//! `obs.record`: the record of what people did during an event, placed and timed, published into
//! the run so the views can show it. It reads GeoJSON files kept in the repository's event folders
//! (`data/events/<event>/...`, copied from their publishers with the folder's README naming each),
//! keeps the features inside the run's window, and publishes each one as a `record.event` at its
//! time and at the cell it stands in (a point) or starts in (a line). Nothing in the run reacts to
//! it yet; it is the record drawn beside the model, not an intervention in the model.
//!
//! The time is whatever field the manifest names. For NIFC's event lines and points that is
//! `CreateDate`, when the GIS feature was made, which lags the work on the ground; the dataset's
//! own time fields are empty for this incident (checked 2026-11-03).

use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module, Time};
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
struct Feature {
    t: i64,
    cell: usize,
    kind: String,           // point | line
    category: String,
    label: String,
    note: String,
    status: String,
    lat: f64, lon: f64,
    path: Vec<[f64; 2]>,    // [lat, lon] along a line, decimated
    length_m: f64,
    file: String,
}

#[derive(Serialize, Deserialize)]
pub struct Record {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    files: Vec<String>,
    time_field: String,
    category_field: String,
    label_field: String,
    note_field: String,
    status_field: String,
    max_vertices: usize,
    time_shift_h: f64,
    scheduled: bool,
    #[serde(skip)] feats: Vec<Feature>,
}

fn param_list(p: &toml::Value, key: &str) -> Vec<String> {
    p.get(key).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default()
}

impl Record {
    pub fn new(name: &str, p: &toml::Value) -> Result<Record, String> {
        let mut o = Record {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 34.0), lat_n: param_f64(p, "lat_n", 34.16), lon_w: param_f64(p, "lon_w", -118.72), lon_e: param_f64(p, "lon_e", -118.46),
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            files: param_list(p, "files"),
            time_field: param_str(p, "time_field", "CreateDate").to_string(),
            category_field: param_str(p, "category_field", "FeatureCategory").to_string(),
            label_field: param_str(p, "label_field", "Label").to_string(),
            note_field: param_str(p, "note_field", "Comments").to_string(),
            status_field: param_str(p, "status_field", "FeatureStatus").to_string(),
            max_vertices: param_f64(p, "max_vertices", 40.0) as usize,
            time_shift_h: param_f64(p, "time_shift_h", 0.0),
            scheduled: false,
            feats: Vec::new(),
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
        let inside = |lat: f64, lon: f64| lat >= self.lat_s && lat <= self.lat_n && lon >= self.lon_w && lon <= self.lon_e;
        let cell_of = |lat: f64, lon: f64| {
            let c = (((lon - grid.lon_w) / grid.dlon).floor().max(0.0) as usize).min(grid.nx - 1);
            let r = (((grid.lat_n - lat) / grid.dlat).floor().max(0.0) as usize).min(grid.ny - 1);
            r * grid.nx + c
        };
        let s = |p: &serde_json::Value, k: &str| p.get(k).map(|v| match v { serde_json::Value::String(x) => x.clone(), serde_json::Value::Null => String::new(), x => x.to_string() }).unwrap_or_default();
        let mut feats = Vec::new();
        for f in &self.files {
            let path = root.join("data/events").join(f);
            let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let g: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
            let Some(list) = g.get("features").and_then(|v| v.as_array()) else { return Err(format!("{}: no features", path.display())) };
            for ft in list {
                let p = ft.get("properties").cloned().unwrap_or(json!({}));
                let Some(tv) = p.get(&self.time_field) else { continue };
                // epoch milliseconds (ArcGIS) or seconds or an ISO string
                let t = match tv {
                    serde_json::Value::Number(n) => { let x = n.as_f64().unwrap_or(0.0); if x > 1e11 { (x / 1000.0) as i64 } else { x as i64 } }
                    serde_json::Value::String(x) => match Time::parse(x) { Ok(tt) => tt.0, Err(_) => continue },
                    _ => continue,
                } - (self.time_shift_h * 3600.0) as i64;
                let geom = ft.get("geometry").cloned().unwrap_or(json!(null));
                let gtype = s(&geom, "type");
                let coords = geom.get("coordinates").cloned().unwrap_or(json!([]));
                let pt = |c: &serde_json::Value| -> Option<(f64, f64)> { let a = c.as_array()?; Some((a.get(1)?.as_f64()?, a.get(0)?.as_f64()?)) };
                let (kind, lat, lon, path, length_m) = match gtype.as_str() {
                    "Point" => { let Some((lat, lon)) = pt(&coords) else { continue }; if !inside(lat, lon) { continue } ("point", lat, lon, Vec::new(), 0.0) }
                    "LineString" | "MultiLineString" => {
                        let parts: Vec<&serde_json::Value> = if gtype == "LineString" { vec![&coords] } else { coords.as_array().map(|a| a.iter().collect()).unwrap_or_default() };
                        let mut pts: Vec<(f64, f64)> = Vec::new();
                        for part in parts { if let Some(a) = part.as_array() { for c in a { if let Some(q) = pt(c) { pts.push(q); } } } }
                        if !pts.iter().any(|&(la, lo)| inside(la, lo)) { continue }
                        let mut len = 0.0;
                        for w in pts.windows(2) { let (a, b) = (w[0], w[1]); let m = 111_320.0; let dx = (b.1 - a.1) * m * a.0.to_radians().cos(); let dy = (b.0 - a.0) * m; len += (dx * dx + dy * dy).sqrt(); }
                        let first = *pts.iter().find(|&&(la, lo)| inside(la, lo)).unwrap();
                        let step = ((pts.len() + self.max_vertices - 1) / self.max_vertices).max(1);
                        let mut path: Vec<[f64; 2]> = pts.iter().step_by(step).map(|&(la, lo)| [la, lo]).collect();
                        if let Some(&last) = pts.last() { if path.last() != Some(&[last.0, last.1]) { path.push([last.0, last.1]); } }
                        ("line", first.0, first.1, path, len)
                    }
                    _ => continue,
                };
                feats.push(Feature {
                    t, cell: cell_of(lat, lon), kind: kind.into(),
                    category: s(&p, &self.category_field), label: s(&p, &self.label_field), note: s(&p, &self.note_field), status: s(&p, &self.status_field),
                    lat, lon, path, length_m, file: f.clone(),
                });
            }
        }
        feats.sort_by_key(|f| f.t);
        self.feats = feats;
        Ok(())
    }
}

impl Module for Record {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        let now = ctx.now();
        let mut by: std::collections::BTreeMap<String, usize> = Default::default();
        for f in &self.feats { *by.entry(f.category.clone()).or_default() += 1; }
        ctx.publish("record.loaded", json!({ "files": self.files, "count": self.feats.len(), "first": self.feats.first().map(|f| Time(f.t).to_iso()), "last": self.feats.last().map(|f| Time(f.t).to_iso()), "by_category": by, "time_field": self.time_field, "time_shift_h": self.time_shift_h }));
        if !self.scheduled {
            for (i, f) in self.feats.iter().enumerate() {
                if f.t >= now.0 { ctx.schedule_with_priority(Duration(f.t - now.0), 9, "record", json!(i)); }
            }
            self.scheduled = true;
        }
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "record" { return; }
        let Some(i) = ev.data.as_u64().map(|x| x as usize) else { return };
        let Some(f) = self.feats.get(i) else { return };
        ctx.publish_at(f.cell, "record.event", json!({
            "kind": f.kind, "category": f.category, "label": f.label, "note": f.note, "status": f.status,
            "lat": f.lat, "lon": f.lon, "path": f.path, "length_m": f.length_m, "file": f.file, "time": Time(f.t).to_iso(),
        }));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Record = serde_json::from_value(v).unwrap(); o.load().expect("record reload"); *self = o; }
}

impl Record {
    pub fn describe(name: &str, p: &toml::Value) -> esim_kernel::describe::Describe {
        esim_kernel::describe::Describe::new("obs.record", name, "The record of what people did during the event, placed and timed: GeoJSON points and lines copied into the repository's event folder (for the Palisades fire, NIFC's event points and lines: dozer pushes, drop points, helispots, division breaks, completed dozer and hand line), kept to the run's window and published into the run at the time each was made, at the cell it stands or starts in, so the views can show them against the model. Nothing in the model reacts to them yet.")
            .priority(9).dt("at each feature's time")
            .param("lat_s", 34.0, "deg", "").param("lat_n", 34.16, "deg", "").param("lon_w", -118.72, "deg", "").param("lon_e", -118.46, "deg", "").param("res_arcsec", 1.0, "arcsec", "")
            .param("files", json!(param_list(p, "files")), "", "GeoJSON files under data/events, as the event folder's README lists them")
            .param("time_field", "CreateDate", "", "the property that times a feature: epoch milliseconds, seconds or ISO. For NIFC event features this is when the GIS feature was made, which lags the work; the dataset's own LineDateTime and PointDateTime are empty for this incident")
            .param("category_field", "FeatureCategory", "", "").param("label_field", "Label", "", "").param("note_field", "Comments", "", "").param("status_field", "FeatureStatus", "", "Approved, Proposed, Archive as the GIS unit set it")
            .param("max_vertices", 40.0, "", "a line's path is thinned to about this many vertices in the event").param("time_shift_h", 0.0, "h", "hours taken off every feature's time, for a time field known to lag the work it records; 0 is the record as it stands, anything else is a guess and says so here")
            .publish("record.loaded", "{ files, count, first, last, by_category, time_field }")
            .publish("record.event", "at the feature's cell: { kind point|line, category, label, note, status, lat, lon, path [[lat, lon]], length_m, file, time }")
            .asset("copernicus-dem-glo30", "the grid")
            .note(&format!("files: {}", param_list(p, "files").join(", ")))
    }
}
