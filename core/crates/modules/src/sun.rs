//! M1: clear-sky solar irradiance over real terrain.
//!
//! Solar position by the NOAA (Spencer 1971) formulas, good to a fraction of a degree.
//! Air mass by Kasten and Young (1989). Clear-sky direct normal irradiance by the
//! Meinel formula DNI = E0 * 0.7^(AM^0.678), a simple broadband model; diffuse taken as
//! one tenth of DNI on the horizontal. Both are placeholders for a proper clear-sky
//! model (REST2 or Ineichen) once there is a radiation record to score against (M4).
//! Terrain enters through slope and aspect (Horn) for the incidence angle and through
//! horizon angles for shadowing. Sky-view factor and elevation correction are omitted.
//!
//! Energy ledger (joules), per step and per cell footprint of area A:
//!   top of atmosphere on the horizontal       E0 cos z A dt      from `sun`
//!   reaching the surface                      (DNI cos z + DHI) A dt   to `surface`
//!   terrain shadow (beam lands on a neighbour) DNI cos z A dt     to `terrain_shadow`
//!   the rest                                                      to `atmosphere`
//! `surface` is a declared stock (cumulative), so the close checks the module's own
//! accounting against the flows.

use esim_kernel::manifest::{param_duration, param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module, Time};
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

const LEDGER: &str = "energy";
const SOLAR_CONSTANT: f64 = 1361.0;

#[derive(Serialize, Deserialize)]
pub struct Sun {
    name: String,
    // configuration
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64,
    res_arcsec: f64,
    dt: Duration,
    report_every: Duration,
    n_az: usize,
    horizon_km: f64,
    transmittance: f64,
    diffuse_fraction: f64,
    // state
    absorbed_j: f64,
    day_sum: Vec<f32>,
    day_steps: u32,
    next_report: Option<Time>,
    // derived terrain, rebuilt from the cache on restore (not serialised)
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] areas: Vec<f64>,
    #[serde(skip)] slope: Vec<f32>,
    #[serde(skip)] aspect: Vec<f32>,
    #[serde(skip)] horizon: Vec<f32>,
    #[serde(skip)] provenance: String,
}

impl Sun {
    pub fn new(name: &str, p: &toml::Value) -> Result<Sun, String> {
        let mut s = Sun {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 42.3), lat_n: param_f64(p, "lat_n", 43.2),
            lon_w: param_f64(p, "lon_w", -94.2), lon_e: param_f64(p, "lon_e", -93.5),
            res_arcsec: param_f64(p, "res_arcsec", 30.0),
            dt: param_duration(p, "dt", "1h")?,
            report_every: param_duration(p, "report_every", "1d")?,
            n_az: param_f64(p, "horizon_azimuths", 16.0) as usize,
            horizon_km: param_f64(p, "horizon_km", 20.0),
            transmittance: param_f64(p, "transmittance", 0.7),
            diffuse_fraction: param_f64(p, "diffuse_fraction", 0.1),
            absorbed_j: 0.0, day_sum: Vec::new(), day_steps: 0, next_report: None,
            grid: None, areas: Vec::new(), slope: Vec::new(), aspect: Vec::new(), horizon: Vec::new(), provenance: String::new(),
        };
        let root = param_str(p, "data_root", "").to_string();
        s.load_terrain(if root.is_empty() { None } else { Some(PathBuf::from(root)) })?;
        Ok(s)
    }

    fn load_terrain(&mut self, root: Option<PathBuf>) -> Result<(), String> {
        let root = root.or_else(|| std::env::var("ESIM_ROOT").ok().map(PathBuf::from)).unwrap_or_else(|| PathBuf::from("."));
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let dem = cat.asset("copernicus-dem-glo30").ok_or("catalog has no copernicus-dem-glo30")?;
        let mut f = Fetcher::default_for(&root)?;
        let w = dem_window(&mut f, dem, self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let (slope, aspect) = w.slope_aspect(&f);
        let horizon = w.horizons(&f, self.n_az, self.horizon_km);
        let grid = LatLonGrid { nx: w.nx, ny: w.ny, dlon: w.dlon, dlat: w.dlat, lon_w: w.lon_w, lat_n: w.lat_n };
        self.areas = (0..grid.ny).flat_map(|r| std::iter::repeat_n(grid.row_area(r), grid.nx)).collect();
        if self.day_sum.len() != grid.nx * grid.ny { self.day_sum = vec![0.0; grid.nx * grid.ny]; }
        self.provenance = w.provenance.clone();
        self.grid = Some(grid); self.slope = slope; self.aspect = aspect; self.horizon = horizon;
        Ok(())
    }
}

/// Sun direction in the local east-north-up frame, and the top-of-atmosphere normal
/// irradiance, at UTC time `t` for a point. Returns (east, north, up, e0).
pub fn solar_position(t: Time, lat_deg: f64, lon_deg: f64) -> (f64, f64, f64, f64) {
    let secs = t.0.rem_euclid(86_400) as f64;
    let days = t.0.div_euclid(86_400);
    // day of year from civil date
    let (y, m, d) = civil(days);
    let doy = day_of_year(y, m, d) as f64;
    let hour = secs / 3600.0;
    let gamma = std::f64::consts::TAU / 365.0 * (doy - 1.0 + (hour - 12.0) / 24.0);
    let eqtime = 229.18 * (0.000075 + 0.001868 * gamma.cos() - 0.032077 * gamma.sin() - 0.014615 * (2.0 * gamma).cos() - 0.040849 * (2.0 * gamma).sin());
    let decl = 0.006918 - 0.399912 * gamma.cos() + 0.070257 * gamma.sin() - 0.006758 * (2.0 * gamma).cos() + 0.000907 * (2.0 * gamma).sin() - 0.002697 * (3.0 * gamma).cos() + 0.00148 * (3.0 * gamma).sin();
    let tst = secs / 60.0 + eqtime + 4.0 * lon_deg; // true solar time, minutes
    let ha = (tst / 4.0 - 180.0).to_radians();
    let (phi, dl) = (lat_deg.to_radians(), decl);
    let east = -dl.cos() * ha.sin();
    let north = dl.sin() * phi.cos() - dl.cos() * phi.sin() * ha.cos();
    let up = dl.sin() * phi.sin() + dl.cos() * phi.cos() * ha.cos();
    let e0 = SOLAR_CONSTANT * (1.0 + 0.033 * (std::f64::consts::TAU * doy / 365.0).cos());
    (east, north, up, e0)
}

fn civil(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}
fn day_of_year(y: i64, m: i64, d: i64) -> i64 {
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let cum = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    cum[(m - 1) as usize] + d + if leap && m > 2 { 1 } else { 0 }
}

/// Kasten and Young 1989 relative air mass from the zenith angle in degrees.
pub fn air_mass(zenith_deg: f64) -> f64 {
    let cz = zenith_deg.to_radians().cos();
    1.0 / (cz + 0.50572 * (96.07995 - zenith_deg).powf(-1.6364))
}

impl Module for Sun {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.ledger(LEDGER, "J", 1e6); // 1 MJ against ~1e17 J per basin-day: 1e-11 relative
        ctx.declare_stock(LEDGER, "surface", self.absorbed_j);
        let grid = self.grid.as_ref().unwrap();
        ctx.field("elevation", "m", grid, {
            // re-read the cached window for the first frame: elevation is a base layer, shown once
            let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
            let cat = Catalog::load(&root.join("data/catalog.toml")).unwrap();
            let mut f = Fetcher::default_for(&root).unwrap();
            dem_window(&mut f, cat.asset("copernicus-dem-glo30").unwrap(), self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec).unwrap().elev
        });
        ctx.field("slope", "rad", grid, self.slope.clone());
        self.next_report = Some(ctx.now().add(self.report_every));
        ctx.schedule(self.dt, "tick", json!(null));
    }

    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" { return; }
        let grid = self.grid.as_ref().unwrap().clone();
        let dt = self.dt.as_secs_f64();
        // position at the step's midpoint for the domain centre's longitude; per-cell lat/lon below
        let t_mid = Time(ctx.now().0 - (self.dt.0 / 2));
        let (mut toa, mut surf, mut shadow, mut atm) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        let mut ghi_sum = 0.0f64;
        let n_az = self.n_az;
        let mut incident = vec![0f32; grid.nx * grid.ny];
        for row in 0..grid.ny {
            let lat = grid.lat(row);
            for col in 0..grid.nx {
                let i = row * grid.nx + col;
                let lon = grid.lon(col);
                let (e, n, up, e0) = solar_position(t_mid, lat, lon);
                if up <= 0.0 { continue; }
                let a = self.areas[i];
                let cosz = up;
                let z_deg = cosz.acos().to_degrees();
                let dni = e0 * self.transmittance.powf(air_mass(z_deg).powf(0.678));
                let dhi = self.diffuse_fraction * dni;
                // shadow test: solar elevation vs horizon angle at the sun's azimuth
                let mut az = e.atan2(n); if az < 0.0 { az += std::f64::consts::TAU; }
                let k = ((az / std::f64::consts::TAU) * n_az as f64).round() as usize % n_az;
                let elev_angle = (std::f64::consts::FRAC_PI_2 - cosz.acos()) as f32;
                let shadowed = elev_angle < self.horizon[i * n_az + k];
                // irradiance felt by the sloped surface (diagnostic field)
                let s = self.slope[i] as f64;
                let asp = self.aspect[i];
                let cos_inc = if asp.is_nan() { cosz } else {
                    (cosz * s.cos() + (1.0 - cosz * cosz).max(0.0).sqrt() * s.sin() * (az - asp as f64).cos()).max(0.0)
                };
                let slope_irr = if shadowed { dhi } else { dni * cos_inc + dhi };
                self.day_sum[i] += slope_irr as f32;
                // ledger on the footprint
                let toa_c = e0 * cosz * a * dt;
                let direct_c = dni * cosz * a * dt;
                let diffuse_c = dhi * a * dt;
                toa += toa_c;
                let surf_c = if shadowed { diffuse_c } else { direct_c + diffuse_c };
                // Publish f32; account the f32 value, so readers and the ledger see the same joules.
                incident[i] = surf_c as f32;
                let surf_c = incident[i] as f64;
                if shadowed { shadow += direct_c; }
                surf += surf_c;
                atm += toa_c - surf_c - if shadowed { direct_c } else { 0.0 };
                ghi_sum += (if shadowed { dhi } else { dni * cosz + dhi }) * a;
            }
        }
        self.day_steps += 1;
        // Shared state for whoever heats, grows or evaporates with this energy: joules
        // per cell footprint over this step. Summed in f64 above for the ledger; the
        // f32 field is what readers get, so readers must account in f32-consistent sums.
        ctx.world_write("sun.incident_j", "J", &grid, incident);
        if toa > 0.0 {
            ctx.transfer(LEDGER, "sun", "surface", surf);
            ctx.transfer(LEDGER, "sun", "terrain_shadow", shadow);
            ctx.transfer(LEDGER, "sun", "atmosphere", atm);
            self.absorbed_j += surf;
        }
        ctx.declare_stock(LEDGER, "surface", self.absorbed_j);
        let total_area: f64 = self.areas.iter().sum();
        ctx.report("sun.ghi_mean_w_m2", "W/m2", ghi_sum / total_area);
        if let Some(nr) = self.next_report {
            if ctx.now() >= nr {
                let steps = self.day_steps.max(1) as f32;
                let frame: Vec<f32> = self.day_sum.iter().map(|v| v / steps).collect();
                let mean = frame.iter().map(|&v| v as f64).sum::<f64>() / frame.len() as f64;
                ctx.field("irradiance_daily_mean", "W/m2", &grid, frame);
                ctx.report("sun.irradiance_daily_mean_w_m2", "W/m2", mean);
                self.day_sum.iter_mut().for_each(|v| *v = 0.0);
                self.day_steps = 0;
                self.next_report = Some(nr.add(self.report_every));
            }
        }
        ctx.schedule(self.dt, "tick", json!(null));
    }

    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) {
        let mut s: Sun = serde_json::from_value(v).unwrap();
        s.load_terrain(None).expect("terrain reload on restore");
        *self = s;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn noon_zenith_on_the_solstice_at_the_gauge() {
        // 2025-06-21, solar noon at 93.806 W is about 18:15 UTC. Zenith should be near
        // latitude minus declination: 42.43 - 23.44 = 19.0 degrees.
        let t = Time::parse("2025-06-21T18:15:00Z").unwrap();
        let (_, _, up, _) = solar_position(t, 42.4319722, -93.8059444);
        let z = up.acos().to_degrees();
        assert!((z - 19.0).abs() < 0.6, "zenith {z}");
    }
    #[test]
    fn midnight_is_dark() {
        let t = Time::parse("2025-06-21T06:00:00Z").unwrap();
        let (_, _, up, _) = solar_position(t, 42.43, -93.81);
        assert!(up < 0.0);
    }
    #[test]
    fn air_mass_is_one_overhead_and_about_38_at_the_horizon() {
        assert!((air_mass(0.0) - 1.0).abs() < 0.01);
        let h = air_mass(90.0);
        assert!(h > 30.0 && h < 45.0, "{h}");
    }
}

impl Sun {
    pub fn describe(name: &str, p: &toml::Value) -> esim_kernel::describe::Describe {
        esim_kernel::describe::Describe::new("sun.irradiance", name, "Clear-sky solar irradiance on real terrain every tick: sun position, air mass, slope incidence and horizon shadow; absorbed energy through the energy ledger.")
            .priority(0).dt(param_str(p, "dt", "1h"))
            .param("lat_s", 42.3, "deg", "").param("lat_n", 43.2, "deg", "").param("lon_w", -94.2, "deg", "").param("lon_e", -93.5, "deg", "")
            .param("res_arcsec", 30.0, "arcsec", "grid cell; the DEM is aggregated to it; every module in a manifest must use one window and resolution, so state them once in [place]")
            .param("dt", "1h", "", "").param("report_every", "1d", "", "")
            .param("horizon_azimuths", 16, "", "").param("horizon_km", 20.0, "km", "how far the horizon search looks")
            .param("transmittance", 0.7, "", "Meinel clear-sky placeholder; winter clearness above one exposed it (devlog 20261101-m4-weather)")
            .param("diffuse_fraction", 0.1, "", "placeholder")
            .param("data_root", "", "", "overrides ESIM_ROOT")
            .own("sun.incident_j", "J")
            .ledger("energy", "J", 1e6)
            .frame("elevation", "m").frame("slope", "rad").frame("irradiance_daily_mean", "W/m2")
            .report("sun.ghi_mean_w_m2", "W/m2").report("sun.irradiance_daily_mean_w_m2", "W/m2")
            .asset("copernicus-dem-glo30", "terrain window, slope, aspect and horizons")
            .note("Parameter provenance: devlog/20261101-m1-sun.md")
    }
}
