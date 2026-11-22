//! `fuel.depot`: a fuel depot's account. One stock of fuel in gallons, held in a ledger that must
//! close: what the tanks release at the earthquake leaves it for the ground and the water, the
//! emergency response draws on it each day once the roads let a truck out, anyone may ask it for a
//! delivery, and resupply tops it back up from a stated day. It also carries the published range
//! of deaths and injuries at the depot itself. The depot is the event folder's `hub.toml`; the code
//! names no place. Pulled out of `portland.lifelines` on 2026-11-14.
use crate::fragility::{or, table, tf, ts};
use crate::lifelines::commas;
use crate::quake::{cell_of, window_grid};
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Describe, Duration, Event, Module};
use serde::{Deserialize, Serialize};
use serde_json::json;

const FUEL: &str = "fuel";

#[derive(Serialize, Deserialize)]
pub struct Depot {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    folder: String, storage_cut: f64, emergency_gal_day: f64, resupply_day: i64, resupply_gal_day: f64,
    place: String, lat: f64, lon: f64, cell: usize, contents_gal: f64, release_max_gal: f64,
    toll_workers: f64, toll_deaths: (f64, f64), toll_injuries: (f64, f64), toll_source: String,
    stock_gal: f64, water_gal: f64, ground_gal: f64, used_gal: f64, delivered_gal: f64, asked_gal: f64,
    deaths: f64, injuries: f64,
    fired: bool, scheduled: bool, day: i64, trucks: bool, said: Vec<String>,
}

impl Depot {
    pub fn new(name: &str, p: &toml::Value) -> Result<Depot, String> {
        let mut o = Depot {
            name: name.to_string(),
            lat_s: crate::fragility::need_f64(p, "lat_s", name)?, lat_n: crate::fragility::need_f64(p, "lat_n", name)?, lon_w: crate::fragility::need_f64(p, "lon_w", name)?, lon_e: crate::fragility::need_f64(p, "lon_e", name)?, res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "").to_string(), storage_cut: param_f64(p, "storage_cut", 0.0),
            emergency_gal_day: param_f64(p, "emergency_gal_day", 300000.0), resupply_day: param_f64(p, "resupply_day", 14.0) as i64, resupply_gal_day: param_f64(p, "resupply_gal_day", 1000000.0),
            place: String::new(), lat: 0.0, lon: 0.0, cell: 0, contents_gal: 0.0, release_max_gal: 0.0,
            toll_workers: 0.0, toll_deaths: (0.0, 0.0), toll_injuries: (0.0, 0.0), toll_source: String::new(),
            stock_gal: 0.0, water_gal: 0.0, ground_gal: 0.0, used_gal: 0.0, delivered_gal: 0.0, asked_gal: 0.0, deaths: 0.0, injuries: 0.0,
            fired: false, scheduled: false, day: 0, trucks: false, said: Vec::new(),
        };
        let (g, _) = window_grid(o.lat_s, o.lat_n, o.lon_w, o.lon_e, o.res_arcsec)?;
        let hb = table(&o.folder, "hub.toml")?; let h = hb.get("hub").ok_or("hub.toml: [hub]")?;
        o.place = or(ts(h, "name"), "the depot"); o.lat = tf(h, "lat"); o.lon = tf(h, "lon"); o.cell = cell_of(&g, o.lat, o.lon);
        o.contents_gal = tf(h, "contents_gal"); o.release_max_gal = tf(h, "release_max_gal");
        o.toll_workers = tf(h, "toll_workers"); o.toll_deaths = (tf(h, "toll_deaths_min"), tf(h, "toll_deaths_max")); o.toll_injuries = (tf(h, "toll_injuries_min"), tf(h, "toll_injuries_max"));
        o.toll_source = ts(h, "toll_source");
        Ok(o)
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let folder = param_str(p, "folder", "");
        let h = table(folder, "hub.toml").ok().and_then(|t| t.get("hub").cloned());
        let mut summary = String::from("This module keeps the account of one fuel depot, in gallons, in a ledger that must balance at every close. The depot starts with the contents the event folder gives it, less any cut made before the earthquake. When the tanks module reports what the tanks released, that fuel leaves the stock for the ground and for the water. Each day afterwards the emergency response draws a fixed amount, but only once the roads module reports that a truck can get from the depot to at least one place. Any module may ask for a delivery, and the depot grants it while it has the fuel. From a stated day, resupply tops the stock back up toward what it held. The module also carries a published range for deaths and injuries at the depot itself, where the folder gives one: it draws a value in the range at random and scales it by the size of the release. It does not model fire, and it does not follow the spilled fuel.");
        if let Some(h) = &h { summary.push_str(&format!(" In this run the depot is {}, holding {:.1} million gallons before any cut.", or(ts(h, "name"), "the depot"), tf(h, "contents_gal") / 1e6)); }
        Describe::new("fuel.depot", name, &summary)
            .priority(2).dt("1 day after the rupture")
            .param("folder", folder, "", "the folder under data/events that holds this scenario's hub.toml: where the depot is, what it holds, and the published range of deaths there")
            .param("storage_cut", param_f64(p, "storage_cut", 0.0), "", "the fraction of the depot's fuel removed before the earthquake")
            .param("emergency_gal_day", param_f64(p, "emergency_gal_day", 300000.0), "gal/d", "gallons the emergency response (fire, police, utilities, shelters) takes each day once a truck can leave the depot. A guess")
            .param("resupply_day", param_f64(p, "resupply_day", 14.0), "d", "the day fuel deliveries to the depot resume. A guess")
            .param("resupply_gal_day", param_f64(p, "resupply_gal_day", 1000000.0), "gal/d", "gallons arriving each day after that. A guess")
            .subscribe("quake.rupture", "the instant").subscribe("tanks.released", "what the tanks released").subscribe("routes.state", "whether a truck can get from the depot to any place").subscribe("fuel.request", "{ who, gal }: a module asking for a delivery")
            .publish("fuel.granted", "{ who, gal } when a request is met")
            .publish("fuel.state", "{ day, stock_gal, trucks } each day")
            .publish("depot.toll", "{ deaths, injuries } once, at the depot itself")
            .publish("lifeline.marked", "{ kind: hub, name, lat, lon } where the depot is, at the start").publish("lifeline.failed", "{ kind: hub, name, released_gal, river_gal } when the tanks release")
            .publish("depot.crossing", "{ kind, day, sentence, source, confidence } the sentences shown on the run page: the spill, deaths at the depot, resupply, the stock running out")
            .ledger(FUEL, "gal", 1.0)
            .report("fuel.hub_gal", "gal").report("fuel.river_gal", "gal").report("fuel.ground_gal", "gal").report("fuel.consumed_gal", "gal").report("fuel.delivered_gal", "gal").report("fuel.days_left", "d")
            .report("hub.deaths", "people").report("hub.injuries", "people")
            .asset(&format!("events/{folder}"), "hub.toml")
            .view("counter", "fuel at the depot", "fuel.hub_gal", "gallons still in the tanks")
            .view("counter", "fuel spilled toward water", "fuel.river_gal", "gallons the tanks released toward water")
            .view("counter", "fuel spilled on the ground", "fuel.ground_gal", "")
            .view("counter", "fuel used by the response", "fuel.consumed_gal", "the emergency response's daily draw and the deliveries granted")
            .view("counter", "days of fuel left", "fuel.days_left", "the stock divided by the day's use")
            .view("chart", "fuel at the depot", "fuel.hub_gal", "")
            .view("chart", "fuel used by the response", "fuel.consumed_gal", "")
            .view("text", "what this module leaves out", "The fuel is one stock. There are no pipes, no individual trucks, no grades of fuel and no priorities among users: a request is met if the gallons are there. The response's daily use and the day and rate of resupply are guesses, so the module can say how much is left but not how long it would really last. The response draws nothing until a truck can reach a place by road, which leaves it using no fuel for as long as the roads are shut. Fire in the spilled fuel is not modelled; the deaths at the depot are a published range, not a computed result.", "")
    }
    fn say(&mut self, ctx: &mut Ctx, id: &str, sentence: String, source: &str, confidence: &str) {
        if self.said.iter().any(|s| s == id) { return; }
        self.said.push(id.to_string());
        ctx.publish("depot.crossing", json!({ "id": id, "title": id, "name": "depot", "kind": "depot", "day": self.day, "time": ctx.now().to_iso(), "sentence": sentence, "source": source, "confidence": confidence }));
    }
    fn report(&self, ctx: &mut Ctx) {
        // the stocks as held, for every close: the ledger compares them with what the flows imply
        ctx.declare_stock(FUEL, "hub", self.stock_gal);
        ctx.declare_stock(FUEL, "ground", self.ground_gal); ctx.declare_stock(FUEL, "river", self.water_gal);
        ctx.declare_stock(FUEL, "burned", self.used_gal); ctx.declare_stock(FUEL, "pipeline", 1.0e9 - self.delivered_gal);
        ctx.report("fuel.hub_gal", "gal", self.stock_gal); ctx.report("fuel.river_gal", "gal", self.water_gal); ctx.report("fuel.ground_gal", "gal", self.ground_gal);
        ctx.report("fuel.consumed_gal", "gal", self.used_gal); ctx.report("fuel.delivered_gal", "gal", self.delivered_gal);
        let use_day = self.emergency_gal_day + self.asked_gal;
        ctx.report("fuel.days_left", "d", if use_day > 0.0 { self.stock_gal / use_day } else { f64::NAN });
        ctx.report("hub.deaths", "people", self.deaths); ctx.report("hub.injuries", "people", self.injuries);
    }
}

impl Module for Depot {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.subscribe("quake.rupture"); ctx.subscribe("tanks.released"); ctx.subscribe("routes.state"); ctx.subscribe("fuel.request");
        ctx.ledger(FUEL, "gal", 1.0);
        if !self.scheduled {
            self.scheduled = true;
            self.stock_gal = self.contents_gal * (1.0 - self.storage_cut);
            ctx.declare_stock(FUEL, "hub", self.stock_gal);
            ctx.declare_stock(FUEL, "ground", 0.0); ctx.declare_stock(FUEL, "river", 0.0); ctx.declare_stock(FUEL, "burned", 0.0);
            ctx.declare_stock(FUEL, "pipeline", 1.0e9);   // the world outside the modelled area: the pipeline and the barges
        }
        ctx.publish_at(self.cell, "lifeline.marked", json!({ "kind": "hub", "name": self.place, "lat": self.lat, "lon": self.lon, "contents_gal": self.stock_gal }));
        self.report(ctx);
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        let f = |k: &str| ev.data.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
        match ev.topic.as_str() {
            "tanks.released" => {
                let before = self.stock_gal;
                let released = f("gal").min(self.stock_gal); let water = f("river_gal").min(released); let ground = released - water;
                ctx.transfer(FUEL, "hub", "river", water); ctx.transfer(FUEL, "hub", "ground", ground);
                self.stock_gal -= released; self.water_gal += water; self.ground_gal += ground;
                ctx.publish_at(self.cell, "lifeline.failed", json!({ "kind": "hub", "name": self.place, "released_gal": released, "river_gal": water }));
                let lp = ctx.world_read("quake.liq_p").and_then(|w| w.data.get(self.cell).copied()).unwrap_or(f32::NAN);
                let n = ev.data.get("tanks_failed").and_then(|v| v.as_u64()).unwrap_or(0);
                let place = self.place.clone();
                self.say(ctx, "spill", format!("Within minutes of the earthquake, about {:.0} million gallons of fuel escape from the tanks at the {}, {:.0} percent of the {:.0} million gallons stored there. About {:.0} million gallons spill onto the ground and {:.0} million flow toward water. The ground maps give a {:.0} percent chance that the ground under the depot spreads sideways as the soil liquefies.", released / 1e6, place, if before > 0.0 { released / before * 100.0 } else { 0.0 }, before / 1e6, ground / 1e6, water / 1e6, if lp.is_nan() { 0.0 } else { lp as f64 * 100.0 }),
                    &format!("the fuel.tanks module: {n} tanks failed, each by a random draw against a fragility curve at the ground motion at its location. A failed tank releases a share of its contents set by its age, and the fuel goes toward water where the tank's row says it would"), "computed tank by tank");
                // deaths at the depot itself: a published range, drawn at random and scaled by the release against the largest the source considers
                let scale = if self.release_max_gal > 0.0 { (released / self.release_max_gal).clamp(0.0, 1.0) } else { 0.0 };
                let (ud, ui) = (ctx.rng().f64(), ctx.rng().f64());
                self.deaths = (self.toll_deaths.0 + (self.toll_deaths.1 - self.toll_deaths.0) * ud) * scale;
                self.injuries = (self.toll_injuries.0 + (self.toll_injuries.1 - self.toll_injuries.0) * ui) * scale;
                if released > 0.0 && self.toll_workers > 0.0 {
                    let (d, i, w, src) = (self.deaths, self.injuries, self.toll_workers, if self.toll_source.is_empty() { "a published range in the event folder, drawn at random and scaled by the size of the release".to_string() } else { self.toll_source.clone() });
                    self.say(ctx, "depot-toll", format!("At the {}, the count is about {:.0} killed and {:.0} injured among roughly {} workers, on the assumption that the spilled fuel ignites.", place, d, i, commas(w)), &src, "a published range, scaled by a guess");
                }
                ctx.publish("depot.toll", json!({ "deaths": self.deaths, "injuries": self.injuries }));
                self.report(ctx);
            }
            "quake.rupture" => {
                if self.fired { return; }
                self.fired = true; self.day = 0;
                ctx.schedule_with_priority(Duration::days(1), 5, "day", json!(null));
            }
            "routes.state" => { self.trucks = ev.data.get("any_reachable").and_then(|v| v.as_bool()).unwrap_or(false); }
            "fuel.request" => {
                let gal = f("gal");
                if gal > 0.0 && self.stock_gal >= gal {
                    ctx.transfer(FUEL, "hub", "burned", gal); self.stock_gal -= gal; self.used_gal += gal; self.asked_gal += gal;
                    ctx.publish("fuel.granted", json!({ "who": ev.data.get("who").cloned().unwrap_or(json!("")), "gal": gal }));
                }
            }
            "day" => {
                self.day += 1; let d = self.day; self.asked_gal = 0.0;
                if d >= self.resupply_day {
                    // the tanks are topped back up toward what they held before the day, never past it; with no
                    // storage at all, a week of deliveries is the working stock the trucks draw from
                    let cap = (self.contents_gal * (1.0 - self.storage_cut)).max(self.resupply_gal_day * 7.0);
                    let add = self.resupply_gal_day.min((cap - self.stock_gal).max(0.0));
                    if add > 0.0 { ctx.transfer(FUEL, "pipeline", "hub", add); self.stock_gal += add; self.delivered_gal += add; }
                    let rate = self.resupply_gal_day;
                    self.say(ctx, "resupply", format!("Day {}: fuel deliveries to the depot resume, at up to {:.1} million gallons a day.", d, rate / 1e6), "a guess at when the supply into the depot works again", "a guess");
                }
                if self.trucks {
                    let draw = self.emergency_gal_day.min(self.stock_gal);
                    if draw > 0.0 { ctx.transfer(FUEL, "hub", "burned", draw); self.stock_gal -= draw; self.used_gal += draw; }
                }
                if self.stock_gal <= 0.0 { self.say(ctx, "empty", format!("Day {}: the fuel left at the depot is used up.", d), "the module's fuel account", "arithmetic on guessed rates"); }
                self.report(ctx);
                ctx.publish("fuel.state", json!({ "day": d, "stock_gal": self.stock_gal, "trucks": self.trucks }));
                ctx.schedule_with_priority(Duration::days(1), 5, "day", json!(null));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { *self = serde_json::from_value(v).unwrap(); }
}
