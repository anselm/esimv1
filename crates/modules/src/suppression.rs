//! Interventions, as a module of their own: what people did to the fire, taken from the record
//! and turned into something any fire engine understands. The record (`obs.record`) says where a
//! line was cut and when the GIS unit drew it; this module turns each completed line into a
//! `ground.break`, the cells under it to a stated width, of a stated strength, from that time on.
//! A fire engine that subscribes to `ground.break` refuses heat and embers on those cells (or, for
//! Rothermel, treats them as unburnable); it never reads the record itself, so the same engine runs
//! another fire with another record, or with none. Resources as agents (engines, dozers, aircraft
//! placed by rule from the reports' counts, held in a ledger) are the next form; this one is the
//! lines as the record has them, nothing more.
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Event, LatLonGrid, Module, Time};
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Suppression {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    mode: String,            // off | hold
    line_width_m: f64,
    strength: f64,
    category_prefix: String,
    skip_status: String,
    lines: usize, cells: usize,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] metres: Vec<(f64, f64)>,
    #[serde(skip)] held: Vec<bool>,
}

impl Suppression {
    pub fn new(name: &str, p: &toml::Value) -> Result<Suppression, String> {
        let mut o = Suppression {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 34.0), lat_n: param_f64(p, "lat_n", 34.16), lon_w: param_f64(p, "lon_w", -118.72), lon_e: param_f64(p, "lon_e", -118.46),
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            mode: param_str(p, "mode", "off").to_string(),
            line_width_m: param_f64(p, "line_width_m", 30.0),
            strength: param_f64(p, "strength", 1.0),
            category_prefix: param_str(p, "category_prefix", "Completed").to_string(),
            skip_status: param_str(p, "skip_status", "Archive").to_string(),
            lines: 0, cells: 0,
            grid: None, metres: Vec::new(), held: Vec::new(),
        };
        if o.mode != "off" { o.load()?; }
        Ok(o)
    }
    fn load(&mut self) -> Result<(), String> {
        let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let dem = cat.asset("copernicus-dem-glo30").ok_or("catalog has no copernicus-dem-glo30")?;
        let mut fe = Fetcher::default_for(&root)?;
        let w = dem_window(&mut fe, dem, self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let grid = LatLonGrid { nx: w.nx, ny: w.ny, dlon: w.dlon, dlat: w.dlat, lon_w: w.lon_w, lat_n: w.lat_n };
        self.metres = (0..grid.ny).map(|r| { let lat = grid.lat_n - (r as f64 + 0.5) * grid.dlat; (grid.dlon * 111_320.0 * lat.to_radians().cos(), grid.dlat * 111_320.0) }).collect();
        self.held = vec![false; grid.nx * grid.ny];
        self.grid = Some(grid);
        Ok(())
    }
    /// The cells under a path, to line_width_m, that are not already held.
    fn cells_under(&mut self, pts: &[(f64, f64)]) -> Vec<usize> {
        let Some(grid) = self.grid.clone() else { return Vec::new() };
        let half = self.line_width_m / 2.0;
        let mut out = Vec::new();
        for seg in pts.windows(2) {
            let (ax, ay) = seg[0]; let (bx, by) = seg[1];
            let row_mid = (((grid.lat_n - (ay + by) / 2.0) / grid.dlat).max(0.0) as usize).min(grid.ny - 1);
            let (mx, my) = self.metres[row_mid];
            let len = (((bx - ax) * mx / grid.dlon).powi(2) + ((by - ay) * my / grid.dlat).powi(2)).sqrt();
            let steps = (len / 5.0).ceil().max(1.0) as usize;
            for st in 0..=steps {
                let t = st as f64 / steps as f64; let (x, y) = (ax + (bx - ax) * t, ay + (by - ay) * t);
                if x < grid.lon_w || x >= grid.lon_e() || y <= grid.lat_s() || y > grid.lat_n { continue; }
                let c = ((x - grid.lon_w) / grid.dlon) as usize; let r = ((grid.lat_n - y) / grid.dlat) as usize;
                let (rc, rr) = ((half / mx).ceil() as isize, (half / my).ceil() as isize);
                for dr in -rr..=rr { for dc in -rc..=rc {
                    let (cc, rrr) = (c as isize + dc, r as isize + dr);
                    if cc < 0 || rrr < 0 || cc >= grid.nx as isize || rrr >= grid.ny as isize { continue; }
                    if ((dc as f64 * mx).powi(2) + (dr as f64 * my).powi(2)).sqrt() <= half.max(mx * 0.5) {
                        let k = rrr as usize * grid.nx + cc as usize;
                        if !self.held[k] { self.held[k] = true; out.push(k); }
                    }
                } }
            }
        }
        out
    }
}

impl Module for Suppression {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.publish("suppression.ready", json!({ "mode": self.mode, "line_width_m": self.line_width_m, "strength": self.strength, "source": "record.event" }));
        if self.mode != "off" { ctx.subscribe("record.event"); }
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "record.event" || self.mode == "off" { return; }
        let d = &ev.data;
        if d.get("kind").and_then(|v| v.as_str()) != Some("line") { return; }
        let cat = d.get("category").and_then(|v| v.as_str()).unwrap_or("");
        let status = d.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if !cat.starts_with(&self.category_prefix) || status == self.skip_status { return; }
        let Some(path) = d.get("path").and_then(|v| v.as_array()) else { return };
        let pts: Vec<(f64, f64)> = path.iter().filter_map(|p| { let a = p.as_array()?; Some((a.get(1)?.as_f64()?, a.get(0)?.as_f64()?)) }).collect(); // (lon, lat)
        let cells = self.cells_under(&pts);
        if cells.is_empty() { return; }
        self.lines += 1; self.cells += cells.len();
        let at = cells[0];
        ctx.publish_at(at, "ground.break", json!({
            "cells": cells, "strength": self.strength, "from": Time(ev.t.0).to_iso(), "until": serde_json::Value::Null,
            "category": cat, "label": d.get("label").cloned().unwrap_or(json!("")), "length_m": d.get("length_m").cloned().unwrap_or(json!(0.0)),
        }));
        ctx.report("suppression.lines", "lines", self.lines as f64);
        ctx.report("suppression.cells_held", "cells", self.cells as f64);
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Suppression = serde_json::from_value(v).unwrap(); if o.mode != "off" { o.load().expect("suppression reload"); } *self = o; }
}

impl Suppression {
    pub fn describe(name: &str, p: &toml::Value) -> esim_kernel::describe::Describe {
        let mode = param_str(p, "mode", "off").to_string();
        let mut d = esim_kernel::describe::Describe::new("fire.suppression", name, "What people did to the fire, as a module of its own: each completed line in the record (obs.record's record.event of kind line) becomes a ground.break, the cells under it to a stated width, of a stated strength, from the line's recorded time on. Any fire engine that subscribes to ground.break refuses heat and embers there; the engine never reads the record itself, so the same engine runs another fire with another record, or none. The record's times are when the GIS feature was made, which lags the work; obs.record's time_shift_h is the declared way to move them. Resources as agents with a ledger are the next form.")
            .param("lat_s", 34.0, "deg", "the fire's window, so the cells match the engine's").param("lat_n", 34.16, "deg", "").param("lon_w", -118.72, "deg", "").param("lon_e", -118.46, "deg", "").param("res_arcsec", 1.0, "arcsec", "")
            .param("mode", "off", "", "off: nothing is published, the record is drawn and not defended; hold: completed lines hold from their recorded time")
            .param("line_width_m", 30.0, "m", "guess: the width held along a completed line").param("strength", 1.0, "", "the break's strength; engines today treat any strength above zero as unburnable")
            .param("category_prefix", "Completed", "", "which record categories are lines that hold (Completed Dozer Line, Completed Hand Line, Completed Road as Line); planned lines do not").param("skip_status", "Archive", "", "features with this status are ignored")
            .priority(9).dt("at each line's recorded time")
            .publish("suppression.ready", "{ mode, line_width_m, strength, source }")
            .note("Nothing here is suppression as it happened: no crews on the line, no burnover, no retardant drops, no aircraft, no engines at houses. It is the record's lines as barriers, from the time the record gives.");
        if mode != "off" {
            d = d.subscribe("record.event", "lines from the record")
                .publish("ground.break", "at a cell: { cells, strength, from, until, category, label, length_m }; the cells a line holds from this time")
                .report("suppression.lines", "lines").report("suppression.cells_held", "cells");
        }
        d
    }
}
