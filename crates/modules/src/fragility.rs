//! What the subject modules that break things share: a fragility curve read from an event folder's
//! `fragility.toml`, the draw of a damage state from the ground at a cell, and the small readers for
//! the folder's tables. Not a module. Until 2026-11-14 this lived inside `hazard.components`, which
//! held tanks, bridges, power structures and road segments in one kind because they share this
//! method; they are now `fuel.tanks`, `roads.bridges`, `power.structures` and `roads.routes`.
use crate::quake::{phi, root};
use esim_kernel::{Ctx, LatLonGrid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lognormal exceedance medians for four damage states (slight to complete) in g, a ground-failure
/// median in cm of permanent deformation, days to restore by state, and the state at which the
/// thing is out of service.
#[derive(Clone, Serialize, Deserialize, Default)]
pub(crate) struct Curve { pub medians: Vec<f64>, pub beta: f64, pub gf_median: f64, pub gf_beta: f64, pub restore: Vec<i64>, pub fail_state: u8 }

pub(crate) fn tf(t: &toml::Value, k: &str) -> f64 { t.get(k).and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64))).unwrap_or(0.0) }
pub(crate) fn ts(t: &toml::Value, k: &str) -> String { t.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string() }
pub(crate) fn tarr(t: &toml::Value, k: &str) -> Vec<f64> { t.get(k).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_float().or_else(|| x.as_integer().map(|i| i as f64))).collect()).unwrap_or_default() }
pub(crate) fn or(s: String, d: &str) -> String { if s.is_empty() { d.to_string() } else { s } }

pub(crate) fn table(folder: &str, file: &str) -> Result<toml::Value, String> {
    let path = root().join("data/events").join(folder).join(file);
    std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?.parse::<toml::Value>().map_err(|e| format!("{file}: {e}"))
}
pub(crate) fn csv(folder: &str, file: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let path = root().join("data/events").join(folder).join(file);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut lines = text.lines();
    let head: Vec<String> = lines.next().ok_or("empty csv")?.split(',').map(|s| s.trim().to_string()).collect();
    Ok((head, lines.filter(|l| !l.trim().is_empty()).map(|l| l.split(',').map(|s| s.trim().to_string()).collect()).collect()))
}
pub(crate) fn curve(t: &toml::Value) -> Curve {
    Curve { medians: tarr(t, "medians_g"), beta: tf(t, "beta"), gf_median: tf(t, "gf_median_cm"), gf_beta: tf(t, "gf_beta"), restore: tarr(t, "restore_days").iter().map(|&d| d as i64).collect(), fail_state: tf(t, "fail_state").max(tf(t, "closed_state")) as u8 }
}

/// The ground module's fields as they stand at the rupture.
pub(crate) struct Shaking { pga: Vec<f32>, liq_p: Vec<f32>, liq_pgd: Vec<f32>, ls_p: Vec<f32>, ls_pgd: Vec<f32> }
impl Shaking {
    pub(crate) fn read(ctx: &Ctx, n: usize) -> Shaking {
        let take = |name: &str| -> Vec<f32> { ctx.world_read(name).map(|f| f.data.clone()).unwrap_or_else(|| vec![f32::NAN; n]) };
        Shaking { pga: take("quake.pga_g"), liq_p: take("quake.liq_p"), liq_pgd: take("quake.liq_pgd_cm"), ls_p: take("quake.ls_p"), ls_pgd: take("quake.ls_pgd_cm") }
    }
    /// The damage state (0 none to 4 complete) of a thing with this curve at this cell, by the draw `u`.
    pub(crate) fn draw(&self, c: &Curve, cell: usize, u: f64) -> u8 {
        let at = |v: &Vec<f32>| v.get(cell).copied().unwrap_or(f32::NAN);
        let (pga, liq_p, liq_pgd, ls_p, ls_pgd) = (at(&self.pga), at(&self.liq_p), at(&self.liq_pgd), at(&self.ls_p), at(&self.ls_pgd));
        if pga.is_nan() || pga <= 0.0 || c.medians.len() < 4 { return 0; }
        let a = pga as f64;
        let ex: Vec<f64> = c.medians.iter().map(|m| phi((a / m).ln() / c.beta)).collect();
        let gf = |p: f32, pgd: f32| -> f64 { if p.is_nan() || pgd.is_nan() || p <= 0.0 || pgd <= 0.0 || c.gf_median <= 0.0 { 0.0 } else { p as f64 * phi((pgd as f64 / c.gf_median).ln() / c.gf_beta) } };
        let pg = 1.0 - (1.0 - gf(liq_p, liq_pgd)) * (1.0 - gf(ls_p, ls_pgd));
        // ground failure takes any state to complete; otherwise the shaking's state by the draw
        if u < pg { return 4; }
        let v = (u - pg) / (1.0 - pg).max(1e-9);
        let mut s = 0u8; for (k, e) in ex.iter().enumerate() { if v < *e { s = k as u8 + 1; } }
        s
    }
}

/// `force = ["kind:id:state"]`: the entries for one kind, as id to state.
pub(crate) fn forced(list: &[String], kind: &str) -> HashMap<String, u8> {
    list.iter().filter_map(|s| { let p: Vec<&str> = s.splitn(3, ':').collect(); if p.len() == 3 && p[0] == kind { Some((p[1].to_string(), p[2].parse().unwrap_or(4))) } else { None } }).collect()
}
pub(crate) fn force_param(p: &toml::Value) -> Vec<String> { p.get("force").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default() }

/// A grid whose cells are `coarse_arcsec` across, over the same window.
pub(crate) fn coarse_of(g: &LatLonGrid, coarse_arcsec: f64, res_arcsec: f64) -> (LatLonGrid, usize) {
    let step = (coarse_arcsec / res_arcsec).round().max(1.0) as usize;
    (LatLonGrid { nx: (g.nx + step - 1) / step, ny: (g.ny + step - 1) / step, dlon: g.dlon * step as f64, dlat: g.dlat * step as f64, lon_w: g.lon_w, lat_n: g.lat_n }, step)
}
