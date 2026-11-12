//! `lifelines.recovery`: the months after the earthquake, as a clock. The shaking is an instant
//! and the state has mapped it; what nobody has drawn for a reader is the time after: when the
//! bridges, the power, the water and the fuel come back, and what the fuel hub does on the first
//! afternoon. This module holds the lifelines as components with a declared fate each, a
//! restoration time drawn by the run's seed from the Oregon Resilience Plan's published ranges,
//! and a fuel ledger in gallons that must close: what the hub's tanks held, what left them in
//! the first minutes (onto the ground and into the river, by the county's 2022 study), what the
//! hospitals' generators and the emergency fleet burn, and what comes back up the pipeline.
//! Every constant is either a published range or a guess, and the describe says which.

use crate::quake::{cell_of, phi, root, sample, window_grid};
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Describe, Duration, Event, LatLonGrid, Module};
use serde::{Deserialize, Serialize};
use serde_json::json;

const FUEL: &str = "fuel";

#[derive(Clone, Serialize, Deserialize)]
struct Bridge { name: String, lat: f64, lon: f64, opened: i64, cell: usize, reopen_day: i64, open: bool }
#[derive(Clone, Serialize, Deserialize)]
struct Hospital { name: String, lat: f64, lon: f64, emergency: bool, cell: usize, east: bool, damaged: bool, p_damage: f64, grid_day: i64, generator_h: f64, status: String }
#[derive(Clone, Serialize, Deserialize, Default)]
struct Hub { lat: f64, lon: f64, contents_gal: f64, release_min: f64, release_max: f64, river_share: f64, utilization: f64, terminals: i64 }
#[derive(Clone, Serialize, Deserialize)]
struct Service { name: String, min_days: i64, max_days: i64, text: String, days: i64 }

fn tf(t: &toml::Value, k: &str) -> f64 { t.get(k).and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64))).unwrap_or(0.0) }
fn ts(t: &toml::Value, k: &str) -> String { t.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string() }

#[derive(Serialize, Deserialize)]
pub struct Lifelines {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    folder: String,
    storage_cut: f64, retrofit: bool, retrofit_release: f64,
    bridge_ready: Vec<String>, bridge_min_days: f64, bridge_max_days: f64, new_bridge_year: i64,
    hospital_median_scale: f64, beta: f64,
    generator_h: f64, hospital_gal_day: f64, emergency_gal_day: f64, resupply_day: i64, resupply_gal_day: f64,
    route_min_days: f64, route_max_days: f64,
    bridges: Vec<Bridge>, hospitals: Vec<Hospital>, hub: Hub, services: Vec<Service>,
    hub_cell: usize,
    fired: bool, scheduled: bool, day: i64, t0: i64,
    route_day: i64,
    hub_gal: f64, river_gal: f64, ground_gal: f64, consumed_gal: f64, delivered_gal: f64,
    said: Vec<String>,
    residents_total: f64,
    #[serde(skip)] grid: Option<LatLonGrid>,
}

impl Lifelines {
    pub fn new(name: &str, p: &toml::Value) -> Result<Lifelines, String> {
        let ready: Vec<String> = p.get("bridge_ready").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        let mut o = Lifelines {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 45.43), lat_n: param_f64(p, "lat_n", 45.66), lon_w: param_f64(p, "lon_w", -122.85), lon_e: param_f64(p, "lon_e", -122.47),
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "portland-cascadia").to_string(),
            storage_cut: param_f64(p, "storage_cut", 0.0),
            retrofit: p.get("retrofit").and_then(|v| v.as_bool()).unwrap_or(false),
            retrofit_release: param_f64(p, "retrofit_release", 0.10),
            bridge_ready: ready, bridge_min_days: param_f64(p, "bridge_min_days", 21.0), bridge_max_days: param_f64(p, "bridge_max_days", 730.0), new_bridge_year: param_f64(p, "new_bridge_year", 2015.0) as i64,
            hospital_median_scale: param_f64(p, "hospital_median_scale", 1.5), beta: param_f64(p, "beta", 0.7),
            generator_h: param_f64(p, "generator_h", 72.0), hospital_gal_day: param_f64(p, "hospital_gal_day", 2000.0), emergency_gal_day: param_f64(p, "emergency_gal_day", 300000.0),
            resupply_day: param_f64(p, "resupply_day", 14.0) as i64, resupply_gal_day: param_f64(p, "resupply_gal_day", 1000000.0),
            route_min_days: param_f64(p, "route_min_days", 7.0), route_max_days: param_f64(p, "route_max_days", 30.0),
            bridges: Vec::new(), hospitals: Vec::new(), hub: Hub::default(), services: Vec::new(), hub_cell: 0,
            fired: false, scheduled: false, day: 0, t0: 0, route_day: 0,
            hub_gal: 0.0, river_gal: 0.0, ground_gal: 0.0, consumed_gal: 0.0, delivered_gal: 0.0,
            said: Vec::new(), residents_total: 0.0, grid: None,
        };
        o.load()?;
        Ok(o)
    }
    fn load(&mut self) -> Result<(), String> {
        let (g, _) = window_grid(self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let dir = root().join("data/events").join(&self.folder);
        let read = |f: &str| -> Result<toml::Value, String> { toml::from_str(&std::fs::read_to_string(dir.join(f)).map_err(|e| format!("{f}: {e}"))?).map_err(|e| format!("{f}: {e}")) };
        let fresh = self.bridges.is_empty();
        if fresh {
            let b = read("bridges.toml")?;
            self.bridges = b.get("bridge").and_then(|v| v.as_array()).ok_or("bridges.toml has no [[bridge]]")?.iter().map(|t| Bridge { name: ts(t, "name"), lat: tf(t, "lat"), lon: tf(t, "lon"), opened: tf(t, "opened") as i64, cell: cell_of(&g, tf(t, "lat"), tf(t, "lon")), reopen_day: 0, open: true }).collect();
            let h = read("hospitals.toml")?;
            self.hospitals = h.get("hospital").and_then(|v| v.as_array()).ok_or("hospitals.toml has no [[hospital]]")?.iter().map(|t| Hospital { name: ts(t, "name"), lat: tf(t, "lat"), lon: tf(t, "lon"), emergency: t.get("emergency").and_then(|v| v.as_bool()).unwrap_or(false), cell: cell_of(&g, tf(t, "lat"), tf(t, "lon")), east: tf(t, "lon") >= -122.675, damaged: false, p_damage: 0.0, grid_day: 0, generator_h: 0.0, status: "open".into() }).collect();
            let hb = read("hub.toml")?; let hb = hb.get("hub").ok_or("hub.toml has no [hub]")?;
            let contents = tf(hb, "contents_gal");
            self.hub = Hub { lat: tf(hb, "lat"), lon: tf(hb, "lon"), contents_gal: contents, release_min: tf(hb, "release_min_gal") / contents, release_max: tf(hb, "release_max_gal") / contents, river_share: tf(hb, "release_to_river_share"), utilization: tf(hb, "utilization_today"), terminals: tf(hb, "terminals") as i64 };
            let r = read("restoration.toml")?;
            self.services = r.get("service").and_then(|v| v.as_array()).ok_or("restoration.toml has no [[service]]")?.iter().filter(|t| ts(t, "zone") == "valley").map(|t| Service { name: ts(t, "name"), min_days: tf(t, "min_days") as i64, max_days: tf(t, "max_days") as i64, text: ts(t, "text"), days: 0 }).collect();
            let res = sample(&self.folder, "people_resident.tif", &g)?;
            self.residents_total = res.iter().filter(|v| !v.is_nan()).map(|&v| v as f64).sum();
        }
        self.hub_cell = cell_of(&g, self.hub.lat, self.hub.lon);
        self.grid = Some(g);
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let folder = param_str(p, "folder", "portland-cascadia");
        let mut d = Describe::new("lifelines.recovery", name, "The months after the earthquake as a clock: the Willamette crossings, the hospitals, the power and the water, and the fuel hub, each a component with a declared fate drawn by the run's seed from published ranges (the county on its bridges, the Oregon Resilience Plan on restoration times, the county's 2022 study on the hub's tanks) and a fuel ledger in gallons that must close. Nothing here is engineered: it is the state's own estimates arranged in time, with the guesses named.")
            .priority(2).dt("1 day after the rupture")
            .param("folder", folder, "", "the event folder with bridges.toml, hospitals.toml, hub.toml and restoration.toml")
            .param("storage_cut", param_f64(p, "storage_cut", 0.0), "", "fraction of the hub's contents removed before the day (0.2 is Ordinance 2026-312's cut by 2036, applied as if to the stock)")
            .param("retrofit", p.get("retrofit").and_then(|v| v.as_bool()).unwrap_or(false), "", "every tank behaves as one built after 2004: at most retrofit_release of its contents leaves (the study's post-2004 rule)")
            .param("retrofit_release", param_f64(p, "retrofit_release", 0.10), "", "the study's up-to-10-percent release for tanks designed for shaking and deformation")
            .param("bridge_ready", json!(p.get("bridge_ready").cloned().map(|v| serde_json::to_value(v).unwrap_or(json!([]))).unwrap_or(json!([]))), "", "crossings usable at once (none today; the Burnside replacement would be the first)")
            .param("bridge_min_days", param_f64(p, "bridge_min_days", 21.0), "d", "the county's 'weeks' (a guess at the number)")
            .param("bridge_max_days", param_f64(p, "bridge_max_days", 730.0), "d", "the county's 'years' (a guess at the number)")
            .param("new_bridge_year", param_f64(p, "new_bridge_year", 2015.0), "yr", "crossings opened in or after this year draw from the lower half of the range (Sellwood 2016, Tilikum 2015; the county expects even these to be unusable at first, so they are closed too; a guess)")
            .param("hospital_median_scale", param_f64(p, "hospital_median_scale", 1.5), "", "hospitals' fragility medians are the generic ones times this (newer code; a guess)")
            .param("generator_h", param_f64(p, "generator_h", 72.0), "h", "on-site generator fuel at each hospital (a common planning figure; a guess)")
            .param("hospital_gal_day", param_f64(p, "hospital_gal_day", 2000.0), "gal/d", "diesel a hospital on generators burns (a guess)")
            .param("emergency_gal_day", param_f64(p, "emergency_gal_day", 300000.0), "gal/d", "the response's draw on the hub once trucks can move: fire, police, utilities, shelters (a guess; the region's normal use is several million a day)")
            .param("resupply_day", param_f64(p, "resupply_day", 14.0), "d", "when fuel first arrives again (the Olympic pipeline from Puget Sound and the river terminals, both damaged; a guess)")
            .param("resupply_gal_day", param_f64(p, "resupply_gal_day", 1000000.0), "gal/d", "what arrives after that (a guess)")
            .param("route_min_days", param_f64(p, "route_min_days", 7.0), "d", "earliest day trucks move on the emergency routes (debris and ground failure on 74 percent of segments; a guess)")
            .param("route_max_days", param_f64(p, "route_max_days", 30.0), "d", "latest (a guess)")
            .subscribe("quake.rupture", "the instant")
            .read_on("quake.pga_g", "g", true, "quake.rupture", "at each hospital").read_on("quake.liq_p", "1", true, "quake.rupture", "at each hospital and the hub").read_on("quake.liq_pgd_cm", "cm", true, "quake.rupture", "")
            .publish("lifeline.marked", "{ kind: bridge | hospital | hub, name, lat, lon } at each component's cell at the start")
            .publish("lifeline.failed", "{ kind, name, reopen_day } at the cell, on the rupture").publish("lifeline.restored", "{ kind, name, day } at the cell, when it comes back")
            .publish("lifeline.crossing", "{ kind, day, sentence, source, confidence } the spill, the closures, each restoration milestone, the fuel")
            .ledger(FUEL, "gal", 1.0)
            .report("lifeline.crossings_open", "crossings").report("lifeline.hospitals_open", "hospitals").report("lifeline.hospitals_dark", "hospitals").report("lifeline.hospitals_closed", "hospitals")
            .report("service.power_share", "1").report("service.water_share", "1").report("service.people_with_power", "people").report("service.people_with_water", "people").report("service.routes_open", "1")
            .report("fuel.hub_gal", "gal").report("fuel.river_gal", "gal").report("fuel.ground_gal", "gal").report("fuel.consumed_gal", "gal").report("fuel.delivered_gal", "gal").report("fuel.days_left", "d")
            .asset(&format!("events/{folder}"), "the component tables and the residents raster")
            .view("story", "what happens", "crossing", "one sentence per event, newest first; the source beside each")
            .view("counter", "Willamette crossings open", "lifeline.crossings_open", "of the twelve in the window")
            .view("counter", "hospitals open", "lifeline.hospitals_open", "undamaged, with grid power or a running generator")
            .view("counter", "hospitals dark", "lifeline.hospitals_dark", "standing, no power")
            .view("counter", "people with power", "service.people_with_power", "residents times the restored share")
            .view("counter", "people with water", "service.people_with_water", "")
            .view("counter", "fuel at the hub", "fuel.hub_gal", "gallons still in the tanks")
            .view("counter", "fuel in the river", "fuel.river_gal", "gallons, by the county's 2022 study's rule")
            .view("counter", "days of fuel left", "fuel.days_left", "the hub's stock over the response's daily draw")
            .view("chart", "share of people with power", "service.power_share", "a ramp to the restoration day drawn from the Resilience Plan's valley range")
            .view("chart", "share of people with water", "service.water_share", "")
            .view("chart", "crossings open", "lifeline.crossings_open", "")
            .view("chart", "hospitals open", "lifeline.hospitals_open", "")
            .view("chart", "fuel at the hub", "fuel.hub_gal", "")
            .view("text", "what this does not measure", "Not in this run: the tsunami (not Portland's), aftershocks, fire following the earthquake, the Columbia bridges and I-5 north, the water system's pipes (the bureau's 1,500 to 3,000 breaks are in the record's README, not here), sewage, telephones and data, food, the airport, the port, the people who leave. The restoration ramps are straight lines to a day drawn from a published range; a real recovery is lumpy and unequal. The hospitals' fates come from the generic fragility at their cells and a guessed code factor. The fuel is a stock, not a network: no pipes, no trucks, no priorities. The spill is a number from the county's study, not a flow on the river.", "");
        let dir = root().join("data/events").join(folder);
        if let Ok(t) = std::fs::read_to_string(dir.join("bridges.toml")).map_err(|e| e.to_string()).and_then(|s| toml::from_str::<toml::Value>(&s).map_err(|e| e.to_string())) {
            if let Some(a) = t.get("bridge").and_then(|v| v.as_array()) { d = d.table("bridges", json!(a.iter().map(|b| json!({ "name": ts(b, "name"), "opened": tf(b, "opened"), "carries": ts(b, "carries"), "owner": ts(b, "owner") })).collect::<Vec<_>>())); }
        }
        if let Ok(t) = std::fs::read_to_string(dir.join("restoration.toml")).map_err(|e| e.to_string()).and_then(|s| toml::from_str::<toml::Value>(&s).map_err(|e| e.to_string())) {
            if let Some(a) = t.get("service").and_then(|v| v.as_array()) { d = d.table("restoration", json!(a.iter().map(|b| json!({ "service": ts(b, "name"), "zone": ts(b, "zone"), "estimate": ts(b, "text") })).collect::<Vec<_>>())); }
        }
        d
    }
    fn say(&mut self, ctx: &mut Ctx, id: &str, kind: &str, sentence: String, source: &str, confidence: &str) {
        if self.said.iter().any(|s| s == id) { return; }
        self.said.push(id.to_string());
        ctx.publish("lifeline.crossing", json!({ "id": id, "title": id, "name": kind, "kind": kind, "day": self.day, "time": ctx.now().to_iso(), "sentence": sentence, "source": source, "confidence": confidence }));
    }
    fn service_days(&self, name: &str) -> i64 { self.services.iter().find(|s| s.name == name).map(|s| s.days).unwrap_or(0) }
    fn demand_gal_day(&self) -> f64 { self.emergency_gal_day + self.hospitals.iter().filter(|h| h.status == "generator").count() as f64 * self.hospital_gal_day }
    fn report_all(&self, ctx: &mut Ctx) {
        // the stocks as held, for every close: the ledger compares them with what the flows imply
        ctx.declare_stock(FUEL, "hub", self.hub_gal);
        ctx.declare_stock(FUEL, "ground", self.ground_gal); ctx.declare_stock(FUEL, "river", self.river_gal);
        ctx.declare_stock(FUEL, "burned", self.consumed_gal); ctx.declare_stock(FUEL, "pipeline", 1.0e9 - self.delivered_gal);
        let open = self.bridges.iter().filter(|b| b.open).count();
        ctx.report("lifeline.crossings_open", "crossings", open as f64);
        ctx.report("lifeline.hospitals_open", "hospitals", self.hospitals.iter().filter(|h| h.status == "open" || h.status == "generator").count() as f64);
        ctx.report("lifeline.hospitals_dark", "hospitals", self.hospitals.iter().filter(|h| h.status == "dark").count() as f64);
        ctx.report("lifeline.hospitals_closed", "hospitals", self.hospitals.iter().filter(|h| h.status == "closed").count() as f64);
        let ramp = |t: i64| -> f64 { if !self.fired { 1.0 } else if t <= 0 { 1.0 } else { (self.day as f64 / t as f64).clamp(0.0, 1.0) } };
        let (pw, wa) = (ramp(self.service_days("electricity")), ramp(self.service_days("drinking water and sewer")));
        ctx.report("service.power_share", "1", pw); ctx.report("service.water_share", "1", wa);
        ctx.report("service.people_with_power", "people", pw * self.residents_total); ctx.report("service.people_with_water", "people", wa * self.residents_total);
        ctx.report("service.routes_open", "1", if !self.fired || self.day >= self.route_day { 1.0 } else { 0.0 });
        ctx.report("fuel.hub_gal", "gal", self.hub_gal); ctx.report("fuel.river_gal", "gal", self.river_gal); ctx.report("fuel.ground_gal", "gal", self.ground_gal);
        ctx.report("fuel.consumed_gal", "gal", self.consumed_gal); ctx.report("fuel.delivered_gal", "gal", self.delivered_gal);
        let dem = self.demand_gal_day();
        ctx.report("fuel.days_left", "d", if dem > 0.0 { self.hub_gal / dem } else { f64::NAN });
    }
}

impl Module for Lifelines {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.subscribe("quake.rupture");
        ctx.ledger(FUEL, "gal", 1.0);
        if !self.scheduled {
            self.hub_gal = self.hub.contents_gal * (1.0 - self.storage_cut);
            ctx.declare_stock(FUEL, "hub", self.hub_gal);
            ctx.declare_stock(FUEL, "ground", 0.0); ctx.declare_stock(FUEL, "river", 0.0); ctx.declare_stock(FUEL, "burned", 0.0);
            ctx.declare_stock(FUEL, "pipeline", 1.0e9);   // the world outside the window: the pipeline and the barges
            // the fates, drawn once by the seed
            for s in &mut self.services { let u = ctx.rng().f64(); s.days = s.min_days + ((s.max_days - s.min_days) as f64 * u).round() as i64; }
            let (lo, hi) = (self.bridge_min_days, self.bridge_max_days);
            for b in &mut self.bridges {
                let u = ctx.rng().f64();
                let (a, c) = if b.opened >= self.new_bridge_year { (lo, (lo + hi) / 2.0) } else { (lo, hi) };
                // log-uniform between the ends: "weeks to years"
                b.reopen_day = (a * (c / a).powf(u)).round() as i64;
                if self.bridge_ready.iter().any(|n| n == &b.name) { b.reopen_day = 0; }
            }
            let ru = ctx.rng().f64();
            self.route_day = (self.route_min_days + (self.route_max_days - self.route_min_days) * ru).round() as i64;
            let power = self.service_days("electricity");
            for h in &mut self.hospitals { let u = ctx.rng().f64(); h.grid_day = (power as f64 * u).round() as i64; h.generator_h = self.generator_h; }
            self.scheduled = true;
        }
        for b in &self.bridges { ctx.publish_at(b.cell, "lifeline.marked", json!({ "kind": "bridge", "name": b.name, "lat": b.lat, "lon": b.lon, "opened": b.opened })); }
        for h in &self.hospitals { ctx.publish_at(h.cell, "lifeline.marked", json!({ "kind": "hospital", "name": h.name, "lat": h.lat, "lon": h.lon })); }
        ctx.publish_at(self.hub_cell, "lifeline.marked", json!({ "kind": "hub", "name": "Critical Energy Infrastructure Hub", "lat": self.hub.lat, "lon": self.hub.lon, "contents_gal": self.hub_gal }));
        self.report_all(ctx);
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "quake.rupture" => {
                if self.fired { return; }
                self.fired = true; self.day = 0; self.t0 = ctx.now().0;
                let n = self.grid.as_ref().map(|g| g.nx * g.ny).unwrap_or(0);
                let take = |name: &str| -> Vec<f32> { ctx.world_read(name).map(|f| f.data.clone()).unwrap_or_else(|| vec![0.0; n]) };
                let (pga, liq_p, liq_pgd) = (take("quake.pga_g"), take("quake.liq_p"), take("quake.liq_pgd_cm"));
                // the hub: the study's release range, drawn by the seed; the retrofit rule caps it
                let u = ctx.rng().f64();
                let mut frac = self.hub.release_min + (self.hub.release_max - self.hub.release_min) * u;
                if self.retrofit { frac = frac.min(self.retrofit_release); }
                let released = self.hub_gal * frac;
                let river = released * self.hub.river_share; let ground = released - river;
                ctx.transfer(FUEL, "hub", "river", river); ctx.transfer(FUEL, "hub", "ground", ground);
                self.hub_gal -= released; self.river_gal += river; self.ground_gal += ground;
                let lp = liq_p.get(self.hub_cell).copied().unwrap_or(f32::NAN);
                ctx.publish_at(self.hub_cell, "lifeline.failed", json!({ "kind": "hub", "name": "Critical Energy Infrastructure Hub", "released_gal": released, "river_gal": river }));
                self.say(ctx, "spill", "hub", format!("Within minutes, about {:.0} million gallons leave the hub's tanks ({:.0} percent of the {:.0} million stored): {:.0} million onto the ground, {:.0} million toward the Willamette. DOGAMI maps a {:.0} percent chance of lateral spread under the hub.", released / 1e6, frac * 100.0, (self.hub_gal + released) / 1e6, ground / 1e6, river / 1e6, if lp.is_nan() { 0.0 } else { lp as f64 * 100.0 }), "CEI Hub Seismic Risk Analysis (ECONorthwest, Salus Resilience, Enduring Econometrics, 2022): 94.6 to 193.7 million gallons released of 233.5 million held (the report's 67 percent fill of 332.7 million of active capacity), 43 percent to the river; the draw within the range is the seed's", "a published range");
                // the crossings
                for b in &mut self.bridges { b.open = b.reopen_day == 0; }
                for b in self.bridges.clone() { if !b.open { ctx.publish_at(b.cell, "lifeline.failed", json!({ "kind": "bridge", "name": b.name, "reopen_day": b.reopen_day })); } }
                let closed = self.bridges.iter().filter(|b| !b.open).count();
                let soonest = self.bridges.iter().filter(|b| !b.open).map(|b| b.reopen_day).min().unwrap_or(0);
                let latest = self.bridges.iter().map(|b| b.reopen_day).max().unwrap_or(0);
                self.say(ctx, "crossings-closed", "bridges", format!("{} of the {} Willamette crossings in the window are closed; the first reopens on day {}, the last on day {}.", closed, self.bridges.len(), soonest, latest), "Multnomah County: no downtown bridge is expected to be usable at once; weeks to years before any is; the days are drawn log-uniformly in that range by the seed", "a published statement, the numbers guessed");
                // the hospitals: damaged by the generic fragility with a code factor, at their cells
                let mut closed_h = Vec::new();
                for h in &mut self.hospitals {
                    let a = pga.get(h.cell).copied().unwrap_or(f32::NAN) as f64;
                    let p_shake = if a.is_nan() || a <= 0.0 { 0.0 } else { phi((a / (0.80 * self.hospital_median_scale)).ln() / self.beta) };
                    let (lp, pg) = (liq_p.get(h.cell).copied().unwrap_or(0.0) as f64, liq_pgd.get(h.cell).copied().unwrap_or(0.0) as f64);
                    let p_gf = if lp > 0.0 && pg > 0.0 { lp * phi((pg / 152.0).ln() / 1.2) } else { 0.0 };
                    h.p_damage = 1.0 - (1.0 - p_shake) * (1.0 - p_gf);
                    let u = ctx.rng().f64();
                    h.damaged = u < h.p_damage;
                    h.status = if h.damaged { "closed".into() } else { "generator".into() };
                    if h.damaged { closed_h.push(h.name.clone()); }
                }
                for h in self.hospitals.clone() { if h.damaged { ctx.publish_at(h.cell, "lifeline.failed", json!({ "kind": "hospital", "name": h.name, "p_damage": h.p_damage })); } }
                let n_open = self.hospitals.len() - closed_h.len();
                self.say(ctx, "hospitals", "hospitals", format!("{} of {} hospitals stand and run on generators; {} closed by damage{}{}. The grid is down everywhere.", n_open, self.hospitals.len(), closed_h.len(), if closed_h.is_empty() { "" } else { ": " }, closed_h.join(", ")), "the generic fragility at each hospital's cell with a guessed code factor, drawn by the seed; generator fuel for 72 hours (a guess)", "guess");
                self.report_all(ctx);
                ctx.schedule_with_priority(Duration::days(1), 5, "day", json!(null));
            }
            "day" => {
                self.day += 1;
                let d = self.day;
                // crossings
                for b in self.bridges.clone().iter().enumerate().map(|(i, b)| (i, b.clone())).collect::<Vec<_>>() {
                    let (i, b) = b;
                    if !b.open && d >= b.reopen_day { self.bridges[i].open = true; ctx.publish_at(b.cell, "lifeline.restored", json!({ "kind": "bridge", "name": b.name, "day": d })); let s = format!("Day {}: the {} reopens, the first crossing back.", d, b.name); self.say(ctx, "first-crossing", "bridges", s, "the seed's draw within the county's weeks-to-years", "guess"); }
                }
                if self.bridges.iter().all(|b| b.open) { self.say(ctx, "all-crossings", "bridges", format!("Day {}: every crossing in the window is open again.", d), "the seed's draws", "guess"); }
                let routes = d >= self.route_day;
                if routes { self.say(ctx, "routes", "routes", format!("Day {}: trucks move on the emergency routes.", d), "a guess within 7 to 30 days; DOGAMI: 74 percent of segments at risk of ground failure", "guess"); }
                // fuel: resupply, then the response's draw, then the hospitals' generators
                if d >= self.resupply_day {
                    // the tanks are topped back up toward what they held before the day, never past it
                    let cap = self.hub.contents_gal * (1.0 - self.storage_cut);
                    let add = self.resupply_gal_day.min((cap - self.hub_gal).max(0.0));
                    if add > 0.0 { ctx.transfer(FUEL, "pipeline", "hub", add); self.hub_gal += add; self.delivered_gal += add; }
                    self.say(ctx, "resupply", "hub", format!("Day {}: fuel arrives again, up to {:.1} million gallons a day.", d, self.resupply_gal_day / 1e6), "a guess at when the pipeline and the river terminals work", "guess");
                }
                if routes {
                    let draw = self.emergency_gal_day.min(self.hub_gal);
                    if draw > 0.0 { ctx.transfer(FUEL, "hub", "burned", draw); self.hub_gal -= draw; self.consumed_gal += draw; }
                }
                let any_bridge = self.bridges.iter().any(|b| b.open);
                let power_day = self.service_days("electricity");
                let mut dark_now = Vec::new();
                for i in 0..self.hospitals.len() {
                    let h = self.hospitals[i].clone();
                    if h.damaged { continue; }
                    let grid = d >= h.grid_day.max(1);
                    if grid { self.hospitals[i].status = "open".into(); continue; }
                    let mut left = h.generator_h - 24.0;
                    let reachable = routes && (!h.east || any_bridge);
                    if reachable && left < self.generator_h && self.hub_gal >= self.hospital_gal_day {
                        ctx.transfer(FUEL, "hub", "burned", self.hospital_gal_day); self.hub_gal -= self.hospital_gal_day; self.consumed_gal += self.hospital_gal_day;
                        left = self.generator_h;
                    }
                    self.hospitals[i].generator_h = left.max(0.0);
                    if left <= 0.0 { if self.hospitals[i].status != "dark" { dark_now.push(h.name.clone()); } self.hospitals[i].status = "dark".into(); } else { self.hospitals[i].status = "generator".into(); }
                }
                if !dark_now.is_empty() { self.say(ctx, "hospitals-dark", "hospitals", format!("Day {}: generators run dry with no fuel delivered: {} go dark.", d, dark_now.join(", ")), "72 hours of generator fuel (a guess), deliveries only once trucks move and, east of the river, a crossing is open", "guess"); }
                if d == power_day / 2 { self.say(ctx, "power-half", "power", format!("Day {}: power is back for half the people.", d), "the Oregon Resilience Plan's 1 to 3 months for the valley, the day drawn by the seed, a straight ramp", "a published range"); }
                if d == power_day { self.say(ctx, "power", "power", format!("Day {}: power is back for everyone.", d), "the Oregon Resilience Plan's 1 to 3 months for the valley", "a published range"); }
                let water_day = self.service_days("drinking water and sewer");
                if d == water_day / 2 { self.say(ctx, "water-half", "water", format!("Day {}: water is back for half the people.", d), "the Oregon Resilience Plan's 1 month to 1 year for the valley, the day drawn by the seed", "a published range"); }
                if d == water_day { self.say(ctx, "water", "water", format!("Day {}: water and sewer are back for everyone.", d), "the Oregon Resilience Plan", "a published range"); }
                if self.hub_gal <= 0.0 { self.say(ctx, "hub-empty", "hub", format!("Day {}: the hub's remaining stock is gone.", d), "the ledger", "arithmetic on guesses"); }
                self.report_all(ctx);
                ctx.schedule_with_priority(Duration::days(1), 5, "day", json!(null));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Lifelines = serde_json::from_value(v).unwrap(); o.load().expect("lifelines reload"); *self = o; }
}
