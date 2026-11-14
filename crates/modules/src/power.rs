//! `power.structures`: the electricity service from the transmission structures that stand. Each
//! structure fails at the rupture with the probability its row gives; a cell has electricity when
//! the structures nearest it all stand; crews repair a stated number a day, those serving the most
//! people first. The structures are a table in the event folder and the rule its `fragility.toml`.
//! The module knows no utility and no city, and it has no lines, substations or distribution.
//! One of the four subject modules that replaced `hazard.components` on 2026-11-14.
use crate::fragility::{coarse_of, csv, or, table, tf, ts};
use crate::quake::{sample, window_grid};
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Describe, Duration, Event, LatLonGrid, Module};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, Default)]
struct Structure { lat: f64, lon: f64, prob: f64, down: bool, people: f64 }

#[derive(Serialize, Deserialize)]
pub struct Power {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    folder: String, soils: String, coarse_arcsec: f64,
    crews_per_day: f64, served: usize,
    structures: Vec<Structure>,
    coarse: Option<LatLonGrid>, coarse_people: Vec<f32>, nearest: Vec<Vec<usize>>,
    fired: bool, scheduled: bool, day: i64, failed: i64, repaired: i64,
}

impl Power {
    pub fn new(name: &str, p: &toml::Value) -> Result<Power, String> {
        let mut o = Power {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 0.0), lat_n: param_f64(p, "lat_n", 0.0), lon_w: param_f64(p, "lon_w", 0.0), lon_e: param_f64(p, "lon_e", 0.0), res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "").to_string(), soils: param_str(p, "soils", "wet").to_string(), coarse_arcsec: param_f64(p, "coarse_arcsec", 10.0),
            crews_per_day: 12.0, served: 2, structures: Vec::new(), coarse: None, coarse_people: Vec::new(), nearest: Vec::new(),
            fired: false, scheduled: false, day: 0, failed: 0, repaired: 0,
        };
        o.load()?;
        Ok(o)
    }
    fn rule(folder: &str) -> Result<toml::Value, String> { table(folder, "fragility.toml")?.get("pole").cloned().ok_or("fragility.toml: [pole]".into()) }
    fn load(&mut self) -> Result<(), String> {
        let (g, _) = window_grid(self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let t = Self::rule(&self.folder)?;
        self.crews_per_day = tf(&t, "crews_per_day"); self.served = tf(&t, "served_poles").max(1.0) as usize;
        // the table and its column names are the folder's to say; the defaults are the names in the first folder written
        let (file, cx, cy) = (or(ts(&t, "file"), "poles.csv"), or(ts(&t, "x"), "X"), or(ts(&t, "y"), "Y"));
        let cprob = or(ts(&t, &format!("prob_{}", self.soils)), if self.soils == "wet" { "CSZ_wet_Prob" } else { "CSZ_dry_Prob" });
        let (h, rows) = csv(&self.folder, &file)?;
        let col = |k: &str| h.iter().position(|x| x == k).ok_or(format!("{file}: no column {k}"));
        let (ix, iy, ip) = (col(&cx)?, col(&cy)?, col(&cprob)?);
        let mut st: Vec<Structure> = rows.iter().map(|r| Structure { lon: r[ix].parse().unwrap_or(0.0), lat: r[iy].parse().unwrap_or(0.0), prob: r[ip].parse::<f64>().unwrap_or(0.0) / 100.0, down: false, people: 0.0 }).collect();
        // a coarser grid for the service: the people in each cell and the structures nearest it
        let (cg, step) = coarse_of(&g, self.coarse_arcsec, self.res_arcsec);
        let res = sample(&self.folder, "people_resident.tif", &g)?;
        let mut people = vec![0.0f32; cg.nx * cg.ny];
        for r in 0..g.ny { for c in 0..g.nx { let v = res[r * g.nx + c]; if !v.is_nan() { people[(r / step) * cg.nx + c / step] += v; } } }
        let mut nearest = Vec::with_capacity(cg.nx * cg.ny);
        for r in 0..cg.ny { for c in 0..cg.nx {
            let (lat, lon) = (cg.lat_n - (r as f64 + 0.5) * cg.dlat, cg.lon_w + (c as f64 + 0.5) * cg.dlon);
            let kx = 111.0 * lat.to_radians().cos();
            let mut best: Vec<(f64, usize)> = st.iter().enumerate().map(|(i, s)| (((s.lat - lat) * 111.0).powi(2) + ((s.lon - lon) * kx).powi(2), i)).collect();
            best.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            nearest.push(best.iter().take(self.served).map(|x| x.1).collect::<Vec<_>>());
        } }
        for (ci, ps) in nearest.iter().enumerate() { for &pi in ps { st[pi].people += people[ci] as f64; } }
        self.coarse = Some(cg); self.coarse_people = people; self.nearest = nearest; self.structures = st;
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let folder = param_str(p, "folder", "");
        let rule = Self::rule(folder).ok();
        let (crews, served) = rule.as_ref().map(|t| (tf(t, "crews_per_day"), tf(t, "served_poles").max(1.0))).unwrap_or((0.0, 0.0));
        let mut summary = String::from("This module computes electricity service from the transmission structures that are standing. Each structure, a tower or a pole, is a row in a table in the event folder, with a location and a probability of failure for saturated and for dry soil. At the earthquake each structure fails or stands by a random draw against its probability. The modelled area is divided into coarse cells, and a cell has electricity when the structures nearest to it are all standing. Afterwards crews repair a fixed number of structures a day, starting with those that serve the most people. Each day the module writes a map of where electricity is on, which other modules read at the places they care about, and reports the share of residents served. The structures are not connected into a network: the module has no power lines, substations, generation or neighbourhood distribution, so none of those can fail in it and it restores service sooner than a real grid would.");
        if crews > 0.0 { summary.push_str(&format!(" In this run a cell depends on its {served:.0} nearest structures and crews repair {crews:.0} a day; both numbers are in the folder's fragility.toml, which says where they came from.")); }
        let mut d = Describe::new("power.structures", name, &summary)
            .priority(1).dt("1 day after the rupture")
            .param("folder", folder, "", "the folder under data/events that holds this scenario's table of structures, its fragility.toml and its map of residents")
            .param("soils", param_str(p, "soils", "wet"), "", "which of the table's failure probabilities to use: wet (saturated soil) or dry")
            .param("coarse_arcsec", param_f64(p, "coarse_arcsec", 10.0), "arcsec", "the cell size of the grid on which service is computed")
            .subscribe("quake.rupture", "the instant")
            .publish("power.state", "{ day, share } each day: the share of residents with electricity")
            .own("power.up", "1")
            .frame("power_up", "1")
            .report("power.share", "1").report("power.poles_failed", "poles").report("power.poles_repaired", "poles")
            .asset(&format!("events/{folder}"), "the table of structures, fragility.toml and the map of residents")
            .view("map", "electricity service", "power_up", "1 where the structures nearest the cell are standing, on a coarser grid than the other maps")
            .view("counter", "share of people with power", "power.share", "residents in cells with service, as a share of all residents")
            .view("counter", "transmission structures failed", "power.poles_failed", "each failing with the probability its row gives")
            .view("counter", "transmission structures repaired", "power.poles_repaired", "so far, at the crews' daily rate")
            .view("chart", "share of people with power", "power.share", "computed each day from the structures standing")
            .view("text", "what this module leaves out", "Power lines, substations, generation, the switching a real grid does, and the distribution network inside neighbourhoods. A cell's service depends only on its nearest transmission structures. The repair rate is constant and does not depend on roads, fuel or crews' access.", "");
        if let Some(t) = rule { d = d.table("fragility", json!({ "pole": t })); }
        d
    }
    fn state(&self, ctx: &mut Ctx) {
        let cg = self.coarse.clone().unwrap();
        let (mut up_people, mut tot) = (0.0, 0.0);
        let mut up = vec![0.0f32; cg.nx * cg.ny];
        for (ci, ps) in self.nearest.iter().enumerate() { let ok = ps.iter().all(|&pi| !self.structures[pi].down); let pp = self.coarse_people[ci] as f64; tot += pp; if ok { up_people += pp; up[ci] = 1.0; } }
        let share = if tot > 0.0 { up_people / tot } else { 1.0 };
        ctx.world_write("power.up", "1", &cg, up.clone());
        ctx.field("power_up", "1", &cg, up);
        ctx.report("power.share", "1", share); ctx.report("power.poles_failed", "poles", self.failed as f64); ctx.report("power.poles_repaired", "poles", self.repaired as f64);
        ctx.publish("power.state", json!({ "day": self.day, "share": share }));
    }
}

impl Module for Power {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) { ctx.subscribe("quake.rupture"); self.state(ctx); }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "quake.rupture" => {
                if self.fired { return; }
                self.fired = true; self.day = 0;
                let mut failed = 0;
                for s in &mut self.structures { let u = ctx.rng().f64(); s.down = u < s.prob; if s.down { failed += 1; } }
                self.failed = failed;
                self.state(ctx);
                if !self.scheduled { self.scheduled = true; ctx.schedule_with_priority(Duration::days(1), 3, "day", json!(null)); }
            }
            "day" => {
                self.day += 1;
                let mut idx: Vec<usize> = self.structures.iter().enumerate().filter(|(_, s)| s.down).map(|(i, _)| i).collect();
                idx.sort_by(|a, b| self.structures[*b].people.partial_cmp(&self.structures[*a].people).unwrap());
                for &i in idx.iter().take(self.crews_per_day.round() as usize) { self.structures[i].down = false; self.repaired += 1; }
                self.state(ctx);
                ctx.schedule_with_priority(Duration::days(1), 3, "day", json!(null));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { *self = serde_json::from_value(v).unwrap(); }
}
