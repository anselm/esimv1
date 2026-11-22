//! `roads.routes`: a road network as segments in a graph, and what can be reached over it. Each
//! segment closes at the rupture for a number of days drawn from the range for its class of ground
//! deformation; a bridge links a segment on each bank and is open or closed as the bridges module
//! says; each day the module works out which named places can be reached from one origin over open
//! segments and open bridges. The graph is the event folder's `routes.toml` and the closure ranges
//! its `fragility.toml`. The module knows no city. One of the four subject modules that replaced
//! `hazard.components` on 2026-11-14.
use crate::fragility::{or, table, tf, ts};
use crate::quake::{cell_of, root, window_grid};
use esim_registry::wildfire::{cached_context, context};
use esim_registry::Fetcher;
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Describe, Duration, Event, Module};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, HashMap, VecDeque};

#[derive(Clone, Serialize, Deserialize, Default)]
struct Seg { id: i64, lat: f64, lon: f64, cell: usize, class: i64, neighbours: Vec<usize>, length_m: f64, down: bool, repair_day: i64 }
#[derive(Clone, Serialize, Deserialize, Default)]
struct Link { bridge: String, a: usize, b: usize, open: bool }

#[derive(Serialize, Deserialize)]
pub struct Routes {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    folder: String, soils: String,
    closure_days: Vec<(f64, f64)>,
    segs: Vec<Seg>, links: Vec<Link>, origin: Option<usize>, origin_name: String, places: BTreeMap<String, usize>,
    fired: bool, scheduled: bool, day: i64,
}

impl Routes {
    pub fn new(name: &str, p: &toml::Value) -> Result<Routes, String> {
        let mut o = Routes {
            name: name.to_string(),
            lat_s: crate::fragility::need_f64(p, "lat_s", name)?, lat_n: crate::fragility::need_f64(p, "lat_n", name)?, lon_w: crate::fragility::need_f64(p, "lon_w", name)?, lon_e: crate::fragility::need_f64(p, "lon_e", name)?, res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "").to_string(), soils: param_str(p, "soils", "wet").to_string(),
            closure_days: Vec::new(), segs: Vec::new(), links: Vec::new(), origin: None, origin_name: String::new(), places: BTreeMap::new(),
            fired: false, scheduled: false, day: 0,
        };
        o.load()?;
        Ok(o)
    }
    fn load(&mut self) -> Result<(), String> {
        let (g, _) = window_grid(self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let fr = table(&self.folder, "fragility.toml")?;
        self.closure_days = fr.get("route").and_then(|r| r.get("closure_days_by_class")).and_then(|v| v.as_array()).map(|a| a.iter().map(|x| { let p = x.as_array().map(|q| q.iter().filter_map(|y| y.as_float().or_else(|| y.as_integer().map(|i| i as f64))).collect::<Vec<_>>()).unwrap_or_default(); (p.first().copied().unwrap_or(0.0), p.get(1).copied().unwrap_or(0.0)) }).collect()).unwrap_or_default();
        let rt = table(&self.folder, "routes.toml")?;
        let segs = rt.get("segment").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let index: HashMap<i64, usize> = segs.iter().enumerate().map(|(k, s)| (tf(s, "id") as i64, k)).collect();
        self.segs = segs.iter().map(|s| {
            let (lat, lon) = (tf(s, "lat"), tf(s, "lon"));
            let nb = s.get("neighbours").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_integer()).filter_map(|i| index.get(&i).copied()).collect()).unwrap_or_default();
            Seg { id: tf(s, "id") as i64, lat, lon, cell: cell_of(&g, lat, lon), class: tf(s, if self.soils == "wet" { "class" } else { "class_dry" }) as i64, neighbours: nb, length_m: tf(s, "length_m"), down: false, repair_day: -1 }
        }).collect();
        // a bridge joins the segment nearest it on each bank; it is open until the bridges module says otherwise
        self.links = rt.get("link").and_then(|v| v.as_array()).cloned().unwrap_or_default().iter().filter_map(|l| { let (a, b) = (index.get(&(tf(l, "west") as i64)).copied()?, index.get(&(tf(l, "east") as i64)).copied()?); Some(Link { bridge: ts(l, "bridge"), a, b, open: true }) }).collect();
        // named places and the segment nearest each; one of them is the origin
        self.origin_name = or(ts(&rt, "origin"), "hub");
        if let Some(t) = rt.get("nearest").and_then(|n| n.as_table()) { for (k, v) in t { if let Some(i) = v.as_integer().and_then(|i| index.get(&i)) { if *k == self.origin_name { self.origin = Some(*i); } else { self.places.insert(k.clone(), *i); } } } }
        // the area's roads and place names for a map, from OpenStreetMap; fetched once, carried by the describe from the cache
        match Fetcher::default_for(&root()) { Ok(mut fe) => { if let Err(e) = context(&mut fe, self.lat_s, self.lat_n, self.lon_w, self.lon_e) { eprintln!("context: {e} (the map will have no roads or names)"); } } Err(e) => eprintln!("context: {e}") }
        Ok(())
    }
    /// A city's road context is megabytes; a run's describe carries the heavier roads only, each
    /// thinned to a point every 40 m or so and without its name, and the place names.
    fn slim(c: serde_json::Value) -> serde_json::Value {
        let roads: Vec<serde_json::Value> = c["roads"].as_array().into_iter().flatten().filter(|r| matches!(r["class"].as_str(), Some("motorway" | "trunk" | "primary" | "secondary"))).filter_map(|r| {
            let pts: Vec<(f64, f64)> = r["pts"].as_array()?.iter().filter_map(|q| Some((q[0].as_f64()?, q[1].as_f64()?))).collect();
            if pts.len() < 2 { return None; }
            let kx = 111_320.0 * pts[0].1.to_radians().cos();
            let mut keep = vec![pts[0]];
            for q in &pts[1..pts.len() - 1] { let l = keep[keep.len() - 1]; if (((q.0 - l.0) * kx).powi(2) + ((q.1 - l.1) * 110_570.0).powi(2)).sqrt() >= 40.0 { keep.push(*q); } }
            keep.push(pts[pts.len() - 1]);
            Some(json!({ "class": r["class"], "pts": keep.into_iter().map(|(x, y)| [x, y]).collect::<Vec<_>>() }))
        }).collect();
        json!({ "roads": roads, "places": c["places"], "boundaries": [], "source": c["source"] })
    }
    /// The segments' outlines for a map, from the folder's GeoJSON named in routes.toml
    /// (`geometry`, with `geometry_id` the property that holds a segment's id): each polygon ring
    /// thinned to one point every 30 m or so. A drawing aid; the model uses the graph.
    fn geometry(folder: &str, rt: &toml::Value) -> Option<serde_json::Value> {
        let file = or(ts(rt, "geometry"), "routes.geojson"); let idp = or(ts(rt, "geometry_id"), "SegmentID");
        let text = std::fs::read_to_string(esim_registry::paths::event(folder).join(&file)).ok()?;
        let g: serde_json::Value = serde_json::from_str(&text).ok()?;
        fn rings(geom: &serde_json::Value, out: &mut Vec<Vec<(f64, f64)>>) {
            let ring = |r: &serde_json::Value| -> Vec<(f64, f64)> { r.as_array().into_iter().flatten().filter_map(|q| Some((q[0].as_f64()?, q[1].as_f64()?))).collect() };
            match geom["type"].as_str() {
                Some("Polygon") => for r in geom["coordinates"].as_array().into_iter().flatten() { out.push(ring(r)); },
                Some("MultiPolygon") => for p in geom["coordinates"].as_array().into_iter().flatten() { for r in p.as_array().into_iter().flatten() { out.push(ring(r)); } },
                Some("GeometryCollection") => for x in geom["geometries"].as_array().into_iter().flatten() { rings(x, out); },
                _ => {}
            }
        }
        let r5 = |x: f64| (x * 1e5).round() / 1e5;
        let mut segs = Vec::new();
        for f in g["features"].as_array()? {
            let id = f["properties"][idp.as_str()].as_i64().or(f["properties"][idp.as_str()].as_f64().map(|x| x as i64))?;
            let mut rs = Vec::new(); rings(&f["geometry"], &mut rs);
            let lines: Vec<Vec<[f64; 2]>> = rs.into_iter().filter(|r| r.len() > 3).map(|r| {
                let kx = 111_320.0 * r[0].1.to_radians().cos();
                let mut keep: Vec<(f64, f64)> = vec![r[0]];
                for q in &r[1..] { let l = keep[keep.len() - 1]; if (((q.0 - l.0) * kx).powi(2) + ((q.1 - l.1) * 110_570.0).powi(2)).sqrt() >= 30.0 { keep.push(*q); } }
                keep.push(r[r.len() - 1]);
                keep.into_iter().map(|(x, y)| [r5(x), r5(y)]).collect()
            }).collect();
            segs.push(json!({ "id": format!("{id}"), "lines": lines }));
        }
        Some(json!({ "segments": segs }))
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let folder = param_str(p, "folder", "");
        let rt = table(folder, "routes.toml").ok();
        let rule = table(folder, "fragility.toml").ok().and_then(|t| t.get("route").cloned());
        let (n, origin) = rt.as_ref().map(|t| (t.get("segment").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0), or(ts(t, "origin"), "hub"))).unwrap_or((0, String::new()));
        let mut summary = String::from("This module holds a road network as segments joined in a graph and works out what can be reached over it. Each segment has a class of expected ground deformation. At the earthquake a segment with a class above zero closes for a number of days drawn at random from the range for its class, and it reopens when those days have passed. A bridge joins one segment on each bank; it is open or closed as the bridges module reports. Each day the module finds which named places can be reached from one origin over open segments and open bridges, and reports the share of the network's length that is open. The graph and the named places are the event folder's routes.toml, and the closure ranges are its fragility.toml. Debris from damaged buildings does not block a segment, and the module has no traffic, no detours off the listed segments and no vehicles.");
        if n > 0 { summary.push_str(&format!(" In this run the network has {n} segments and the origin is the place the folder calls {origin}.")); }
        let mut d = Describe::new("roads.routes", name, &summary)
            .priority(1).dt("1 day after the rupture")
            .param("folder", folder, "", "the folder under data/events that holds this scenario's routes.toml and fragility.toml")
            .param("soils", param_str(p, "soils", "wet"), "", "which of each segment's deformation classes to use: wet (saturated soil) or dry")
            .subscribe("quake.rupture", "the instant").subscribe("component.state", "the bridges' states, which open and close the links between banks")
            .publish("component.state", "{ kind: route, id, name, lat, lon, state, down, repair_day, day } at the segment's location: at the earthquake and when it reopens")
            .publish("routes.state", "{ day, open_share, reachable: { place: bool }, any_reachable } each day")
            .report("routes.open_share", "1").report("routes.places_reachable", "places")
            .asset(&format!("events/{folder}"), "routes.toml and fragility.toml")
            .view("counter", "routes open", "routes.open_share", "the share of the network's total length that is open")
            .view("counter", "places reachable from the origin", "routes.places_reachable", "of the named places in the folder's routes.toml")
            .view("chart", "routes open", "routes.open_share", "")
            .view("chart", "places reachable from the origin", "routes.places_reachable", "")
            .view("text", "what this module leaves out", "A segment is open or closed along its whole length. The days closed are a range per deformation class, and the folder's fragility.toml says whether those ranges are published or guessed. Debris, traffic, fuel for vehicles and roads outside the listed network are not represented, so a place that is unreachable here may have another way in.", "");
        if let Some(t) = rule { d = d.table("fragility", json!({ "route": t })); }
        if let Some(g) = rt.as_ref().and_then(|t| Self::geometry(folder, t)) { d = d.table("geometry", g); }
        if let Some(c) = Fetcher::default_for(&root()).ok().and_then(|f| cached_context(&f, param_f64(p, "lat_s", 0.0), param_f64(p, "lat_n", 0.0), param_f64(p, "lon_w", 0.0), param_f64(p, "lon_e", 0.0))) { d = d.table("context", Self::slim(c)).asset("osm-overpass", "the area's roads and place names for the map, from OpenStreetMap"); }
        d
    }
    fn say(&self, ctx: &mut Ctx, s: &Seg, day: i64) { ctx.publish_at(s.cell, "component.state", json!({ "kind": "route", "id": format!("{}", s.id), "name": format!("segment {}", s.id), "lat": s.lat, "lon": s.lon, "state": if s.class > 0 { 3 } else { 0 }, "down": s.down, "repair_day": s.repair_day, "day": day })); }
    fn state(&self, ctx: &mut Ctx) {
        let (mut open, mut tot) = (0.0, 0.0);
        for s in &self.segs { tot += s.length_m; if !s.down { open += s.length_m; } }
        // reachability from the origin's segment over open segments and open bridges
        let mut seen = vec![false; self.segs.len()];
        if let Some(o) = self.origin { if !self.segs[o].down {
            let mut q = VecDeque::new(); q.push_back(o); seen[o] = true;
            while let Some(i) = q.pop_front() {
                let mut next: Vec<usize> = self.segs[i].neighbours.iter().copied().filter(|&j| !self.segs[j].down).collect();
                for l in self.links.iter().filter(|l| l.open) { if l.a == i && !self.segs[l.b].down { next.push(l.b); } if l.b == i && !self.segs[l.a].down { next.push(l.a); } }
                for j in next { if !seen[j] { seen[j] = true; q.push_back(j); } }
            }
        } }
        let reachable: BTreeMap<String, bool> = self.places.iter().map(|(k, &s)| (k.clone(), seen[s])).collect();
        let n = reachable.values().filter(|&&b| b).count();
        let share = if tot > 0.0 { open / tot } else { 1.0 };
        ctx.report("routes.open_share", "1", share); ctx.report("routes.places_reachable", "places", n as f64);
        ctx.publish("routes.state", json!({ "day": self.day, "open_share": share, "reachable": reachable, "any_reachable": n > 0 }));
    }
}

impl Module for Routes {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) { ctx.subscribe("quake.rupture"); ctx.subscribe("component.state"); self.state(ctx); }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "component.state" => {
                if ev.data.get("kind").and_then(|v| v.as_str()) != Some("bridge") { return; }
                let (name, down) = (ev.data.get("name").and_then(|v| v.as_str()).unwrap_or(""), ev.data.get("down").and_then(|v| v.as_bool()).unwrap_or(false));
                for l in self.links.iter_mut().filter(|l| l.bridge == name) { l.open = !down; }
            }
            "quake.rupture" => {
                if self.fired { return; }
                self.fired = true; self.day = 0;
                for i in 0..self.segs.len() {
                    let (lo, hi) = self.closure_days.get(self.segs[i].class as usize).copied().unwrap_or((0.0, 0.0));
                    let v = ctx.rng().f64();
                    let days = (lo + (hi - lo) * v).round() as i64;
                    self.segs[i].down = days > 0; self.segs[i].repair_day = if days > 0 { days } else { -1 };
                }
                for s in &self.segs { self.say(ctx, s, 0); }
                // the day's state waits for the bridges' own reports of this instant, which arrive before this tick
                if !self.scheduled { self.scheduled = true; ctx.schedule_with_priority(Duration::secs(0), 4, "settle", json!(null)); ctx.schedule_with_priority(Duration::days(1), 4, "day", json!(null)); }
            }
            "settle" => self.state(ctx),
            "day" => {
                self.day += 1; let d = self.day;
                for i in 0..self.segs.len() { if self.segs[i].down && self.segs[i].repair_day >= 0 && d >= self.segs[i].repair_day { self.segs[i].down = false; let s = self.segs[i].clone(); self.say(ctx, &s, d); } }
                self.state(ctx);
                ctx.schedule_with_priority(Duration::days(1), 4, "day", json!(null));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { *self = serde_json::from_value(v).unwrap(); }
}
