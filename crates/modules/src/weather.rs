//! M4: weather from outside. Hourly ERA5 reanalysis (through Open-Meteo's archive API)
//! on a coarse grid of points over the window, interpolated to the run's grid and
//! published each hour as shared fields for the sun, soil and anything else that needs
//! air: wx.t2m_c, wx.precip_mm (this hour), wx.sw_w_m2, wx.rh_pct, wx.wind_m_s. The
//! module owns no conserved quantity; the water it reports is accounted by whoever
//! takes it from the atmosphere. Runs at priority -1 so readers at the same instant see
//! this hour's weather. Forecast ensembles arrive as a second source later.

use esim_kernel::manifest::{param_duration, param_f64};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module, Time};
use esim_registry::obs::{era5_hourly, gefs_ensemble, WeatherGrid};
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Weather {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64,
    res_arcsec: f64,
    point_step_deg: f64,
    dt: Duration,
    start: String,
    end: String,
    /// "era5": reanalysis only. "chain": ERA5 where it exists, then a GEFS ensemble member
    /// chosen by the run seed, for the recent past and the forecast window.
    source: String,
    past_days: u32,
    forecast_days: u32,
    seed: u64,
    member: usize,
    missing_hours: u32,
    era5_hours: u32,
    member_hours: u32,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] wx: Option<WeatherGrid>,
    #[serde(skip)] fc: Option<WeatherGrid>,
    #[serde(skip)] areas: Vec<f64>,
}

impl Weather {
    pub fn new(name: &str, p: &toml::Value) -> Result<Weather, String> {
        let mut w = Weather {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 42.3), lat_n: param_f64(p, "lat_n", 43.2),
            lon_w: param_f64(p, "lon_w", -94.2), lon_e: param_f64(p, "lon_e", -93.5),
            res_arcsec: param_f64(p, "res_arcsec", 30.0),
            point_step_deg: param_f64(p, "point_step_deg", 0.25),
            dt: param_duration(p, "dt", "1h")?,
            start: p.get("start").and_then(|v| v.as_str()).unwrap_or("2025-01-01").to_string(),
            end: p.get("end").and_then(|v| v.as_str()).unwrap_or("2025-12-31").to_string(),
            source: p.get("source").and_then(|v| v.as_str()).unwrap_or("era5").to_string(),
            past_days: param_f64(p, "past_days", 10.0) as u32,
            forecast_days: param_f64(p, "forecast_days", 10.0) as u32,
            seed: p.get("seed").and_then(|v| v.as_integer()).unwrap_or(1) as u64,
            member: 0,
            missing_hours: 0, era5_hours: 0, member_hours: 0, grid: None, wx: None, fc: None, areas: Vec::new(),
        };
        w.load()?;
        Ok(w)
    }
    fn load(&mut self) -> Result<(), String> {
        let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let dem = cat.asset("copernicus-dem-glo30").ok_or("catalog has no copernicus-dem-glo30")?;
        let mut f = Fetcher::default_for(&root)?;
        let w = dem_window(&mut f, dem, self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let grid = LatLonGrid { nx: w.nx, ny: w.ny, dlon: w.dlon, dlat: w.dlat, lon_w: w.lon_w, lat_n: w.lat_n };
        self.areas = (0..grid.ny).flat_map(|r| std::iter::repeat_n(grid.row_area(r), grid.nx)).collect();
        self.wx = Some(era5_hourly(&mut f, self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.point_step_deg, &self.start, &self.end)?);
        if self.source == "chain" {
            let members = gefs_ensemble(&mut f, self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.point_step_deg, self.past_days, self.forecast_days)?;
            self.member = ((self.seed.max(1) - 1) as usize) % members.len();
            self.fc = members.into_iter().nth(self.member);
        }
        self.grid = Some(grid);
        Ok(())
    }
}

impl Module for Weather {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        let wx = self.wx.as_ref().unwrap();
        ctx.report("wx.points", "1", (wx.lats.len() * wx.lons.len()) as f64);
        ctx.schedule_with_priority(self.dt, -1, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" { return; }
        let grid = self.grid.clone().unwrap();
        let wx = self.wx.as_ref().unwrap();
        // ERA5 hourly values are labelled by the hour they end (accumulations) or the
        // instant (states); our step ending at `now` takes the record labelled now - 1h,
        // which Open-Meteo labels by the start of the hour.
        let t: Time = ctx.now();
        let n = grid.nx * grid.ny;
        let (mut t2m, mut pr, mut sw, mut rh, mut wind) = (vec![f32::NAN; n], vec![0f32; n], vec![0f32; n], vec![f32::NAN; n], vec![f32::NAN; n]);
        // ERA5 if it has a finite temperature for this hour, else the forecast member.
        let era_h = wx.hour_index(t.0 - 3600).filter(|&h| wx.t2m_c.first().and_then(|v| v.get(h)).map(|x| x.is_finite()).unwrap_or(false));
        let (src, h): (&WeatherGrid, Option<usize>) = match (era_h, &self.fc) {
            (Some(h), _) => { self.era5_hours += 1; (wx, Some(h)) }
            (None, Some(fc)) => { let h = fc.hour_index(t.0 - 3600).filter(|&h| fc.t2m_c.first().and_then(|v| v.get(h)).map(|x| x.is_finite()).unwrap_or(false)); if h.is_some() { self.member_hours += 1; } (fc, h) }
            (None, None) => (wx, None),
        };
        ctx.report("wx.source", "0=era5 1=member", if era_h.is_some() { 0.0 } else { 1.0 });
        match h {
            None => { self.missing_hours += 1; }
            Some(h) => {
                let wx = src;
                for row in 0..grid.ny {
                    let lat = grid.lat(row);
                    for col in 0..grid.nx {
                        let lon = grid.lon(col);
                        let i = row * grid.nx + col;
                        t2m[i] = wx.at(&wx.t2m_c, h, lat, lon);
                        let p = wx.at(&wx.precip_mm, h, lat, lon); pr[i] = if p.is_finite() { p.max(0.0) } else { 0.0 };
                        let s = wx.at(&wx.sw_w_m2, h, lat, lon); sw[i] = if s.is_finite() { s.max(0.0) } else { 0.0 };
                        rh[i] = wx.at(&wx.rh_pct, h, lat, lon);
                        wind[i] = wx.at(&wx.wind_m_s, h, lat, lon);
                    }
                }
            }
        }
        let area: f64 = self.areas.iter().sum();
        let mean = |v: &[f32]| v.iter().zip(&self.areas).filter(|(x, _)| x.is_finite()).map(|(x, a)| *x as f64 * a).sum::<f64>() / area;
        ctx.report("wx.sw_mean_w_m2", "W/m2", mean(&sw));
        ctx.report("wx.t2m_mean_c", "degC", mean(&t2m));
        ctx.report("wx.precip_mean_mm", "mm", mean(&pr));
        ctx.world_write("wx.t2m_c", "degC", &grid, t2m);
        ctx.world_write("wx.precip_mm", "mm", &grid, pr);
        ctx.world_write("wx.sw_w_m2", "W/m2", &grid, sw);
        ctx.world_write("wx.rh_pct", "%", &grid, rh);
        ctx.world_write("wx.wind_m_s", "m/s", &grid, wind);
        ctx.schedule_with_priority(self.dt, -1, "tick", json!(null));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) {
        let mut w: Weather = serde_json::from_value(v).unwrap();
        w.load().expect("weather reload on restore");
        *self = w;
    }
}

impl Weather {
    pub fn describe(name: &str, p: &toml::Value) -> esim_kernel::describe::Describe {
        let source = p.get("source").and_then(|v| v.as_str()).unwrap_or("era5");
        let d = esim_kernel::describe::Describe::new("weather.era5", name, "Observed weather as fields: ERA5 reanalysis hourly on a point grid, bilinear to the cells; with source = chain, ERA5 up to the archive's edge then one GEFS ensemble member chosen by the seed.")
            .priority(-1).dt(esim_kernel::manifest::param_str(p, "dt", "1h"))
            .param("lat_s", 42.3, "deg", "").param("lat_n", 43.2, "deg", "").param("lon_w", -94.2, "deg", "").param("lon_e", -93.5, "deg", "")
            .param("res_arcsec", 30.0, "arcsec", "").param("point_step_deg", 0.25, "deg", "spacing of the Open-Meteo points")
            .param("dt", "1h", "", "").param("start", "2025-01-01", "date", "").param("end", "2025-12-31", "date", "")
            .param("source", "era5", "", "era5 | chain").param("past_days", 10, "d", "chain: GEFS past days").param("forecast_days", 10, "d", "chain: GEFS horizon")
            .param("seed", 1, "", "folded in from the run seed; chain picks the GEFS member from it")
            .own("wx.t2m_c", "degC").own("wx.precip_mm", "mm").own("wx.sw_w_m2", "W/m2").own("wx.rh_pct", "%").own("wx.wind_m_s", "m/s")
            .report("wx.points", "1").report("wx.source", "0=era5 1=member")
            .report("wx.sw_mean_w_m2", "W/m2").report("wx.t2m_mean_c", "degC").report("wx.precip_mean_mm", "mm")
            .asset("copernicus-dem-glo30", "the cell grid")
            .asset("open-meteo-era5", "hourly archive; lags about five days");
        if source == "chain" { d.asset("open-meteo-gefs", "31-member ensemble, one member per branch") } else { d }
    }
}
