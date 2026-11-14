//! `roads.bridges`: bridges, one by one. Each reads the ground at its place at the rupture, is given
//! a damage state by a draw against the fragility curve of its seismic class, closes at or beyond
//! the curve's closing state and reopens after the repair time for its state. The bridges are the
//! event folder's `bridges.toml` and the curves its `fragility.toml`. The module knows no river and
//! no city. One of the four subject modules that replaced `hazard.components` on 2026-11-14.
use crate::fragility::{curve, force_param, forced, table, tf, ts, Curve, Shaking};
use crate::quake::{cell_of, root, window_grid};
use esim_registry::wildfire::{cached_osm_bridges, osm_bridges};
use esim_registry::Fetcher;
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Describe, Duration, Event, LatLonGrid, Module};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Clone, Serialize, Deserialize, Default)]
struct Bridge { name: String, lat: f64, lon: f64, cell: usize, class: String, state: u8, down: bool, repair_day: i64 }

#[derive(Serialize, Deserialize)]
pub struct Bridges {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    folder: String, force: Vec<String>,
    default: Curve, classes: BTreeMap<String, Curve>,
    bridges: Vec<Bridge>, fired: bool, scheduled: bool, day: i64,
    #[serde(skip)] grid: Option<LatLonGrid>,
}

impl Bridges {
    pub fn new(name: &str, p: &toml::Value) -> Result<Bridges, String> {
        let mut o = Bridges {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 0.0), lat_n: param_f64(p, "lat_n", 0.0), lon_w: param_f64(p, "lon_w", 0.0), lon_e: param_f64(p, "lon_e", 0.0), res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "").to_string(), force: force_param(p),
            default: Curve::default(), classes: BTreeMap::new(), bridges: Vec::new(), fired: false, scheduled: false, day: 0, grid: None,
        };
        o.load()?;
        Ok(o)
    }
    fn load(&mut self) -> Result<(), String> {
        let (g, _) = window_grid(self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let fr = table(&self.folder, "fragility.toml")?;
        self.default = curve(fr.get("bridge").ok_or("fragility.toml: [bridge]")?);
        if let Some(t) = fr.get("bridge_class").and_then(|v| v.as_table()) { for (k, v) in t { self.classes.insert(k.clone(), curve(v)); } }
        let br = table(&self.folder, "bridges.toml")?;
        self.bridges = br.get("bridge").and_then(|v| v.as_array()).cloned().unwrap_or_default().iter().map(|b| { let (lat, lon) = (tf(b, "lat"), tf(b, "lon")); Bridge { name: ts(b, "name"), lat, lon, cell: cell_of(&g, lat, lon), class: ts(b, "seismic_class"), state: 0, down: false, repair_day: -1 } }).collect();
        // the bridges' lines for a map, from OpenStreetMap; fetched once, carried by the describe from the cache
        match Fetcher::default_for(&root()) { Ok(mut fe) => { if let Err(e) = osm_bridges(&mut fe, self.lat_s, self.lat_n, self.lon_w, self.lon_e) { eprintln!("bridge lines: {e} (the map will draw each bridge as a point)"); } } Err(e) => eprintln!("bridge lines: {e}") }
        self.grid = Some(g);
        Ok(())
    }
    /// Each bridge's lines for a map: the OpenStreetMap bridge ways that pass close to the bridge's
    /// point, each way given to the bridge it is nearest. A drawing aid; the model uses the point.
    fn geometry(folder: &str, p: &toml::Value) -> Option<serde_json::Value> {
        let br = table(folder, "bridges.toml").ok()?;
        let list: Vec<(String, f64, f64)> = br.get("bridge")?.as_array()?.iter().map(|b| (ts(b, "name"), tf(b, "lat"), tf(b, "lon"))).collect();
        let fe = Fetcher::default_for(&root()).ok()?;
        let osm = cached_osm_bridges(&fe, param_f64(p, "lat_s", 0.0), param_f64(p, "lat_n", 0.0), param_f64(p, "lon_w", 0.0), param_f64(p, "lon_e", 0.0))?;
        let mut lines: Vec<Vec<serde_json::Value>> = vec![Vec::new(); list.len()];
        for w in osm.get("ways")?.as_array()? {
            let pts: Vec<(f64, f64)> = w["pts"].as_array().into_iter().flatten().filter_map(|q| Some((q[0].as_f64()?, q[1].as_f64()?))).collect();
            // the nearest bridge point to any vertex of the way, in metres
            let mut best = (f64::INFINITY, 0usize);
            for (k, (_, lat, lon)) in list.iter().enumerate() {
                let kx = 111_320.0 * lat.to_radians().cos();
                let d = pts.iter().map(|(x, y)| (((x - lon) * kx).powi(2) + ((y - lat) * 110_570.0).powi(2)).sqrt()).fold(f64::INFINITY, f64::min);
                if d < best.0 { best = (d, k); }
            }
            if best.0 <= 180.0 { lines[best.1].push(w["pts"].clone()); }
        }
        Some(json!({ "source": osm.get("source").cloned().unwrap_or(json!("")), "bridges": list.iter().zip(lines).map(|((name, lat, lon), l)| json!({ "name": name, "lat": lat, "lon": lon, "lines": l })).collect::<Vec<_>>() }))
    }
    fn curve_of(&self, class: &str) -> Curve { self.classes.get(class).cloned().unwrap_or_else(|| self.default.clone()) }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let folder = param_str(p, "folder", "");
        let fr = table(folder, "fragility.toml").ok();
        let n = table(folder, "bridges.toml").ok().and_then(|t| t.get("bridge").and_then(|v| v.as_array()).map(|a| a.len())).unwrap_or(0);
        let mut d = Describe::new("roads.bridges", name, "This module damages bridges one at a time. Each bridge sits at its own coordinates and carries a seismic class. At the earthquake it reads the ground motion there and is assigned a damage state by a random draw against the fragility curve of its class. A bridge at or beyond the curve's closing state is closed, and it reopens after the repair time the curve gives for its damage state. Each change is published, so a module that routes traffic or draws a map can follow it. The bridges are the event folder's bridges.toml, and the curves and repair times are its fragility.toml, which says where each came from.")
            .priority(1).dt("1 day after the rupture")
            .param("folder", folder, "", "the folder under data/events that holds this scenario's bridges.toml and fragility.toml")
            .param("force", json!(force_param(p)), "", "bridges whose damage state is set in advance instead of drawn, written as bridge:name:state")
            .subscribe("quake.rupture", "the instant")
            .read_on("quake.pga_g", "g", true, "quake.rupture", "at each bridge").read_on("quake.liq_p", "1", true, "quake.rupture", "").read_on("quake.liq_pgd_cm", "cm", true, "quake.rupture", "").read_on("quake.ls_p", "1", true, "quake.rupture", "").read_on("quake.ls_pgd_cm", "cm", true, "quake.rupture", "")
            .publish("component.state", "{ kind: bridge, id, name, lat, lon, state, down, repair_day, day } at the bridge's location: at the start, at the earthquake, and when it reopens")
            .report("bridges.open", "crossings")
            .asset(&format!("events/{folder}"), "bridges.toml and fragility.toml")
            .view("counter", "bridges open", "bridges.open", &format!("of {n}, each according to its own damage state and repair time"))
            .view("chart", "bridges open", "bridges.open", "")
            .view("text", "what this module leaves out", "A bridge is one point with one curve for its class. The module does not represent spans, approaches, piers or movable parts separately, and a bridge is either open or closed, with no reduced capacity. Repair times are fixed by damage state and do not depend on how many other bridges need repair at once.", "");
        if let Some(g) = Self::geometry(folder, p) { d = d.table("geometry", g).asset("osm-overpass", "the bridges' lines for the map, from OpenStreetMap"); }
        if let Some(fr) = fr { let mut t = serde_json::Map::new(); for k in ["bridge", "bridge_class"] { if let Some(v) = fr.get(k) { t.insert(k.to_string(), json!(v)); } } d = d.table("fragility", serde_json::Value::Object(t)); }
        d
    }
    fn say(&self, ctx: &mut Ctx, b: &Bridge, day: i64) { ctx.publish_at(b.cell, "component.state", json!({ "kind": "bridge", "id": b.name, "name": b.name, "lat": b.lat, "lon": b.lon, "state": b.state, "down": b.down, "repair_day": b.repair_day, "day": day })); }
    fn report(&self, ctx: &mut Ctx) { ctx.report("bridges.open", "crossings", self.bridges.iter().filter(|b| !b.down).count() as f64); }
}

impl Module for Bridges {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.subscribe("quake.rupture");
        self.report(ctx);
        for b in &self.bridges { self.say(ctx, b, 0); }
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "quake.rupture" => {
                if self.fired { return; }
                self.fired = true; self.day = 0;
                let g = self.grid.clone().unwrap();
                let shaking = Shaking::read(ctx, g.nx * g.ny);
                let forced = forced(&self.force, "bridge");
                for i in 0..self.bridges.len() {
                    let u = ctx.rng().f64();
                    let cv = self.curve_of(&self.bridges[i].class);
                    let mut state = shaking.draw(&cv, self.bridges[i].cell, u);
                    if let Some(s) = forced.get(&self.bridges[i].name) { state = *s; }
                    let b = &mut self.bridges[i];
                    b.state = state; b.down = state >= cv.fail_state;
                    b.repair_day = if b.down { cv.restore.get((state as usize).saturating_sub(1)).copied().unwrap_or(230) } else { -1 };
                }
                for b in &self.bridges { self.say(ctx, b, 0); }
                self.report(ctx);
                if !self.scheduled { self.scheduled = true; ctx.schedule_with_priority(Duration::days(1), 3, "day", json!(null)); }
            }
            "day" => {
                self.day += 1; let d = self.day;
                for i in 0..self.bridges.len() {
                    if self.bridges[i].down && self.bridges[i].repair_day >= 0 && d >= self.bridges[i].repair_day { self.bridges[i].down = false; let b = self.bridges[i].clone(); self.say(ctx, &b, d); }
                }
                self.report(ctx);
                ctx.schedule_with_priority(Duration::days(1), 3, "day", json!(null));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Bridges = serde_json::from_value(v).unwrap(); o.grid = window_grid(o.lat_s, o.lat_n, o.lon_w, o.lon_e, o.res_arcsec).ok().map(|x| x.0); *self = o; }
}
