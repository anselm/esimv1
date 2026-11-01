//! M10 Rules: the jurisdiction layer. A rules module reads calendar rows addressed to it,
//! and on each row's effective date publishes `rule.effective` and writes the constraint
//! field the row describes. The first rule is a land-use constraint: no cropping within
//! a buffer of the channel network, which the farm agents obey (grass instead of crop),
//! which the crop module grows, which the water module's evaporation and the nitrogen
//! module feel. Seam test 5 of the brief, in this basin's terms: a rule changes land
//! cover, land cover changes water.

use esim_kernel::manifest::{param_duration, param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module, Time};
use esim_registry::events::EventTable;
use esim_registry::hydro::delineate;
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Rules {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64, gauge_lat: f64, gauge_lon: f64, snap_radius: usize,
    dt: Duration,
    rule_id: String,
    buffer_cells: usize,
    channel_km2: f64,
    effective: String,
    applied: bool,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] buffer: Vec<usize>,
    #[serde(skip)] cells: Vec<usize>,
    #[serde(skip)] row_title: String, #[serde(skip)] row_source: String, #[serde(skip)] row_confidence: String, #[serde(skip)] row_known_since: String,
}

impl Rules {
    pub fn new(name: &str, p: &toml::Value) -> Result<Rules, String> {
        let mut r = Rules {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 42.3), lat_n: param_f64(p, "lat_n", 43.2), lon_w: param_f64(p, "lon_w", -94.2), lon_e: param_f64(p, "lon_e", -93.5),
            res_arcsec: param_f64(p, "res_arcsec", 30.0), gauge_lat: param_f64(p, "gauge_lat", 42.4319722), gauge_lon: param_f64(p, "gauge_lon", -93.8059444), snap_radius: param_f64(p, "snap_radius", 3.0) as usize,
            dt: param_duration(p, "dt", "1d")?,
            rule_id: param_str(p, "rule_id", "whatif-riparian-buffer-boone-2025").to_string(),
            buffer_cells: param_f64(p, "buffer_cells", 1.0) as usize,
            channel_km2: param_f64(p, "channel_km2", 2.0),
            effective: String::new(), applied: false,
            grid: None, buffer: Vec::new(), cells: Vec::new(), row_title: String::new(), row_source: String::new(), row_confidence: String::new(), row_known_since: String::new(),
        };
        r.load()?;
        Ok(r)
    }
    fn load(&mut self) -> Result<(), String> {
        let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let table = EventTable::load(&root.join("data/events/calendar.toml"))?;
        let row = table.get(&self.rule_id).ok_or_else(|| format!("calendar.toml has no row {}", self.rule_id))?;
        self.effective = row.date.clone().ok_or("rule row has no date")?;
        self.row_title = row.title.clone(); self.row_source = row.source.clone(); self.row_confidence = row.confidence.clone(); self.row_known_since = row.known_since.clone();
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let dem = cat.asset("copernicus-dem-glo30").ok_or("catalog has no copernicus-dem-glo30")?;
        let mut f = Fetcher::default_for(&root)?;
        let w = dem_window(&mut f, dem, self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let grid = LatLonGrid { nx: w.nx, ny: w.ny, dlon: w.dlon, dlat: w.dlat, lon_w: w.lon_w, lat_n: w.lat_n };
        let row_area: Vec<f64> = (0..grid.ny).map(|r| grid.row_area(r)).collect();
        let (net, _, cells, mask) = delineate(&f, &w, &row_area, self.gauge_lat, self.gauge_lon, self.snap_radius);
        // buffer: basin cells within buffer_cells (Chebyshev) of a channel cell
        let channel: Vec<usize> = cells.iter().copied().filter(|&i| net.accum_m2[i] as f64 >= self.channel_km2 * 1e6).collect();
        let mut inb = vec![false; grid.nx * grid.ny];
        let b = self.buffer_cells as isize;
        for &i in &channel {
            let (c, r) = ((i % grid.nx) as isize, (i / grid.nx) as isize);
            for dr in -b..=b { for dc in -b..=b {
                let (cc, rr) = (c + dc, r + dr);
                if cc < 0 || rr < 0 || cc as usize >= grid.nx || rr as usize >= grid.ny { continue; }
                let j = rr as usize * grid.nx + cc as usize;
                if mask[j] { inb[j] = true; }
            } }
        }
        self.buffer = cells.iter().copied().filter(|&i| inb[i]).collect();
        self.grid = Some(grid); self.cells = cells;
        Ok(())
    }
}

impl Module for Rules {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.publish("rule.loaded", json!({ "id": self.rule_id, "title": self.row_title, "effective": self.effective, "confidence": self.row_confidence, "source": self.row_source, "known_since": self.row_known_since }));
        let g = self.grid.clone().unwrap();
        // before the effective date the constraint field exists and is zero everywhere
        let mut f = vec![f32::NAN; g.nx * g.ny];
        for &i in &self.cells { f[i] = 0.0; }
        ctx.world_write("rules.no_crop", "1", &g, f);
        ctx.report("rules.buffer_cells", "1", self.buffer.len() as f64);
        ctx.schedule_with_priority(self.dt, 1, "tick", json!(null)); // before agents (2)
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" { return; }
        if !self.applied {
            if let Ok(t) = Time::parse(&self.effective) {
                if ctx.now().0 >= t.0 {
                    let g = self.grid.clone().unwrap();
                    let mut f = vec![f32::NAN; g.nx * g.ny];
                    for &i in &self.cells { f[i] = 0.0; }
                    for &i in &self.buffer { f[i] = 1.0; }
                    ctx.world_write("rules.no_crop", "1", &g, f.clone());
                    ctx.field("rule_no_crop", "1", &g, f);
                    ctx.publish("rule.effective", json!({ "id": self.rule_id, "cells": self.buffer.len() }));
                    ctx.report("rules.effective", "1", 1.0);
                    self.applied = true;
                }
            }
        }
        ctx.schedule_with_priority(self.dt, 1, "tick", json!(null));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut r: Rules = serde_json::from_value(v).unwrap(); r.load().expect("rules reload"); *self = r; }
}
