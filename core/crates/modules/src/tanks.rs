//! `fuel.tanks`: storage tanks, one by one. Each tank sits at its own coordinates, reads the ground
//! there at the rupture and is given a damage state by a draw against a fragility curve; a tank that
//! fails releases a share of its contents set by its age, toward water where its row says so. The
//! tanks are the event folder's `tanks.csv` and the curve and the age rule its `fragility.toml`.
//! The module knows no terminal and no city. One of the four subject modules that replaced
//! `hazard.components` on 2026-11-14.
use crate::fragility::{coarse_of, csv, curve, force_param, forced, table, tf, Curve, Shaking};
use crate::quake::{cell_of, window_grid};
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Describe, Event, LatLonGrid, Module};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, Default)]
struct Tank { id: String, name: String, lat: f64, lon: f64, cell: usize, year: i64, fill_gal: f64, pct_min: f64, pct_max: f64, water: bool, state: u8, down: bool }

#[derive(Serialize, Deserialize)]
pub struct Tanks {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    folder: String, coarse_arcsec: f64, force: Vec<String>, storage_cut: f64, retrofit: bool,
    curve: Curve, design_year: i64, newer_release_max: f64, study_min_gal: f64, study_max_gal: f64,
    tanks: Vec<Tank>, fired: bool, release_gal: f64, release_water_gal: f64,
    coarse: Option<LatLonGrid>,
    #[serde(skip)] grid: Option<LatLonGrid>,
}

impl Tanks {
    pub fn new(name: &str, p: &toml::Value) -> Result<Tanks, String> {
        let mut o = Tanks {
            name: name.to_string(),
            lat_s: crate::fragility::need_f64(p, "lat_s", name)?, lat_n: crate::fragility::need_f64(p, "lat_n", name)?, lon_w: crate::fragility::need_f64(p, "lon_w", name)?, lon_e: crate::fragility::need_f64(p, "lon_e", name)?, res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "").to_string(), coarse_arcsec: param_f64(p, "coarse_arcsec", 10.0), force: force_param(p),
            storage_cut: param_f64(p, "storage_cut", 0.0), retrofit: p.get("retrofit").and_then(|v| v.as_bool()).unwrap_or(false),
            curve: Curve::default(), design_year: 1993, newer_release_max: 0.1, study_min_gal: 0.0, study_max_gal: 0.0,
            tanks: Vec::new(), fired: false, release_gal: 0.0, release_water_gal: 0.0, coarse: None, grid: None,
        };
        o.load()?;
        Ok(o)
    }
    fn rule(folder: &str) -> Result<toml::Value, String> { table(folder, "fragility.toml")?.get("tank").cloned().ok_or("fragility.toml: [tank]".into()) }
    fn load(&mut self) -> Result<(), String> {
        let (g, _) = window_grid(self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let t = Self::rule(&self.folder)?;
        self.curve = curve(&t);
        self.newer_release_max = tf(&t, "post_1993_release_max");
        if tf(&t, "design_year") > 0.0 { self.design_year = tf(&t, "design_year") as i64; }
        self.study_min_gal = tf(&t, "study_release_min_gal"); self.study_max_gal = tf(&t, "study_release_max_gal");
        let (h, rows) = csv(&self.folder, "tanks.csv")?;
        let col = |k: &str| h.iter().position(|x| x == k).ok_or(format!("tanks.csv: no column {k}"));
        let (ci, clat, clon, cfill, cyear, czone, cmin, cmax, cprop, cfail) = (col("id")?, col("lat")?, col("lon")?, col("fill_gal")?, col("year")?, col("zone")?, col("pct_min")?, col("pct_max")?, col("property")?, col("failure")?);
        let mut tanks = Vec::new();
        for r in &rows {
            let f = |i: usize| r.get(i).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            let (lat, lon) = (f(clat), f(clon));
            // a row with no place, or one the source says cannot fail, is not a tank here
            if lat == 0.0 || r[cfail].starts_with("No") { continue; }
            tanks.push(Tank { id: r[ci].clone(), name: format!("{} {}", r[cprop], r[ci]), lat, lon, cell: cell_of(&g, lat, lon), year: r.get(cyear).and_then(|s| s.parse().ok()).unwrap_or(0), fill_gal: f(cfill), pct_min: f(cmin) / 100.0, pct_max: f(cmax) / 100.0, water: r[czone].contains("Water"), state: 0, down: false });
        }
        self.coarse = Some(coarse_of(&g, self.coarse_arcsec, self.res_arcsec).0);
        self.tanks = tanks; self.grid = Some(g);
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let folder = param_str(p, "folder", "");
        let rule = Self::rule(folder).ok();
        let study = rule.as_ref().map(|t| (tf(t, "study_release_min_gal"), tf(t, "study_release_max_gal"))).filter(|s| s.1 > 0.0);
        let mut summary = String::from("This module damages storage tanks one at a time. Each tank sits at its own coordinates. At the earthquake it reads the ground motion there and is assigned a damage state by a random draw against a fragility curve, which gives the probability of each state at a given strength of shaking and of ground failure. A tank at or beyond the curve's failing state releases a share of its contents. The share depends on the tank's age: a tank built before the design year in the folder's rule, or with no recorded year, draws its share from the range in its own row of the table, and a newer tank releases at most the rule's smaller maximum. The release goes toward water where the tank's row says so. The tanks are the event folder's tanks.csv, and the curve and the age rule are its fragility.toml. The module does not follow the fuel after it leaves the tank, and it has no fire.");
        if let Some((lo, hi)) = study { summary.push_str(&format!(" The folder gives a published range for the total release, {:.1} to {:.1} million gallons, and the module reports its own total against both ends.", lo / 1e6, hi / 1e6)); }
        let mut d = Describe::new("fuel.tanks", name, &summary)
            .priority(1).dt("once, at the rupture")
            .param("folder", folder, "", "the folder under data/events that holds this scenario's tanks.csv and fragility.toml")
            .param("coarse_arcsec", param_f64(p, "coarse_arcsec", 10.0), "arcsec", "the cell size of the map of releases")
            .param("storage_cut", param_f64(p, "storage_cut", 0.0), "", "the fraction of every tank's contents removed before the earthquake")
            .param("retrofit", p.get("retrofit").and_then(|v| v.as_bool()).unwrap_or(false), "", "if true, every tank releases as a newer tank does, whatever its year")
            .param("force", json!(force_param(p)), "", "tanks whose damage state is set in advance instead of drawn, written as tank:id:state")
            .subscribe("quake.rupture", "the instant")
            .read_on("quake.pga_g", "g", true, "quake.rupture", "at each tank").read_on("quake.liq_p", "1", true, "quake.rupture", "").read_on("quake.liq_pgd_cm", "cm", true, "quake.rupture", "").read_on("quake.ls_p", "1", true, "quake.rupture", "").read_on("quake.ls_pgd_cm", "cm", true, "quake.rupture", "")
            .publish("component.state", "{ kind: tank, id, name, lat, lon, state, down } at the tank's location, at the start and at the earthquake")
            .publish("tank.release", "{ id, name, gal, river, year } at the tank's location, for each failed tank")
            .publish("tanks.released", "{ gal, river_gal, tanks_failed } once, the total over all tanks, for a module that keeps a fuel account")
            .frame("tank_release_gal", "gal")
            .report("tanks.failed", "tanks").report("tanks.release_gal", "gal").report("tanks.release_river_gal", "gal")
            .asset(&format!("events/{folder}"), "tanks.csv and fragility.toml")
            .view("map", "fuel released, by tank", "tank_release_gal", "gallons escaping from each failed tank at the earthquake, on a coarser grid than the other maps")
            .view("counter", "tanks failed", "tanks.failed", "of the tanks in the folder's table that can fail")
            .view("counter", "fuel released", "tanks.release_gal", "the total over the failed tanks")
            .view("counter", "fuel released toward water", "tanks.release_river_gal", "from the failed tanks whose row places them where a spill reaches water")
            .view("text", "what this module leaves out", "The fragility curve is one curve for every tank, whatever its size, fill or anchorage. Its values are whatever the folder's fragility.toml says, and that file says where they came from. A tank is either failed or not: there is no partial leak, no fire, no containment berm and no account of where the fuel goes after it leaves the tank.", "");
        if study.is_some() { d = d.report("fit.release_vs_study_min", "1").report("fit.release_vs_study_max", "1"); }
        if let Some(t) = rule { d = d.table("fragility", json!({ "tank": t })); }
        d
    }
    fn say(&self, ctx: &mut Ctx, t: &Tank) { ctx.publish_at(t.cell, "component.state", json!({ "kind": "tank", "id": t.id, "name": t.name, "lat": t.lat, "lon": t.lon, "state": t.state, "down": t.down, "repair_day": -1, "day": 0 })); }
    fn report(&self, ctx: &mut Ctx) {
        ctx.report("tanks.failed", "tanks", self.tanks.iter().filter(|t| t.down).count() as f64);
        ctx.report("tanks.release_gal", "gal", self.release_gal); ctx.report("tanks.release_river_gal", "gal", self.release_water_gal);
    }
}

impl Module for Tanks {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.subscribe("quake.rupture");
        let cg = self.coarse.clone().unwrap();
        ctx.field("tank_release_gal", "gal", &cg, vec![0.0; cg.nx * cg.ny]);
        self.report(ctx);
        for t in &self.tanks { self.say(ctx, t); }
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "quake.rupture" || self.fired { return; }
        self.fired = true;
        let g = self.grid.clone().unwrap();
        let shaking = Shaking::read(ctx, g.nx * g.ny);
        let forced = forced(&self.force, "tank");
        let cg = self.coarse.clone().unwrap();
        let mut map = vec![0.0f32; cg.nx * cg.ny];
        let (mut rel, mut rel_water) = (0.0, 0.0);
        let mut releases = Vec::new();
        for i in 0..self.tanks.len() {
            let u = ctx.rng().f64();
            let mut state = shaking.draw(&self.curve, self.tanks[i].cell, u);
            if let Some(s) = forced.get(&self.tanks[i].id) { state = *s; }
            let down = state >= self.curve.fail_state;
            let t = &mut self.tanks[i];
            t.state = state; t.down = down;
            if down {
                let v = ctx.rng().f64();
                let older = t.year == 0 || t.year < self.design_year;
                let frac = if !self.retrofit && older { t.pct_min + (t.pct_max - t.pct_min) * v } else { self.newer_release_max * v };
                let gal = t.fill_gal * (1.0 - self.storage_cut) * frac;
                rel += gal; if t.water { rel_water += gal; }
                map[cell_of(&cg, t.lat, t.lon)] += gal as f32;
                releases.push((t.cell, json!({ "id": t.id, "name": t.name, "gal": gal, "river": t.water, "year": t.year })));
            }
        }
        self.release_gal = rel; self.release_water_gal = rel_water;
        for t in &self.tanks { self.say(ctx, t); }
        for (cell, e) in releases { ctx.publish_at(cell, "tank.release", e); }
        ctx.publish("tanks.released", json!({ "gal": rel, "river_gal": rel_water, "tanks_failed": self.tanks.iter().filter(|t| t.down).count() }));
        ctx.field("tank_release_gal", "gal", &cg, map);
        if self.study_max_gal > 0.0 { ctx.report("fit.release_vs_study_min", "1", rel / self.study_min_gal); ctx.report("fit.release_vs_study_max", "1", rel / self.study_max_gal); }
        self.report(ctx);
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Tanks = serde_json::from_value(v).unwrap(); o.grid = window_grid(o.lat_s, o.lat_n, o.lon_w, o.lon_e, o.res_arcsec).ok().map(|x| x.0); *self = o; }
}
