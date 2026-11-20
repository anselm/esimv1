//! Interventions, as a module of their own: what people did to the fire, taken from the record
//! and turned into something any fire engine understands. The record (`obs.record`) says where a
//! line was cut and when the GIS unit drew it; this module turns each completed line into a
//! `ground.break`, the cells under it to a stated width, of a stated strength, from that time on.
//! A fire engine that subscribes to `ground.break` refuses heat and embers on those cells (or, for
//! Rothermel, treats them as unburnable); it never reads the record itself, so the same engine runs
//! another fire with another record, or with none. Resources as agents (engines, dozers, aircraft
//! placed by rule from the reports' counts, held in a ledger) are a later form.
//!
//! `mode = "resources"` is the first form in which a fire can be stopped. The incident's status
//! reports say how many engines, dozers, hand crews and helicopters were assigned at each time.
//! Each has a rate at which it can hold fire edge; a declared share of the total is at work on the
//! edge in any hour. Every hour the module reads where the fire is (`fire.burned_frac`), measures
//! how far each stretch of edge moved in the hour, and holds as much edge as the hour's capacity
//! buys, taking the most active stretches that are slow enough to hold first. A stretch moving
//! faster than `hold_rate_m_h` cannot be held and is left. The held ground goes to the engine as a
//! `ground.break`. The idea of closing a perimeter at a production rate is Fried and Fried's
//! (1996, recalled); working the active edge first is our choice.
use esim_kernel::manifest::{param_duration, param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module, Time};
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Suppression {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    mode: String,            // off | hold | resources
    line_width_m: f64,
    strength: f64,
    category_prefix: String,
    skip_status: String,
    lines: usize, cells: usize,
    // resources mode
    reports: String, reports_tz_offset_h: f64,
    share: f64, engine_m_h: f64, dozer_m_h: f64, crew_m_h: f64, helicopter_m_h: f64,
    hold_rate_m_h: f64, depth_m: f64, every: Duration, frame_res_arcsec: f64,
    #[serde(default)] scheduled: bool, #[serde(default)] held_m: f64, #[serde(default)] first_work: Option<i64>, #[serde(default)] last_held_cells: usize, #[serde(default)] framed: bool,
    #[serde(skip)] rows: Vec<Row>,
    #[serde(skip)] prev: Vec<bool>,
    #[serde(skip)] land: Vec<bool>,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] metres: Vec<(f64, f64)>,
    #[serde(skip)] held: Vec<bool>,
}

/// One status report: its instant and what it counts. A blank count is zero.
#[derive(Clone, Default)]
struct Row { t: i64, engines: f64, dozers: f64, crews: f64, helicopters: f64, containment_pct: f64 }

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
            reports: param_str(p, "reports", "").to_string(), reports_tz_offset_h: param_f64(p, "reports_tz_offset_h", 0.0),
            share: param_f64(p, "share", 0.02), engine_m_h: param_f64(p, "engine_m_h", 240.0), dozer_m_h: param_f64(p, "dozer_m_h", 600.0), crew_m_h: param_f64(p, "crew_m_h", 100.0), helicopter_m_h: param_f64(p, "helicopter_m_h", 200.0),
            hold_rate_m_h: param_f64(p, "hold_rate_m_h", 100.0), depth_m: param_f64(p, "depth_m", 60.0), every: param_duration(p, "every", "1h")?, frame_res_arcsec: param_f64(p, "frame_res_arcsec", 3.0),
            scheduled: false, held_m: 0.0, first_work: None, last_held_cells: 0, framed: false, rows: Vec::new(), prev: Vec::new(), land: Vec::new(),
            grid: None, metres: Vec::new(), held: Vec::new(),
        };
        if !["off", "hold", "resources"].contains(&o.mode.as_str()) { return Err(format!("fire.suppression: mode is off, hold or resources, not {}", o.mode)); }
        if o.mode == "resources" && o.reports.is_empty() { return Err("fire.suppression: mode resources needs reports, a status-report table under data/events/".into()); }
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
        self.land = w.elev.iter().map(|e| *e > 0.0).collect();
        self.grid = Some(grid);
        if self.mode == "resources" {
            let path = root.join("data/events").join(&self.reports);
            let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let mut lines = text.lines();
            let head: Vec<&str> = lines.next().unwrap_or("").split(',').collect();
            let col = |n: &str| head.iter().position(|h| *h == n).ok_or(format!("{}: no column {n}", path.display()));
            let (ct, ce, cd, cc, ch, cp) = (col("as_of_local")?, col("engines")?, col("dozers")?, col("hand_crews")?, col("helicopters")?, col("containment_pct")?);
            for l in lines {
                let f: Vec<&str> = l.split(',').collect();
                let num = |i: usize| f.get(i).and_then(|v| v.trim().parse::<f64>().ok()).unwrap_or(0.0);
                let Some(t) = f.get(ct).and_then(|v| Time::parse(&format!("{}Z", v.trim())).ok()) else { continue };
                self.rows.push(Row { t: t.0 - (self.reports_tz_offset_h * 3600.0) as i64, engines: num(ce), dozers: num(cd), crews: num(cc), helicopters: num(ch), containment_pct: num(cp) });
            }
            self.rows.sort_by_key(|r| r.t);
            if self.rows.is_empty() { return Err(format!("{}: no report rows", path.display())); }
        }
        Ok(())
    }
    /// The latest report at or before an instant.
    fn row_at(&self, t: i64) -> Row { self.rows.iter().take_while(|r| r.t <= t).last().cloned().unwrap_or_default() }
    /// Metres from every cell to the nearest cell of a set: a two-pass chamfer transform.
    fn distance_m(&self, from: &[bool]) -> Vec<f32> {
        let g = self.grid.as_ref().unwrap();
        let (mx, my) = self.metres[g.ny / 2]; let (mx, my) = (mx as f32, my as f32); let md = (mx * mx + my * my).sqrt();
        let mut d: Vec<f32> = from.iter().map(|b| if *b { 0.0 } else { f32::INFINITY }).collect();
        let (nx, ny) = (g.nx, g.ny);
        for r in 0..ny { for c in 0..nx {
            let k = r * nx + c; let mut v = d[k];
            if c > 0 { v = v.min(d[k - 1] + mx); }
            if r > 0 { v = v.min(d[k - nx] + my); if c > 0 { v = v.min(d[k - nx - 1] + md); } if c + 1 < nx { v = v.min(d[k - nx + 1] + md); } }
            d[k] = v;
        } }
        for r in (0..ny).rev() { for c in (0..nx).rev() {
            let k = r * nx + c; let mut v = d[k];
            if c + 1 < nx { v = v.min(d[k + 1] + mx); }
            if r + 1 < ny { v = v.min(d[k + nx] + my); if c > 0 { v = v.min(d[k + nx - 1] + md); } if c + 1 < nx { v = v.min(d[k + nx + 1] + md); } }
            d[k] = v;
        } }
        d
    }
    /// One hour of work on the fire's edge.
    fn work(&mut self, ctx: &mut Ctx) {
        let now = ctx.now().0;
        let Some(g) = self.grid.clone() else { return };
        let n = g.nx * g.ny;
        let Some(burned) = ctx.world_read("fire.burned_frac").filter(|f| f.data.len() == n).map(|f| f.data.iter().map(|x| *x >= 0.5).collect::<Vec<bool>>()) else { return };
        let row = self.row_at(now);
        let hours = self.every.as_secs_f64() / 3600.0;
        let capacity = self.share * hours * (row.engines * self.engine_m_h + row.dozers * self.dozer_m_h + row.crews * self.crew_m_h + row.helicopters * self.helicopter_m_h);
        let (mx, my) = self.metres[g.ny / 2]; let cell = (mx * my).sqrt();
        // the edge: unburned land beside burned ground
        let mut edge = Vec::new();
        for r in 0..g.ny { for c in 0..g.nx {
            let k = r * g.nx + c;
            if burned[k] || !self.land[k] { continue; }
            let mut beside = false;
            'o: for dr in -1i64..=1 { for dc in -1i64..=1 {
                let (rr, cc) = (r as i64 + dr, c as i64 + dc);
                if rr < 0 || cc < 0 || rr >= g.ny as i64 || cc >= g.nx as i64 { continue; }
                if burned[rr as usize * g.nx + cc as usize] { beside = true; break 'o; }
            } }
            if beside { edge.push(k); }
        } }
        let had = self.prev.len() == n && self.prev.iter().any(|b| *b);
        let (mut used, mut holdable_m, mut fast_m) = (0.0f64, 0.0f64, 0.0f64);
        let mut cells = Vec::new();
        if had {
            // how far the fire came to reach each stretch of edge in the hour: the distance to the ground
            // that was burned an hour ago, less the one cell an unmoved edge already stands off
            let dist = self.distance_m(&self.prev);
            let mut cand: Vec<(f32, usize)> = Vec::new();
            for &k in &edge {
                if self.held[k] { continue; }
                let adv = ((dist[k] as f64 - 1.5 * cell).max(0.0) / hours) as f32;
                if adv as f64 <= self.hold_rate_m_h { cand.push((adv, k)); holdable_m += cell; } else { fast_m += cell; }
            }
            cand.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap().then(a.1.cmp(&b.1)));
            let (rc, rr) = ((self.depth_m / mx).ceil() as i64, (self.depth_m / my).ceil() as i64);
            for (_, k) in cand {
                if used + cell > capacity { break; }
                used += cell;
                if self.held[k] { continue; } // already under a neighbour's hold this hour; the work is still counted
                let (c, r) = ((k % g.nx) as i64, (k / g.nx) as i64);
                for dr in -rr..=rr { for dc in -rc..=rc {
                    let (cc, r2) = (c + dc, r + dr);
                    if cc < 0 || r2 < 0 || cc >= g.nx as i64 || r2 >= g.ny as i64 { continue; }
                    if ((dc as f64 * mx).powi(2) + (dr as f64 * my).powi(2)).sqrt() > self.depth_m.max(cell * 0.5) { continue; }
                    let j = r2 as usize * g.nx + cc as usize;
                    if !burned[j] && !self.held[j] { self.held[j] = true; cells.push(j); }
                } }
            }
        }
        if !cells.is_empty() {
            if self.first_work.is_none() {
                self.first_work = Some(now);
                ctx.publish("suppression.crossing", json!({ "kind": "first_hold", "time": Time(now).to_iso(), "sentence": format!("Crews first hold a stretch of the fire's edge at {} UTC, with {:.0} engines, {:.0} dozers, {:.0} hand crews and {:.0} helicopters in the latest report.", &Time(now).to_iso()[..16].replace('T', " "), row.engines, row.dozers, row.crews, row.helicopters) }));
            }
            self.lines += 1; self.cells += cells.len(); self.held_m += used;
            ctx.publish_at(cells[0], "ground.break", json!({ "cells": cells, "strength": self.strength, "from": Time(now).to_iso(), "until": serde_json::Value::Null, "category": "Held edge", "label": "", "length_m": used }));
        }
        let held_edge = edge.iter().filter(|k| self.held[**k]).count();
        ctx.report("suppression.engines", "count", row.engines); ctx.report("suppression.dozers", "count", row.dozers);
        ctx.report("suppression.hand_crews", "count", row.crews); ctx.report("suppression.helicopters", "count", row.helicopters);
        ctx.report("suppression.capacity_m_h", "m/h", capacity / hours);
        ctx.report("suppression.held_m_h", "m/h", used / hours);
        ctx.report("suppression.held_m", "m", self.held_m);
        ctx.report("suppression.edge_m", "m", edge.len() as f64 * cell);
        ctx.report("suppression.edge_too_fast_m", "m", fast_m);
        ctx.report("suppression.edge_holdable_m", "m", holdable_m);
        ctx.report("suppression.containment_pct", "%", if edge.is_empty() { 0.0 } else { 100.0 * held_edge as f64 / edge.len() as f64 });
        ctx.report("suppression.reported_containment_pct", "%", row.containment_pct);
        ctx.report("suppression.cells_held", "cells", self.cells as f64);
        if self.cells != self.last_held_cells || !self.scheduled_frame() {
            let k = (self.frame_res_arcsec / self.res_arcsec).round().max(1.0) as usize;
            let fg = LatLonGrid { nx: g.nx.div_ceil(k), ny: g.ny.div_ceil(k), dlon: g.dlon * k as f64, dlat: g.dlat * k as f64, lon_w: g.lon_w, lat_n: g.lat_n };
            let mut frame = vec![0f32; fg.nx * fg.ny];
            for fr in 0..fg.ny { for fc in 0..fg.nx {
                let (mut cnt, mut inside) = (0usize, 0usize);
                for r in fr * k..((fr + 1) * k).min(g.ny) { for c in fc * k..((fc + 1) * k).min(g.nx) { cnt += 1; if self.held[r * g.nx + c] { inside += 1; } } }
                frame[fr * fg.nx + fc] = if cnt > 0 { inside as f32 / cnt as f32 } else { 0.0 };
            } }
            ctx.field("edge_held", "1", &fg, frame);
            self.last_held_cells = self.cells; self.framed = true;
        }
        self.prev = burned;
    }
    fn scheduled_frame(&self) -> bool { self.framed }
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
        if self.mode == "hold" { ctx.subscribe("record.event"); }
        if self.mode == "resources" && !self.scheduled { ctx.schedule_with_priority(self.every, 9, "suppression.hour", json!(null)); self.scheduled = true; }
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic == "suppression.hour" && self.mode == "resources" {
            self.work(ctx);
            ctx.schedule_with_priority(self.every, 9, "suppression.hour", json!(null));
            return;
        }
        if ev.topic != "record.event" || self.mode != "hold" { return; }
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
    fn snapshot(&self) -> serde_json::Value {
        let ix = |v: &[bool]| v.iter().enumerate().filter(|(_, b)| **b).map(|(i, _)| i as u32).collect::<Vec<u32>>();
        let mut v = serde_json::to_value(self).unwrap();
        v["held_ix"] = json!(ix(&self.held)); v["prev_ix"] = json!(ix(&self.prev)); v["prev_len"] = json!(self.prev.len());
        v
    }
    fn restore(&mut self, v: serde_json::Value) {
        let mut o: Suppression = serde_json::from_value(v.clone()).unwrap();
        if o.mode != "off" { o.load().expect("suppression reload"); }
        let ix = |k: &str| v.get(k).and_then(|a| a.as_array()).map(|a| a.iter().filter_map(|x| x.as_u64().map(|x| x as usize)).collect::<Vec<usize>>()).unwrap_or_default();
        for k in ix("held_ix") { if k < o.held.len() { o.held[k] = true; } }
        o.prev = vec![false; v.get("prev_len").and_then(|x| x.as_u64()).unwrap_or(0) as usize];
        for k in ix("prev_ix") { if k < o.prev.len() { o.prev[k] = true; } }
        *self = o;
    }
}

impl Suppression {
    pub fn describe(name: &str, p: &toml::Value) -> esim_kernel::describe::Describe {
        let mode = param_str(p, "mode", "off").to_string();
        let mut d = esim_kernel::describe::Describe::new("fire.suppression", name, "What people did to the fire, as a module of its own: each completed line in the record (obs.record's record.event of kind line) becomes a ground.break, the cells under it to a stated width, of a stated strength, from the line's recorded time on. Any fire engine that subscribes to ground.break refuses heat and embers there; the engine never reads the record itself, so the same engine runs another fire with another record, or none. The record's times are when the GIS feature was made, which lags the work; obs.record's time_shift_h is the declared way to move them. Resources as agents with a ledger are the next form.")
            .param("lat_s", 34.0, "deg", "the fire's window, so the cells match the engine's").param("lat_n", 34.16, "deg", "").param("lon_w", -118.72, "deg", "").param("lon_e", -118.46, "deg", "").param("res_arcsec", 1.0, "arcsec", "")
            .param("mode", "off", "", "off: nothing is published, the record is drawn and not defended; hold: completed lines hold from their recorded time; resources: the reported engines, dozers, crews and helicopters hold the fire's edge hour by hour, where it moves slowly enough to be held")
            .param("line_width_m", 30.0, "m", "guess: the width held along a completed line").param("strength", 1.0, "", "the break's strength; engines today treat any strength above zero as unburnable")
            .param("category_prefix", "Completed", "", "which record categories are lines that hold (Completed Dozer Line, Completed Hand Line, Completed Road as Line); planned lines do not").param("skip_status", "Archive", "", "features with this status are ignored")
            .param("reports", "", "", "resources mode: the incident's status reports, a CSV under data/events/ with as_of_local, engines, dozers, hand_crews, helicopters, containment_pct; a blank count is zero, so nothing works the edge before the first report that counts resources")
            .param("reports_tz_offset_h", 0.0, "h", "the reports' clock against UTC")
            .param("share", 0.02, "", "guess, to be fitted: the share of the listed rates that is at work on the fire's edge in an hour. It stands for everything the counts do not say: shifts, travel, crews defending houses and not the edge, and the model's edge being longer than a mapped one because it follows every cell")
            .param("engine_m_h", 240.0, "m/h", "recalled, unchecked: an engine crew's line rate in brush, about 12 chains an hour (NWCG Fireline Handbook, Appendix A)").param("dozer_m_h", 600.0, "m/h", "recalled, unchecked: about 30 chains an hour in brush on a moderate slope").param("crew_m_h", 100.0, "m/h", "recalled, unchecked: a hand crew in chaparral, about 5 chains an hour").param("helicopter_m_h", 200.0, "m/h", "guess: no table gives edge cooled per helicopter hour")
            .param("hold_rate_m_h", 100.0, "m/h", "guess, to be fitted: a stretch of edge that advanced faster than this in the last hour cannot be held and is left to run")
            .param("depth_m", 60.0, "m", "guess: the depth of ground held behind a stretch of edge; it has to exceed the engine's reach or heat crosses it")
            .param("every", "1h", "", "how often the edge is read and worked").param("frame_res_arcsec", 3.0, "arcsec", "the held-ground frames are the share of each block held, on this grid")
            .priority(9).dt("at each line's recorded time")
            .publish("suppression.ready", "{ mode, line_width_m, strength, source }")
            .note("Nothing here is suppression as it happened: no crews on the line, no burnover, no retardant drops, no aircraft, no engines at houses. It is the record's lines as barriers, from the time the record gives.");
        d.layer = "people".into();
        if mode == "resources" {
            d = d.dt("every hour")
                .read("fire.burned_frac", "1", false, "the fire module's burned cells; without them nothing is held")
                .publish("ground.break", "at a cell: { cells, strength, from, until, category, label, length_m }; the ground held behind the stretches of edge worked this hour")
                .publish("suppression.crossing", "{ kind, time, sentence }: the first hour any edge is held")
                .frame("edge_held", "1")
                .asset("copernicus-dem-glo30", "the grid, and which cells are sea")
                .report("suppression.engines", "count").report("suppression.dozers", "count").report("suppression.hand_crews", "count").report("suppression.helicopters", "count")
                .report("suppression.capacity_m_h", "m/h").report("suppression.held_m_h", "m/h").report("suppression.held_m", "m")
                .report("suppression.edge_m", "m").report("suppression.edge_too_fast_m", "m").report("suppression.edge_holdable_m", "m")
                .report("suppression.containment_pct", "%").report("suppression.reported_containment_pct", "%").report("suppression.cells_held", "cells")
                .view_vs("The share of the fire's edge that is held", "suppression.containment_pct", "suppression.reported_containment_pct", "The model's figure is held edge over all edge on land. The incident's containment is a judgement that a line will hold, made by people, and the two are not the same measure; neither is used to set anything.")
                .note("What this is: a rule for where a stated amount of work goes. The counts are the incident's; the rates are recalled from a handbook and unchecked; the share at work and the fastest edge that can be held are guesses to be fitted against the mapped perimeters. Nothing here places a crew, flies an aircraft, defends a house or runs out of water.");
        } else if mode != "off" {
            d = d.subscribe("record.event", "lines from the record")
                .publish("ground.break", "at a cell: { cells, strength, from, until, category, label, length_m }; the cells a line holds from this time")
                .report("suppression.lines", "lines").report("suppression.cells_held", "cells");
        }
        d
    }
}
