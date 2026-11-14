//! `health.hospitals`: hospitals and whether each has power. A hospital building is damaged or not
//! at the rupture by a draw against a generic building curve; an undamaged one is open while the
//! grid reaches it, runs on its generator when it does not, asks the depot for fuel when a truck
//! can reach it, and goes dark when the generator is empty. The hospitals are the event folder's
//! `hospitals.toml`; the code names none. Pulled out of `portland.lifelines` on 2026-11-14.
use crate::fragility::{table, tf, ts};
use crate::quake::{cell_of, phi, window_grid};
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Describe, Duration, Event, Module};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Clone, Serialize, Deserialize)]
struct Hospital { name: String, lat: f64, lon: f64, cell: usize, damaged: bool, p_damage: f64, left_h: f64, on_grid: bool, status: String }

#[derive(Serialize, Deserialize)]
pub struct Hospitals {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    folder: String, median_scale: f64, beta: f64, generator_h: f64, gal_day: f64,
    hospitals: Vec<Hospital>, reachable: BTreeMap<String, bool>,
    fired: bool, scheduled: bool, day: i64, said: Vec<String>,
}

impl Hospitals {
    pub fn new(name: &str, p: &toml::Value) -> Result<Hospitals, String> {
        let mut o = Hospitals {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 0.0), lat_n: param_f64(p, "lat_n", 0.0), lon_w: param_f64(p, "lon_w", 0.0), lon_e: param_f64(p, "lon_e", 0.0), res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "").to_string(), median_scale: param_f64(p, "median_scale", 1.5), beta: param_f64(p, "beta", 0.70),
            generator_h: param_f64(p, "generator_h", 72.0), gal_day: param_f64(p, "gal_day", 2000.0),
            hospitals: Vec::new(), reachable: BTreeMap::new(), fired: false, scheduled: false, day: 0, said: Vec::new(),
        };
        let (g, _) = window_grid(o.lat_s, o.lat_n, o.lon_w, o.lon_e, o.res_arcsec)?;
        let h = table(&o.folder, "hospitals.toml")?;
        let full = o.generator_h;
        o.hospitals = h.get("hospital").and_then(|v| v.as_array()).ok_or("hospitals.toml has no [[hospital]]")?.iter().map(|t| { let (lat, lon) = (tf(t, "lat"), tf(t, "lon")); Hospital { name: ts(t, "name"), lat, lon, cell: cell_of(&g, lat, lon), damaged: false, p_damage: 0.0, left_h: full, on_grid: true, status: "open".into() } }).collect();
        Ok(o)
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let folder = param_str(p, "folder", "");
        let n = table(folder, "hospitals.toml").ok().and_then(|t| t.get("hospital").and_then(|v| v.as_array()).map(|a| a.len())).unwrap_or(0);
        let mut summary = String::from("This module follows hospitals and whether each one has power. At the earthquake each hospital building is damaged or not by a random draw against a generic building fragility curve at the ground motion at its location, with an allowance for hospitals being built to a newer code. An undamaged hospital is open while the grid reaches it, which the module learns by reading the power module's map of service at the hospital. When the grid does not reach it, the hospital runs on its generator and uses up its stored fuel. If the roads module reports that a truck can reach it, it asks the fuel depot for a day's delivery, and a granted delivery refills the generator. A hospital whose generator fuel runs out is without power until the grid or a delivery returns. The module does not represent beds, staff, supplies or damage inside a building that still stands, so an open hospital here is a building with power and nothing more.");
        if n > 0 { summary.push_str(&format!(" In this run there are {n} hospitals, from the event folder's hospitals.toml.")); }
        Describe::new("health.hospitals", name, &summary)
            .priority(2).dt("1 day after the rupture")
            .param("folder", folder, "", "the folder under data/events that holds this scenario's hospitals.toml")
            .param("median_scale", param_f64(p, "median_scale", 1.5), "", "hospitals are assumed to be stronger than the generic building: the shaking needed to damage one is multiplied by this. A guess, standing for newer building codes")
            .param("beta", param_f64(p, "beta", 0.70), "", "the spread of the fragility curve (recalled)")
            .param("generator_h", param_f64(p, "generator_h", 72.0), "h", "hours of generator fuel stored at each hospital. 72 hours is a common planning figure, used here as a guess")
            .param("gal_day", param_f64(p, "gal_day", 2000.0), "gal/d", "gallons of diesel one hospital burns in a day on its generators, and so the size of a delivery it asks for (a guess)")
            .subscribe("quake.rupture", "the instant").subscribe("routes.state", "which hospitals a truck can reach from the depot").subscribe("fuel.granted", "a delivery the depot has met")
            .read_on("quake.pga_g", "g", true, "quake.rupture", "at each hospital").read_on("quake.liq_p", "1", true, "quake.rupture", "").read_on("quake.liq_pgd_cm", "cm", true, "quake.rupture", "")
            .read_on("power.up", "1", false, "day", "whether the grid reaches each hospital")
            .publish("fuel.request", "{ who, gal } a day's generator fuel, asked of the depot when a truck can reach the hospital")
            .publish("hospitals.state", "{ day, open, dark, closed } each day")
            .publish("lifeline.marked", "{ kind: hospital, name, lat, lon } at the start").publish("lifeline.failed", "{ kind: hospital, name, status } closed by damage, or without power").publish("lifeline.restored", "{ kind: hospital, name, day, status } with power again")
            .publish("hospital.crossing", "{ kind, day, sentence, source, confidence } the sentences shown on the run page")
            .report("lifeline.hospitals_open", "hospitals").report("lifeline.hospitals_dark", "hospitals").report("lifeline.hospitals_closed", "hospitals")
            .asset(&format!("events/{folder}"), "hospitals.toml")
            .view("counter", "hospitals open", "lifeline.hospitals_open", "undamaged, with grid electricity or a generator that has fuel")
            .view("counter", "hospitals without power", "lifeline.hospitals_dark", "undamaged, but with no grid electricity and no generator fuel")
            .view("counter", "hospitals closed by damage", "lifeline.hospitals_closed", "")
            .view("chart", "hospitals open", "lifeline.hospitals_open", "")
            .view("chart", "hospitals without power", "lifeline.hospitals_dark", "")
            .view("text", "what this module leaves out", "Beds, staff, supplies, water, and damage to equipment inside a building that still stands. Every hospital uses the same generic curve, the same generator fuel and the same daily burn, all of them guesses. A delivery is all or nothing and arrives the day it is asked for. Patients do not travel: the module does not know which hospital anyone can reach.", "")
    }
    fn say(&mut self, ctx: &mut Ctx, id: &str, sentence: String, source: &str, confidence: &str) {
        if self.said.iter().any(|s| s == id) { return; }
        self.said.push(id.to_string());
        ctx.publish("hospital.crossing", json!({ "id": id, "title": id, "name": "hospitals", "kind": "hospitals", "day": self.day, "time": ctx.now().to_iso(), "sentence": sentence, "source": source, "confidence": confidence }));
    }
    fn count(&self, s: &[&str]) -> usize { self.hospitals.iter().filter(|h| s.contains(&h.status.as_str())).count() }
    fn report(&self, ctx: &mut Ctx) {
        ctx.report("lifeline.hospitals_open", "hospitals", self.count(&["open", "generator"]) as f64);
        ctx.report("lifeline.hospitals_dark", "hospitals", self.count(&["dark"]) as f64);
        ctx.report("lifeline.hospitals_closed", "hospitals", self.count(&["closed"]) as f64);
    }
    fn state(&self, ctx: &mut Ctx) { ctx.publish("hospitals.state", json!({ "day": self.day, "open": self.count(&["open", "generator"]), "dark": self.count(&["dark"]), "closed": self.count(&["closed"]) })); }
}

impl Module for Hospitals {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.subscribe("quake.rupture"); ctx.subscribe("routes.state"); ctx.subscribe("fuel.granted");
        for h in &self.hospitals { ctx.publish_at(h.cell, "lifeline.marked", json!({ "kind": "hospital", "name": h.name, "lat": h.lat, "lon": h.lon })); }
        self.report(ctx);
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "routes.state" => { if let Some(m) = ev.data.get("reachable").and_then(|v| v.as_object()) { self.reachable = m.iter().map(|(k, v)| (k.clone(), v.as_bool().unwrap_or(false))).collect(); } }
            "fuel.granted" => { let who = ev.data.get("who").and_then(|v| v.as_str()).unwrap_or(""); let full = self.generator_h; for h in self.hospitals.iter_mut().filter(|h| h.name == who) { h.left_h = full; } }
            "quake.rupture" => {
                if self.fired { return; }
                self.fired = true; self.day = 0;
                let take = |name: &str| -> Vec<f32> { ctx.world_read(name).map(|f| f.data.clone()).unwrap_or_default() };
                let (pga, liq_p, liq_pgd) = (take("quake.pga_g"), take("quake.liq_p"), take("quake.liq_pgd_cm"));
                let (scale, beta) = (self.median_scale, self.beta);
                let mut closed = Vec::new();
                for i in 0..self.hospitals.len() {
                    let cell = self.hospitals[i].cell;
                    // the generic building's median for extensive damage, 0.80 g, raised by the allowance; ground failure as the buildings module has it
                    let a = pga.get(cell).copied().unwrap_or(f32::NAN) as f64;
                    let p_shake = if a.is_nan() || a <= 0.0 { 0.0 } else { phi((a / (0.80 * scale)).ln() / beta) };
                    let (lp, pg) = (liq_p.get(cell).copied().unwrap_or(0.0) as f64, liq_pgd.get(cell).copied().unwrap_or(0.0) as f64);
                    let p_gf = if lp > 0.0 && pg > 0.0 { lp * phi((pg / 152.0).ln() / 1.2) } else { 0.0 };
                    let u = ctx.rng().f64();
                    let h = &mut self.hospitals[i];
                    h.p_damage = 1.0 - (1.0 - p_shake) * (1.0 - p_gf);
                    h.damaged = u < h.p_damage;
                    h.status = if h.damaged { "closed".into() } else { "generator".into() };
                    if h.damaged { closed.push(h.name.clone()); }
                }
                for h in self.hospitals.clone() { if h.damaged { ctx.publish_at(h.cell, "lifeline.failed", json!({ "kind": "hospital", "name": h.name, "status": "closed", "p_damage": h.p_damage })); } }
                let (n, open) = (self.hospitals.len(), self.hospitals.len() - closed.len());
                let hours = self.generator_h;
                self.say(ctx, "hospitals", format!("{} of {} hospitals are undamaged{}{}. Each has a backup generator.", open, n, if closed.is_empty() { String::new() } else { format!(" and {} are closed by damage: ", closed.len()) }, closed.join(", ")),
                    &format!("A generic building fragility curve at each hospital's location, with a guessed allowance for newer building codes, and a random draw. Each hospital is assumed to hold generator fuel for {hours:.0} hours, which is a guess"), "a guess");
                self.report(ctx); self.state(ctx);
                if !self.scheduled { self.scheduled = true; ctx.schedule_with_priority(Duration::days(1), 6, "day", json!(null)); }
            }
            "day" => {
                self.day += 1;
                // whether the grid reaches a place: the power module's map of service, read at the place
                let power = ctx.world_read("power.up").map(|f| (f.grid.clone(), f.data.clone()));
                let grid_at = |lat: f64, lon: f64| power.as_ref().map(|(g, d)| d.get(cell_of(g, lat, lon)).copied().unwrap_or(0.0) > 0.5).unwrap_or(false);
                let (d, full, gal) = (self.day, self.generator_h, self.gal_day);
                for i in 0..self.hospitals.len() {
                    let h = self.hospitals[i].clone();
                    if h.damaged { continue; }
                    if grid_at(h.lat, h.lon) {
                        if h.status != "open" { ctx.publish_at(h.cell, "lifeline.restored", json!({ "kind": "hospital", "name": h.name, "day": d, "status": "open" })); }
                        self.hospitals[i].status = "open".into(); self.hospitals[i].on_grid = true;
                        continue;
                    }
                    // a day on the generator; a delivery, if a truck can reach it and the depot grants one, refills it before the day is settled
                    let left = h.left_h - 24.0;
                    self.hospitals[i].left_h = left; self.hospitals[i].on_grid = false;
                    if self.reachable.get(&h.name).copied().unwrap_or(false) && left < full { ctx.publish("fuel.request", json!({ "who": h.name, "gal": gal })); }
                }
                ctx.schedule_with_priority(Duration::secs(0), 7, "settle", json!(null));
                ctx.schedule_with_priority(Duration::days(1), 6, "day", json!(null));
            }
            "settle" => {
                let d = self.day; let mut dark_now = Vec::new();
                for i in 0..self.hospitals.len() {
                    let h = self.hospitals[i].clone();
                    if h.damaged || h.on_grid { continue; }
                    if h.left_h <= 0.0 {
                        if h.status != "dark" { dark_now.push(h.name.clone()); ctx.publish_at(h.cell, "lifeline.failed", json!({ "kind": "hospital", "name": h.name, "status": "dark", "day": d })); }
                        self.hospitals[i].status = "dark".into(); self.hospitals[i].left_h = 0.0;
                    } else {
                        if h.status == "dark" { ctx.publish_at(h.cell, "lifeline.restored", json!({ "kind": "hospital", "name": h.name, "day": d, "status": "generator" })); }
                        self.hospitals[i].status = "generator".into();
                    }
                }
                if !dark_now.is_empty() { let hours = self.generator_h; self.say(ctx, "dark", format!("Day {}: these hospitals use up their generator fuel before any is delivered, and lose power: {}.", d, dark_now.join(", ")), &format!("Each hospital is assumed to hold {hours:.0} hours of generator fuel, which is a guess. A delivery needs a route of open road segments and bridges from the depot to the hospital, and fuel left at the depot"), "a guess"); }
                self.report(ctx); self.state(ctx);
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { *self = serde_json::from_value(v).unwrap(); }
}
