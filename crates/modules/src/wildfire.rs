//! A wildfire on real terrain: Rothermel's surface fire spread model (1972, in the form
//! Andrews 2018 sets out) on a one-arc-second window of the DEM, with a fuel model per
//! cell from ESA WorldCover land cover, dead fuel moisture from the hour's humidity and
//! temperature, and the wind's speed and direction from the ERA5 archive at one point.
//! Fire moves cell to cell: a burning cell pushes progress into each unburned neighbour
//! at the rate Rothermel gives in that direction (an ellipse whose head points down the
//! sum of the wind and slope vectors), and the neighbour ignites when its progress
//! reaches one. No suppression, no spotting, no crown fire, no structure-to-structure
//! physics: built-up land carries fire at a guessed rate the manifest can set to zero.
//!
//! `obs.perimeters` beside it fetches the incident's mapped perimeters (NIFC) and, at the
//! time each was current, scores the run's burned area against it: acres, the overlap
//! (Sørensen), what was missed and what was overrun.
//!
//! Every fuel parameter is recalled from Scott and Burgan 2005 and flagged so in
//! data/events/fuels.toml; the moisture equations (Simard 1968), the wind adjustment
//! (Albini and Baughman 1979) and the ellipse (Anderson 1983) are recalled too. Nothing is
//! tuned.

use esim_kernel::manifest::{param_duration, param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module, Time};
use esim_registry::wildfire::{daily_perimeters, landcover_window, rasterize, wfigs_perimeters, wind_hourly, Perimeter, WindSeries};
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

const ACRE_M2: f64 = 4046.8564224;
const FT_PER_M: f64 = 3.280839895;

// ---- fuel models -------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FuelModel {
    pub id: String,
    pub title: String,
    /// loads lb/ft2: dead 1-h, 10-h, 100-h, live herbaceous, live woody
    pub w: [f64; 5],
    /// surface-area-to-volume 1/ft for the same five classes
    pub s: [f64; 5],
    pub depth_ft: f64,
    pub mx_dead: f64,
    pub confidence: String,
    pub source: String,
}

fn load_fuels(root: &std::path::Path) -> Result<(Vec<FuelModel>, Vec<(u8, usize, String)>), String> {
    let text = std::fs::read_to_string(root.join("data/events/fuels.toml")).map_err(|e| format!("fuels.toml: {e}"))?;
    let v: toml::Value = toml::from_str(&text).map_err(|e| format!("fuels.toml: {e}"))?;
    let f = |t: &toml::Value, k: &str| t.get(k).and_then(|x| x.as_float().or_else(|| x.as_integer().map(|i| i as f64))).unwrap_or(0.0);
    const T_AC_TO_LB_FT2: f64 = 2000.0 / 43560.0;
    let mut fuels = Vec::new();
    for t in v.get("fuel").and_then(|x| x.as_array()).into_iter().flatten() {
        fuels.push(FuelModel {
            id: t.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            title: t.get("title").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            w: [f(t, "dead_1h_t_ac"), f(t, "dead_10h_t_ac"), f(t, "dead_100h_t_ac"), f(t, "live_herb_t_ac"), f(t, "live_woody_t_ac")].map(|x| x * T_AC_TO_LB_FT2),
            s: [f(t, "sav_1h"), 109.0, 30.0, f(t, "sav_herb"), f(t, "sav_woody")],
            depth_ft: f(t, "depth_ft"),
            mx_dead: f(t, "mx_dead_pct") / 100.0,
            confidence: t.get("confidence").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            source: t.get("source").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        });
    }
    let mut covers = Vec::new();
    for t in v.get("cover").and_then(|x| x.as_array()).into_iter().flatten() {
        let code = f(t, "code") as u8;
        let id = t.get("fuel").and_then(|x| x.as_str()).unwrap_or("");
        let Some(i) = fuels.iter().position(|m| m.id == id) else { return Err(format!("fuels.toml: cover {code} names unknown fuel {id}")) };
        covers.push((code, i, t.get("cover").and_then(|x| x.as_str()).unwrap_or("").to_string()));
    }
    Ok((fuels, covers))
}

/// What Rothermel gives for a fuel bed at given moistures, before wind and slope: the
/// no-wind rate (ft/min), the reaction intensity (Btu/ft2/min), and the coefficients the
/// wind and slope factors need.
#[derive(Clone, Copy, Debug, Default)]
struct Bed { r0: f64, ir: f64, beta: f64, ratio: f64, b: f64, c: f64, e: f64, waf: f64 }

/// Rothermel 1972 as Andrews 2018 sets it out, two categories (dead, live) of up to three
/// and two size classes. `m` are moisture fractions for the five classes; `cured` moves
/// that share of the live herbaceous load into the dead 1-h class.
fn rothermel(fm: &FuelModel, m: [f64; 5], cured: f64) -> Option<Bed> {
    const RHO_P: f64 = 32.0; const ST: f64 = 0.0555; const SE: f64 = 0.010; const H: f64 = 8000.0;
    let mut w = fm.w;
    w[0] += w[3] * cured; w[3] *= 1.0 - cured;
    let s = fm.s;
    let total: f64 = w.iter().sum();
    if total <= 0.0 || fm.depth_ft <= 0.0 { return None; }
    // weighting by surface area within each category
    let a: Vec<f64> = (0..5).map(|j| if s[j] > 0.0 { s[j] * w[j] / RHO_P } else { 0.0 }).collect();
    let a_dead: f64 = a[..3].iter().sum(); let a_live: f64 = a[3..].iter().sum();
    let f = |j: usize| -> f64 { let ai = if j < 3 { a_dead } else { a_live }; if ai > 0.0 { a[j] / ai } else { 0.0 } };
    let f_dead = a_dead / (a_dead + a_live); let f_live = 1.0 - f_dead;
    let sigma_dead: f64 = (0..3).map(|j| f(j) * s[j]).sum();
    let sigma_live: f64 = (3..5).map(|j| f(j) * s[j]).sum();
    let sigma = f_dead * sigma_dead + f_live * sigma_live;
    if sigma <= 0.0 { return None; }
    let rho_b = total / fm.depth_ft;
    let beta = rho_b / RHO_P;
    let beta_op = 3.348 * sigma.powf(-0.8189);
    let ratio = beta / beta_op;
    let wn_dead: f64 = (0..3).map(|j| f(j) * w[j] * (1.0 - ST)).sum();
    let wn_live: f64 = (3..5).map(|j| f(j) * w[j] * (1.0 - ST)).sum();
    let mf_dead: f64 = (0..3).map(|j| f(j) * m[j]).sum();
    let mf_live: f64 = (3..5).map(|j| f(j) * m[j]).sum();
    // live moisture of extinction from the dead fine fuel's state
    let live_load: f64 = w[3..].iter().sum();
    let mx_live = if live_load > 0.0 {
        let num: f64 = (0..3).map(|j| if s[j] > 0.0 { w[j] * (-138.0 / s[j]).exp() } else { 0.0 }).sum();
        let den: f64 = (3..5).map(|j| if s[j] > 0.0 { w[j] * (-500.0 / s[j]).exp() } else { 0.0 }).sum();
        let wr = if den > 0.0 { num / den } else { 0.0 };
        let mf_fine = if num > 0.0 { (0..3).map(|j| if s[j] > 0.0 { w[j] * m[j] * (-138.0 / s[j]).exp() } else { 0.0 }).sum::<f64>() / num } else { mf_dead };
        (2.9 * wr * (1.0 - mf_fine / fm.mx_dead) - 0.226).max(fm.mx_dead)
    } else { fm.mx_dead };
    let eta_m = |mf: f64, mx: f64| -> f64 { if mx <= 0.0 { return 0.0; } let r = (mf / mx).min(1.0); (1.0 - 2.59 * r + 5.11 * r * r - 3.52 * r * r * r).max(0.0) };
    let eta_s = 0.174 * SE.powf(-0.19);
    let gamma_max = sigma.powf(1.5) / (495.0 + 0.0594 * sigma.powf(1.5));
    let aa = 133.0 * sigma.powf(-0.7913);
    let gamma = gamma_max * ratio.powf(aa) * (aa * (1.0 - ratio)).exp();
    let ir = gamma * H * eta_s * (wn_dead * eta_m(mf_dead, fm.mx_dead) + wn_live * eta_m(mf_live, mx_live));
    if ir <= 0.0 { return None; }
    let xi = ((0.792 + 0.681 * sigma.sqrt()) * (beta + 0.1)).exp() / (192.0 + 0.2595 * sigma);
    let sink_dead: f64 = (0..3).map(|j| if s[j] > 0.0 { f(j) * (-138.0 / s[j]).exp() * (250.0 + 1116.0 * m[j]) } else { 0.0 }).sum();
    let sink_live: f64 = (3..5).map(|j| if s[j] > 0.0 { f(j) * (-138.0 / s[j]).exp() * (250.0 + 1116.0 * m[j]) } else { 0.0 }).sum();
    let sink = rho_b * (f_dead * sink_dead + f_live * sink_live);
    if sink <= 0.0 { return None; }
    let r0 = ir * xi / sink;
    let c = 7.47 * (-0.133 * sigma.powf(0.55)).exp();
    let b = 0.02526 * sigma.powf(0.54);
    let e = 0.715 * (-3.59e-4 * sigma).exp();
    // unsheltered wind adjustment from 20 ft to midflame, by fuel depth (Albini and Baughman 1979)
    let h = fm.depth_ft.max(0.1);
    let waf = 1.83 / ((20.0 + 0.36 * h) / (0.13 * h)).ln();
    Some(Bed { r0, ir, beta, ratio, b, c, e, waf })
}

/// Dead fine fuel moisture (fraction) from relative humidity and temperature: Simard's
/// equilibrium moisture content, T in degF, H in percent. Recalled.
fn dead_moisture(t_c: f64, rh: f64) -> f64 {
    let t = t_c * 9.0 / 5.0 + 32.0;
    let h = rh.clamp(0.0, 100.0);
    let m = if h < 10.0 { 0.03229 + 0.281073 * h - 0.000578 * h * t }
        else if h < 50.0 { 2.22749 + 0.160107 * h - 0.01478 * t }
        else { 21.0606 + 0.005565 * h * h - 0.00035 * h * t - 0.483199 * h };
    (m / 100.0).clamp(0.01, 0.35)
}

/// Length-to-breadth of the fire ellipse from the midflame wind in m/s (Anderson 1983 as
/// FARSITE states it, recalled), held between 1 and 8.
fn length_to_breadth(u_m_s: f64) -> f64 {
    (0.936 * (0.2566 * u_m_s).exp() + 0.461 * (-0.1548 * u_m_s).exp() - 0.397).clamp(1.0, 8.0)
}

// ---- the fire -----------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
pub struct Rothermel {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    dt: Duration,
    frame_every: Duration,
    ignition_time: String,
    ignition_lat: f64, ignition_lon: f64,
    wind_start: String, wind_end: String,
    wind_source: String,       // "mean" | "gust"
    wind_scale: f64,
    live_moisture_pct: f64,
    herb_cured_frac: f64,
    shrub_fuel: String,        // fuel id for WorldCover shrubland, SH5 by default
    built_ros_m_min: f64,
    max_residence_min: f64,
    substep_s: f64,
    reach_cells: i64,
    // state
    ignited: bool,
    ignited_at: Vec<f32>,      // seconds since ignition per cell, NaN unburned
    progress: Vec<f32>,        // per cell and approach direction (cell * n_dirs + k): the share of the distance fire has covered
    active: Vec<usize>,
    since_ignition_s: f64,
    last_frame: i64,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] fuel_of: Vec<u8>,          // index into fuels, 255 none
    #[serde(skip)] cover: Vec<u8>,
    #[serde(skip)] fuels: Vec<FuelModel>,
    #[serde(skip)] slope: Vec<f32>,
    #[serde(skip)] aspect: Vec<f32>,
    #[serde(skip)] wind: Option<WindSeries>,
    #[serde(skip)] area_m2: Vec<f64>,
    #[serde(skip)] metres: Vec<(f64, f64)>,
    #[serde(skip)] beds: Vec<Option<Bed>>,    // per fuel model, this hour
    #[serde(skip)] cell_rate: Vec<(f32, f32, f32)>, // per cell this hour: head m/min, head bearing rad, eccentricity
    #[serde(skip)] stamp: Vec<u32>,
    #[serde(skip)] hour_stamp: u32,
    #[serde(skip)] dirs: Vec<(isize, isize)>,
}

impl Rothermel {
    pub fn new(name: &str, p: &toml::Value) -> Result<Rothermel, String> {
        let mut f = Rothermel {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 34.0), lat_n: param_f64(p, "lat_n", 34.16), lon_w: param_f64(p, "lon_w", -118.72), lon_e: param_f64(p, "lon_e", -118.46),
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            dt: param_duration(p, "dt", "1h")?,
            frame_every: param_duration(p, "frame_every", "6h")?,
            ignition_time: param_str(p, "ignition_time", "2025-01-07T18:30:00Z").to_string(),
            ignition_lat: param_f64(p, "ignition_lat", 34.0723), ignition_lon: param_f64(p, "ignition_lon", -118.5451),
            wind_start: param_str(p, "wind_start", "2025-01-06").to_string(), wind_end: param_str(p, "wind_end", "2025-01-14").to_string(),
            wind_source: param_str(p, "wind_source", "mean").to_string(),
            wind_scale: param_f64(p, "wind_scale", 1.0),
            live_moisture_pct: param_f64(p, "live_moisture_pct", 60.0),
            herb_cured_frac: param_f64(p, "herb_cured_frac", 0.9),
            shrub_fuel: param_str(p, "shrub_fuel", "SH5").to_string(),
            built_ros_m_min: param_f64(p, "built_ros_m_min", 1.0),
            max_residence_min: param_f64(p, "max_residence_min", 120.0),
            substep_s: param_f64(p, "substep_s", 15.0),
            reach_cells: param_f64(p, "reach_cells", 3.0) as i64,
            ignited: false, ignited_at: Vec::new(), progress: Vec::new(), active: Vec::new(), since_ignition_s: 0.0, last_frame: i64::MIN / 4,
            grid: None, fuel_of: Vec::new(), cover: Vec::new(), fuels: Vec::new(), slope: Vec::new(), aspect: Vec::new(), wind: None, area_m2: Vec::new(), metres: Vec::new(),
            beds: Vec::new(), cell_rate: Vec::new(), stamp: Vec::new(), hour_stamp: 0, dirs: Vec::new(),
        };
        f.load()?;
        Ok(f)
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
        self.cover = landcover_window(&mut fe, lc, grid.nx, grid.ny, grid.dlon, grid.dlat, grid.lon_w, grid.lat_n)?;
        let (mut fuels, covers) = load_fuels(&root)?;
        // the manifest may choose the shrub model
        if let Some(i) = fuels.iter().position(|m| m.id == self.shrub_fuel) { let _ = i; } else { return Err(format!("fuels.toml has no fuel {}", self.shrub_fuel)); }
        let shrub = fuels.iter().position(|m| m.id == self.shrub_fuel).unwrap();
        self.fuel_of = self.cover.iter().map(|&c| {
            if c == 20 { return shrub as u8; }
            covers.iter().find(|(code, _, _)| *code == c).map(|(_, i, _)| *i as u8).unwrap_or(255)
        }).collect();
        let wind_lat = (self.lat_s + self.lat_n) / 2.0; let wind_lon = (self.lon_w + self.lon_e) / 2.0;
        self.wind = Some(wind_hourly(&mut fe, wind_lat, wind_lon, &self.wind_start, &self.wind_end)?);
        self.area_m2 = (0..grid.ny).flat_map(|r| std::iter::repeat_n(grid.row_area(r), grid.nx)).collect();
        self.metres = (0..grid.ny).map(|r| { let (mx, my) = grid.metres_per_degree(r); (mx * grid.dlon, my * grid.dlat) }).collect();
        // every primitive lattice direction within reach: (1,0), (1,1), (1,2), (2,1), (1,3), ... in all quadrants
        let r = self.reach_cells.max(1);
        let gcd = |mut a: i64, mut b: i64| { a = a.abs(); b = b.abs(); while b != 0 { let t = a % b; a = b; b = t; } a };
        self.dirs = (-r..=r).flat_map(|dc| (-r..=r).map(move |dr| (dc, dr))).filter(|&(dc, dr)| (dc != 0 || dr != 0) && dc * dc + dr * dr <= r * r && gcd(dc, dr) == 1).map(|(dc, dr)| (dc as isize, dr as isize)).collect();
        let n = grid.nx * grid.ny;
        let nd = self.dirs.len();
        if self.ignited_at.len() != n || self.progress.len() != n * nd { self.ignited_at = vec![f32::NAN; n]; self.progress = vec![0.0; n * nd]; }
        self.beds = vec![None; fuels.len()];
        self.cell_rate = vec![(0.0, 0.0, 0.0); n];
        self.stamp = vec![0; n];
        fuels.truncate(fuels.len());
        self.fuels = fuels;
        self.slope = slope; self.aspect = aspect;
        self.grid = Some(grid);
        Ok(())
    }
    fn cell_of(&self, lat: f64, lon: f64) -> Option<usize> {
        let g = self.grid.as_ref()?;
        if lat > g.lat_n || lat < g.lat_s() || lon < g.lon_w || lon > g.lon_e() { return None; }
        let col = (((lon - g.lon_w) / g.dlon) as usize).min(g.nx - 1);
        let row = (((g.lat_n - lat) / g.dlat) as usize).min(g.ny - 1);
        Some(row * g.nx + col)
    }
    /// This hour's weather: (wind m/s, direction the wind comes from deg, gust m/s, rh %, t2m C)
    fn weather_at(&self, t: Time) -> (f64, f64, f64, f64, f64) {
        let Some(w) = &self.wind else { return (0.0, 0.0, 0.0, 50.0, 15.0) };
        let Some(i) = w.index_at(t.0) else { return (0.0, 0.0, 0.0, 50.0, 15.0) };
        let g = |v: &Vec<f32>| v.get(i).copied().unwrap_or(f32::NAN) as f64;
        (g(&w.wind_m_s), g(&w.dir_deg), g(&w.gust_m_s), g(&w.rh_pct), g(&w.t2m_c))
    }
    /// Rate at a cell this hour, cached: (head rate m/min, head bearing rad clockwise from north, eccentricity).
    fn rate(&mut self, cell: usize, u10: f64, dir_from: f64) -> (f32, f32, f32) {
        if self.stamp[cell] == self.hour_stamp { return self.cell_rate[cell]; }
        self.stamp[cell] = self.hour_stamp;
        let fi = self.fuel_of[cell];
        let out = if fi == 255 { (0.0, 0.0, 0.0) } else {
            let fm = &self.fuels[fi as usize];
            let downwind = (dir_from + 180.0).to_radians();
            if fm.id == "URB" {
                // not Rothermel: a guessed structure-to-structure rate, scaled by the wind
                let head = self.built_ros_m_min * (u10 / 10.0).max(0.2);
                let lb = length_to_breadth(u10 * 0.4);
                let ecc = (1.0 - 1.0 / (lb * lb)).max(0.0).sqrt();
                (head as f32, downwind as f32, ecc as f32)
            } else if let Some(bed) = self.beds[fi as usize] {
                // midflame wind ft/min, capped at Rothermel's effective wind limit
                let mut u = u10 * FT_PER_M * 60.0 * bed.waf;
                u = u.min(0.9 * bed.ir);
                let phi_w = bed.c * u.powf(bed.b) * bed.ratio.powf(-bed.e);
                let slope = self.slope[cell] as f64;
                let phi_s = 5.275 * bed.beta.powf(-0.3) * slope.tan().powi(2);
                // vector sum of wind (downwind) and slope (uphill) effects
                let (mut vx, mut vy) = (phi_w * downwind.sin(), phi_w * downwind.cos());
                let asp = self.aspect[cell] as f64;
                if asp.is_finite() && phi_s > 0.0 { let up = asp + std::f64::consts::PI; vx += phi_s * up.sin(); vy += phi_s * up.cos(); }
                let phi = (vx * vx + vy * vy).sqrt();
                let head_dir = if phi > 1e-9 { vx.atan2(vy) } else { downwind };
                let head = bed.r0 * (1.0 + phi) / FT_PER_M;
                let lb = length_to_breadth(u / (FT_PER_M * 60.0));
                let ecc = (1.0 - 1.0 / (lb * lb)).max(0.0).sqrt();
                (head as f32, head_dir as f32, ecc as f32)
            } else { (0.0, 0.0, 0.0) }
        };
        self.cell_rate[cell] = out;
        out
    }
    fn burned_field(&self) -> Vec<f32> { self.ignited_at.iter().map(|v| if v.is_nan() { 0.0 } else { 1.0 }).collect() }
    fn burned_m2(&self) -> f64 { self.ignited_at.iter().zip(&self.area_m2).filter(|(v, _)| !v.is_nan()).map(|(_, a)| a).sum() }
}

impl Module for Rothermel {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        let g = self.grid.clone().unwrap();
        ctx.publish("fuel.loaded", json!({ "fuels": self.fuels.iter().map(|f| json!({ "id": f.id, "confidence": f.confidence })).collect::<Vec<_>>(), "shrub": self.shrub_fuel, "cells": g.nx * g.ny, "burnable_cells": self.fuel_of.iter().filter(|&&f| f != 255).count() }));
        ctx.field("fire_fuel_class", "worldcover code", &g, self.cover.iter().map(|&c| c as f32).collect());
        let mut km2 = std::collections::BTreeMap::new();
        for (i, &c) in self.cover.iter().enumerate() { *km2.entry(c).or_insert(0.0) += self.area_m2[i] / 1e6; }
        ctx.publish("cover.summary", json!(km2.iter().map(|(c, a)| json!({ "code": c, "km2": (a * 100.0).round() / 100.0 })).collect::<Vec<_>>()));
        ctx.world_write("fire.burned_frac", "1", &g, self.burned_field());
        ctx.schedule_with_priority(self.dt, 6, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" { return; }
        let g = self.grid.clone().unwrap();
        let now = ctx.now();
        let t_ign = Time::parse(&self.ignition_time).unwrap_or(Time(i64::MAX));
        // the hour's weather and fuel beds
        let (mean, dir, gust, rh, t2m) = self.weather_at(now);
        let u10 = (if self.wind_source == "gust" { gust } else { mean }).max(0.0) * self.wind_scale;
        let u10 = if u10.is_finite() { u10 } else { 0.0 };
        let md = dead_moisture(t2m, rh);
        let m = [md, md + 0.01, md + 0.02, self.live_moisture_pct / 100.0, self.live_moisture_pct / 100.0];
        self.hour_stamp = self.hour_stamp.wrapping_add(1);
        for (i, fm) in self.fuels.iter().enumerate() { self.beds[i] = if fm.id == "URB" { None } else { rothermel(fm, m, self.herb_cured_frac) }; }
        let shrub_bed = self.fuels.iter().position(|f| f.id == self.shrub_fuel).and_then(|i| self.beds[i]);
        // ignition
        if !self.ignited && now.0 >= t_ign.0 {
            if let Some(c) = self.cell_of(self.ignition_lat, self.ignition_lon) {
                self.ignited = true;
                self.ignited_at[c] = 0.0;
                self.active.push(c);
                ctx.publish_at(c, "fire.ignition", json!({ "lat": self.ignition_lat, "lon": self.ignition_lon, "time": self.ignition_time, "cover": self.cover[c], "fuel": self.fuel_of.get(c).and_then(|&f| self.fuels.get(f as usize)).map(|f| f.id.clone()) }));
            }
        }
        // spread through the hour in substeps
        let mut newly = 0usize;
        if self.ignited {
            let steps = (self.dt.as_secs_f64() / self.substep_s).round().max(1.0) as usize;
            let dt = self.dt.as_secs_f64() / steps as f64;
            let residence = self.max_residence_min * 60.0;
            let dirs = self.dirs.clone();
            let nd = dirs.len();
            for _ in 0..steps {
                let t_s = self.since_ignition_s;
                let mut keep: Vec<usize> = Vec::with_capacity(self.active.len());
                let mut lit: Vec<usize> = Vec::new();
                let active = std::mem::take(&mut self.active);
                for &b in &active {
                    let (bc, br) = (b % g.nx, b / g.nx);
                    let mut any_open = false;
                    for (k, (dc, dr)) in dirs.iter().copied().enumerate() {
                        let (nc, nr) = (bc as isize + dc, br as isize + dr);
                        if nc < 0 || nr < 0 || nc >= g.nx as isize || nr >= g.ny as isize { continue; }
                        let n = nr as usize * g.nx + nc as usize;
                        if !self.ignited_at[n].is_nan() || self.fuel_of[n] == 255 { continue; }
                        // the straight path must not cross a cell that cannot burn (water, bare rock, a barrier)
                        let steps = dc.abs().max(dr.abs());
                        let mut blocked = false;
                        for t in 1..steps {
                            let (ic, ir) = (bc as isize + (dc * t) / steps, br as isize + (dr * t) / steps);
                            if self.fuel_of[ir as usize * g.nx + ic as usize] == 255 { blocked = true; break; }
                        }
                        if blocked { continue; }
                        any_open = true;
                        let (head, theta_h, ecc) = self.rate(n, u10, dir);
                        if head <= 0.0 { continue; }
                        let (mx, my) = self.metres[br];
                        let (ex, ny_m) = (dc as f64 * mx, -(dr as f64) * my);
                        let dist = (ex * ex + ny_m * ny_m).sqrt();
                        let theta = ex.atan2(ny_m);
                        let r = head as f64 * (1.0 - ecc as f64) / (1.0 - ecc as f64 * (theta - theta_h as f64).cos());
                        let slot = n * nd + k;
                        self.progress[slot] += (r / 60.0 * dt / dist) as f32;
                        if self.progress[slot] >= 1.0 { self.ignited_at[n] = (t_s + dt) as f32; lit.push(n); }
                    }
                    if any_open && (t_s - self.ignited_at[b] as f64) < residence { keep.push(b); }
                }
                lit.sort_unstable(); lit.dedup();
                newly += lit.len();
                keep.extend(lit);
                self.active = keep;
                self.since_ignition_s += dt;
            }
        }
        let burned_m2 = self.burned_m2();
        ctx.world_write("fire.burned_frac", "1", &g, self.burned_field());
        if self.ignited && now.0.saturating_sub(self.last_frame) >= self.frame_every.0 {
            ctx.field("fire_arrival_h", "h", &g, self.ignited_at.iter().map(|v| v / 3600.0).collect());
            self.last_frame = now.0;
        }
        let head_shrub = shrub_bed.map(|bed| { let u = (u10 * FT_PER_M * 60.0 * bed.waf).min(0.9 * bed.ir); bed.r0 * (1.0 + bed.c * u.powf(bed.b) * bed.ratio.powf(-bed.e)) / FT_PER_M }).unwrap_or(0.0);
        ctx.report("fire.burned_acres", "acres", burned_m2 / ACRE_M2);
        ctx.report("fire.burned_km2", "km2", burned_m2 / 1e6);
        ctx.report("fire.active_cells", "cells", self.active.len() as f64);
        ctx.report("fire.ignited_cells_hour", "cells/h", newly as f64);
        ctx.report("fire.wind_m_s", "m/s", mean);
        ctx.report("fire.gust_m_s", "m/s", gust);
        ctx.report("fire.wind_dir_deg", "deg", dir);
        ctx.report("fire.rh_pct", "%", rh);
        ctx.report("fire.t2m_c", "degC", t2m);
        ctx.report("fire.dead_moisture_pct", "%", md * 100.0);
        ctx.report("fire.head_ros_shrub_m_min", "m/min", head_shrub);
        ctx.schedule_with_priority(self.dt, 6, "tick", json!(null));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut f: Rothermel = serde_json::from_value(v).unwrap(); f.load().expect("fire reload"); *self = f; }
}

impl Rothermel {
    pub fn describe(name: &str, p: &toml::Value) -> esim_kernel::describe::Describe {
        let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let fuels = load_fuels(&root).map(|(f, _)| serde_json::to_value(f).unwrap_or(serde_json::Value::Array(vec![]))).unwrap_or(serde_json::Value::Array(vec![]));
        esim_kernel::describe::Describe::new("fire.rothermel", name, "A wildfire on real terrain: Rothermel's surface spread model on a one-arc-second DEM window, a fuel model per cell from WorldCover land cover, dead fuel moisture from the hour's humidity and temperature, wind speed and direction from the ERA5 archive at the window's centre; fire moves cell to cell down an ellipse headed along the wind-plus-slope vector. No suppression, no spotting, no crown fire; built-up land burns at a guessed rate the manifest can set to zero.")
            .priority(6).dt(param_str(p, "dt", "1h"))
            .param("lat_s", 34.0, "deg", "").param("lat_n", 34.16, "deg", "").param("lon_w", -118.72, "deg", "").param("lon_e", -118.46, "deg", "")
            .param("res_arcsec", 1.0, "arcsec", "one arc-second is the DEM's own 30 m")
            .param("dt", "1h", "", "the weather step; spread runs in substeps inside it").param("frame_every", "6h", "", "arrival-time frames")
            .param("ignition_time", "2025-01-07T18:30:00Z", "", "").param("ignition_lat", 34.0723, "deg", "").param("ignition_lon", -118.5451, "deg", "the ignition point in the California WFTIIC record for the Palisades fire")
            .param("wind_start", "2025-01-06", "date", "").param("wind_end", "2025-01-14", "date", "")
            .param("wind_source", "mean", "", "mean: the hourly 10 m wind; gust: the hourly gust, as a what-if for a reanalysis that reads a Santa Ana event mild")
            .param("wind_scale", 1.0, "", "multiplier on the wind, for sensitivity runs; 1 is the data")
            .param("live_moisture_pct", 60.0, "%", "live fuel moisture; a guess for chaparral in January after a dry autumn")
            .param("herb_cured_frac", 0.9, "", "share of live herbaceous load counted as dead; a guess for January")
            .param("shrub_fuel", "SH5", "", "fuel model for WorldCover shrubland: SH5 or SH7")
            .param("built_ros_m_min", 1.0, "m/min", "guessed head rate through built-up land at 10 m/s wind, scaled by the wind; 0 makes built-up land a barrier")
            .param("max_residence_min", 120.0, "min", "how long a burning cell keeps pushing fire into neighbours it has not yet lit")
            .param("substep_s", 15.0, "s", "spread substep; a 30 m cell at 100 m/min is crossed in 18 s")
            .param("reach_cells", 3, "cells", "fire travels straight to every cell within this radius along primitive lattice directions (32 at 3), so a narrow wind-driven ellipse is not starved by the eight neighbours; the path must not cross an unburnable cell")
            .publish("fuel.loaded", "the fuel models in use and their confidence").publish("cover.summary", "km2 per land-cover class in the window").publish("fire.ignition", "at a cell: { lat, lon, time, cover, fuel }")
            .own("fire.burned_frac", "1")
            .frame("fire_fuel_class", "worldcover code").frame("fire_arrival_h", "h")
            .report("fire.burned_acres", "acres").report("fire.burned_km2", "km2").report("fire.active_cells", "cells").report("fire.ignited_cells_hour", "cells/h")
            .report("fire.wind_m_s", "m/s").report("fire.gust_m_s", "m/s").report("fire.wind_dir_deg", "deg").report("fire.rh_pct", "%").report("fire.t2m_c", "degC")
            .report("fire.dead_moisture_pct", "%").report("fire.head_ros_shrub_m_min", "m/min")
            .asset("copernicus-dem-glo30", "terrain, slope and aspect").asset("esa-worldcover-2021", "fuel class per cell").asset("open-meteo-era5-wind", "hourly wind, direction, gusts, humidity, temperature at the window's centre")
            .asset("events/fuels.toml", "fuel models, every row recalled from Scott and Burgan 2005 and not yet checked")
            .table("fuels", fuels)
            .view("map", "Arrival time", "fire_arrival_h", "hours since ignition, by cell")
            .view("chart", "Burned area", "fire.burned_acres", "")
            .view("chart", "Wind at the window's centre", "fire.wind_m_s", "ERA5 reanalysis, 25 km")
            .note("Rothermel 1972 as Andrews 2018 sets it out; moisture by Simard 1968; wind adjustment by Albini and Baughman 1979; ellipse by Anderson 1983. All recalled; nothing tuned. The reanalysis wind under-reads Santa Ana gusts; the gust variant is the honest upper bound the data allows.")
    }
}

// ---- the observed perimeters -------------------------------------------------------

#[derive(Serialize, Deserialize)]
pub struct Perimeters {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    incident: String,
    year: i64,
    irwin: String,
    scheduled: bool,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] perims: Vec<Perimeter>,
    #[serde(skip)] sources: Vec<String>,
    #[serde(skip)] area_m2: Vec<f64>,
}

impl Perimeters {
    pub fn new(name: &str, p: &toml::Value) -> Result<Perimeters, String> {
        let mut o = Perimeters {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 34.0), lat_n: param_f64(p, "lat_n", 34.16), lon_w: param_f64(p, "lon_w", -118.72), lon_e: param_f64(p, "lon_e", -118.46),
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            incident: param_str(p, "incident", "Palisades").to_string(),
            year: p.get("year").and_then(|v| v.as_integer()).unwrap_or(2025),
            irwin: param_str(p, "irwin", "{A7EA5D21-F882-44B8-BF64-44AB11059DC1}").to_string(),
            scheduled: false,
            grid: None, perims: Vec::new(), sources: Vec::new(), area_m2: Vec::new(),
        };
        o.load()?;
        Ok(o)
    }
    fn load(&mut self) -> Result<(), String> {
        let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let dem = cat.asset("copernicus-dem-glo30").ok_or("catalog has no copernicus-dem-glo30")?;
        let mut fe = Fetcher::default_for(&root)?;
        let w = dem_window(&mut fe, dem, self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let grid = LatLonGrid { nx: w.nx, ny: w.ny, dlon: w.dlon, dlat: w.dlat, lon_w: w.lon_w, lat_n: w.lat_n };
        self.area_m2 = (0..grid.ny).flat_map(|r| std::iter::repeat_n(grid.row_area(r), grid.nx)).collect();
        let mut all: Vec<(Perimeter, String)> = Vec::new();
        for p in daily_perimeters(&mut fe, &self.irwin, self.year as i32)? { all.push((p, "nifc-operational-archive".into())); }
        for p in wfigs_perimeters(&mut fe, &self.incident, self.year as i32)? { all.push((p, "nifc-wfigs-perimeters".into())); }
        all.sort_by_key(|(p, _)| p.time);
        self.perims = all.iter().map(|(p, _)| p.clone()).collect();
        self.sources = all.into_iter().map(|(_, s)| s).collect();
        self.grid = Some(grid);
        Ok(())
    }
}

impl Module for Perimeters {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        let now = ctx.now();
        ctx.publish("perimeters.loaded", json!({ "incident": self.incident, "irwin": self.irwin, "count": self.perims.len(), "first": self.perims.first().map(|p| Time(p.time).to_iso()), "last": self.perims.last().map(|p| Time(p.time).to_iso()) }));
        if !self.scheduled {
            for (i, p) in self.perims.iter().enumerate() {
                if p.time >= now.0 { ctx.schedule_with_priority(Duration(p.time - now.0), 9, "perimeter", json!(i)); }
            }
            self.scheduled = true;
        }
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "perimeter" { return; }
        let Some(i) = ev.data.as_u64().map(|x| x as usize) else { return };
        let Some(p) = self.perims.get(i) else { return };
        let g = self.grid.clone().unwrap();
        let mask = rasterize(&p.rings, g.nx, g.ny, g.dlon, g.dlat, g.lon_w, g.lat_n);
        let obs_m2: f64 = mask.iter().zip(&self.area_m2).filter(|(m, _)| **m).map(|(_, a)| a).sum();
        ctx.field("obs_perimeter", "1", &g, mask.iter().map(|&m| if m { 1.0 } else { 0.0 }).collect());
        ctx.report("obs.acres", "acres", p.acres);
        ctx.report("obs.acres_in_window", "acres", obs_m2 / ACRE_M2);
        if let Some(f) = ctx.world_read("fire.burned_frac") {
            if f.data.len() == mask.len() {
                let (mut both, mut sim, mut missed, mut over) = (0.0, 0.0, 0.0, 0.0);
                for k in 0..mask.len() {
                    let s = f.data[k] >= 0.5;
                    let a = self.area_m2[k];
                    if s { sim += a; }
                    match (s, mask[k]) { (true, true) => both += a, (false, true) => missed += a, (true, false) => over += a, _ => {} }
                }
                let sorensen = if sim + obs_m2 > 0.0 { 2.0 * both / (sim + obs_m2) } else { 0.0 };
                ctx.report("sim.acres", "acres", sim / ACRE_M2);
                ctx.report("fit.sorensen", "1", sorensen);
                ctx.report("fit.missed_acres", "acres", missed / ACRE_M2);
                ctx.report("fit.over_acres", "acres", over / ACRE_M2);
                ctx.publish("perimeter.scored", json!({ "time": Time(p.time).to_iso(), "obs_acres": p.acres, "sim_acres": sim / ACRE_M2, "sorensen": sorensen, "method": p.method, "source": self.sources.get(i) }));
            }
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Perimeters = serde_json::from_value(v).unwrap(); o.load().expect("perimeters reload"); *self = o; }
}

impl Perimeters {
    pub fn describe(name: &str, p: &toml::Value) -> esim_kernel::describe::Describe {
        esim_kernel::describe::Describe::new("obs.perimeters", name, "The mapped perimeters of a wildfire incident from NIFC (the daily operational perimeters by IRWIN id and the WFIGS perimeter of record), each rasterised on the run's grid at the time it was current and scored against the fire module's burned area: acres, overlap (Sørensen), missed and overrun acres.")
            .priority(9).dt("at each mapped perimeter's time")
            .param("lat_s", 34.0, "deg", "").param("lat_n", 34.16, "deg", "").param("lon_w", -118.72, "deg", "").param("lon_e", -118.46, "deg", "").param("res_arcsec", 1.0, "arcsec", "")
            .param("incident", "Palisades", "", "incident name as WFIGS spells it").param("year", 2025, "", "discovery year").param("irwin", "{A7EA5D21-F882-44B8-BF64-44AB11059DC1}", "", "IRWIN id of the incident")
            .publish("perimeters.loaded", "{ incident, irwin, count, first, last }").publish("perimeter.scored", "{ time, obs_acres, sim_acres, sorensen, method, source }")
            .read("fire.burned_frac", "1", false, "the fire module's burned cells")
            .frame("obs_perimeter", "1")
            .report("obs.acres", "acres").report("obs.acres_in_window", "acres").report("sim.acres", "acres").report("fit.sorensen", "1").report("fit.missed_acres", "acres").report("fit.over_acres", "acres")
            .asset("copernicus-dem-glo30", "the grid").asset("nifc-operational-archive", "daily perimeters").asset("nifc-wfigs-perimeters", "the perimeter of record")
            .view_vs("Burned area against the mapped perimeters", "sim.acres", "obs.acres", "at the times a perimeter was current")
            .view("map", "Mapped perimeter", "obs_perimeter", "the incident's perimeter current at this time")
            .note(&format!("Incident {} ({}), IRWIN {}", param_str(p, "incident", "Palisades"), p.get("year").and_then(|v| v.as_integer()).unwrap_or(2025), param_str(p, "irwin", "{A7EA5D21-F882-44B8-BF64-44AB11059DC1}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sh5() -> FuelModel {
        const K: f64 = 2000.0 / 43560.0;
        FuelModel { id: "SH5".into(), title: String::new(), w: [3.6 * K, 2.1 * K, 0.0, 0.0, 2.9 * K], s: [750.0, 109.0, 30.0, 9999.0, 1600.0], depth_ft: 6.0, mx_dead: 0.15, confidence: String::new(), source: String::new() }
    }
    #[test]
    fn dry_chaparral_spreads_and_wet_does_not() {
        let dry = rothermel(&sh5(), [0.05, 0.06, 0.07, 0.6, 0.6], 0.9).expect("dry bed burns");
        assert!(dry.r0 > 0.5 && dry.r0 < 50.0, "no-wind rate {} ft/min", dry.r0);
        assert!(dry.ir > 100.0, "reaction intensity {}", dry.ir);
        // 10 m/s at midflame: a head rate of tens of metres a minute
        let u = 10.0 * FT_PER_M * 60.0 * dry.waf;
        let head = dry.r0 * (1.0 + dry.c * u.powf(dry.b) * dry.ratio.powf(-dry.e)) / FT_PER_M;
        assert!(head > 5.0 && head < 200.0, "head rate {head} m/min");
        let wet = rothermel(&sh5(), [0.30, 0.31, 0.32, 1.2, 1.2], 0.0);
        assert!(wet.is_none() || wet.unwrap().r0 < 1e-6);
    }
    #[test]
    fn moisture_and_ellipse_are_sane() {
        assert!(dead_moisture(20.0, 15.0) < 0.06 && dead_moisture(20.0, 15.0) > 0.02);
        assert!(dead_moisture(10.0, 90.0) > 0.15);
        assert!((length_to_breadth(0.0) - 1.0).abs() < 0.01);
        assert!(length_to_breadth(4.5) > 2.5 && length_to_breadth(4.5) < 3.2, "{}", length_to_breadth(4.5));
    }
}
