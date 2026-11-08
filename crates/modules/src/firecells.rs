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
use esim_registry::wildfire::{building_footprints, cached_context, context, hrrr_hour, landcover_window, osm_buildings, wind_hourly, HrrrHour, WindSeries};
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
    wind_at: String,         // the simulated instant wind_start stands for; empty: wind_start itself
    buildings: String,       // lariac | osm | none
    buildings_service: String, buildings_where: String, buildings_key: String,
    // the guesses
    reach_m: f64, lambda_m: f64, wind_gain: f64, slope_gain: f64, slope_rate_gain: f64, downslope_gain: f64,
    form: String,            // reach | vector
    cooling: String,         // always | idle
    wind_phi: f64, wind_phi_exp: f64, phi_cap: f64, waf: f64,
    moisture_gain: f64, live_moisture_pct: f64,
    grass_ignite: f64, grass_intensity: f64, grass_burn_min: f64,
    shrub_ignite: f64, shrub_intensity: f64, shrub_burn_min: f64,
    tree_ignite: f64, tree_intensity: f64, tree_burn_min: f64,
    built_ignite: f64, built_intensity: f64, built_burn_min: f64,
    cool_min: f64,
    ignite_scale: f64,
    threshold_jitter: f64,
    ember_built_p: f64,
    built_to_built_gain: f64,
    spot_rate: f64, spot_mean_m: f64, spot_land_p: f64, spot_min_intensity: f64,
    road_widths_m: String,
    // structures as bodies: every footprint its own thing to ignite and burn
    structures: String,          // on | off
    structure_ignite: f64, structure_intensity: f64, structure_burn_min: f64, structure_jitter: f64,
    house_reach_m: f64, house_lambda_m: f64, exposure_m: f64, exposure_lambda_m: f64, ember_accept: f64,
    structure_frame_every: Duration,
    // state
    ignited: bool,
    s_t_ign: Vec<f32>, s_t_out: Vec<f32>, s_heat: Vec<f32>, s_heat_t: Vec<f32>, s_active: Vec<u32>,
    last_s_frame: i64,
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
    #[serde(skip)] held: Vec<bool>,      // under a ground.break (fire.suppression's lines), from its time on
    held_cells: usize, breaks_taken: usize,
    #[serde(skip)] kind: Vec<u8>,          // 0 none, 1 grass, 2 shrub, 3 tree, 4 built
    #[serde(skip)] area_m2: Vec<f64>,
    #[serde(skip)] metres: Vec<(f64, f64)>,
    #[serde(skip)] hrrr: Vec<Option<HrrrHour>>,  // per hour from wind_start
    #[serde(skip)] hrrr_t0: i64,
    #[serde(skip)] wind_shift: i64,              // record time minus simulated time, s
    #[serde(skip)] era5: Option<WindSeries>,
    #[serde(skip)] cell_wind: Vec<(f32, f32)>,   // this hour: (speed m/s, direction the wind blows TOWARD, rad)
    #[serde(skip)] cell_moist: Vec<f32>,
    #[serde(skip)] jitter: Vec<f32>,        // per-cell multiplier on the ignition threshold: the micro-conditions nobody mapped
    #[serde(skip)] offsets: Vec<(isize, isize, f32, f32)>, // dc, dr, distance factor (cells), bearing rad
    // the bodies
    #[serde(skip)] s_cell: Vec<usize>,                 // the cell under each structure's centroid
    #[serde(skip)] s_area: Vec<f32>,                   // footprint m2
    #[serde(skip)] s_fate: Vec<u8>,                    // the inspectors' word: 0 unknown, 1 no damage, 2 affected to major, 3 destroyed
    #[serde(skip)] s_jitter: Vec<f32>,
    #[serde(skip)] s_neigh: Vec<Vec<(u32, f32, f32)>>, // neighbour structure, gap between footprints m, bearing from this one rad
    #[serde(skip)] cell_structs: std::collections::HashMap<usize, Vec<(u32, f32)>>, // cells whose fire a structure feels, with the distance
    #[serde(skip)] cell_home: std::collections::HashMap<usize, Vec<u32>>,           // structures standing in a cell
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
            wind_at: param_str(p, "wind_at", "").to_string(),
            buildings: param_str(p, "buildings", "lariac").to_string(),
            buildings_service: param_str(p, "buildings_service", "https://services.arcgis.com/RmCCgQtiZLDCtblq/arcgis/rest/services/2025_WildFire_LARIAC_BUILDINGS_100M_Plus_DINS_MaxMin_DAMAGE_NEW/FeatureServer/0").to_string(),
            buildings_where: param_str(p, "buildings_where", "Incident_Name LIKE '%Palisades%'").to_string(),
            buildings_key: param_str(p, "buildings_key", "lacounty-lariac/2025-palisades").to_string(),
            reach_m: param_f64(p, "reach_m", 90.0), lambda_m: param_f64(p, "lambda_m", 20.0), wind_gain: param_f64(p, "wind_gain", 0.15), slope_gain: param_f64(p, "slope_gain", 1.5), slope_rate_gain: param_f64(p, "slope_rate_gain", 0.0), downslope_gain: param_f64(p, "downslope_gain", -1.0),
            form: param_str(p, "form", "reach").to_string(), cooling: param_str(p, "cooling", "always").to_string(), wind_phi: param_f64(p, "wind_phi", 1.9), wind_phi_exp: param_f64(p, "wind_phi_exp", 0.9), phi_cap: param_f64(p, "phi_cap", 30.0), waf: param_f64(p, "waf", 0.4),
            moisture_gain: param_f64(p, "moisture_gain", 10.0), live_moisture_pct: param_f64(p, "live_moisture_pct", 60.0),
            grass_ignite: param_f64(p, "grass_ignite", 0.6), grass_intensity: param_f64(p, "grass_intensity", 1.0), grass_burn_min: param_f64(p, "grass_burn_min", 5.0),
            shrub_ignite: param_f64(p, "shrub_ignite", 1.5), shrub_intensity: param_f64(p, "shrub_intensity", 2.5), shrub_burn_min: param_f64(p, "shrub_burn_min", 20.0),
            tree_ignite: param_f64(p, "tree_ignite", 2.0), tree_intensity: param_f64(p, "tree_intensity", 2.0), tree_burn_min: param_f64(p, "tree_burn_min", 30.0),
            built_ignite: param_f64(p, "built_ignite", 4.0), built_intensity: param_f64(p, "built_intensity", 3.0), built_burn_min: param_f64(p, "built_burn_min", 90.0),
            cool_min: param_f64(p, "cool_min", 15.0),
            ignite_scale: param_f64(p, "ignite_scale", 1.0),
            threshold_jitter: param_f64(p, "threshold_jitter", 0.0),
            ember_built_p: param_f64(p, "ember_built_p", 0.0),
            built_to_built_gain: param_f64(p, "built_to_built_gain", 1.0),
            spot_rate: param_f64(p, "spot_rate", 0.003), spot_mean_m: param_f64(p, "spot_mean_m", 25.0), spot_land_p: param_f64(p, "spot_land_p", 0.3), spot_min_intensity: param_f64(p, "spot_min_intensity", 2.0),
            road_widths_m: param_str(p, "road_widths_m", "motorway=30,trunk=20,primary=14,secondary=10,tertiary=8").to_string(),
            structures: param_str(p, "structures", "off").to_string(),
            structure_ignite: param_f64(p, "structure_ignite", 3.0), structure_intensity: param_f64(p, "structure_intensity", 3.0), structure_burn_min: param_f64(p, "structure_burn_min", 60.0), structure_jitter: param_f64(p, "structure_jitter", 0.3),
            house_reach_m: param_f64(p, "house_reach_m", 40.0), house_lambda_m: param_f64(p, "house_lambda_m", 8.0), exposure_m: param_f64(p, "exposure_m", 35.0), exposure_lambda_m: param_f64(p, "exposure_lambda_m", 12.0), ember_accept: param_f64(p, "ember_accept", 0.25),
            structure_frame_every: param_duration(p, "structure_frame_every", "6h")?,
            s_t_ign: Vec::new(), s_t_out: Vec::new(), s_heat: Vec::new(), s_heat_t: Vec::new(), s_active: Vec::new(), last_s_frame: i64::MIN / 4,
            ignited: false, since_ignition_s: 0.0, last_frame: i64::MIN / 4,
            t_ign: Vec::new(), t_out: Vec::new(), heat: Vec::new(), heat_t: Vec::new(), active: Vec::new(), spots: 0, held_cells: 0, breaks_taken: 0,
            grid: None, frame_grid: None, elev: Vec::new(), slope: Vec::new(), aspect: Vec::new(), cover: Vec::new(), built_frac: Vec::new(), built_height: Vec::new(), road: Vec::new(), held: Vec::new(), kind: Vec::new(),
            area_m2: Vec::new(), metres: Vec::new(), hrrr: Vec::new(), hrrr_t0: 0, wind_shift: 0, era5: None, cell_wind: Vec::new(), cell_moist: Vec::new(), jitter: Vec::new(), offsets: Vec::new(),
            s_cell: Vec::new(), s_area: Vec::new(), s_fate: Vec::new(), s_jitter: Vec::new(), s_neigh: Vec::new(), cell_structs: Default::default(), cell_home: Default::default(),
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
        let mut bodies_src: Option<Vec<esim_registry::wildfire::Building>> = None;
        let fetched = match self.buildings.as_str() {
            "osm" => osm_buildings(&mut fe, self.lat_s, self.lat_n, self.lon_w, self.lon_e),
            "none" => Err("no building source asked for".to_string()),
            _ => building_footprints(&mut fe, &self.buildings_service, &self.buildings_key, &self.buildings_where),
        };
        match fetched {
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
                bodies_src = Some(bs);
            }
            Err(e) => eprintln!("buildings: {e}; built-up land from land cover only"),
        }
        // roads as breaks: the major roads of the window, buffered by class
        self.road = vec![false; n]; self.held = vec![false; n];
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
        if self.structures == "on" { if let Some(bs) = &bodies_src { self.build_bodies(bs, &grid); } }
        // the wind: HRRR hours over the window, or the ERA5 point series
        let t0 = Time::parse(&self.wind_start)?.0; let t1 = Time::parse(&self.wind_end)?.0;
        // a borrowed wind: the record's hours from wind_start are played from wind_at in the run
        let t_at = if self.wind_at.trim().is_empty() { t0 } else { Time::parse(&self.wind_at)?.0 };
        self.hrrr_t0 = t_at;
        self.wind_shift = t0 - t_at;
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
        // micro-conditions: a fixed log-normal factor per cell from a hash of its index, so a run is reproducible and a
        // neighbour can be easier or harder to light than its twin; sigma is the knob
        self.jitter = (0..n).map(|k| {
            if self.threshold_jitter <= 0.0 { return 1.0; }
            let mut x = (k as u64).wrapping_mul(0x9E3779B97F4A7C15) ^ 0xD1B54A32D192ED03; x ^= x >> 29; x = x.wrapping_mul(0xBF58476D1CE4E5B9); x ^= x >> 32;
            let u1 = ((x & 0xFFFF_FFFF) as f64 + 0.5) / 4294967296.0; let u2 = (((x >> 32) & 0xFFFF_FFFF) as f64 + 0.5) / 4294967296.0;
            let z = (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos();
            (self.threshold_jitter * z).exp() as f32
        }).collect();
        let k = (self.frame_res_arcsec / self.res_arcsec).round().max(1.0) as usize;
        self.frame_grid = Some(LatLonGrid { nx: grid.nx.div_ceil(k), ny: grid.ny.div_ceil(k), dlon: grid.dlon * k as f64, dlat: grid.dlat * k as f64, lon_w: grid.lon_w, lat_n: grid.lat_n });
        self.slope = slope; self.aspect = aspect; self.grid = Some(grid);
        Ok(())
    }

    /// Every footprint becomes a body: a centroid, an area, the cell it stands in, the inspectors'
    /// word on its fate, a fixed vulnerability factor, its neighbours within `house_reach_m` (with the
    /// gap between footprints, not between centres), and the wildland cells within `exposure_m`
    /// whose fire it feels.
    fn build_bodies(&mut self, bs: &[esim_registry::wildfire::Building], grid: &LatLonGrid) {
        let mut cent: Vec<(f64, f64)> = Vec::new();
        self.s_cell.clear(); self.s_area.clear(); self.s_fate.clear();
        for b in bs {
            if b.ring.len() < 3 { continue; }
            let (cx, cy) = (b.ring.iter().map(|p| p.0).sum::<f64>() / b.ring.len() as f64, b.ring.iter().map(|p| p.1).sum::<f64>() / b.ring.len() as f64);
            if cy > grid.lat_n || cy < grid.lat_s() || cx < grid.lon_w || cx > grid.lon_e() { continue; }
            let r = (((grid.lat_n - cy) / grid.dlat) as usize).min(grid.ny - 1); let c = (((cx - grid.lon_w) / grid.dlon) as usize).min(grid.nx - 1);
            let (mx, my) = (self.metres[r].0 / grid.dlon, self.metres[r].1 / grid.dlat); // metres per degree here
            let mut a = 0.0; for i in 0..b.ring.len() { let (x0, y0) = b.ring[i]; let (x1, y1) = b.ring[(i + 1) % b.ring.len()]; a += (x0 * mx) * (y1 * my) - (x1 * mx) * (y0 * my); }
            let area = (a.abs() / 2.0) as f32;
            if !(area > 5.0) { continue; }
            let fate = if b.damage.starts_with("Destroyed") { 3 } else if b.damage.starts_with("No Damage") { 1 } else if b.damage.is_empty() { 0 } else { 2 };
            cent.push((cx, cy)); self.s_cell.push(r * grid.nx + c); self.s_area.push(area); self.s_fate.push(fate);
        }
        let m = cent.len();
        self.s_jitter = (0..m).map(|i| {
            if self.structure_jitter <= 0.0 { return 1.0; }
            let mut x = (i as u64 + 7_919).wrapping_mul(0x9E3779B97F4A7C15) ^ 0xD1B54A32D192ED03; x ^= x >> 29; x = x.wrapping_mul(0xBF58476D1CE4E5B9); x ^= x >> 32;
            let u1 = ((x & 0xFFFF_FFFF) as f64 + 0.5) / 4294967296.0; let u2 = (((x >> 32) & 0xFFFF_FFFF) as f64 + 0.5) / 4294967296.0;
            ((self.structure_jitter * (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()).exp()) as f32
        }).collect();
        if self.s_t_ign.len() != m { self.s_t_ign = vec![f32::NAN; m]; self.s_t_out = vec![0.0; m]; self.s_heat = vec![0.0; m]; self.s_heat_t = vec![0.0; m]; self.s_active.clear(); }
        // a bucket grid in cells for the neighbour search
        self.cell_home.clear();
        for (i, &k) in self.s_cell.iter().enumerate() { self.cell_home.entry(k).or_default().push(i as u32); }
        let (mx0, my0) = self.metres[grid.ny / 2];
        let reach_c = (self.house_reach_m / mx0).ceil() as isize + 1; let reach_r = (self.house_reach_m / my0).ceil() as isize + 1;
        self.s_neigh = vec![Vec::new(); m];
        for i in 0..m {
            let (r, c) = ((self.s_cell[i] / grid.nx) as isize, (self.s_cell[i] % grid.nx) as isize);
            let (mx, my) = (self.metres[r as usize].0 / grid.dlon, self.metres[r as usize].1 / grid.dlat);
            let half_i = (self.s_area[i] as f64).sqrt() / 2.0;
            for dr in -reach_r..=reach_r { for dc in -reach_c..=reach_c {
                let (rr, cc) = (r + dr, c + dc);
                if rr < 0 || cc < 0 || rr >= grid.ny as isize || cc >= grid.nx as isize { continue; }
                let Some(list) = self.cell_home.get(&(rr as usize * grid.nx + cc as usize)) else { continue };
                for &j in list {
                    let j = j as usize; if j == i { continue; }
                    let (ex, ny_m) = ((cent[j].0 - cent[i].0) * mx, (cent[j].1 - cent[i].1) * my);
                    let d = (ex * ex + ny_m * ny_m).sqrt();
                    let gap = (d - half_i - (self.s_area[j] as f64).sqrt() / 2.0).max(0.0);
                    if gap <= self.house_reach_m { self.s_neigh[i].push((j as u32, gap as f32, ex.atan2(ny_m) as f32)); }
                }
            } }
        }
        // the cells a structure feels
        self.cell_structs.clear();
        let exp_c = (self.exposure_m / mx0).ceil() as isize; let exp_r = (self.exposure_m / my0).ceil() as isize;
        for i in 0..m {
            let (r, c) = ((self.s_cell[i] / grid.nx) as isize, (self.s_cell[i] % grid.nx) as isize);
            let (mx, my) = (self.metres[r as usize].0 / grid.dlon, self.metres[r as usize].1 / grid.dlat);
            let half_i = (self.s_area[i] as f64).sqrt() / 2.0;
            for dr in -exp_r..=exp_r { for dc in -exp_c..=exp_c {
                let (rr, cc) = (r + dr, c + dc);
                if rr < 0 || cc < 0 || rr >= grid.ny as isize || cc >= grid.nx as isize { continue; }
                let k = rr as usize * grid.nx + cc as usize;
                if self.kind[k] == 0 { continue; }
                let (ex, ny_m) = ((grid.lon(cc as usize) - cent[i].0) * mx, (grid.lat(rr as usize) - cent[i].1) * my);
                let d = ((ex * ex + ny_m * ny_m).sqrt() - half_i).max(0.0);
                if d <= self.exposure_m { self.cell_structs.entry(k).or_default().push((i as u32, d as f32)); }
            } }
        }
        let pairs: usize = self.cell_structs.values().map(|v| v.len()).sum();
        eprintln!("bodies: {} structures ({} destroyed, {} undamaged by the inspectors), {} neighbour links, {} cell exposures", m, self.s_fate.iter().filter(|&&f| f == 3).count(), self.s_fate.iter().filter(|&&f| f == 1).count(), self.s_neigh.iter().map(|v| v.len()).sum::<usize>(), pairs);
    }
    fn ignite_struct(&mut self, i: usize, t: f32) {
        self.s_t_ign[i] = t; self.s_t_out[i] = t + self.structure_burn_min as f32 * 60.0; self.s_active.push(i as u32);
        // the ground it stands on burns with it, so the cell engine carries the fire on
        let k = self.s_cell[i];
        if self.t_ign[k].is_nan() && self.kind[k] != 0 { self.ignite(k, t); }
    }
    /// A one-arc-second frame holding, per cell, the structures standing in it reduced by `f`.
    fn struct_frame(&self, f: impl Fn(&[u32]) -> f32) -> Vec<f32> {
        let n = self.t_ign.len();
        let mut out = vec![f32::NAN; n];
        for (&k, list) in &self.cell_home { out[k] = f(list); }
        out
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
            let Some(i) = w.index_at(now.0 + self.wind_shift) else { return };
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
    /// A ground.break from an interventions module (fire.suppression): the cells it names take no heat
    /// and no ember from now on. The engine knows nothing of the record, only of breaks.
    fn take_break(&mut self, ev: &Event) {
        let Some(cells) = ev.data.get("cells").and_then(|v| v.as_array()) else { return };
        let strength = ev.data.get("strength").and_then(|v| v.as_f64()).unwrap_or(1.0);
        if strength <= 0.0 { return; }
        let mut taken = 0usize;
        for c in cells {
            let Some(k) = c.as_u64().map(|x| x as usize) else { continue };
            if k < self.held.len() && !self.held[k] { self.held[k] = true; taken += 1; }
        }
        if taken > 0 { self.breaks_taken += 1; self.held_cells += taken; }
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
        ctx.publish("ground.loaded", json!({ "cells": n, "wind_start": self.wind_start, "wind_at": if self.wind_at.trim().is_empty() { self.wind_start.clone() } else { self.wind_at.clone() }, "grass": counts(1), "shrub": counts(2), "tree": counts(3), "built": counts(4), "road_cells": self.road.iter().filter(|&&r| r).count(), "hrrr_hours": self.hrrr.iter().filter(|h| h.is_some()).count(), "wind": self.wind }));
        ctx.field("terrain_elev_m", "m", &g, self.elev.clone());
        ctx.field("building_height", "ft (as stated)", &g, self.built_height.clone());
        let fg = self.frame_grid.clone().unwrap();
        let kinds: Vec<f32> = self.kind.iter().map(|&k| k as f32).collect();
        ctx.field("fire_fuel_kind", "0 none 1 grass 2 shrub 3 tree 4 built", &fg, self.to_frame(&kinds, |b| { let mut best = (0.0f32, 0usize); for &x in b { let c = b.iter().filter(|&&y| y == x).count(); if c > best.1 { best = (x, c); } } best.0 }));
        ctx.world_write("fire.burned_frac", "1", &g, vec![0.0; n]);
        if self.structures == "on" {
            ctx.publish("bodies.loaded", json!({ "structures": self.s_cell.len(), "destroyed": self.s_fate.iter().filter(|&&f| f == 3).count(), "undamaged": self.s_fate.iter().filter(|&&f| f == 1).count(), "neighbour_links": self.s_neigh.iter().map(|v| v.len()).sum::<usize>() }));
            ctx.field("building_fate", "0 unknown 1 no damage 2 damaged 3 destroyed (the inspectors)", &g, self.struct_frame(|l| l.iter().map(|&i| self.s_fate[i as usize]).max().unwrap_or(0) as f32));
            ctx.field("building_arrival_h", "h", &g, vec![f32::NAN; n]);
        }
        ctx.subscribe("ground.break");
        ctx.schedule_with_priority(self.dt, 6, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic == "ground.break" { self.take_break(ev); return; }
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
            let vector = self.form == "vector";
            let idle = self.cooling == "idle";
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
                        if !self.t_ign[n].is_nan() || self.kind[n] == 0 || self.held[n] { continue; }
                        let asp = self.aspect[n];
                        let mut lambda = self.lambda_m as f32;
                        let mut sfac = 1.0f32;
                        if vector {
                            // the vector form: heat is I f(d) times a directional rate factor with Rothermel's shape.
                            // Wind (downwind) and slope (uphill, the receiving cell's) add as vectors, the head rate
                            // is 1 + |phi|, and the rate in every other direction follows Anderson's ellipse from
                            // the head, so the backing rate is the ellipse's, not a division
                            let phi_w = if wspd > 0.0 { self.wind_phi as f32 * wspd.powf(self.wind_phi_exp as f32) } else { 0.0 };
                            let (mut vx, mut vy) = (phi_w * wto.sin(), phi_w * wto.cos());
                            if asp.is_finite() && self.slope_rate_gain > 0.0 {
                                let phi_s = self.slope_rate_gain as f32 * self.slope[n].tan().powi(2);
                                let up = asp + std::f32::consts::PI;
                                vx += phi_s * up.sin(); vy += phi_s * up.cos();
                            }
                            let phi = (vx * vx + vy * vy).sqrt().min(self.phi_cap as f32);
                            if phi > 1e-6 {
                                let head = vx.atan2(vy);
                                // the ellipse's eccentricity from the wind that would give this factor alone
                                let u_eff = (phi / self.wind_phi as f32).powf(1.0 / self.wind_phi_exp as f32);
                                let lb = crate::wildfire::length_to_breadth((u_eff * self.waf as f32) as f64) as f32;
                                let ecc = (1.0 - 1.0 / (lb * lb)).max(0.0).sqrt();
                                sfac = (1.0 + phi) * (1.0 - ecc) / (1.0 - ecc * (theta - head).cos());
                            }
                        } else {
                            // the reach form: longer downwind and uphill
                            let along = (theta - wto).cos().max(0.0);
                            lambda *= 1.0 + self.wind_gain as f32 * wspd * along;
                            // slope on the rate itself: Rothermel's shape, a gain times tan²(slope), uphill multiplies
                            // the heat delivered and downhill divides it (the downhill half is a guess)
                            if asp.is_finite() {
                                let c = (theta - (asp + std::f32::consts::PI)).cos();
                                lambda *= 1.0 + self.slope_gain as f32 * self.slope[n].tan() * c.max(0.0);
                                if self.slope_rate_gain > 0.0 {
                                    let t2 = self.slope[n].tan().powi(2);
                                    let down = if self.downslope_gain < 0.0 { self.slope_rate_gain } else { self.downslope_gain } as f32;
                                    sfac = if c >= 0.0 { 1.0 + self.slope_rate_gain as f32 * t2 * c } else { 1.0 / (1.0 + down * t2 * -c) };
                                }
                            }
                        }
                        if self.kind[b] == 4 && self.kind[n] == 4 { lambda *= self.built_to_built_gain as f32; }
                        let q = ib * (-d / lambda).exp() * sfac * dt / 60.0;
                        // the cell cools between contributions
                        let elapsed = t - self.heat_t[n];
                        if elapsed > 0.0 {
                            // idle cooling: heat decays only over the time no dose arrived, so a cell under steady
                            // heating accumulates it and the time to ignite is threshold over rate
                            let span = if idle { (elapsed - dt).max(0.0) } else { elapsed };
                            if span > 0.0 { self.heat[n] *= (-(span / 60.0) / self.cool_min as f32).exp(); }
                            self.heat_t[n] = t;
                        }
                        self.heat[n] += q;
                        let need = self.jitter[n] * self.ignite_scale as f32 * self.fuel(self.kind[n]).ignite * (1.0 + self.moisture_gain as f32 * self.cell_moist[n]) * if self.kind[n] == 4 { 1.0 } else { 1.0 + self.live_moisture_pct as f32 / 200.0 * (self.kind[n] == 2) as u8 as f32 };
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
                                // a structure standing where the ember lands may take it
                                if self.structures == "on" {
                                    if let Some(home) = self.cell_home.get(&l) {
                                        let pick = home[(ctx.rng().f64() * home.len() as f64) as usize % home.len()] as usize;
                                        if self.s_t_ign[pick].is_nan() && ctx.rng().f64() < (self.ember_accept as f32 / self.s_jitter[pick]) as f64 {
                                            self.ignite_struct(pick, t + dt); self.spots += 1; spots_hour += 1; newly += 1;
                                            ctx.publish_at(l, "fire.spot", json!({ "from": b, "distance_m": dist, "structure": pick }));
                                            continue;
                                        }
                                    }
                                }
                                let p_land = if self.kind[l] == 4 && self.ember_built_p > 0.0 { self.ember_built_p as f32 * self.built_frac[l].clamp(0.2, 1.0) } else { self.spot_land_p as f32 * (1.0 - 4.0 * self.cell_moist[l]).max(0.0) };
                                if self.t_ign[l].is_nan() && self.kind[l] != 0 && !self.held[l] && ctx.rng().f64() < p_land as f64 {
                                    self.ignite(l, t + dt); self.spots += 1; spots_hour += 1; newly += 1;
                                    ctx.publish_at(l, "fire.spot", json!({ "from": b, "distance_m": dist }));
                                }
                            }
                        }
                    }
                }
                let _ = cool;
                if self.structures == "on" {
                    // what the bodies feel from the burning ground
                    for &b in &keep {
                        let Some(list) = self.cell_structs.get(&b) else { continue };
                        let ib = self.fuel(self.kind[b]).intensity * if self.kind[b] == 4 { self.built_frac[b].max(0.3) } else { 1.0 };
                        let (wspd, wto) = self.cell_wind[b];
                        let (bc, br) = ((b % g.nx) as f64, (b / g.nx) as f64);
                        for &(i, d) in list.clone().iter() {
                            let i = i as usize;
                            if !self.s_t_ign[i].is_nan() { continue; }
                            // downwind of the burning cell the heat and embers reach further
                            let (sc, sr) = ((self.s_cell[i] % g.nx) as f64, (self.s_cell[i] / g.nx) as f64);
                            let theta = (sc - bc).atan2(br - sr) as f32;
                            let lambda = self.exposure_lambda_m as f32 * (1.0 + self.wind_gain as f32 * wspd * (theta - wto).cos().max(0.0));
                            let q = ib * (-d / lambda).exp() * dt / 60.0;
                            let elapsed = t - self.s_heat_t[i];
                            if elapsed > 0.0 { self.s_heat[i] *= (-(elapsed / 60.0) / self.cool_min as f32).exp(); self.s_heat_t[i] = t; }
                            self.s_heat[i] += q;
                            if self.s_heat[i] >= self.s_jitter[i] * self.structure_ignite as f32 { self.ignite_struct(i, t + dt); newly += 1; }
                        }
                    }
                    // burning bodies heat their neighbours across the gap
                    let s_active = std::mem::take(&mut self.s_active);
                    let mut s_keep = Vec::with_capacity(s_active.len());
                    for &si in &s_active {
                        let si = si as usize;
                        if t >= self.s_t_out[si] { continue; }
                        s_keep.push(si as u32);
                        let is = self.structure_intensity as f32 * (self.s_area[si] / 200.0).sqrt();
                        let (wspd, wto) = self.cell_wind[self.s_cell[si]];
                        for &(j, gap, bearing) in self.s_neigh[si].clone().iter() {
                            let j = j as usize;
                            if !self.s_t_ign[j].is_nan() { continue; }
                            let lambda = self.house_lambda_m as f32 * (1.0 + self.wind_gain as f32 * wspd * (bearing - wto).cos().max(0.0));
                            let q = is * (-gap / lambda).exp() * dt / 60.0;
                            let elapsed = t - self.s_heat_t[j];
                            if elapsed > 0.0 { self.s_heat[j] *= (-(elapsed / 60.0) / self.cool_min as f32).exp(); self.s_heat_t[j] = t; }
                            self.s_heat[j] += q;
                            if self.s_heat[j] >= self.s_jitter[j] * self.structure_ignite as f32 { self.ignite_struct(j, t + dt); newly += 1; }
                        }
                    }
                    s_keep.extend(self.s_active.drain(..));
                    self.s_active = s_keep;
                }
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
        if self.structures == "on" {
            if self.ignited && now.0.saturating_sub(self.last_s_frame) >= self.structure_frame_every.0 {
                ctx.field("building_arrival_h", "h", &g, self.struct_frame(|l| l.iter().map(|&i| self.s_t_ign[i as usize]).filter(|v| !v.is_nan()).fold(f32::NAN, |m, v| if m.is_nan() || v < m { v } else { m }) / 3600.0));
                self.last_s_frame = now.0;
            }
            let (mut lit, mut hit, mut intact_hit) = (0usize, 0usize, 0usize);
            let (mut destroyed, mut intact) = (0usize, 0usize);
            for i in 0..self.s_cell.len() {
                let f = self.s_fate[i]; if f == 3 { destroyed += 1; } else if f == 1 { intact += 1; }
                if !self.s_t_ign[i].is_nan() { lit += 1; if f == 3 { hit += 1; } else if f == 1 { intact_hit += 1; } }
            }
            ctx.report("fire.buildings_ignited", "count", lit as f64);
            ctx.report("fire.buildings_burning", "count", self.s_active.len() as f64);
            ctx.report("fit.buildings_destroyed_reached", "count", hit as f64);
            ctx.report("fit.buildings_undamaged_reached", "count", intact_hit as f64);
            ctx.report("fit.buildings_recall", "1", if destroyed > 0 { hit as f64 / destroyed as f64 } else { 0.0 });
            ctx.report("fit.buildings_precision", "1", if lit > 0 { hit as f64 / lit as f64 } else { 0.0 });
            let _ = intact;
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
        if self.breaks_taken > 0 { ctx.report("fire.held_cells", "cells", self.held_cells as f64); ctx.report("fire.breaks_held", "breaks", self.breaks_taken as f64); }
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
            .param("wind_start", "2025-01-07T18:00:00Z", "", "the first hour of the wind record fetched").param("wind_end", "2025-01-12T00:00:00Z", "", "").param("wind_at", "", "", "the simulated instant wind_start stands for, when a run borrows another day's weather (a scenario in a year with no record yet); empty means wind_start itself, so a run in the record's own time is unchanged. A borrowed wind is a borrowed wind: the page must say whose day it was")
            .param("buildings", "lariac", "", "lariac: LA County's footprints with heights from the feature service below (the Palisades window); osm: OpenStreetMap buildings through Overpass for the window, heights from the height tag or building:levels times 3.2 m, else two storeys, a guess; none: built-up land from land cover alone")
            .param("buildings_service", "https://services.arcgis.com/RmCCgQtiZLDCtblq/arcgis/rest/services/2025_WildFire_LARIAC_BUILDINGS_100M_Plus_DINS_MaxMin_DAMAGE_NEW/FeatureServer/0", "", "").param("buildings_where", "Incident_Name LIKE '%Palisades%'", "", "").param("buildings_key", "lacounty-lariac/2025-palisades", "", "")
            .param("reach_m", 90.0, "m", "guess: how far a burning cell's heat is felt").param("lambda_m", 20.0, "m", "guess: e-folding distance of that heat in still air on the flat")
            .param("wind_gain", 0.15, "1/(m/s)", "guess: the reach grows by this per m/s of wind in the downwind direction").param("slope_gain", 1.5, "", "guess: the reach grows by this times tan(slope) uphill").param("slope_rate_gain", 0.0, "", "the heat a cell delivers uphill is multiplied by 1 + this times tan²(slope) and downhill divided by it; 0 leaves slope to the reach alone. Rothermel's slope factor has this shape with 5.275 β^-0.3 as the gain, about 20 for a shrub bed (recalled); the downhill half is a guess").param("downslope_gain", -1.0, "", "the gain on the downhill division; -1 means the same as slope_rate_gain")
            .param("form", "reach", "", "reach: wind and slope stretch the e-folding distance of the heat (the first form, capped at a few times the flat rate); vector: the heat is I·f(d) times a directional rate factor with Rothermel's shape, wind (downwind) and slope (uphill) added as vectors, head rate 1 + |φ|, every other direction by Anderson's ellipse from the head, so the backing rate is the ellipse's. wind_gain and slope_gain are not used in the vector form")
            .param("wind_phi", 1.9, "per (m/s)^wind_phi_exp", "vector form: φ_w = wind_phi · wind^wind_phi_exp on the 10 m wind; 1.9 and 0.9 are the shape Rothermel's C U^B gives a shrub bed (σ about 750) under a 0.4 wind adjustment, recalled and rounded, so about 15 at 10 m/s and 28 at 20 m/s").param("wind_phi_exp", 0.9, "", "see wind_phi")
            .param("phi_cap", 30.0, "", "vector form: the largest rate factor allowed, standing in for Rothermel's effective-wind limit; a guess").param("waf", 0.4, "", "vector form: the wind adjustment from 10 m to midflame used to pick the ellipse's length-to-breadth (Anderson 1983, recalled); 0.4 is the unsheltered shrub value, recalled")
            .param("moisture_gain", 10.0, "", "guess: ignition energy grows by this times the dead fuel moisture fraction").param("live_moisture_pct", 60.0, "%", "guess: chaparral in January after a dry autumn")
            .param("grass_ignite", 0.6, "heat", "guess").param("grass_intensity", 1.0, "heat/min", "guess").param("grass_burn_min", 5.0, "min", "guess")
            .param("shrub_ignite", 1.5, "heat", "guess").param("shrub_intensity", 2.5, "heat/min", "guess").param("shrub_burn_min", 20.0, "min", "guess")
            .param("tree_ignite", 2.0, "heat", "guess").param("tree_intensity", 2.0, "heat/min", "guess").param("tree_burn_min", 30.0, "min", "guess")
            .param("built_ignite", 4.0, "heat", "guess: a structure's threshold, scaled by nothing yet").param("built_intensity", 3.0, "heat/min", "guess, scaled by the cell's built fraction").param("built_burn_min", 90.0, "min", "guess, scaled by the built fraction")
            .param("cool_min", 15.0, "min", "guess: a cell's received heat decays with this e-folding time").param("cooling", "always", "", "always: a cell's heat decays between doses whatever is happening, so under steady heating it settles at dose times cool_min and a slow flank below the threshold never lights (the cliff); idle: it decays only over the time no dose arrived, so steady heating accumulates and the time to ignite is threshold over rate, as a spread rate is")
            .param("ignite_scale", 1.0, "", "one multiplier on every ignition threshold: the single number fitted to the record (the first infrared perimeter's acreage), declared as such; everything else is the test")
            .param("threshold_jitter", 0.0, "", "sigma of a fixed log-normal factor on each cell's ignition threshold: the micro-conditions nobody mapped (slope detail, what a house is made of); 0 is none")
            .param("ember_built_p", 0.0, "", "chance an ember landing on a built cell lights it, times the cell's built fraction; 0 uses the ordinary landing chance")
            .param("built_to_built_gain", 1.0, "", "heat reaches this many times further from a burning built cell into the next built cell: house to house")
            .param("spot_rate", 0.003, "1/min", "guess: embers per burning cell per minute at 10 m/s, times (wind/10)^2").param("spot_mean_m", 25.0, "m per m/s", "guess: mean ember flight per m/s of wind").param("spot_land_p", 0.3, "", "guess: chance a landed ember lights dry fuel").param("spot_min_intensity", 2.0, "heat/min", "")
            .param("road_widths_m", "motorway=30,trunk=20,primary=14,secondary=10,tertiary=8", "", "roads this wide carry no fuel")
            .param("structures", "off", "", "on: every building footprint is a body of its own that feels the burning ground within exposure_m and its burning neighbours across the gap between footprints, takes embers that land on it, ignites past its own threshold, burns for structure_burn_min and lights the ground it stands on; its fate is scored against the inspectors' word for that footprint")
            .param("structure_ignite", 3.0, "heat", "guess: a structure's ignition threshold, times its own fixed factor").param("structure_intensity", 3.0, "heat/min", "guess at 200 m2 of footprint, scaled by the square root of the area").param("structure_burn_min", 60.0, "min", "guess: how long a burning structure heats its neighbours")
            .param("structure_jitter", 0.3, "", "sigma of a fixed log-normal factor per structure on its threshold: what it is made of, its vents and eaves, which nobody mapped")
            .param("house_reach_m", 40.0, "m", "guess: gap between footprints across which a burning structure's heat is felt").param("house_lambda_m", 8.0, "m", "guess: e-folding gap of that heat, longer downwind")
            .param("exposure_m", 35.0, "m", "guess: the burning ground within this of a footprint heats the structure").param("exposure_lambda_m", 12.0, "m", "guess: e-folding distance of that heat, longer downwind")
            .param("ember_accept", 0.25, "", "guess: chance a structure takes an ember that lands in its cell, divided by its own factor")
            .param("structure_frame_every", "6h", "", "the one-arc-second building_arrival_h frame is written this often")
            .publish("ground.loaded", "cell counts by fuel kind, road cells, HRRR hours").publish("bodies.loaded", "structures, their recorded fates, neighbour links").publish("fire.ignition", "at a cell").publish("fire.spot", "at a cell: { from, distance_m, structure? }")
            .own("fire.burned_frac", "1")
            .frame("terrain_elev_m", "m").frame("building_height", "ft (as stated)").frame("fire_fuel_kind", "0 none 1 grass 2 shrub 3 tree 4 built").frame("fire_arrival_h", "h")
            .frame("building_fate", "0 unknown 1 no damage 2 damaged 3 destroyed (the inspectors)").frame("building_arrival_h", "h")
            .report("fire.buildings_ignited", "count").report("fire.buildings_burning", "count").report("fit.buildings_destroyed_reached", "count").report("fit.buildings_undamaged_reached", "count").report("fit.buildings_recall", "1").report("fit.buildings_precision", "1")
            .report("fire.burned_acres", "acres").report("fire.burned_km2", "km2").report("fire.active_cells", "cells").report("fire.ignited_cells_hour", "cells/h").report("fire.spots_hour", "1/h")
            .report("fire.wind_m_s", "m/s").report("fire.wind_from_deg", "deg").report("fire.dead_moisture_pct", "%").report("fire.built_cells_burned", "cells").report("fire.held_cells", "cells").report("fire.breaks_held", "breaks")
            .subscribe("ground.break", "from an interventions module (fire.suppression): { cells, strength } the cells take no heat and no ember from the break's time on; the engine never reads the record itself")
            .asset("copernicus-dem-glo30", "elevation, slope, aspect").asset("esa-worldcover-2021", "land cover").asset("lacounty-lariac-buildings-2025-fires", "building footprints and heights").asset("osm-overpass", "major roads as breaks; the map's context").asset("noaa-hrrr", "the 3 km wind, gust, temperature and humidity each hour").asset("open-meteo-era5-wind", "the one-point alternative")
            .view("scene", "The fire on the ground", "fire_arrival_h", "terrain, buildings and the fire's arrival, in three dimensions")
            .view("map", "Arrival time", "fire_arrival_h", "hours since ignition").view("map", "What each cell is", "fire_fuel_kind", "")
            .view("chart", "Burned area", "fire.burned_acres", "").view("chart", "Buildings the model has lit", "fire.buildings_ignited", "against the inspectors' record: recall and precision in the series").view("chart", "Wind at the ignition point", "fire.wind_m_s", "HRRR 3 km").view("chart", "Spot fires", "fire.spots_hour", "")
            .note("Not Rothermel: a heat budget with guessed constants, built to be fitted against the mapped perimeters and the inspected structures. The HRRR reader was cross-checked against GDAL at the ignition point. Nothing physical about structures beyond a threshold, an intensity and a burn time.");
        if let Some(c) = ctx_table { d = d.table("context", c); }
        d
    }
}
