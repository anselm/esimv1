//! M4: weather from outside. Hourly ERA5 reanalysis (through Open-Meteo's archive API)
//! on a coarse grid of points over the window, interpolated to the run's grid and
//! published each hour as shared fields for the sun, soil and anything else that needs
//! air: wx.t2m_c, wx.precip_mm (this hour), wx.sw_w_m2, wx.rh_pct, wx.wind_m_s. The
//! module owns no conserved quantity; the water it reports is accounted by whoever
//! takes it from the atmosphere. Runs at priority -1 so readers at the same instant see
//! this hour's weather. Forecast ensembles arrive as a second source later.

use esim_kernel::manifest::{param_duration, param_f64};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module, Time};
use esim_registry::obs::{era5_hourly, WeatherGrid};
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
    missing_hours: u32,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] wx: Option<WeatherGrid>,
    #[serde(skip)] areas: Vec<f64>,
}

impl Weather {
    pub fn new(name: &str, p: &toml::Value) -> Result<Weather, String> {
        let mut w = Weather {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 42.3), lat_n: param_f64(p, "lat_n", 43.2),
            lon_w: param_f64(p, "lon_w", -94.2), lon_e: param_f64(p, "lon_e", -93.5),
            res_arcsec: param_f64(p, "res_arcsec", 15.0),
            point_step_deg: param_f64(p, "point_step_deg", 0.25),
            dt: param_duration(p, "dt", "1h")?,
            start: p.get("start").and_then(|v| v.as_str()).unwrap_or("2025-01-01").to_string(),
            end: p.get("end").and_then(|v| v.as_str()).unwrap_or("2025-12-31").to_string(),
            missing_hours: 0, grid: None, wx: None, areas: Vec::new(),
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
        let h = wx.hour_index(t.0 - 3600);
        let n = grid.nx * grid.ny;
        let (mut t2m, mut pr, mut sw, mut rh, mut wind) = (vec![f32::NAN; n], vec![0f32; n], vec![0f32; n], vec![f32::NAN; n], vec![f32::NAN; n]);
        match h {
            None => { self.missing_hours += 1; }
            Some(h) => {
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
