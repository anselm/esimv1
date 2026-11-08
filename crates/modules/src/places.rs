//! `obs.places`: named places in a fire's window, the stakes of a scenario. Each place is a point
//! the manifest names (a house, a campground, a town hall), with a note saying what it is and where
//! the position came from. At the start every place is published as a `place.marked` at its cell,
//! so the views can draw and label it; each hour the module reads the fire module's burned cells,
//! reports how far the nearest burned cell is from each place, and when a place's own cell has
//! burned publishes `place.reached` there and a `place.crossing` with a sentence, once. Nothing in
//! the model reacts to a place; this is the reader's stake, drawn beside the model.

use esim_kernel::manifest::{param_duration, param_f64};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module};
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub struct Place { pub name: String, pub slug: String, pub lat: f64, pub lon: f64, pub note: String }

#[derive(Serialize, Deserialize)]
pub struct Places {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    every: Duration,
    reach_m: f64,
    places: Vec<Place>,
    reached: Vec<Option<i64>>,   // the instant each place's cell was first seen burned
    start: i64,
    scheduled: bool,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] cells: Vec<usize>,
}

fn slug_of(name: &str) -> String {
    let mut s: String = name.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    while s.contains("__") { s = s.replace("__", "_"); }
    s.trim_matches('_').to_string()
}

pub fn param_places(p: &toml::Value) -> Vec<Place> {
    p.get("places").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| {
        let t = x.as_table()?;
        let name = t.get("name")?.as_str()?.to_string();
        Some(Place { slug: slug_of(&name), name, lat: t.get("lat")?.as_float().or_else(|| t.get("lat")?.as_integer().map(|i| i as f64))?, lon: t.get("lon")?.as_float().or_else(|| t.get("lon")?.as_integer().map(|i| i as f64))?, note: t.get("note").and_then(|v| v.as_str()).unwrap_or("").to_string() })
    }).collect()).unwrap_or_default()
}

impl Places {
    pub fn new(name: &str, p: &toml::Value) -> Result<Places, String> {
        let places = param_places(p);
        if places.is_empty() { return Err("obs.places: no places given (places = [{ name, lat, lon, note }])".into()); }
        let mut o = Places {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 37.97), lat_n: param_f64(p, "lat_n", 38.08), lon_w: param_f64(p, "lon_w", -122.79), lon_e: param_f64(p, "lon_e", -122.61),
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            every: param_duration(p, "every", "1h")?,
            reach_m: param_f64(p, "reach_m", 50.0),
            reached: vec![None; places.len()],
            places, start: 0, scheduled: false, grid: None, cells: Vec::new(),
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
        for pl in &self.places {
            if pl.lat < self.lat_s || pl.lat > self.lat_n || pl.lon < self.lon_w || pl.lon > self.lon_e { return Err(format!("obs.places: {} ({}, {}) is outside the window", pl.name, pl.lat, pl.lon)); }
        }
        self.cells = self.places.iter().map(|pl| {
            let c = (((pl.lon - grid.lon_w) / grid.dlon).floor().max(0.0) as usize).min(grid.nx - 1);
            let r = (((grid.lat_n - pl.lat) / grid.dlat).floor().max(0.0) as usize).min(grid.ny - 1);
            r * grid.nx + c
        }).collect();
        self.grid = Some(grid);
        Ok(())
    }
    /// Metres from a place to the nearest burned cell, or none if nothing has burned.
    fn nearest_m(&self, i: usize, burned: &[f32]) -> Option<f64> {
        let g = self.grid.as_ref()?;
        let pl = &self.places[i];
        let m = 111_320.0;
        let cosl = pl.lat.to_radians().cos();
        let mut best: Option<f64> = None;
        for (k, &b) in burned.iter().enumerate() {
            if b < 0.5 { continue; }
            let (r, c) = (k / g.nx, k % g.nx);
            let dx = (g.lon(c) - pl.lon) * m * cosl; let dy = (g.lat(r) - pl.lat) * m;
            let d = (dx * dx + dy * dy).sqrt();
            if best.map_or(true, |x| d < x) { best = Some(d); }
        }
        best
    }
}

impl Module for Places {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        let now = ctx.now();
        if !self.scheduled { self.start = now.0; }
        for (i, pl) in self.places.iter().enumerate() {
            ctx.publish_at(self.cells[i], "place.marked", json!({ "name": pl.name, "slug": pl.slug, "lat": pl.lat, "lon": pl.lon, "note": pl.note }));
        }
        if !self.scheduled {
            ctx.schedule_with_priority(self.every, 9, "places", json!(null));
            self.scheduled = true;
        }
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "places" { return; }
        let now = ctx.now();
        let burned: Option<Vec<f32>> = ctx.world_read("fire.burned_frac").map(|f| f.data.clone());
        for i in 0..self.places.len() {
            let pl = self.places[i].clone();
            let d = burned.as_ref().and_then(|b| self.nearest_m(i, b));
            if let Some(d) = d { ctx.report(&format!("place.{}.distance_m", pl.slug), "m", d); }
            if self.reached[i].is_none() {
                if let Some(b) = &burned {
                    // reached: its own cell has burned, or a burned cell lies within reach_m (a geocoded
                    // address sits on the road's centreline, which carries no fuel; the house is beside it)
                    if b.get(self.cells[i]).map_or(false, |&x| x >= 0.5) || d.map_or(false, |x| x <= self.reach_m) {
                        self.reached[i] = Some(now.0);
                        let h = (now.0 - self.start) as f64 / 3600.0;
                        let sentence = format!("The fire reaches {} at {} UTC, {:.0} hours into the run.", pl.name, &now.to_iso()[..16].replace('T', " "), h);
                        ctx.publish_at(self.cells[i], "place.reached", json!({ "name": pl.name, "slug": pl.slug, "lat": pl.lat, "lon": pl.lon, "note": pl.note, "time": now.to_iso(), "hours": h }));
                        ctx.publish("place.crossing", json!({ "name": pl.name, "slug": pl.slug, "kind": "reached", "time": now.to_iso(), "hours": h, "sentence": sentence }));
                    }
                }
            }
            ctx.report(&format!("place.{}.reached", pl.slug), "1", if self.reached[i].is_some() { 1.0 } else { 0.0 });
        }
        ctx.report("place.reached_count", "places", self.reached.iter().filter(|r| r.is_some()).count() as f64);
        ctx.schedule_with_priority(self.every, 9, "places", json!(null));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Places = serde_json::from_value(v).unwrap(); o.load().expect("places reload"); *self = o; }
}

impl Places {
    pub fn describe(name: &str, p: &toml::Value) -> esim_kernel::describe::Describe {
        let places = param_places(p);
        let mut d = esim_kernel::describe::Describe::new("obs.places", name, "Named places in the window, the stakes of a scenario: each a point the manifest names with a note on what it is and where its position came from. Every place is published at the start so the views can draw and label it; each hour the module reads the fire module's burned cells, reports how far the nearest burned cell is from each place, and the hour a place's own cell burns, or a burned cell lies within reach_m of it, publishes that it was reached, once, with a sentence. Nothing in the model reacts to a place.")
            .priority(9).dt("every hour")
            .param("lat_s", 37.97, "deg", "").param("lat_n", 38.08, "deg", "").param("lon_w", -122.79, "deg", "").param("lon_e", -122.61, "deg", "").param("res_arcsec", 1.0, "arcsec", "")
            .param("every", "1h", "", "").param("reach_m", 50.0, "m", "a place counts as reached when a burned cell lies within this of it, or its own cell burns; a geocoded address sits on the road's centreline, which carries no fuel, and the house stands beside it").param("places", json!(places.iter().map(|pl| json!({ "name": pl.name, "lat": pl.lat, "lon": pl.lon, "note": pl.note })).collect::<Vec<_>>()), "", "[{ name, lat, lon, note }]: the note says where the position came from")
            .publish("place.marked", "at the place's cell, at the start: { name, slug, lat, lon, note }")
            .publish("place.reached", "at the place's cell, once: { name, slug, lat, lon, note, time, hours }")
            .publish("place.crossing", "{ name, slug, kind: reached, time, hours, sentence }")
            .read("fire.burned_frac", "1", false, "the fire module's burned cells")
            .report("place.reached_count", "places")
            .asset("copernicus-dem-glo30", "the grid");
        for pl in &places {
            d = d.report(&format!("place.{}.distance_m", pl.slug), "m").report(&format!("place.{}.reached", pl.slug), "1")
                .view("chart", &format!("Distance of the fire from {}", pl.name), &format!("place.{}.distance_m", pl.slug), "metres from the place to the nearest burned cell");
            d = d.note(&format!("{}: {}, {} ({})", pl.name, pl.lat, pl.lon, pl.note));
        }
        d = d.view("crossings", "Places reached", "place.crossing", "the hour the fire reached each place, as a sentence");
        d
    }
}
