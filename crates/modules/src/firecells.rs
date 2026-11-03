//! Our own fire engine: a heat budget on cells, from the facts we have about the ground.
//!
//! Every cell of a one-arc-second window knows its elevation and slope (Copernicus DEM), its
//! land cover (ESA WorldCover), the share of it under buildings and their height (LA County
//! footprints), whether a major road crosses it (OpenStreetMap), and, hour by hour, the 3 km
//! wind, gust, temperature and humidity over it (NOAA HRRR). A burning cell radiates heat
//! into the cells around it, further downwind and upslope; an unburned cell ignites when the
//! heat it has received outweighs what its fuel and moisture can absorb, and it then burns
//! with an intensity and for a time set by what is in it: grass briefly, chaparral longer,
//! a built-up cell for hours. Burning cells in a strong wind loft embers that land downwind
//! and may start a spot fire. Roads and water carry nothing. Nobody fights it.
//!
//! Nothing here is Rothermel. Every rate and threshold is a guess, declared so, and the
//! point of the engine is that the record (the mapped perimeters, the inspected structures)
//! can be held against it and the guesses turned into fitted numbers, which is the work.

use esim_kernel::manifest::{param_duration, param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module, Time};
use esim_registry::wildfire::{building_footprints, cached_context, context, hrrr_hour, landcover_window, wind_hourly, HrrrHour, WindSeries};
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

const ACRE_M2: f64 = 4046.8564224;

/// What a kind of fuel does once lit: how hard it is to light, how hot it burns, for how long.
#[derive(Clone, Copy)]
struct FuelKind { ignite: f32, intensity: f32, burn_min: f32 }

#[derive(Serialize, Deserialize)]
pub struct Cells {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    dt: Duration, substep_s: f64, frame_every: Duration, frame_res_arcsec: f64,
    ignition_time: String, ignition_lat: f64, ignition_lon: f64,
    wind: String,            // hrrr | era5
    wind_use: String,        // mean | gust
    wind_start: String, wind_end: String,
    buildings_service: String, buildings_where: String, buildings_key: String,
    // the guesses
    reach_m: f64, lambda_m: f64, wind_gain: f64, slope_gain: f64,
    moisture_gain: f64, live_moisture_pct: f64,
    grass_ignite: f64, grass_intensity: f64, grass_burn_min: f64,
    shrub_ignite: f64, shrub_intensity: f64, shrub_burn_min: f64,
    tree_ignite: f64, tree_intensity: f64, tree_burn_min: f64,
    built_ignite: f64, built_intensity: f64, built_burn_min: f64,
    cool_min: f64,
    ignite_scale: f64,
    spot_rate: f64, spot_mean_m: f64, spot_land_p: f64, spot_min_intensity: f64,
    road_widths_m: String,
    // state
    ignited: bool,
    since_ignition_s: f64,
    last_frame: i64,
    t_ign: Vec<f32>,         // seconds since ignition, NaN unburned
    t_out: Vec<f32>,         // when a burning cell goes out
    heat: Vec<f32>,
    heat_t: Vec<f32>,
    active: Vec<usize>,
    spots: u64,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] frame_grid: Option<LatLonGrid>,
    #[serde(skip)] elev: Vec<f32>,
    #[serde(skip)] slope: Vec<f32>,
    #[serde(skip)] aspect: Vec<f32>,
    #[serde(skip)] cover: Vec<u8>,
    #[serde(skip)] built_frac: Vec<f32>,
    #[serde(skip)] built_height: Vec<f32>,
    #[serde(skip)] road: Vec<bool>,
    #[serde(skip)] kind: Vec<u8>,          // 0 none, 1 grass, 2 shrub, 3 tree, 4 built
    #[serde(skip)] area_m2: Vec<f64>,
    #[serde(skip)] metres: Vec<(f64, f64)>,
    #[serde(skip)] hrrr: Vec<Option<HrrrHour>>,  // per hour from wind_start
    #[serde(skip)] hrrr_t0: i64,
    #[serde(skip)] era5: Option<WindSeries>,
    #[serde(skip)] cell_wind: Vec<(f32, f32)>,   // this hour: (speed m/s, direction the wind blows TOWARD, rad)
    #[serde(skip)] cell_moist: Vec<f32>,
    #[serde(skip)] offsets: Vec<(isize, isize, f32, f32)>, // dc, dr, distance factor (cells), bearing rad
}

fn civil(t: i64) -> (String, u8) {
    let iso = Time(t).to_iso();
    (iso[..10].replace('-', ""), iso[11..13].parse().unwrap_or(0))
}

impl Cells {
    pub fn new(name: &str, p: &toml::Value) -> Result<Cells, String> {
        let mut c = Cells {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 34.0), lat_n: param_f64(p, "lat_n", 34.16), lon_w: param_f64(p, "lon_w", -118.72), lon_e: param_f64(p, "lon_e", -118.46),
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            dt: param_duration(p, "dt", "1h")?, substep_s: param_f64(p, "substep_s", 30.0),
            frame_every: param_duration(p, "frame_every", "1h")?, frame_res_arcsec: param_f64(p, "frame_res_arcsec", 3.0),
            ignition_time: param_str(p, "ignition_time", "2025-01-07T18:30:00Z").to_string(),
            ignition_lat: param_f64(p, "ignition_lat", 34.0723), ignition_lon: param_f64(p, "ignition_lon", -118.5451),
            wind: param_str(p, "wind", "hrrr").to_string(), wind_use: param_str(p, "wind_use", "mean").to_string(),
            wind_start: param_str(p, "wind_start", "2025-01-07T18:00:00Z").to_string(), wind_end: param_str(p, "wind_end", "2025-01-12T00:00:00Z").to_string(),
            buildings_service: param_str(p, "buildings_service", "https://services.arcgis.com/RmCCgQtiZLDCtblq/arcgis/rest/services/2025_WildFire_LARIAC_BUILDINGS_100M_Plus_DINS_MaxMin_DAMAGE_NEW/FeatureServer/0").to_string(),
            buildings_where: param_str(p, "buildings_where", "Incident_Name LIKE '%Palisades%'").to_string(),
            buildings_key: param_str(p, "buildings_key", "lacounty-lariac/2025-palisades").to_string(),
            reach_m: param_f64(p, "reach_m", 90.0), lambda_m: param_f64(p, "lambda_m", 20.0), wind_gain: param_f64(p, "wind_gain", 0.15), slope_gain: param_f64(p, "slope_gain", 1.5),
            moisture_gain: param_f64(p, "moisture_gain", 10.0), live_moisture_pct: param_f64(p, "live_moisture_pct", 60.0),
            grass_ignite: param_f64(p, "grass_ignite", 0.6), grass_intensity: param_f64(p, "grass_intensity", 1.0), grass_burn_min: param_f64(p, "grass_burn_min", 5.0),
            shrub_ignite: param_f64(p, "shrub_ignite", 1.5), shrub_intensity: param_f64(p, "shrub_intensity", 2.5), shrub_burn_min: param_f64(p, "shrub_burn_min", 20.0),
            tree_ignite: param_f64(p, "tree_ignite", 2.0), tree_intensity: param_f64(p, "tree_intensity", 2.0), tree_burn_min: param_f64(p, "tree_burn_min", 30.0),
            built_ignite: param_f64(p, "built_ignite", 4.0), built_intensity: param_f64(p, "built_intensity", 3.0), built_burn_min: param_f64(p, "built_burn_min", 90.0),
            cool_min: param_f64(p, "cool_min", 15.0),
            ignite_scale: param_f64(p, "ignite_scale", 1.0),
            spot_rate: param_f64(p, "spot_rate", 0.003), spot_mean_m: param_f64(p, "spot_mean_m", 25.0), spot_land_p: param_f64(p, "spot_land_p", 0.3), spot_min_intensity: param_f64(p, "spot_min_intensity", 2.0),
            road_widths_m: param_str(p, "road_widths_m", "motorway=30,trunk=20,primary=14,secondary=10,tertiary=8").to_string(),
            ignited: false, since_ignition_s: 0.0, last_frame: i64::MIN / 4,
            t_ign: Vec::new(), t_out: Vec::new(), heat: Vec::new(), heat_t: Vec::new(), active: Vec::new(), spots: 0,
            grid: None, frame_grid: None, elev: Vec::new(), slope: Vec::new(), aspect: Vec::new(), cover: Vec::new(), built_frac: Vec::new(), built_height: Vec::new(), road: Vec::new(), kind: Vec::new(),
            area_m2: Vec::new(), metres: Vec::new(), hrrr: Vec::new(), hrrr_t0: 0, era5: None, cell_wind: Vec::new(), cell_moist: Vec::new(), offsets: Vec::new(),
        };
        c.load()?;
        Ok(c)
    }

    fn load(&mut self) -> Result<(), String> {
        let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let dem = cat.asset("copernicus-dem-glo30").ok_or("catalog has no copernicus-dem-glo30")?;
        let lc = cat.asset("esa-worldcover-2021").ok_or("catalog has no esa-worldcover-2021")?;
        let mut fe = Fetcher::default_for(&root)?;
        let w = dem_window(&mut fe, dem, self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let grid = LatLonGrid { nx: w.nx, ny: w.ny, dlon: w.dlon, dlat: w.dlat, lon_w: w.lon_w, lat_n: w.lat_n };
        let (slope, aspect) = w.slope_aspect(&fe);
        self.elev = w.elev.clone();
        self.cover = landcover_window(&mut fe, lc, grid.nx, grid.ny, grid.dlon, grid.dlat, grid.lon_w, grid.lat_n)?;
        let n = grid.nx * grid.ny;
        self.area_m2 = (0..grid.ny).flat_map(|r| std::iter::repeat_n(grid.row_area(r), grid.nx)).collect();
        self.metres = (0..grid.ny).map(|r| { let (mx, my) = grid.metres_per_degree(r); (mx * grid.dlon, my * grid.dlat) }).collect();
        // buildings: fraction of each cell under a footprint, and the mean height there
        self.built_frac = vec![0.0; n]; self.built_height = vec![f32::NAN; n];
        let mut hsum = vec![0f32; n]; let mut hcnt = vec![0u16; n];
        match building_footprints(&mut fe, &self.buildings_service, &self.buildings_key, &self.buildings_where) {
            Ok(bs) => {
                for b in &bs {
                    // the footprint's bounding cells, then its area share by sub-sampling each cell 3x3
                    let (xs, ys): (Vec<f64>, Vec<f64>) = b.ring.iter().cloned().unzip();
                    let (x0, x1) = (xs.iter().cloned().fold(f64::INFINITY, f64::min), xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
                    let (y0, y1) = (ys.iter().cloned().fold(f64::INFINITY, f64::min), ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
                    if x1 < grid.lon_w || x0 > grid.lon_e() || y1 < grid.lat_s() || y0 > grid.lat_n { continue; }
                    let c0 = (((x0 - grid.lon_w) / grid.dlon).floor().max(0.0)) as usize; let c1 = (((x1 - grid.lon_w) / grid.dlon).floor() as usize).min(grid.nx - 1);
                    let r0 = (((grid.lat_n - y1) / grid.dlat).floor().max(0.0)) as usize; let r1 = (((grid.lat_n - y0) / grid.dlat).floor() as usize).min(grid.ny - 1);
                    let inside = |x: f64, y: f64| -> bool { let mut c = false; let m = b.ring.len(); for i in 0..m { let (xi, yi) = b.ring[i]; let (xj, yj) = b.ring[(i + 1) % m]; if (yi > y) != (yj > y) && x < xi + (y - yi) * (xj - xi) / (yj - yi) { c = !c; } } c };
                    for r in r0..=r1 { for c in c0..=c1 {
                        let mut hits = 0;
                        for sy in 0..3 { for sx in 0..3 {
                            let x = grid.lon_w + (c as f64 + (sx as f64 + 0.5) / 3.0) * grid.dlon; let y = grid.lat_n - (r as f64 + (sy as f64 + 0.5) / 3.0) * grid.dlat;
                            if inside(x, y) { hits += 1; }
                        } }
                        if hits > 0 { let k = r * grid.nx + c; self.built_frac[k] = (self.built_frac[k] + hits as f32 / 9.0).min(1.0); if b.height.is_finite() { hsum[k] += b.height as f32; hcnt[k] += 1; } }
                    } }
                }
                for k in 0..n { if hcnt[k] > 0 { self.built_height[k] = hsum[k] / hcnt[k] as f32; } }
                eprintln!("buildings: {} footprints, {} cells touched", bs.len(), self.built_frac.iter().filter(|&&f| f > 0.0).count());
            }
            Err(e) => eprintln!("buildings: {e}; built-up land from land cover only"),
        }
        // roads as breaks: the major roads of the window, buffered by class
        self.road = vec![false; n];
        let widths: std::collections::HashMap<String, f64> = self.road_widths_m.split(',').filter_map(|kv| kv.split_once('=')).map(|(k, v)| (k.trim().to_string(), v.trim().parse().unwrap_or(10.0))).collect();
        let ctxv = context(&mut fe, self.lat_s, self.lat_n, self.lon_w, self.lon_e).ok().or_else(|| cached_context(&fe, self.lat_s, self.lat_n, self.lon_w, self.lon_e));
        if let Some(cv) = &ctxv {
            for rd in cv["roads"].as_array().into_iter().flatten() {
                let Some(&wm) = widths.get(rd["class"].as_str().unwrap_or("")) else { continue };
                let pts: Vec<(f64, f64)> = rd["pts"].as_array().into_iter().flatten().filter_map(|p| Some((p[0].as_f64()?, p[1].as_f64()?))).collect();
                for seg in pts.windows(2) {
                    let (ax, ay) = seg[0]; let (bx, by) = seg[1];
                    let row_mid = (((grid.lat_n - (ay + by) / 2.0) / grid.dlat) as usize).min(grid.ny - 1);
                    let (mx, my) = self.metres[row_mid];
                    let len = (((bx - ax) * mx / grid.dlon).powi(2) + ((by - ay) * my / grid.dlat).powi(2)).sqrt();
                    let steps = (len / 5.0).ceil().max(1.0) as usize;
                    let half = wm / 2.0;
                    for s in 0..=steps {
                        let t = s as f64 / steps as f64; let (x, y) = (ax + (bx - ax) * t, ay + (by - ay) * t);
                        if x < grid.lon_w || x >= grid.lon_e() || y <= grid.lat_s() || y > grid.lat_n { continue; }
                        let c = ((x - grid.lon_w) / grid.dlon) as usize; let r = ((grid.lat_n - y) / grid.dlat) as usize;
                        let (rc, rr) = ((half / mx).ceil() as isize, (half / my).ceil() as isize);
                        for dr in -rr..=rr { for dc in -rc..=rc {
                            let (cc, rrr) = (c as isize + dc, r as isize + dr);
                            if cc < 0 || rrr < 0 || cc >= grid.nx as isize || rrr >= grid.ny as isize { continue; }
                            if ((dc as f64 * mx).powi(2) + (dr as f64 * my).powi(2)).sqrt() <= half.max(mx * 0.5) { self.road[rrr as usize * grid.nx + cc as usize] = true; }
                        } }
                    }
                }
            }
        }
        // the fuel kind of each cell
        self.kind = (0..n).map(|k| {
            if self.road[k] { return 0; }
            if self.built_frac[k] >= 0.05 { return 4; }
            match self.cover[k] { 10 => 3, 20 => 2, 30 | 40 | 90 => 1, 50 => 4, _ => 0 }
        }).collect();
        // the wind: HRRR hours over the window, or the ERA5 point series
        let t0 = Time::parse(&self.wind_start)?.0; let t1 = Time::parse(&self.wind_end)?.0;
        self.hrrr_t0 = t0;
        if self.wind == "hrrr" {
            let hours = ((t1 - t0) / 3600).max(1) as usize + 1;
            self.hrrr = Vec::with_capacity(hours);
            for h in 0..hours {
                let (date, hh) = civil(t0 + h as i64 * 3600);
                match hrrr_hour(&mut fe, &date, hh, self.lat_s, self.lat_n, self.lon_w, self.lon_e) {
                    Ok(x) => self.hrrr.push(Some(x)),
                    Err(e) => { eprintln!("hrrr {date} {hh}: {e}"); self.hrrr.push(None); }
                }
            }
        } else {
            let (c_lat, c_lon) = ((self.lat_s + self.lat_n) / 2.0, (self.lon_w + self.lon_e) / 2.0);
            self.era5 = Some(wind_hourly(&mut fe, c_lat, c_lon, &self.wind_start[..10], &self.wind_end[..10])?);
        }
        // neighbour offsets within reach
        let (mx, my) = self.metres[grid.ny / 2];
        let (rc, rr) = ((self.reach_m / mx).ceil() as isize, (self.reach_m / my).ceil() as isize);
        self.offsets = (-rr..=rr).flat_map(|dr| (-rc..=rc).map(move |dc| (dc, dr))).filter(|&(dc, dr)| dc != 0 || dr != 0)
            .filter_map(|(dc, dr)| { let (ex, ny_m) = (dc as f64 * mx, -(dr as f64) * my); let d = (ex * ex + ny_m * ny_m).sqrt(); if d <= self.reach_m { Some((dc, dr, d as f32, ex.atan2(ny_m) as f32)) } else { None } }).collect();
        if self.t_ign.len() != n { self.t_ign = vec![f32::NAN; n]; self.t_out = vec![0.0; n]; self.heat = vec![0.0; n]; self.heat_t = vec![0.0; n]; }
        self.cell_wind = vec![(0.0, 0.0); n]; self.cell_moist = vec![0.05; n];
        let k = (self.frame_res_arcsec / self.res_arcsec).round().max(1.0) as usize;
        self.frame_grid = Some(LatLonGrid { nx: grid.nx.div_ceil(k), ny: grid.ny.div_ceil(k), dlon: grid.dlon * k as f64, dlat: grid.dlat * k as f64, lon_w: grid.lon_w, lat_n: grid.lat_n });
        self.slope = slope; self.aspect = aspect; self.grid = Some(grid);
        Ok(())
    }

    fn fuel(&self, kind: u8) -> FuelKind {
        match kind {
            1 => FuelKind { ignite: self.grass_ignite as f32, intensity: self.grass_intensity as f32, burn_min: self.grass_burn_min as f32 },
            2 => FuelKind { ignite: self.shrub_ignite as f32, intensity: self.shrub_intensity as f32, burn_min: self.shrub_burn_min as f32 },
            3 => FuelKind { ignite: self.tree_ignite as f32, intensity: self.tree_intensity as f32, burn_min: self.tree_burn_min as f32 },
            4 => FuelKind { ignite: self.built_ignite as f32, intensity: self.built_intensity as f32, burn_min: self.built_burn_min as f32 },
            _ => FuelKind { ignite: f32::INFINITY, intensity: 0.0, burn_min: 0.0 },
        }
    }
    /// Dead fine fuel moisture (fraction) from humidity and temperature: Simard's equilibrium, recalled.
    fn dead_moisture(t_c: f32, rh: f32) -> f32 {
        let t = t_c * 9.0 / 5.0 + 32.0; let h = rh.clamp(0.0, 100.0);
        let m = if h < 10.0 { 0.03229 + 0.281073 * h - 0.000578 * h * t } else if h < 50.0 { 2.22749 + 0.160107 * h - 0.01478 * t } else { 21.0606 + 0.005565 * h * h - 0.00035 * h * t - 0.483199 * h };
        (m / 100.0).clamp(0.01, 0.35)
    }
    /// This hour's wind and moisture on every cell.
    fn set_hour(&mut self, now: Time) {
        let g = self.grid.clone().unwrap();
        let use_gust = self.wind_use == "gust";
        if self.wind == "hrrr" {
            let h = ((now.0 - self.hrrr_t0) / 3600).clamp(0, self.hrrr.len() as i64 - 1) as usize;
            let Some(hh) = &self.hrrr[h] else { return };
            for r in 0..g.ny { for c in 0..g.nx {
                let (lat, lon) = (g.lat(r), g.lon(c));
                let (u, v) = (hh.at(&hh.u, lat, lon), hh.at(&hh.v, lat, lon));
                let mut spd = (u * u + v * v).sqrt();
                if use_gust { let gu = hh.at(&hh.gust, lat, lon); if gu.is_finite() { spd = gu; } }
                let toward = u.atan2(v); // bearing the wind blows toward, clockwise from north
                let k = r * g.nx + c;
                self.cell_wind[k] = (if spd.is_finite() { spd } else { 0.0 }, if toward.is_finite() { toward } else { 0.0 });
                self.cell_moist[k] = Self::dead_moisture(hh.at(&hh.t2m_c, lat, lon), hh.at(&hh.rh_pct, lat, lon));
            } }
        } else if let Some(w) = &self.era5 {
            let Some(i) = w.index_at(now.0) else { return };
            let spd = if use_gust { w.gust_m_s[i] } else { w.wind_m_s[i] };
            let toward = (w.dir_deg[i] + 180.0).to_radians();
            let m = Self::dead_moisture(w.t2m_c[i], w.rh_pct[i]);
            for k in 0..self.cell_wind.len() { self.cell_wind[k] = (spd, toward); self.cell_moist[k] = m; }
        }
    }
    fn to_frame(&self, v: &[f32], f: impl Fn(&[f32]) -> f32) -> Vec<f32> {
        let g = self.grid.as_ref().unwrap(); let fg = self.frame_grid.as_ref().unwrap();
        let k = (self.frame_res_arcsec / self.res_arcsec).round().max(1.0) as usize;
        let mut out = vec![f32::NAN; fg.nx * fg.ny]; let mut block = Vec::with_capacity(k * k);
        for fr in 0..fg.ny { for fc in 0..fg.nx {
            block.clear();
            for r in fr * k..((fr + 1) * k).min(g.ny) { for c in fc * k..((fc + 1) * k).min(g.nx) { block.push(v[r * g.nx + c]); } }
            out[fr * fg.nx + fc] = f(&block);
        } }
        out
    }
    fn cell_of(&self, lat: f64, lon: f64) -> Option<usize> {
        let g = self.grid.as_ref()?;
        if lat > g.lat_n || lat < g.lat_s() || lon < g.lon_w || lon > g.lon_e() { return None; }
        Some((((g.lat_n - lat) / g.dlat) as usize).min(g.ny - 1) * g.nx + (((lon - g.lon_w) / g.dlon) as usize).min(g.nx - 1))
    }
    fn burned_m2(&self) -> f64 { self.t_ign.iter().zip(&self.area_m2).filter(|(v, _)| !v.is_nan()).map(|(_, a)| a).sum() }
    fn ignite(&mut self, k: usize, t: f32) {
        let f = self.fuel(self.kind[k]);
        self.t_ign[k] = t; self.t_out[k] = t + f.burn_min * 60.0 * (if self.kind[k] == 4 { self.built_frac[k].max(0.2) } else { 1.0 });
        self.active.push(k);
    }
}

impl Module for Cells {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        let g = self.grid.clone().unwrap();
        let n = g.nx * g.ny;
        let counts = |kk: u8| self.kind.iter().filter(|&&k| k == kk).count();
        ctx.publish("ground.loaded", json!({ "cells": n, "grass": counts(1), "shrub": counts(2), "tree": counts(3), "built": counts(4), "road_cells": self.road.iter().filter(|&&r| r).count(), "hrrr_hours": self.hrrr.iter().filter(|h| h.is_some()).count(), "wind": self.wind }));
        ctx.field("terrain_elev_m", "m", &g, self.elev.clone());
        ctx.field("building_height", "ft (as stated)", &g, self.built_height.clone());
        let fg = self.frame_grid.clone().unwrap();
        let kinds: Vec<f32> = self.kind.iter().map(|&k| k as f32).collect();
        ctx.field("fire_fuel_kind", "0 none 1 grass 2 shrub 3 tree 4 built", &fg, self.to_frame(&kinds, |b| { let mut best = (0.0f32, 0usize); for &x in b { let c = b.iter().filter(|&&y| y == x).count(); if c > best.1 { best = (x, c); } } best.0 }));
        ctx.world_write("fire.burned_frac", "1", &g, vec![0.0; n]);
        ctx.schedule_with_priority(self.dt, 6, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" { return; }
        let g = self.grid.clone().unwrap();
        let now = ctx.now();
        self.set_hour(now);
        let t_ign = Time::parse(&self.ignition_time).unwrap_or(Time(i64::MAX));
        if !self.ignited && now.0 >= t_ign.0 {
            if let Some(c) = self.cell_of(self.ignition_lat, self.ignition_lon) {
                self.ignited = true;
                let k0 = if self.kind[c] == 0 { self.offsets.iter().map(|&(dc, dr, _, _)| ((c / g.nx) as isize + dr) as usize * g.nx + ((c % g.nx) as isize + dc) as usize).find(|&k| self.kind[k] != 0).unwrap_or(c) } else { c };
                self.ignite(k0, 0.0);
                ctx.publish_at(k0, "fire.ignition", json!({ "lat": self.ignition_lat, "lon": self.ignition_lon, "time": self.ignition_time, "kind": self.kind[k0] }));
            }
        }
        let mut newly = 0usize; let mut spots_hour = 0u64;
        if self.ignited {
            let steps = (self.dt.as_secs_f64() / self.substep_s).round().max(1.0) as usize;
            let dt = (self.dt.as_secs_f64() / steps as f64) as f32;
            let cool = (-(dt / 60.0) / self.cool_min as f32).exp();
            let offsets = self.offsets.clone();
            for _ in 0..steps {
                let t = self.since_ignition_s as f32;
                let active = std::mem::take(&mut self.active);
                let mut keep = Vec::with_capacity(active.len());
                for &b in &active {
                    if t >= self.t_out[b] { continue; }
                    keep.push(b);
                    let ib = self.fuel(self.kind[b]).intensity * if self.kind[b] == 4 { self.built_frac[b].max(0.3) } else { 1.0 };
                    let (bc, br) = ((b % g.nx) as isize, (b / g.nx) as isize);
                    let (wspd, wto) = self.cell_wind[b];
                    for &(dc, dr, d, theta) in &offsets {
                        let (nc, nr) = (bc + dc, br + dr);
                        if nc < 0 || nr < 0 || nc >= g.nx as isize || nr >= g.ny as isize { continue; }
                        let n = nr as usize * g.nx + nc as usize;
                        if !self.t_ign[n].is_nan() || self.kind[n] == 0 { continue; }
                        // reach: longer downwind and uphill
                        let along = (theta - wto).cos().max(0.0);
                        let mut lambda = self.lambda_m as f32 * (1.0 + self.wind_gain as f32 * wspd * along);
                        let asp = self.aspect[n];
                        if asp.is_finite() { let up = (theta - (asp + std::f32::consts::PI)).cos().max(0.0); lambda *= 1.0 + self.slope_gain as f32 * self.slope[n].tan() * up; }
                        let q = ib * (-d / lambda).exp() * dt / 60.0;
                        // the cell cools between contributions
                        let elapsed = t - self.heat_t[n];
                        if elapsed > 0.0 { self.heat[n] *= (-(elapsed / 60.0) / self.cool_min as f32).exp(); self.heat_t[n] = t; }
                        self.heat[n] += q;
                        let need = self.ignite_scale as f32 * self.fuel(self.kind[n]).ignite * (1.0 + self.moisture_gain as f32 * self.cell_moist[n]) * if self.kind[n] == 4 { 1.0 } else { 1.0 + self.live_moisture_pct as f32 / 200.0 * (self.kind[n] == 2) as u8 as f32 };
                        if self.heat[n] >= need { self.ignite(n, t + dt); newly += 1; }
                    }
                    // embers
                    if ib >= self.spot_min_intensity as f32 && wspd > 3.0 {
                        let p = self.spot_rate as f32 * (wspd / 10.0).powi(2) * dt / 60.0;
                        if ctx.rng().f64() < p as f64 {
                            let dist = -(self.spot_mean_m as f32 * wspd / 10.0 * 10.0) * (1.0 - ctx.rng().f64() as f32).ln();
                            let dir = wto + (ctx.rng().f64() as f32 - 0.5) * 0.5;
                            let (mx, my) = self.metres[br as usize];
                            let (lc, lr) = (bc + ((dist * dir.sin()) / mx as f32).round() as isize, br - ((dist * dir.cos()) / my as f32).round() as isize);
                            if lc >= 0 && lr >= 0 && lc < g.nx as isize && lr < g.ny as isize {
                                let l = lr as usize * g.nx + lc as usize;
                                if self.t_ign[l].is_nan() && self.kind[l] != 0 && ctx.rng().f64() < (self.spot_land_p as f32 * (1.0 - 4.0 * self.cell_moist[l]).max(0.0)) as f64 {
                                    self.ignite(l, t + dt); self.spots += 1; spots_hour += 1; newly += 1;
                                    ctx.publish_at(l, "fire.spot", json!({ "from": b, "distance_m": dist }));
                                }
                            }
                        }
                    }
                }
                let _ = cool;
                keep.extend(self.active.drain(..));
                self.active = keep;
                self.since_ignition_s += dt as f64;
            }
        }
        let burned = self.burned_m2();
        ctx.world_write("fire.burned_frac", "1", &g, self.t_ign.iter().map(|v| if v.is_nan() { 0.0 } else { 1.0 }).collect());
        if self.ignited && now.0.saturating_sub(self.last_frame) >= self.frame_every.0 {
            let fg = self.frame_grid.clone().unwrap();
            ctx.field("fire_arrival_h", "h", &fg, self.to_frame(&self.t_ign, |b| b.iter().copied().filter(|v| !v.is_nan()).fold(f32::NAN, |m, v| if m.is_nan() || v < m { v } else { m }) / 3600.0));
            self.last_frame = now.0;
        }
        let k0 = self.cell_of(self.ignition_lat, self.ignition_lon).unwrap_or(0);
        let (ws, wt) = self.cell_wind[k0];
        ctx.report("fire.burned_acres", "acres", burned / ACRE_M2);
        ctx.report("fire.burned_km2", "km2", burned / 1e6);
        ctx.report("fire.active_cells", "cells", self.active.len() as f64);
        ctx.report("fire.ignited_cells_hour", "cells/h", newly as f64);
        ctx.report("fire.spots_hour", "1/h", spots_hour as f64);
        ctx.report("fire.wind_m_s", "m/s", ws as f64);
        ctx.report("fire.wind_from_deg", "deg", (((wt as f64).to_degrees() + 180.0) % 360.0 + 360.0) % 360.0);
        ctx.report("fire.dead_moisture_pct", "%", self.cell_moist[k0] as f64 * 100.0);
        ctx.report("fire.built_cells_burned", "cells", self.t_ign.iter().zip(&self.kind).filter(|(t, &k)| !t.is_nan() && k == 4).count() as f64);
        ctx.schedule_with_priority(self.dt, 6, "tick", json!(null));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut c: Cells = serde_json::from_value(v).unwrap(); c.load().expect("cells reload"); *self = c; }
}

impl Cells {
    pub fn describe(name: &str, p: &toml::Value) -> esim_kernel::describe::Describe {
        let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let (lat_s, lat_n, lon_w, lon_e) = (param_f64(p, "lat_s", 34.0), param_f64(p, "lat_n", 34.16), param_f64(p, "lon_w", -118.72), param_f64(p, "lon_e", -118.46));
        let ctx_table = Fetcher::default_for(&root).ok().and_then(|f| cached_context(&f, lat_s, lat_n, lon_w, lon_e));
        let mut d = esim_kernel::describe::Describe::new("fire.cells", name, "Our own fire engine: a heat budget on one-arc-second cells that know their elevation and slope, land cover, the share under buildings and their height, the major roads, and the hour's 3 km wind, gust, temperature and humidity. A burning cell radiates heat into the cells around it, further downwind and upslope; a cell ignites when the heat it has received outweighs what its fuel and moisture absorb, then burns with the intensity and for the time its contents set; strong winds loft embers that may start spot fires; roads and water carry nothing; nobody fights it. Every rate and threshold is a guess, declared so, to be fitted against the record.")
            .priority(6).dt(param_str(p, "dt", "1h"))
            .param("lat_s", 34.0, "deg", "").param("lat_n", 34.16, "deg", "").param("lon_w", -118.72, "deg", "").param("lon_e", -118.46, "deg", "").param("res_arcsec", 1.0, "arcsec", "")
            .param("dt", "1h", "", "the weather step").param("substep_s", 30.0, "s", "").param("frame_every", "1h", "", "").param("frame_res_arcsec", 3.0, "arcsec", "")
            .param("ignition_time", "2025-01-07T18:30:00Z", "", "").param("ignition_lat", 34.0723, "deg", "").param("ignition_lon", -118.5451, "deg", "")
            .param("wind", "hrrr", "", "hrrr: NOAA's 3 km analysis, a field over the window each hour; era5: one 25 km point").param("wind_use", "mean", "", "mean or gust")
            .param("wind_start", "2025-01-07T18:00:00Z", "", "").param("wind_end", "2025-01-12T00:00:00Z", "", "")
            .param("buildings_service", "https://services.arcgis.com/RmCCgQtiZLDCtblq/arcgis/rest/services/2025_WildFire_LARIAC_BUILDINGS_100M_Plus_DINS_MaxMin_DAMAGE_NEW/FeatureServer/0", "", "").param("buildings_where", "Incident_Name LIKE '%Palisades%'", "", "").param("buildings_key", "lacounty-lariac/2025-palisades", "", "")
            .param("reach_m", 90.0, "m", "guess: how far a burning cell's heat is felt").param("lambda_m", 20.0, "m", "guess: e-folding distance of that heat in still air on the flat")
            .param("wind_gain", 0.15, "1/(m/s)", "guess: the reach grows by this per m/s of wind in the downwind direction").param("slope_gain", 1.5, "", "guess: the reach grows by this times tan(slope) uphill")
            .param("moisture_gain", 10.0, "", "guess: ignition energy grows by this times the dead fuel moisture fraction").param("live_moisture_pct", 60.0, "%", "guess: chaparral in January after a dry autumn")
            .param("grass_ignite", 0.6, "heat", "guess").param("grass_intensity", 1.0, "heat/min", "guess").param("grass_burn_min", 5.0, "min", "guess")
            .param("shrub_ignite", 1.5, "heat", "guess").param("shrub_intensity", 2.5, "heat/min", "guess").param("shrub_burn_min", 20.0, "min", "guess")
            .param("tree_ignite", 2.0, "heat", "guess").param("tree_intensity", 2.0, "heat/min", "guess").param("tree_burn_min", 30.0, "min", "guess")
            .param("built_ignite", 4.0, "heat", "guess: a structure's threshold, scaled by nothing yet").param("built_intensity", 3.0, "heat/min", "guess, scaled by the cell's built fraction").param("built_burn_min", 90.0, "min", "guess, scaled by the built fraction")
            .param("cool_min", 15.0, "min", "guess: a cell's received heat decays with this e-folding time")
            .param("ignite_scale", 1.0, "", "one multiplier on every ignition threshold: the single number fitted to the record (the first infrared perimeter's acreage), declared as such; everything else is the test")
            .param("spot_rate", 0.003, "1/min", "guess: embers per burning cell per minute at 10 m/s, times (wind/10)^2").param("spot_mean_m", 25.0, "m per m/s", "guess: mean ember flight per m/s of wind").param("spot_land_p", 0.3, "", "guess: chance a landed ember lights dry fuel").param("spot_min_intensity", 2.0, "heat/min", "")
            .param("road_widths_m", "motorway=30,trunk=20,primary=14,secondary=10,tertiary=8", "", "roads this wide carry no fuel")
            .publish("ground.loaded", "cell counts by fuel kind, road cells, HRRR hours").publish("fire.ignition", "at a cell").publish("fire.spot", "at a cell: { from, distance_m }")
            .own("fire.burned_frac", "1")
            .frame("terrain_elev_m", "m").frame("building_height", "ft (as stated)").frame("fire_fuel_kind", "0 none 1 grass 2 shrub 3 tree 4 built").frame("fire_arrival_h", "h")
            .report("fire.burned_acres", "acres").report("fire.burned_km2", "km2").report("fire.active_cells", "cells").report("fire.ignited_cells_hour", "cells/h").report("fire.spots_hour", "1/h")
            .report("fire.wind_m_s", "m/s").report("fire.wind_from_deg", "deg").report("fire.dead_moisture_pct", "%").report("fire.built_cells_burned", "cells")
            .asset("copernicus-dem-glo30", "elevation, slope, aspect").asset("esa-worldcover-2021", "land cover").asset("lacounty-lariac-buildings-2025-fires", "building footprints and heights").asset("osm-overpass", "major roads as breaks; the map's context").asset("noaa-hrrr", "the 3 km wind, gust, temperature and humidity each hour").asset("open-meteo-era5-wind", "the one-point alternative")
            .view("scene", "The fire on the ground", "fire_arrival_h", "terrain, buildings and the fire's arrival, in three dimensions")
            .view("map", "Arrival time", "fire_arrival_h", "hours since ignition").view("map", "What each cell is", "fire_fuel_kind", "")
            .view("chart", "Burned area", "fire.burned_acres", "").view("chart", "Wind at the ignition point", "fire.wind_m_s", "HRRR 3 km").view("chart", "Spot fires", "fire.spots_hour", "")
            .note("Not Rothermel: a heat budget with guessed constants, built to be fitted against the mapped perimeters and the inspected structures. The HRRR reader was cross-checked against GDAL at the ignition point. Nothing physical about structures beyond a threshold, an intensity and a burn time.");
        if let Some(c) = ctx_table { d = d.table("context", c); }
        d
    }
}
