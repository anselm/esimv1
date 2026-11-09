//! `coast.oneline`: a one-line shoreline model for a bay behind a headland and a breakwater.
//!
//! The shoreline is a line of transects along today's coast (OpenStreetMap), each a position
//! seaward (+) or landward (-) of where it started. Each hour the deep-water waves of a reanalysis
//! cell (ERA5-Ocean through Open-Meteo) are carried to each transect: a structure (the headland,
//! the breakwater) throws a geometric shadow, and inside it the waves arrive diffracted, lower and
//! from the structure's tip, which is what bends an open coast into the hook-shaped bay the record
//! describes; then linear shoaling and refraction to breaking (Komar and Gaughan's breaker height,
//! Snell's law for the angle); then the CERC formula for the longshore transport. The divergence of
//! that transport moves the shoreline (Pelnard-Considere), the shoreline's own angle feeds back, and
//! where the beach is gone the bluff retreats and yields its sand. At the breakwater's root a declared
//! fraction of the sand moving past is lost through the rubble mound's voids into the harbor, which
//! is the mechanism the Corps named and the one fitted number here. Sea level rise moves the line by
//! Bruun's rule. A dredge every so many years moves harbor sand back onto the first stretch of beach.
//! Every parameter is a published coefficient recalled from the coastal engineering texts or a guess,
//! declared so; the record it is read against is `data/events/halfmoonbay/record.toml`.
//!
//! It is not a fluid dynamics model: no currents, no cross-shore profile, no wave setup, no
//! reflection off the structure, no two-dimensional sediment field. It is the oldest useful
//! approximation, written so the next engine can be scored against the same record.

use esim_kernel::manifest::{param_duration, param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module, Time};
use esim_registry::coast::{osm_shore, waves_hourly, ShoreWay, WaveSeries};
use esim_registry::Fetcher;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

const G: f64 = 9.81;
const M_PER_DEG: f64 = 111_320.0;
const SAND: &str = "sand";

#[derive(Clone, Serialize, Deserialize)]
struct Transect { lat: f64, lon: f64, ex: f64, ny: f64, x_m: f64, tangent_deg: f64, normal_deg: f64 }

/// A sheltering structure: a segment from its landward base to its seaward tip, in local metres.
#[derive(Clone, Serialize, Deserialize)]
struct Structure { name: String, base: (f64, f64), tip: (f64, f64) }

#[derive(Serialize, Deserialize)]
pub struct OneLine {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64, frame_res_arcsec: f64, frame_margin_m: f64,
    root_lat: f64, root_lon: f64, length_m: f64, dx_m: f64,
    breakwater: String, tip_lat: f64, tip_lon: f64,
    headland: String, head_base_lat: f64, head_base_lon: f64, head_tip_lat: f64, head_tip_lon: f64,
    wave_lat: f64, wave_lon: f64, year_from: i32, year_to: i32,
    k_cerc: f64, gamma: f64, d_closure_m: f64, berm_m: f64, beach_width_m: f64, bluff_height_m: f64, bluff_sand_frac: f64,
    shadow_deg: f64, lit_deg: f64, alpha_cap_deg: f64, leak_frac: f64, slr_m_yr: f64, slr_from_year: i32, closure_distance_m: f64,
    dredge_every_years: f64, dredge_m3: f64, dredge_from_year: i32, place_length_m: f64,
    near_m: f64, zones_m: Vec<f64>, far_end: String, leak_reach_m: f64, dir_bias: String, fit_from_m: f64, dir_bias_fit: f64,
    dt: Duration, report_every: Duration, frame_every: Duration,
    // state
    y: Vec<f64>, bluff: Vec<f64>,
    harbor_m3: f64, downcoast_m3: f64, offshore_m3: f64, bluff_m3: f64, placed_m3: f64, upcoast_m3: f64,
    pending: [f64; 6],           // transfers since the last tick: harbor, downcoast, offshore, bluff supply, placed, upcoast
    hs_sum: f64, hs_n: f64, gross_root: f64,
    y_mark: Vec<f64>, harbor_mark: f64, t_mark: i64,
    last_report: i64, last_frame: i64, last_dredge_year: i32, started: bool, t_start: i64,
    #[serde(skip)] waves: Option<WaveSeries>,
    #[serde(skip)] transects: Vec<Transect>,
    #[serde(skip)] structures: Vec<Structure>,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] cos0: f64,
    #[serde(skip)] slr_on: bool,
}

fn bearing_deg(dx_e: f64, dy_n: f64) -> f64 { (dx_e.atan2(dy_n).to_degrees() + 360.0) % 360.0 }
fn wrap180(a: f64) -> f64 { let mut a = (a + 180.0) % 360.0; if a < 0.0 { a += 360.0; } a - 180.0 }

/// Does the half-line from `s` toward the bearing `from_deg` (where the waves come from) cross the segment a-b?
fn ray_hits(s: (f64, f64), from_deg: f64, a: (f64, f64), b: (f64, f64)) -> bool {
    let (dx, dy) = (from_deg.to_radians().sin(), from_deg.to_radians().cos());
    let (ex, ey) = (b.0 - a.0, b.1 - a.1);
    let den = dx * ey - dy * ex;
    if den.abs() < 1e-12 { return false; }
    let (wx, wy) = (a.0 - s.0, a.1 - s.1);
    let t = (wx * ey - wy * ex) / den;      // along the ray
    let u = (wx * dy - wy * dx) / den;      // along the segment
    t > 0.0 && (0.0..=1.0).contains(&u)
}

impl OneLine {
    pub fn new(name: &str, p: &toml::Value) -> Result<OneLine, String> {
        let mut o = OneLine {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 37.42), lat_n: param_f64(p, "lat_n", 37.52), lon_w: param_f64(p, "lon_w", -122.52), lon_e: param_f64(p, "lon_e", -122.42),
            res_arcsec: param_f64(p, "res_arcsec", 1.0), frame_res_arcsec: param_f64(p, "frame_res_arcsec", 0.5), frame_margin_m: param_f64(p, "frame_margin_m", 600.0),
            root_lat: param_f64(p, "root_lat", 37.49995), root_lon: param_f64(p, "root_lon", -122.47432),
            length_m: param_f64(p, "length_m", 8000.0), dx_m: param_f64(p, "dx_m", 50.0),
            breakwater: param_str(p, "breakwater", "on").to_string(), tip_lat: param_f64(p, "tip_lat", 37.49522), tip_lon: param_f64(p, "tip_lon", -122.48573),
            headland: param_str(p, "headland", "on").to_string(), head_base_lat: param_f64(p, "head_base_lat", 37.5031), head_base_lon: param_f64(p, "head_base_lon", -122.4851),
            head_tip_lat: param_f64(p, "head_tip_lat", 37.4927), head_tip_lon: param_f64(p, "head_tip_lon", -122.5005),
            wave_lat: param_f64(p, "wave_lat", 37.45), wave_lon: param_f64(p, "wave_lon", -122.55),
            year_from: param_f64(p, "year_from", 1961.0) as i32, year_to: param_f64(p, "year_to", 2024.0) as i32,
            k_cerc: param_f64(p, "k_cerc", 0.39), gamma: param_f64(p, "gamma", 0.78), d_closure_m: param_f64(p, "d_closure_m", 8.0), berm_m: param_f64(p, "berm_m", 2.0),
            beach_width_m: param_f64(p, "beach_width_m", 20.0), bluff_height_m: param_f64(p, "bluff_height_m", 10.0), bluff_sand_frac: param_f64(p, "bluff_sand_frac", 0.5),
            shadow_deg: param_f64(p, "shadow_deg", 25.0), lit_deg: param_f64(p, "lit_deg", 8.0), alpha_cap_deg: param_f64(p, "alpha_cap_deg", 45.0), leak_frac: param_f64(p, "leak_frac", 0.02),
            slr_m_yr: param_f64(p, "slr_m_yr", 0.0), slr_from_year: param_f64(p, "slr_from_year", 2025.0) as i32, closure_distance_m: param_f64(p, "closure_distance_m", 500.0),
            dredge_every_years: param_f64(p, "dredge_every_years", 0.0), dredge_m3: param_f64(p, "dredge_m3", 76_000.0), dredge_from_year: param_f64(p, "dredge_from_year", 2025.0) as i32,
            place_length_m: param_f64(p, "place_length_m", 305.0),
            near_m: param_f64(p, "near_m", 1400.0), zones_m: vec![0.0, 700.0, 1400.0, 3000.0, 1e9], far_end: param_str(p, "far_end", "open").to_string(), leak_reach_m: param_f64(p, "leak_reach_m", 300.0), dir_bias: param_str(p, "dir_bias", "fit").to_string(), fit_from_m: param_f64(p, "fit_from_m", 3000.0), dir_bias_fit: 0.0,
            dt: param_duration(p, "dt", "1d")?, report_every: param_duration(p, "report_every", "30d")?, frame_every: param_duration(p, "frame_every", "365d")?,
            y: Vec::new(), bluff: Vec::new(), harbor_m3: 0.0, downcoast_m3: 0.0, offshore_m3: 0.0, bluff_m3: 0.0, placed_m3: 0.0, upcoast_m3: 0.0, pending: [0.0; 6],
            hs_sum: 0.0, hs_n: 0.0, gross_root: 0.0, y_mark: Vec::new(), harbor_mark: 0.0, t_mark: 0,
            last_report: 0, last_frame: 0, last_dredge_year: 0, started: false, t_start: 0,
            waves: None, transects: Vec::new(), structures: Vec::new(), grid: None, cos0: 1.0, slr_on: false,
        };
        o.load()?;
        Ok(o)
    }

    fn local(&self, lat: f64, lon: f64) -> (f64, f64) { ((lon - self.root_lon) * M_PER_DEG * self.cos0, (lat - self.root_lat) * M_PER_DEG) }

    fn load(&mut self) -> Result<(), String> {
        let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let mut fe = Fetcher::default_for(&root)?;
        self.cos0 = self.root_lat.to_radians().cos();
        // the shoreline: chain the coastline ways from the node nearest the root, walking away from it
        let ways: Vec<ShoreWay> = osm_shore(&mut fe, self.lat_s, self.lat_n, self.lon_w, self.lon_e)?;
        let coast: Vec<&ShoreWay> = ways.iter().filter(|w| w.kind == "coastline").collect();
        if coast.is_empty() { return Err("coast.oneline: OpenStreetMap has no coastline in the window".into()); }
        let dist = |a: (f64, f64), b: (f64, f64)| { let (ax, ay) = ((a.1 - b.1) * M_PER_DEG * self.cos0, (a.0 - b.0) * M_PER_DEG); (ax * ax + ay * ay).sqrt() };
        let rootpt = (self.root_lat, self.root_lon);
        // the way whose end is nearest the root starts the chain; OSM coastline has water on its right
        let mut best: Option<(usize, bool, f64)> = None;
        for (i, w) in coast.iter().enumerate() {
            let d0 = dist(w.pts[0], rootpt); let d1 = dist(*w.pts.last().unwrap(), rootpt);
            if best.map_or(true, |b| d0 < b.2) { best = Some((i, false, d0)); }
            if best.map_or(true, |b| d1 < b.2) { best = Some((i, true, d1)); }
        }
        let (i0, rev0, d0) = best.unwrap();
        if d0 > 300.0 { return Err(format!("coast.oneline: no coastline way ends within 300 m of the root ({d0:.0} m)")); }
        let mut line: Vec<(f64, f64)> = Vec::new();
        let mut water_right = !rev0;
        let mut used = vec![false; coast.len()];
        let (mut cur, mut rev) = (i0, rev0);
        loop {
            used[cur] = true;
            let mut pts = coast[cur].pts.clone();
            if rev { pts.reverse(); }
            if line.is_empty() { line.extend(pts); } else { line.extend(pts.into_iter().skip(1)); }
            let end = *line.last().unwrap();
            // the next way sharing this end
            let mut next: Option<(usize, bool)> = None;
            for (j, w) in coast.iter().enumerate() {
                if used[j] { continue; }
                if dist(w.pts[0], end) < 2.0 { next = Some((j, false)); break; }
                if dist(*w.pts.last().unwrap(), end) < 2.0 { next = Some((j, true)); break; }
            }
            let Some((j, r)) = next else { break };
            cur = j; rev = r;
            let _ = &mut water_right;
        }
        if rev0 { water_right = false; } else { water_right = true; }
        // resample every dx_m up to length_m
        let mut transects = Vec::new();
        let mut acc = 0.0; let mut next_x = 0.0;
        let pl: Vec<(f64, f64)> = line.iter().map(|&(la, lo)| self.local(la, lo)).collect();
        for k in 0..pl.len() - 1 {
            let (a, b) = (pl[k], pl[k + 1]);
            let seg = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            if seg < 1e-6 { continue; }
            while next_x <= acc + seg && next_x <= self.length_m {
                let f = (next_x - acc) / seg;
                let (ex, ny) = (a.0 + f * (b.0 - a.0), a.1 + f * (b.1 - a.1));
                let tangent = bearing_deg(b.0 - a.0, b.1 - a.1);
                let normal = if water_right { (tangent + 90.0) % 360.0 } else { (tangent + 270.0) % 360.0 };
                let (lat, lon) = (self.root_lat + ny / M_PER_DEG, self.root_lon + ex / (M_PER_DEG * self.cos0));
                transects.push(Transect { lat, lon, ex, ny, x_m: next_x, tangent_deg: tangent, normal_deg: normal });
                next_x += self.dx_m;
            }
            acc += seg;
            if next_x > self.length_m { break; }
        }
        if transects.len() < 10 { return Err(format!("coast.oneline: only {} transects along {:.0} m of coast", transects.len(), acc)); }
        // smooth the normals over five transects: a mapped coastline is jagged at the scale of a transect
        let n = transects.len();
        let raw: Vec<f64> = transects.iter().map(|t| t.normal_deg).collect();
        for i in 0..n {
            let (mut sx, mut sy) = (0.0, 0.0);
            for j in i.saturating_sub(2)..(i + 3).min(n) { sx += raw[j].to_radians().sin(); sy += raw[j].to_radians().cos(); }
            transects[i].normal_deg = bearing_deg(sx, sy);
            transects[i].tangent_deg = (transects[i].normal_deg + if water_right { 270.0 } else { 90.0 }) % 360.0;
        }
        for i in 0..3.min(n.saturating_sub(4)) { transects[i].normal_deg = transects[3].normal_deg; transects[i].tangent_deg = transects[3].tangent_deg; }
        self.transects = transects;
        if self.y.len() != self.transects.len() { self.y = vec![0.0; self.transects.len()]; self.bluff = vec![0.0; self.transects.len()]; self.y_mark = self.y.clone(); }
        // the structures
        let mut st = Vec::new();
        if self.headland == "on" { st.push(Structure { name: "headland".into(), base: self.local(self.head_base_lat, self.head_base_lon), tip: self.local(self.head_tip_lat, self.head_tip_lon) }); }
        if self.breakwater == "on" { st.push(Structure { name: "breakwater".into(), base: self.local(self.root_lat, self.root_lon), tip: self.local(self.tip_lat, self.tip_lon) }); }
        self.structures = st;
        // roads, place names and city limits for the map; fetched once, carried by the describe from the cache
        if let Err(e) = esim_registry::wildfire::context(&mut fe, self.lat_s, self.lat_n, self.lon_w, self.lon_e) { eprintln!("context: {e} (the map will have no roads or names)"); }
        // the waves
        self.waves = Some(waves_hourly(&mut fe, self.wave_lat, self.wave_lon, self.year_from, self.year_to)?);
        // the direction offset: a half-degree cell's deep-water direction is not the direction at a
        // sheltered shore; the record says the bay was in equilibrium before the breakwater, so one angle
        // is fitted that brings the net transport along the open stretch of the line to zero
        if self.dir_bias.trim() == "fit" {
            let w = self.waves.as_ref().unwrap();
            let hours: Vec<usize> = (0..w.times.len().min(3 * 8760)).step_by(3).collect();
            let from = (self.fit_from_m / self.dx_m) as usize;
            let net = |bias: f64, me: &OneLine| -> f64 {
                let mut q = vec![0.0; me.transects.len() + 1];
                let mut sum = 0.0;
                for &h in &hours {
                    let (hs, dir) = (w.hs_m[h] as f64, w.dir_deg[h] as f64);
                    if !hs.is_finite() || !dir.is_finite() { continue; }
                    let tp = if w.tp_s[h].is_finite() { w.tp_s[h] as f64 } else { w.tm_s[h] as f64 };
                    me.transport(hs, tp, (dir + bias).rem_euclid(360.0), &mut q);
                    sum += q.iter().skip(from).sum::<f64>();
                }
                sum
            };
            let mut best = (0.0, f64::INFINITY);
            let mut b = -45.0;
            while b <= 45.0 { let v = net(b, self).abs(); if v < best.1 { best = (b, v); } b += 1.0; }
            let mut bb = best.0 - 1.0;
            while bb <= best.0 + 1.0 { let v = net(bb, self).abs(); if v < best.1 { best = (bb, v); } bb += 0.1; }
            self.dir_bias_fit = best.0;
        } else {
            self.dir_bias_fit = self.dir_bias.trim().parse().unwrap_or(0.0);
        }
        // the frame grid: the line's own box with a margin, at the frame resolution, so the map is the shore
        let d = self.frame_res_arcsec / 3600.0;
        let (mut la0, mut la1, mut lo0, mut lo1) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
        for t in &self.transects { la0 = la0.min(t.lat); la1 = la1.max(t.lat); lo0 = lo0.min(t.lon); lo1 = lo1.max(t.lon); }
        let m = self.frame_margin_m;
        let (la0, la1, lo0, lo1) = ((la0 - m / M_PER_DEG).max(self.lat_s), (la1 + m / M_PER_DEG).min(self.lat_n), (lo0 - m / (M_PER_DEG * self.cos0)).max(self.lon_w), (lo1 + m / (M_PER_DEG * self.cos0)).min(self.lon_e));
        self.grid = Some(LatLonGrid { nx: ((lo1 - lo0) / d).round().max(1.0) as usize, ny: ((la1 - la0) / d).round().max(1.0) as usize, dlon: d, dlat: d, lon_w: lo0, lat_n: la1 });
        Ok(())
    }

    /// The waves at a transect after the structures: (height factor, direction they come from).
    fn sheltered(&self, i: usize, from_deg: f64) -> (f64, f64) {
        let t = &self.transects[i];
        let (nx, ny) = (t.normal_deg.to_radians().sin(), t.normal_deg.to_radians().cos());
        let s = (t.ex + self.y[i] * nx, t.ny + self.y[i] * ny);
        let (mut kd, mut dir) = (1.0, from_deg);
        let (dx, dy) = (from_deg.to_radians().sin(), from_deg.to_radians().cos());
        for st in &self.structures {
            let (ux, uy) = (st.tip.0 - s.0, st.tip.1 - s.1);
            let len = (ux * ux + uy * uy).sqrt().max(1.0);
            let cosphi = ((ux * dx + uy * dy) / len).clamp(-1.0, 1.0);
            let phi = cosphi.acos().to_degrees();
            if ray_hits(s, from_deg, st.base, st.tip) {
                let k = 0.5 * (-phi / self.shadow_deg).exp();
                if k < kd { kd = k; dir = bearing_deg(ux, uy); }
            } else if cosphi > 0.0 {
                let k = 1.0 - 0.5 * (-phi / self.lit_deg).exp();
                if k < kd { kd = k; }
            }
        }
        (kd, dir)
    }

    /// The waves at the face between transects k-1 and k: the mean of the two transects' shelter.
    fn sheltered_face(&self, k: usize, from_deg: f64) -> (f64, f64) {
        let (k1, d1) = self.sheltered(k.saturating_sub(1), from_deg);
        let (k2, d2) = self.sheltered(k.min(self.transects.len() - 1), from_deg);
        let (sx, sy) = (d1.to_radians().sin() + d2.to_radians().sin(), d1.to_radians().cos() + d2.to_radians().cos());
        (0.5 * (k1 + k2), bearing_deg(sx, sy))
    }

    /// Longshore transport this hour at every face between transects (q[k] lies between transects
    /// k-1 and k; q[0] and q[n] are the ends), m3/s, positive toward +x, away from the breakwater.
    /// Staggered on purpose: a transport computed at the transects from a centred slope lets odd and
    /// even transects drift apart (the checkerboard), which the first runs did.
    fn transport(&self, hs: f64, tp: f64, from_deg: f64, q: &mut [f64]) -> f64 {
        let n = self.transects.len();
        let coef = self.k_cerc * 1025.0 * G.sqrt() / (16.0 * self.gamma.sqrt() * (2650.0 - 1025.0) * 0.6);
        let c0 = G * tp.max(2.0) / (2.0 * std::f64::consts::PI);
        let mut eps_max: f64 = 0.0;
        q[0] = 0.0; q[n] = 0.0;
        for k in 1..n {
            let (kd, dir) = self.sheltered_face(k, from_deg);
            let h0 = hs * kd;
            if h0 < 0.05 { q[k] = 0.0; continue; }
            let (a, b) = (&self.transects[k - 1], &self.transects[k]);
            let base = bearing_deg(a.normal_deg.to_radians().sin() + b.normal_deg.to_radians().sin(), a.normal_deg.to_radians().cos() + b.normal_deg.to_radians().cos());
            let tangent = bearing_deg(a.tangent_deg.to_radians().sin() + b.tangent_deg.to_radians().sin(), a.tangent_deg.to_radians().cos() + b.tangent_deg.to_radians().cos());
            // the seaward normal turns away from +x where the line advances along +x: a bulge faces back
            let slope = (self.y[k] - self.y[k - 1]) / self.dx_m;
            let plus_clockwise = wrap180(tangent - base) > 0.0;
            let normal = base - slope.atan().to_degrees() * if plus_clockwise { 1.0 } else { -1.0 };
            // the angle to the mapped shore decides how much of the wave the face sees: beyond alpha_cap the
            // transport is tapered by the cosine to nothing at grazing incidence, because a face that the
            // waves run along is in the shadow of its own coast. The angle to the moving line, held within
            // the cap, carries the feedback that keeps the equation diffusive: beyond the cap the one-line
            // equation is anti-diffusive (Ashton and Murray's high-angle instability, recalled) and the first
            // runs of this module showed what that does to a line of transects
            let alpha_base = wrap180(dir - base);
            if alpha_base.abs() >= 90.0 { q[k] = 0.0; continue; }
            let taper = if alpha_base.abs() > self.alpha_cap_deg { alpha_base.to_radians().cos() / self.alpha_cap_deg.to_radians().cos() } else { 1.0 };
            let alpha0 = wrap180(dir - normal).clamp(-self.alpha_cap_deg, self.alpha_cap_deg);
            let hb = 0.39 * G.powf(0.2) * (tp.max(2.0) * h0 * h0).powf(0.4);
            let cb = (G * hb / self.gamma).sqrt();
            let sinab = ((cb / c0) * alpha0.to_radians().sin()).clamp(-1.0, 1.0);
            let ab = sinab.asin();
            // waves from the +x side (alpha0 > 0 when +x is clockwise from the normal) drive sand toward -x
            let sign = if plus_clockwise { -1.0 } else { 1.0 };
            q[k] = sign * coef * hb.powf(2.5) * (2.0 * ab).sin() * taper;
            let eps = 2.0 * coef * hb.powf(2.5) * (2.0 * ab).cos().abs() / (self.d_closure_m + self.berm_m);
            if eps > eps_max { eps_max = eps; }
        }
        eps_max
    }

    fn hour(&mut self, hs: f64, tp: f64, from_deg: f64) {
        let n = self.transects.len();
        let d = self.d_closure_m + self.berm_m;
        let mut f = vec![0.0; n + 1];
        let eps = self.transport(hs, tp, from_deg, &mut f);
        let sub = ((3600.0 * eps / (0.4 * self.dx_m * self.dx_m)).ceil().max(1.0) as usize).min(24);
        let dt = 3600.0 / sub as f64;
        let bw_on = self.breakwater == "on";
        for _ in 0..sub {
            if sub > 1 { self.transport(hs, tp, from_deg, &mut f); }
            // the ends: the leak, a share of the sand moving along the first stretch of beach, either way;
            // and the far end, closed at a point or open
            let m = (self.leak_reach_m / self.dx_m).round().max(1.0) as usize;
            let gross = f.iter().skip(1).take(m.min(n - 1)).map(|v| v.abs()).sum::<f64>() / m.min(n - 1) as f64;
            f[0] = if bw_on { -self.leak_frac * gross } else { f[1] };
            f[n] = if self.far_end == "closed" { 0.0 } else { f[n - 1] };
            self.gross_root += gross * dt;
            // move the line
            for i in 0..n {
                let dv = -(f[i + 1] - f[i]) * dt / self.dx_m;       // m3 per m gained
                let toe = -self.beach_width_m - self.bluff[i];
                let mut dy = dv / d;
                if dv < 0.0 && self.y[i] + dy < toe {
                    // the beach is gone: the bluff retreats with the line and yields its sand
                    let dsupply = self.bluff_height_m * self.bluff_sand_frac;
                    let need = (-dv) - (self.y[i] - toe).max(0.0) * d;   // the part of the loss the bluff takes
                    let r = need / (d + dsupply);
                    dy = -(self.y[i] - toe).max(0.0) - r;
                    self.bluff[i] += r;
                    self.pending[3] += r * dsupply * self.dx_m;
                }
                self.y[i] += dy;
            }
            if bw_on { self.pending[0] += -f[0] * dt; } else { self.pending[5] += -f[0] * dt; }   // into the harbor through the voids, or, before it was built, along the coast to the north
            self.pending[1] += f[n] * dt;       // past the far end
        }
        if self.slr_m_yr > 0.0 && self.slr_on {
            let dy = -self.slr_m_yr / 8760.0 * self.closure_distance_m / d;
            for i in 0..n { self.y[i] += dy; }
            self.pending[2] += -dy * d * self.dx_m * n as f64;
        }
        self.hs_sum += hs; self.hs_n += 1.0;
    }

    fn dredge(&mut self, year: i32) {
        if self.dredge_every_years <= 0.0 || year < self.dredge_from_year { return; }
        if self.last_dredge_year != 0 && (year - self.last_dredge_year) < self.dredge_every_years as i32 { return; }
        let v = self.dredge_m3.min(self.harbor_m3 + self.pending[0]);
        if v <= 0.0 { return; }
        let idx: Vec<usize> = (0..self.transects.len()).filter(|&i| self.transects[i].x_m <= self.place_length_m).collect();
        let d = self.d_closure_m + self.berm_m;
        let dy = v / (idx.len() as f64 * self.dx_m * d);
        for i in idx { self.y[i] += dy; }
        self.pending[4] += v;
        self.last_dredge_year = year;
    }

    fn zone_mean(&self, lo: f64, hi: f64) -> f64 {
        let v: Vec<f64> = self.transects.iter().zip(&self.y).filter(|(t, _)| t.x_m >= lo && t.x_m < hi).map(|(_, &y)| y).collect();
        if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 }
    }

    /// Two frames: `shore_change_m`, the strip between today's line and the line now, painted with
    /// the change (a cell is at least painted where the line now runs, so a metre shows); and
    /// `shore_line`, the line now (1) and today's line (0.4), for the eye.
    fn frame(&self, ctx: &mut Ctx) {
        let g = self.grid.clone().unwrap();
        let n = self.transects.len();
        let mut change = vec![f32::NAN; g.nx * g.ny];
        let mut line = vec![f32::NAN; g.nx * g.ny];
        let cell_m = self.frame_res_arcsec / 3600.0 * M_PER_DEG;
        let half = 0.75 * cell_m;
        for r in 0..g.ny { for c in 0..g.nx {
            let (lat, lon) = (g.lat(r), g.lon(c));
            let (ex, ny) = self.local(lat, lon);
            // the nearest transect by distance to its base point
            let mut best = (f64::INFINITY, 0usize);
            for (i, t) in self.transects.iter().enumerate() { let dd = (t.ex - ex).powi(2) + (t.ny - ny).powi(2); if dd < best.0 { best = (dd, i); } }
            let i = best.1;
            let t = &self.transects[i];
            // the cell's offset along the transect's normal (seaward +) and along the shore
            let (nx, nyy) = (t.normal_deg.to_radians().sin(), t.normal_deg.to_radians().cos());
            let off = (ex - t.ex) * nx + (ny - t.ny) * nyy;
            let along = ((ex - t.ex) * nyy - (ny - t.ny) * nx).abs();
            if along > self.dx_m * 0.75 && i != 0 && i != n - 1 { continue; }
            let y = self.y[i];
            let k = r * g.nx + c;
            let (lo, hi) = (y.min(0.0), y.max(0.0));
            if (off >= lo - half && off <= hi + half) || (off - y).abs() <= half { change[k] = y as f32; }
            if off.abs() <= half { line[k] = 0.4; }
            if (off - y).abs() <= half { line[k] = 1.0; }
        } }
        ctx.field("shore_change_m", "m", &g, change);
        ctx.field("shore_line", "1", &g, line);
    }
}

impl Module for OneLine {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        let now = ctx.now();
        let d = self.d_closure_m + self.berm_m;
        ctx.ledger(SAND, "m3", 1.0);
        if !self.started {
            let beach = self.beach_width_m * d * self.dx_m * self.transects.len() as f64;
            ctx.declare_stock(SAND, "beach", beach);
            ctx.declare_stock(SAND, "bluff", 1e9);
            ctx.declare_stock(SAND, "harbor", 0.0);
            ctx.declare_stock(SAND, "downcoast", 0.0);
            ctx.declare_stock(SAND, "offshore", 0.0);
            ctx.declare_stock(SAND, "upcoast", 0.0);
            self.t_start = now.0; self.t_mark = now.0; self.last_report = now.0; self.last_frame = now.0;
            ctx.schedule_with_priority(self.dt, 5, "tick", json!(null));
            self.started = true;
        }
        let w = self.waves.as_ref().unwrap();
        ctx.publish("coast.loaded", json!({ "transects": self.transects.len(), "dx_m": self.dx_m, "length_m": self.transects.last().map(|t| t.x_m), "structures": self.structures.iter().map(|s| s.name.clone()).collect::<Vec<_>>(), "wave_cell": [w.cell_lat, w.cell_lon], "wave_hours": w.times.len(), "years": [self.year_from, self.year_to], "dir_bias_deg": self.dir_bias_fit }));
        self.frame(ctx);
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" { return; }
        let now = ctx.now();
        let t0 = now.0 - self.dt.0;
        // the hours of this step, by the record's clock; a year outside the record repeats its last year
        let w = self.waves.take().unwrap();
        let span = w.times.last().copied().unwrap_or(0) - w.times[0] + 3600;
        let mut t = t0;
        while t < now.0 {
            let mut tt = t;
            if span > 0 { while tt > w.times.last().copied().unwrap_or(tt) { tt -= 365 * 86400; } while tt < w.times[0] { tt += 365 * 86400; } }
            if let Some(i) = w.index_at(tt) {
                let (hs, tp, dir) = (w.hs_m[i] as f64, if w.tp_s[i].is_finite() { w.tp_s[i] as f64 } else { w.tm_s[i] as f64 }, w.dir_deg[i] as f64);
                if hs.is_finite() && dir.is_finite() { self.hour(hs, tp, (dir + self.dir_bias_fit).rem_euclid(360.0)); }
            }
            t += 3600;
        }
        self.waves = Some(w);
        // a new calendar year: the dredge
        let year = |t: i64| Time(t).to_iso()[..4].parse::<i32>().unwrap_or(0);
        self.slr_on = year(now.0) >= self.slr_from_year;
        if year(now.0) != year(t0) { self.dredge(year(now.0)); }
        // the books
        let p = self.pending; self.pending = [0.0; 6];
        if p[0] != 0.0 { ctx.transfer(SAND, "beach", "harbor", p[0]); self.harbor_m3 += p[0]; }
        if p[1] != 0.0 { ctx.transfer(SAND, "beach", "downcoast", p[1]); self.downcoast_m3 += p[1]; }
        if p[2] != 0.0 { ctx.transfer(SAND, "beach", "offshore", p[2]); self.offshore_m3 += p[2]; }
        if p[3] != 0.0 { ctx.transfer(SAND, "bluff", "beach", p[3]); self.bluff_m3 += p[3]; }
        if p[5] != 0.0 { ctx.transfer(SAND, "beach", "upcoast", p[5]); self.upcoast_m3 += p[5]; }
        if p[4] != 0.0 { ctx.transfer(SAND, "harbor", "beach", p[4]); self.harbor_m3 -= p[4]; self.placed_m3 += p[4]; ctx.publish("coast.dredged", json!({ "m3": p[4], "time": now.to_iso(), "placed_along_m": self.place_length_m })); }
        let d = self.d_closure_m + self.berm_m;
        let beach: f64 = self.y.iter().map(|y| (self.beach_width_m + y) * d * self.dx_m).sum::<f64>();
        ctx.declare_stock(SAND, "beach", beach);
        ctx.declare_stock(SAND, "harbor", self.harbor_m3);
        ctx.declare_stock(SAND, "downcoast", self.downcoast_m3);
        ctx.declare_stock(SAND, "offshore", self.offshore_m3);
        ctx.declare_stock(SAND, "upcoast", self.upcoast_m3);
        ctx.declare_stock(SAND, "bluff", 1e9 - self.bluff_m3);
        // the reports
        if now.0 - self.last_report >= self.report_every.0 {
            self.last_report = now.0;
            let yrs = ((now.0 - self.t_mark) as f64 / (365.25 * 86400.0)).max(1e-6);
            let near_now: f64 = self.transects.iter().zip(&self.y).filter(|(t, _)| t.x_m < self.near_m).map(|(_, &y)| y * d * self.dx_m).sum();
            let near_then: f64 = self.transects.iter().zip(&self.y_mark).filter(|(t, _)| t.x_m < self.near_m).map(|(_, &y)| y * d * self.dx_m).sum();
            let erosion_rate = (near_now - near_then) / yrs;      // m3/yr, negative = loss
            let harbor_rate = (self.harbor_m3 - self.harbor_mark) / yrs;
            ctx.report("coast.dir_bias_deg", "deg", self.dir_bias_fit);
            ctx.report("coast.hs_mean_m", "m", if self.hs_n > 0.0 { self.hs_sum / self.hs_n } else { 0.0 });
            ctx.report("coast.harbor_sand_m3", "m3", self.harbor_m3);
            ctx.report("coast.harbor_rate_m3_yr", "m3/yr", harbor_rate);
            ctx.report("coast.near_change_m3_yr", "m3/yr", erosion_rate);
            ctx.report("coast.gross_root_m3_yr", "m3/yr", self.gross_root / ((now.0 - self.t_start) as f64 / (365.25 * 86400.0)).max(1e-6));
            let names = ["surfers", "vallejo", "miramar", "south"];
            for k in 0..4 { ctx.report(&format!("coast.shore_{}_m", names[k]), "m", self.zone_mean(self.zones_m[k], self.zones_m[k + 1])); }
            ctx.report("coast.bluff_max_m", "m", self.bluff.iter().cloned().fold(0.0, f64::max));
            ctx.report("coast.bluff_mean_near_m", "m", { let v: Vec<f64> = self.transects.iter().zip(&self.bluff).filter(|(t, _)| t.x_m < self.near_m).map(|(_, &b)| b).collect(); v.iter().sum::<f64>() / v.len().max(1) as f64 });
            ctx.report("coast.downcoast_m3", "m3", self.downcoast_m3);
            ctx.report("coast.upcoast_m3", "m3", self.upcoast_m3);
            ctx.report("coast.placed_m3", "m3", self.placed_m3);
            ctx.report("fit.near_erosion_ratio", "1", -erosion_rate / 3211.0);
            ctx.report("fit.harbor_ratio", "1", harbor_rate / 1529.0);
            // a yearly window for the rates
            if now.0 - self.t_mark >= 365 * 86400 { self.t_mark = now.0; self.y_mark = self.y.clone(); self.harbor_mark = self.harbor_m3; }
            self.hs_sum = 0.0; self.hs_n = 0.0;
        }
        if now.0 - self.last_frame >= self.frame_every.0 { self.last_frame = now.0; self.frame(ctx); }
        ctx.schedule_with_priority(self.dt, 5, "tick", json!(null));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: OneLine = serde_json::from_value(v).unwrap(); o.load().expect("coast reload"); *self = o; }
}

fn record_rows() -> serde_json::Value {
    let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let text = std::fs::read_to_string(root.join("data/events/halfmoonbay/record.toml")).unwrap_or_default();
    let v: toml::Value = toml::from_str(&text).unwrap_or(toml::Value::Table(Default::default()));
    json!(v.get("fact").and_then(|f| f.as_array()).map(|a| a.iter().map(|r| json!({
        "id": r.get("id").and_then(|x| x.as_str()), "title": r.get("title").and_then(|x| x.as_str()), "value": r.get("value").and_then(|x| x.as_float().or_else(|| x.as_integer().map(|i| i as f64))),
        "unit": r.get("unit").and_then(|x| x.as_str()), "source": r.get("source").and_then(|x| x.as_str()), "confidence": r.get("confidence").and_then(|x| x.as_str()) })).collect::<Vec<_>>()).unwrap_or_default())
}

impl OneLine {
    pub fn describe(name: &str, p: &toml::Value) -> esim_kernel::describe::Describe {
        // roads and names for the map, from the cache if a run has fetched them
        let (la_s, la_n, lo_w, lo_e) = (param_f64(p, "lat_s", 37.42), param_f64(p, "lat_n", 37.52), param_f64(p, "lon_w", -122.52), param_f64(p, "lon_e", -122.42));
        let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let ctx_table = Fetcher::default_for(&root).ok().and_then(|f| esim_registry::wildfire::cached_context(&f, la_s, la_n, lo_w, lo_e));
        let mut d = esim_kernel::describe::Describe::new("coast.oneline", name, "A one-line shoreline model for a bay behind a headland and a breakwater: today's coast as a line of transects; each hour the reanalysis cell's deep-water waves carried to each transect through the structures' shadows (diffracted, lower and from the tip inside a shadow), shoaled and refracted to breaking, and turned into longshore transport by the CERC formula; the transport's divergence moves the line, the line's own angle feeds back, the bluff retreats and yields sand where the beach is gone, a declared fraction of the sand moving past the breakwater's root is lost through its voids into the harbor, sea level rise moves the line by Bruun's rule, and a dredge every so many years puts harbor sand back on the first stretch of beach. Not a fluid dynamics model: no currents, no cross-shore profile, no setup, no reflection. Every coefficient is a published value recalled or a guess, declared so; the record it is read against is the Corps' and Griggs's numbers in the table below.")
            .priority(5).dt(&format!("{} (the waves hour by hour within it)", param_str(p, "dt", "1d")))
            .param("lat_s", 37.42, "deg", "").param("lat_n", 37.52, "deg", "").param("lon_w", -122.52, "deg", "").param("lon_e", -122.42, "deg", "").param("res_arcsec", 1.0, "arcsec", "").param("frame_res_arcsec", 0.5, "arcsec", "the frames: about 15 m, so a decade of retreat is a few cells").param("frame_margin_m", 600.0, "m", "the frames cover the line's own box plus this, not the whole window")
            .param("root_lat", 37.49995, "deg", "where the East Breakwater meets the shore: the first node of OpenStreetMap coastline way 638337850 (2026-11-08); the line of transects starts here and runs away from it").param("root_lon", -122.47432, "deg", "")
            .param("length_m", 8000.0, "m", "how far along the coast the line runs: 8 km stops short of Miramontes Point, where the mapped coast turns").param("dx_m", 50.0, "m", "transect spacing")
            .param("breakwater", "on", "", "on: the East Breakwater shelters, from its root to its tip, and leaks sand into the harbor; off: the coast before 1959").param("tip_lat", 37.49522, "deg", "the breakwater's seaward end, OpenStreetMap coastline node (2026-11-08)").param("tip_lon", -122.48573, "deg", "")
            .param("headland", "on", "", "Pillar Point as a sheltering segment from its base to its tip").param("head_base_lat", 37.5031, "deg", "OpenStreetMap, the point's eastern base").param("head_base_lon", -122.4851, "deg", "").param("head_tip_lat", 37.4927, "deg", "Sail Rock, the point's southern tip (OpenStreetMap)").param("head_tip_lon", -122.5005, "deg", "")
            .param("wave_lat", 37.45, "deg", "the reanalysis answers for the half-degree cell that holds the point (37.5, -122.5)").param("wave_lon", -122.55, "deg", "").param("year_from", 1961.0, "", "first year of waves fetched").param("year_to", 2024.0, "", "last; a run past it repeats the last year")
            .param("k_cerc", 0.39, "", "the CERC coefficient for significant wave height (Shore Protection Manual 1984, recalled); 0.2 to 0.4 in use on open coasts, and the number every one-line model is calibrated through. Here it also absorbs what the model lacks: the reanalysis cell's height is the open ocean's, and the point's refraction and the shelf's losses are not modelled, so a sheltered bay fits far below the open-coast value; the manifest says what it fitted").param("gamma", 0.78, "", "breaker index, recalled").param("d_closure_m", 8.0, "m", "depth of closure, a guess for this coast").param("berm_m", 2.0, "m", "berm height, a guess")
            .param("beach_width_m", 20.0, "m", "guess: the beach in front of the bluff today, taken uniform; where the line retreats past it the bluff retreats").param("bluff_height_m", 10.0, "m", "guess: the low marine terrace bluff south of the harbor").param("bluff_sand_frac", 0.5, "", "guess: the sand fraction of the bluff that stays on the beach")
            .param("shadow_deg", 25.0, "deg", "guess: inside a structure's shadow the height factor is 0.5 at the shadow line falling as exp(-angle/this) (the shape of the Penney and Price solution, rounded)").param("lit_deg", 8.0, "deg", "guess: on the lit side the factor rises from 0.5 at the line with this scale")
            .param("alpha_cap_deg", 45.0, "deg", "beyond this deep-water angle to the shore normal the transport is the cap's value tapered by the cosine to nothing at grazing; beyond the cap the one-line equation is anti-diffusive (Ashton and Murray's high-angle instability, recalled) and a line of transects cannot carry what follows, a spit").param("leak_frac", 0.02, "", "the one fitted number: the share of the longshore transport at the breakwater's root, either way, that passes through the rubble mound's voids into the harbor; a guess to be read against the Corps' harbor accretion")
            .param("slr_m_yr", 0.0, "m/yr", "sea level rise from slr_from_year; 0.0044 is the Corps' intermediate 50-year case, 0.0126 its high case (0.71 and 2.06 ft in 50 years)").param("slr_from_year", 2025.0, "", "").param("closure_distance_m", 500.0, "m", "Bruun's rule: retreat = rise times this over the profile height; a guess")
            .param("dredge_every_years", 0.0, "yr", "0: no dredging; otherwise harbor sand is moved to the first place_length_m of beach every this many years from dredge_from_year").param("dredge_m3", 76_000.0, "m3", "the District's pilot: up to 100,000 cubic yards").param("dredge_from_year", 2025.0, "", "").param("place_length_m", 305.0, "m", "1,000 ft of Surfers Beach, the pilot's placement")
            .param("dir_bias", "fit", "deg or fit", "an angle added to the reanalysis cell's wave directions. fit: chosen at the start so that the net transport along the line beyond fit_from_m, over the first three years of waves, is zero, because the record says the bay was in equilibrium before the breakwater and a 50 km cell's deep-water direction is not the direction at a sheltered shore; the fitted angle is reported as coast.dir_bias_deg. A number uses that number").param("fit_from_m", 3000.0, "m", "the open stretch the fit is made on, beyond the breakwater's shadow")
            .param("far_end", "open", "", "open: the transport at the last face passes the end of the line, to or from the coast beyond it toward Miramontes Point (booked as downcoast); closed: a point, nothing passes").param("leak_reach_m", 300.0, "m", "the leak is a share of the mean transport, either way, over this first stretch of beach")
            .param("near_m", 1400.0, "m", "the stretch the near-breakwater erosion rate is summed over: Griggs's 1.4 km")
            .param("dt", "1d", "", "").param("report_every", "30d", "", "").param("frame_every", "365d", "", "")
            .publish("coast.loaded", "{ transects, dx_m, length_m, structures, wave_cell, wave_hours, years, dir_bias_deg }").publish("coast.dredged", "{ m3, time, placed_along_m }")
            .ledger(SAND, "m3", 1.0)
            .frame("shore_change_m", "m").frame("shore_line", "1")
            .report("coast.dir_bias_deg", "deg").report("coast.hs_mean_m", "m").report("coast.harbor_sand_m3", "m3").report("coast.harbor_rate_m3_yr", "m3/yr").report("coast.near_change_m3_yr", "m3/yr").report("coast.gross_root_m3_yr", "m3/yr")
            .report("coast.shore_surfers_m", "m").report("coast.shore_vallejo_m", "m").report("coast.shore_miramar_m", "m").report("coast.shore_south_m", "m")
            .report("coast.bluff_max_m", "m").report("coast.bluff_mean_near_m", "m").report("coast.downcoast_m3", "m3").report("coast.upcoast_m3", "m3").report("coast.placed_m3", "m3")
            .report("fit.near_erosion_ratio", "1").report("fit.harbor_ratio", "1")
            .asset("open-meteo-era5-ocean-waves", "hourly significant wave height, direction and period at the cell").asset("osm-overpass", "the coastline and the breakwater")
            .table("record", record_rows())
            .view("map", "The line", "shore_line", "the shoreline now, dark, and today's line, grey; about 15 m a cell").view("map", "Shoreline change", "shore_change_m", "the strip between today's line and the line now: metres landward (-) or seaward (+)")
            .view("chart", "Surfers Beach: the line", "coast.shore_surfers_m", "mean change of the first 700 m, metres").view("chart", "Vallejo Beach: the line", "coast.shore_vallejo_m", "700 to 1,400 m from the breakwater")
            .view("chart", "Sand in the harbor", "coast.harbor_sand_m3", "through the breakwater's voids, cumulative").view("chart", "Erosion within 1.4 km of the breakwater", "coast.near_change_m3_yr", "m3 a year over the past year; the Corps' figure is a loss of 3,211 m3 a year (4,200 CY)")
            .view("chart", "Bluff retreat, worst transect", "coast.bluff_max_m", "metres since the start").view("chart", "Waves at the cell", "coast.hs_mean_m", "30-day mean significant height")
            .note("Zones: surfers 0 to 700 m from the root, vallejo 700 to 1,400 m, miramar 1,400 to 3,000 m, south beyond; the record's places (the Highway 1 and Mirada Road revetments) are not placed exactly.")
            .note("The shoreline is today's, revetments and all; a run from 1961 on it measures rates, not positions.");
        if let Some(c) = ctx_table { d = d.table("context", c); }
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// One hour of a typical westerly swell on the as-built coast, transect by transect. Needs the cache.
    #[test]
    fn a_westerly_hour_transect_by_transect() {
        let root: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", ".."].iter().collect();
        if !root.join("data/cache").exists() { eprintln!("skipped"); return; }
        std::env::set_var("ESIM_ROOT", &root);
        let p: toml::Value = toml::from_str("year_from = 1993\nyear_to = 1993").unwrap();
        let o = OneLine::new("coast", &p).expect("build");
        let n = o.transects.len();
        let mut q = vec![0.0; n + 1];
        let eps = o.transport(1.65, 10.0, 280.0, &mut q);
        eprintln!("eps {eps:.3}");
        for i in (0..n).step_by(5) {
            let t = &o.transects[i];
            let (kd, dir) = o.sheltered(i, 280.0);
            eprintln!("{i:3} x {:5.0} lat {:.4} lon {:.4} normal {:4.0} kd {kd:.2} dir {dir:4.0} q {:8.4}", t.x_m, t.lat, t.lon, t.normal_deg, q[i]);
        }
        assert!(q.iter().any(|v| *v != 0.0));
    }
    /// Sixty days of real hours, then the line transect by transect: where does it run away?
    #[test]
    fn sixty_days_transect_by_transect() {
        let root: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", ".."].iter().collect();
        if !root.join("data/cache").exists() { eprintln!("skipped"); return; }
        std::env::set_var("ESIM_ROOT", &root);
        let p: toml::Value = toml::from_str("year_from = 1961\nyear_to = 1961").unwrap();
        let mut o = OneLine::new("coast", &p).expect("build");
        let w = o.waves.take().unwrap();
        for h in 0..(60 * 24) {
            let (hs, dir) = (w.hs_m[h] as f64, w.dir_deg[h] as f64);
            let tp = if w.tp_s[h].is_finite() { w.tp_s[h] as f64 } else { w.tm_s[h] as f64 };
            if h % 240 == 0 { eprintln!("day {:3} hs {hs:.2} dir {dir:.0}  y[0..8] {:?}  bluff max {:.1} at {:?}", h / 24, &o.y[0..8].iter().map(|v| (*v * 10.0).round() / 10.0).collect::<Vec<_>>(), o.bluff.iter().cloned().fold(0.0, f64::max), o.bluff.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(i, _)| i)); }
            o.hour(hs, tp, (dir + o.dir_bias_fit).rem_euclid(360.0));
        }
        let n = o.transects.len();
        for i in (0..n).step_by(4) { eprintln!("{i:3} x {:5.0} normal {:4.0} y {:9.1} bluff {:9.1}", o.transects[i].x_m, o.transects[i].normal_deg, o.y[i], o.bluff[i]); }
    }
}
