//! M3: water from the sky to the gauge. Rain and snow from a GHCN-Daily station fall on
//! every cell of the basin delineated from the DEM; a snowpack melts by degree-days; a
//! soil bucket fills, evaporates with the sun's energy, drains, and spills; runoff and
//! drainage route down the D8 network through linear reservoirs to the pour point. The
//! outlet discharge is scored against the USGS daily record by `esim score`.
//!
//! Everything here is the simplest published form of each process and is labelled so:
//! uniform rain from one station over 2,000 km2, a single soil layer, degree-day melt,
//! energy-limited evaporation with one factor, two routing velocities. The water ledger
//! must close regardless; the score says how far the physics is from the river.
//!
//! Water ledger (m3): atmosphere -> snowpack | soil; snowpack -> soil; soil ->
//! evaporation | channel; channel -> outlet. Stocks declared: snowpack, soil, channel.

use esim_kernel::manifest::{param_duration, param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module, Time};
use esim_registry::obs::{ghcn_station, usgs_daily};
use esim_registry::{dem_window, Catalog, Fetcher, FlowNet};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

const LEDGER: &str = "water";
const LATENT_HEAT_J_PER_KG: f64 = 2.45e6;

#[derive(Serialize, Deserialize)]
pub struct Basin {
    name: String,
    // configuration
    station: String,
    usgs_site: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64,
    res_arcsec: f64,
    gauge_lat: f64, gauge_lon: f64,
    snap_radius: usize,
    obs_hour_utc: i64,
    local_offset_h: i64,
    dt: Duration,
    report_every: Duration,
    soil_capacity_mm: f64,
    drain_per_day: f64,
    pet_factor: f64,
    snow_t_c: f64,
    ddf_mm_per_c_day: f64,
    channel_km2: f64,
    v_channel_m_s: f64,
    v_hill_m_s: f64,
    energy_source: String,
    /// "station" (GHCN daily, uniform) or "wx" (hourly fields wx.precip_mm and wx.t2m_c)
    forcing: String,
    /// if a `veg.lai` field exists, evaporation demand scales with leaf area
    lai_coupling: bool,
    // state (per basin cell, in basin order)
    snow_mm: Vec<f64>,
    soil_mm: Vec<f64>,
    chan_m3: Vec<f64>,
    seen_version: u64,
    day_q_sum: f64, day_q_n: u32,
    day_p_mm: f64,
    day_sum_soil: Vec<f32>, day_steps: u32,
    next_report: Option<Time>,
    missing_prcp_days: u32,
    last_tmean: f64,
    // derived, rebuilt on restore
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] cells: Vec<usize>,          // basin cells in upstream-to-downstream order
    #[serde(skip)] pos: Vec<usize>,            // grid index -> position in `cells`, or MAX
    #[serde(skip)] down: Vec<usize>,           // position of downstream cell, or MAX (pour)
    #[serde(skip)] area: Vec<f64>,
    #[serde(skip)] frac: Vec<f64>,             // per-step release fraction of channel storage
    #[serde(skip)] accum_km2: Vec<f32>,
    #[serde(skip)] prcp: BTreeMap<i64, f64>,
    #[serde(skip)] tmax: BTreeMap<i64, f64>,
    #[serde(skip)] tmin: BTreeMap<i64, f64>,
    #[serde(skip)] q_obs: BTreeMap<i64, f64>,
    #[serde(skip)] basin_km2: f64,
}

impl Basin {
    pub fn new(name: &str, p: &toml::Value) -> Result<Basin, String> {
        let mut b = Basin {
            name: name.to_string(),
            station: param_str(p, "station", "USC00138806").to_string(),
            usgs_site: param_str(p, "usgs_site", "05481000").to_string(),
            lat_s: param_f64(p, "lat_s", 42.3), lat_n: param_f64(p, "lat_n", 43.2),
            lon_w: param_f64(p, "lon_w", -94.2), lon_e: param_f64(p, "lon_e", -93.5),
            res_arcsec: param_f64(p, "res_arcsec", 15.0),
            gauge_lat: param_f64(p, "gauge_lat", 42.4319722), gauge_lon: param_f64(p, "gauge_lon", -93.8059444),
            snap_radius: param_f64(p, "snap_radius", 3.0) as usize,
            obs_hour_utc: param_f64(p, "obs_hour_utc", 13.0) as i64,
            local_offset_h: param_f64(p, "local_offset_h", -6.0) as i64,
            dt: param_duration(p, "dt", "1h")?,
            report_every: param_duration(p, "report_every", "1d")?,
            soil_capacity_mm: param_f64(p, "soil_capacity_mm", 150.0),
            drain_per_day: param_f64(p, "drain_per_day", 0.03),
            pet_factor: param_f64(p, "pet_factor", 0.7),
            snow_t_c: param_f64(p, "snow_t_c", 0.0),
            ddf_mm_per_c_day: param_f64(p, "ddf_mm_per_c_day", 3.0),
            channel_km2: param_f64(p, "channel_km2", 2.0),
            v_channel_m_s: param_f64(p, "v_channel_m_s", 0.4),
            v_hill_m_s: param_f64(p, "v_hill_m_s", 0.02),
            energy_source: param_str(p, "energy_source", "sun.incident_j").to_string(),
            forcing: param_str(p, "forcing", "station").to_string(),
            lai_coupling: p.get("lai_coupling").and_then(|v| v.as_bool()).unwrap_or(true),
            snow_mm: Vec::new(), soil_mm: Vec::new(), chan_m3: Vec::new(), seen_version: 0,
            day_q_sum: 0.0, day_q_n: 0, day_p_mm: 0.0, day_sum_soil: Vec::new(), day_steps: 0, next_report: None,
            missing_prcp_days: 0, last_tmean: 0.0,
            grid: None, cells: Vec::new(), pos: Vec::new(), down: Vec::new(), area: Vec::new(), frac: Vec::new(), accum_km2: Vec::new(),
            prcp: BTreeMap::new(), tmax: BTreeMap::new(), tmin: BTreeMap::new(), q_obs: BTreeMap::new(), basin_km2: 0.0,
        };
        b.load()?;
        Ok(b)
    }

    fn load(&mut self) -> Result<(), String> {
        let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let dem = cat.asset("copernicus-dem-glo30").ok_or("catalog has no copernicus-dem-glo30")?;
        let mut f = Fetcher::default_for(&root)?;
        let w = dem_window(&mut f, dem, self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let grid = LatLonGrid { nx: w.nx, ny: w.ny, dlon: w.dlon, dlat: w.dlat, lon_w: w.lon_w, lat_n: w.lat_n };
        let row_area: Vec<f64> = (0..grid.ny).map(|r| grid.row_area(r)).collect();
        let net = FlowNet::build(&f, &w, &row_area);
        let col = ((self.gauge_lon - w.lon_w) / w.dlon) as usize;
        let row = ((w.lat_n - self.gauge_lat) / w.dlat) as usize;
        let pour = net.snap(col, row, self.snap_radius);
        let mask = net.basin(pour);
        let cells: Vec<usize> = net.order.iter().copied().filter(|&i| mask[i]).collect();
        let mut pos = vec![usize::MAX; grid.nx * grid.ny];
        for (k, &i) in cells.iter().enumerate() { pos[i] = k; }
        let down: Vec<usize> = cells.iter().map(|&i| if i == pour { usize::MAX } else { pos[net.down[i]] }).collect();
        let area: Vec<f64> = cells.iter().map(|&i| row_area[i / grid.nx]).collect();
        let dt = self.dt.as_secs_f64();
        let frac: Vec<f64> = cells.iter().map(|&i| {
            let (mpd_lon, mpd_lat) = grid.metres_per_degree(i / grid.nx);
            let len = (mpd_lon * grid.dlon).min(mpd_lat * grid.dlat);
            let v = if net.accum_m2[i] as f64 >= self.channel_km2 * 1e6 { self.v_channel_m_s } else { self.v_hill_m_s };
            1.0 - (-dt * v / len).exp()
        }).collect();
        self.accum_km2 = cells.iter().map(|&i| net.accum_m2[i] / 1e6).collect();
        self.basin_km2 = area.iter().sum::<f64>() / 1e6;
        let n = cells.len();
        if self.soil_mm.len() != n {
            self.snow_mm = vec![0.0; n];
            self.soil_mm = vec![0.5 * self.soil_capacity_mm; n];
            self.chan_m3 = vec![0.0; n];
            self.day_sum_soil = vec![0.0; n];
        }
        let g = ghcn_station(&mut f, &self.station)?;
        self.prcp = g.prcp_mm; self.tmax = g.tmax_c; self.tmin = g.tmin_c;
        // observed discharge for the window of years the run may touch; the CLI knows the
        // run window but the module does not, so fetch a generous span once
        self.q_obs = usgs_daily(&mut f, &self.usgs_site, "00060", "1940-01-01", "2026-12-31").unwrap_or_default();
        self.grid = Some(grid); self.cells = cells; self.pos = pos; self.down = down; self.area = area; self.frac = frac;
        Ok(())
    }

    /// GHCN date (00:00 UTC of the observation day) whose 24-hour total covers the step
    /// ending at `t`: totals end at obs_hour_utc, so shift by (24 - obs_hour) hours.
    fn obs_day(&self, t: Time) -> i64 {
        let shifted = t.0 + (24 - self.obs_hour_utc) * 3600 - 1;
        shifted.div_euclid(86_400) * 86_400
    }
    /// Local calendar day (00:00 UTC key) that the step ending at `t` belongs to.
    fn local_day(&self, t: Time) -> i64 { (t.0 - 1 + self.local_offset_h * 3600).div_euclid(86_400) * 86_400 }

    fn stocks(&self) -> (f64, f64, f64) {
        let snow: f64 = self.snow_mm.iter().zip(&self.area).map(|(s, a)| s / 1000.0 * a).sum();
        let soil: f64 = self.soil_mm.iter().zip(&self.area).map(|(s, a)| s / 1000.0 * a).sum();
        let chan: f64 = self.chan_m3.iter().sum();
        (snow, soil, chan)
    }
}

impl Module for Basin {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.ledger(LEDGER, "m3", 1.0);
        let (s, so, c) = self.stocks();
        ctx.declare_stock(LEDGER, "snowpack", s);
        ctx.declare_stock(LEDGER, "soil", so);
        ctx.declare_stock(LEDGER, "channel", c);
        let grid = self.grid.clone().unwrap();
        let mut basin = vec![f32::NAN; grid.nx * grid.ny];
        let mut acc = vec![f32::NAN; grid.nx * grid.ny];
        for (k, &i) in self.cells.iter().enumerate() { basin[i] = 1.0; acc[i] = self.accum_km2[k].max(0.01).log10(); }
        ctx.field("basin", "1", &grid, basin);
        ctx.field("flow_accum_log10_km2", "log10 km2", &grid, acc);
        ctx.report("basin.area_km2", "km2", self.basin_km2);
        self.next_report = Some(ctx.now().add(self.report_every));
        ctx.schedule_with_priority(self.dt, 2, "tick", json!(null));
    }

    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" { return; }
        let now = ctx.now();
        let dt = self.dt.as_secs_f64();
        let steps_per_day = (86_400.0 / dt).max(1.0);
        let day = self.obs_day(now);
        let p_mm = match self.prcp.get(&day) { Some(v) => *v / steps_per_day, None => { 0.0 } };
        if self.prcp.get(&day).is_none() && now.0 % 86_400 == self.obs_hour_utc * 3600 { self.missing_prcp_days += 1; }
        let tmean = match (self.tmax.get(&day), self.tmin.get(&day)) { (Some(a), Some(b)) => { self.last_tmean = 0.5 * (a + b); self.last_tmean } _ => self.last_tmean };
        let snowing = tmean < self.snow_t_c;
        let melt_mm = if tmean > 0.0 { self.ddf_mm_per_c_day * tmean / steps_per_day } else { 0.0 };
        // energy for evaporation from the sun's field (J per cell footprint this step)
        let energy: Option<Vec<f32>> = ctx.world_read(&self.energy_source).map(|f| f.data.clone());
        let drain_frac = self.drain_per_day * dt / 86_400.0;
        // gridded forcing, if asked for and present: per-cell precipitation this step and air temperature
        let wx_p: Option<Vec<f32>> = if self.forcing == "wx" { ctx.world_read("wx.precip_mm").map(|f| f.data.clone()) } else { None };
        let wx_t: Option<Vec<f32>> = if self.forcing == "wx" { ctx.world_read("wx.t2m_c").map(|f| f.data.clone()) } else { None };
        let steps_per_day_f = steps_per_day;
        let lai: Option<Vec<f32>> = if self.lai_coupling { ctx.world_read("veg.lai").map(|f| f.data.clone()) } else { None };

        let (mut to_snow, mut to_soil, mut melt, mut evap, mut to_chan) = (0.0f64, 0.0, 0.0, 0.0, 0.0);
        let mut outlet = 0.0f64;
        let n = self.cells.len();
        let mut inflow = vec![0.0f64; n];
        for k in 0..n {
            let a = self.area[k];
            let i = self.cells[k];
            // precipitation and phase, from the station day or the gridded hour
            let (p_mm, snowing, melt_mm) = match (&wx_p, &wx_t) {
                (Some(pp), Some(tt)) => {
                    let p = pp[i] as f64; let tc = tt[i] as f64;
                    let tc = if tc.is_finite() { tc } else { tmean };
                    (p, tc < self.snow_t_c, if tc > 0.0 { self.ddf_mm_per_c_day * tc / steps_per_day_f } else { 0.0 })
                }
                _ => (p_mm, snowing, melt_mm),
            };
            if snowing { self.snow_mm[k] += p_mm; to_snow += p_mm / 1000.0 * a; }
            else { self.soil_mm[k] += p_mm; to_soil += p_mm / 1000.0 * a; }
            self.day_p_mm += p_mm * a; // area-weighted, divided out at the day boundary
            // melt
            let m = melt_mm.min(self.snow_mm[k]);
            self.snow_mm[k] -= m; self.soil_mm[k] += m; melt += m / 1000.0 * a;
            // evaporation, energy limited and moisture scaled
            if let Some(e) = &energy {
                let pet_mm = self.pet_factor * (e[i] as f64 / a) / LATENT_HEAT_J_PER_KG * 1000.0 / 1000.0; // J/m2 -> kg/m2 -> mm
                // bare soil evaporates less than a closed canopy transpires
                let cover = lai.as_ref().map(|l| { let v = l[i] as f64; if v.is_finite() { 0.5 + 0.5 * (1.0 - (-0.5 * v).exp()) } else { 1.0 } }).unwrap_or(1.0);
                let et = (pet_mm * cover * (self.soil_mm[k] / self.soil_capacity_mm).clamp(0.0, 1.0)).min(self.soil_mm[k]);
                self.soil_mm[k] -= et; evap += et / 1000.0 * a;
            }
            // saturation excess and drainage to the channel store of this cell
            let excess = (self.soil_mm[k] - self.soil_capacity_mm).max(0.0);
            self.soil_mm[k] -= excess;
            let drain = self.soil_mm[k] * drain_frac;
            self.soil_mm[k] -= drain;
            let gen = (excess + drain) / 1000.0 * a;
            to_chan += gen;
            inflow[k] += gen;
            self.day_sum_soil[k] += self.soil_mm[k] as f32;
        }
        // route: cells are in upstream-to-downstream order
        for k in 0..n {
            self.chan_m3[k] += inflow[k];
            let out = self.chan_m3[k] * self.frac[k];
            self.chan_m3[k] -= out;
            let d = self.down[k];
            if d == usize::MAX { outlet += out; } else { inflow[d] += out; }
        }
        ctx.transfer(LEDGER, "atmosphere", "snowpack", to_snow);
        ctx.transfer(LEDGER, "atmosphere", "soil", to_soil);
        ctx.transfer(LEDGER, "snowpack", "soil", melt);
        ctx.transfer(LEDGER, "soil", "evaporation", evap);
        ctx.transfer(LEDGER, "soil", "channel", to_chan);
        ctx.transfer(LEDGER, "channel", "outlet", outlet);
        let (s, so, c) = self.stocks();
        ctx.declare_stock(LEDGER, "snowpack", s);
        ctx.declare_stock(LEDGER, "soil", so);
        ctx.declare_stock(LEDGER, "channel", c);
        let q = outlet / dt;
        ctx.report("hydro.q_outlet_m3_s", "m3/s", q);
        self.day_q_sum += q; self.day_q_n += 1; self.day_steps += 1;
        // local-day boundary: daily mean simulated and observed discharge for scoring
        let next_local = self.local_day(Time(now.0 + self.dt.0));
        if next_local != self.local_day(now) {
            let ld = self.local_day(now);
            let qd = self.day_q_sum / self.day_q_n.max(1) as f64;
            ctx.report("hydro.q_sim_daily_m3_s", "m3/s", qd);
            if let Some(o) = self.q_obs.get(&ld) { ctx.report("hydro.q_obs_daily_m3_s", "m3/s", *o); }
            ctx.report("hydro.precip_daily_mm", "mm", self.day_p_mm / (self.basin_km2 * 1e6));
            let grid = self.grid.clone().unwrap();
            let mut sf = vec![f32::NAN; grid.nx * grid.ny];
            let mut snf = vec![f32::NAN; grid.nx * grid.ny];
            for (k, &i) in self.cells.iter().enumerate() { sf[i] = self.soil_mm[k] as f32; snf[i] = self.snow_mm[k] as f32; }
            ctx.world_write("water.soil_mm", "mm", &grid, sf);
            ctx.world_write("water.snow_mm", "mm", &grid, snf);
            ctx.report("hydro.snowpack_mean_mm", "mm", s / self.basin_km2 / 1e3);
            ctx.report("hydro.soil_mean_mm", "mm", so / self.basin_km2 / 1e3);
            self.day_q_sum = 0.0; self.day_q_n = 0; self.day_p_mm = 0.0;
        }
        if let Some(nr) = self.next_report {
            if now >= nr {
                let grid = self.grid.clone().unwrap();
                let mut frame = vec![f32::NAN; grid.nx * grid.ny];
                let steps = self.day_steps.max(1) as f32;
                for (k, &i) in self.cells.iter().enumerate() { frame[i] = self.day_sum_soil[k] / steps; }
                ctx.field("soil_daily_mean_mm", "mm", &grid, frame);
                self.day_sum_soil.iter_mut().for_each(|v| *v = 0.0);
                self.day_steps = 0;
                self.next_report = Some(nr.add(self.report_every));
            }
        }
        ctx.schedule_with_priority(self.dt, 2, "tick", json!(null));
    }

    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) {
        let mut b: Basin = serde_json::from_value(v).unwrap();
        b.load().expect("basin reload on restore");
        *self = b;
    }
}
