//! `people.aftermath`: what happens to people after the shaking stops, as claims with sources.
//! The buildings module counts the dead and the injured at the instant; this module keeps the
//! days after: the life-threatening injured who find no open hospital, the deaths that follow the
//! loss of power at a rate taken from a disaster with a record, the people on electricity-dependent
//! medical devices without grid power, the person-days without power and water, and the people
//! whose outage passed the Resilience Plan's one-month rule. Every channel is a row in
//! `data/events/<folder>/aftermath.toml` with its rate, source, confidence and a switch, listed on
//! the run page whether on or off. It reads the lifelines module's daily state and the buildings'
//! toll as events and owns one total: `deaths.total`.
use crate::quake::{root, sample, window_grid};
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Describe, Event, Module};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize)]
struct Claim { id: String, title: String, on: String, rate: f64, low: f64, high: f64, per: String, source: String, confidence: String, status: String }

#[derive(Serialize, Deserialize)]
pub struct Aftermath {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    folder: String,
    month_days: i64,
    claims: Vec<Claim>,
    power_dependent_share: f64,
    residents_total: f64,
    quake_deaths: f64, hub_deaths: f64, injuries_life: f64, injuries_hospital: f64,
    #[serde(default)] agents_dead: Option<f64>,
    untreated: f64,
    deaths: Vec<(String, f64)>, deaths_low: f64, deaths_high: f64,
    person_days_power: f64, person_days_water: f64, devices_days: f64,
    over_month_power: f64, over_month_water: f64,
    day: i64, said: Vec<String>,
}

fn tf(t: &toml::Value, k: &str) -> f64 { t.get(k).and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64))).unwrap_or(0.0) }
fn ts(t: &toml::Value, k: &str) -> String { t.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string() }

impl Aftermath {
    pub fn new(name: &str, p: &toml::Value) -> Result<Aftermath, String> {
        let mut o = Aftermath {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 45.43), lat_n: param_f64(p, "lat_n", 45.66), lon_w: param_f64(p, "lon_w", -122.85), lon_e: param_f64(p, "lon_e", -122.47),
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "portland-cascadia").to_string(),
            month_days: param_f64(p, "month_days", 30.0) as i64,
            claims: Vec::new(), power_dependent_share: 0.0, residents_total: 0.0,
            quake_deaths: 0.0, hub_deaths: 0.0, injuries_life: 0.0, injuries_hospital: 0.0, agents_dead: None, untreated: 0.0,
            deaths: Vec::new(), deaths_low: 0.0, deaths_high: 0.0,
            person_days_power: 0.0, person_days_water: 0.0, devices_days: 0.0, over_month_power: 0.0, over_month_water: 0.0,
            day: 0, said: Vec::new(),
        };
        o.load()?;
        Ok(o)
    }
    fn table(folder: &str) -> Result<toml::Value, String> {
        let path = root().join("data/events").join(folder).join("aftermath.toml");
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        text.parse::<toml::Value>().map_err(|e| format!("aftermath.toml: {e}"))
    }
    fn claims_of(t: &toml::Value) -> Vec<Claim> {
        t.get("claim").and_then(|v| v.as_array()).map(|a| a.iter().map(|c| Claim { id: ts(c, "id"), title: ts(c, "title"), on: ts(c, "on"), rate: tf(c, "rate"), low: tf(c, "low"), high: tf(c, "high"), per: ts(c, "per"), source: ts(c, "source"), confidence: ts(c, "confidence"), status: ts(c, "status") }).collect()).unwrap_or_default()
    }
    fn load(&mut self) -> Result<(), String> {
        let t = Self::table(&self.folder)?;
        self.claims = Self::claims_of(&t);
        if let Some(i) = t.get("inputs") {
            let dme = tf(i, "power_dependent_dme_multnomah") + tf(i, "power_dependent_dme_washington") + tf(i, "power_dependent_dme_clackamas");
            let pop = tf(i, "population_multnomah") + tf(i, "population_washington") + tf(i, "population_clackamas");
            self.power_dependent_share = if pop > 0.0 { dme / pop } else { 0.0 };
        }
        let (g, _) = window_grid(self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let res = sample(&self.folder, "people_resident.tif", &g)?;
        self.residents_total = res.iter().filter(|v| !v.is_nan()).map(|&v| v as f64).sum();
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let folder = param_str(p, "folder", "portland-cascadia");
        let claims = Self::table(folder).map(|t| Self::claims_of(&t)).unwrap_or_default();
        let mut d = Describe::new("people.aftermath", name, "This module counts harm to people after the shaking stops. It does not model how that harm comes about. It applies a short table of rates, each with a source, to counts the run already has. Two rates add deaths. One applies to people with life-threatening injuries on any day when no hospital is open. The other applies to each day a person spends without electricity; it is derived from the record of Puerto Rico after Hurricane Maria in 2017 and stands for interrupted medical care. The module also counts the people who depend on electrically powered medical devices and have no grid electricity (from the emPOWER data of the US Department of Health and Human Services), the person-days spent without electricity and without water, and the people still without a service after one month, which the Oregon Resilience Plan gives as the point past which a business tends to leave. Each rate is a row in the event folder's aftermath.toml, with its source, how far it can be trusted and whether it is switched on. The claims table lists every row, on or off. The module also keeps the run's single death total, deaths.total, which adds deaths in buildings, deaths at the fuel hub and the deaths counted here.")
            .priority(3).dt("1 day, on the lifelines' state")
            .param("folder", folder, "", "the folder of input data for this event, with the table of rates and the map of residents")
            .param("month_days", param_f64(p, "month_days", 30.0), "d", "the Oregon Resilience Plan's one-month rule: a business without services for this long tends to leave. The module counts the people whose electricity or water is still out on this day")
            .subscribe("quake.toll", "the buildings module's dead and injured, once the shaking ends").subscribe("agents.toll", "the agents' dead, which the total uses when a run has an agents module").subscribe("lifeline.state", "the lifelines module's daily state: hospitals open, the shares of people with electricity and water, deaths at the fuel hub")
            .publish("after.crossing", "{ kind, day, sentence, source, confidence } the sentences shown on the run page: the first deaths after the shaking, the one-month count, the count after a year")
            .report("deaths.total", "people").report("after.deaths", "people").report("after.deaths_low", "people").report("after.deaths_high", "people")
            .report("after.injuries_untreated", "people").report("after.person_days_without_power", "person-days").report("after.person_days_without_water", "person-days")
            .report("after.devices_without_power", "people").report("after.people_over_month_without_power", "people").report("after.people_over_month_without_water", "people")
            .asset(&format!("events/{folder}"), "the table of rates and the map of residents")
            .view("counter", "dead after the shaking", "after.deaths", "from the rates that are switched on; the charts show the low and high ends of the range")
            .view("counter", "person-days without power", "after.person_days_without_power", "one person without electricity for one day is one person-day")
            .view("counter", "on medical devices, without power", "after.devices_without_power", "the emPOWER share of the three counties' population, applied to the residents of the modelled area who have no electricity")
            .view("counter", "past a month without water", "after.people_over_month_without_water", "people still without water on the day the one-month rule is checked")
            .view("chart", "dead after the shaking", "after.deaths", "from the rates that are switched on")
            .view("chart", "dead after the shaking, high end of the range", "after.deaths_high", "the uncertainty intervals of the sources, applied as multipliers")
            .view("chart", "person-days without water", "after.person_days_without_water", "")
            .view("text", "what this module leaves out", "Two possible causes of death, cold homes and waterborne illness, are listed in the claims table but switched off, because no rate with a source was found for either. The Puerto Rico rate comes from a different kind of disaster in a different climate and season, and it covers a whole population. It is taken to include every death that follows the loss of electricity, including the deaths of people on medical devices. For that reason the device count is reported as a number of people exposed, and no second death rate is applied to it. The untreated-injury rule treats every injured person on the first day any hospital is open; the model does not represent hospital beds, staff or the distance to a hospital. The module does not move people: no one leaves the area, goes to a shelter or is counted as cold.", "")
            .note("Each row of the claims table is a published rate or a declared guess, applied to a count the run already has. The module does not model how the harm comes about, so a reader who doubts a figure should look at the source of its row.");
        for c in &claims { d = d.report(&format!("after.deaths_{}", c.id.replace('-', "_")), "people"); }
        d = d.table("claims", json!(claims.iter().map(|c| json!({ "id": c.id, "title": c.title, "on": c.on, "rate": c.rate, "low": c.low, "high": c.high, "per": c.per, "source": c.source, "confidence": c.confidence, "status": c.status })).collect::<Vec<_>>()));
        d
    }
    fn say(&mut self, ctx: &mut Ctx, id: &str, sentence: String, source: &str, confidence: &str) {
        if self.said.iter().any(|s| s == id) { return; }
        self.said.push(id.to_string());
        ctx.publish("after.crossing", json!({ "id": id, "title": id, "name": "after", "kind": "after", "day": self.day, "time": ctx.now().to_iso(), "sentence": sentence, "source": source, "confidence": confidence }));
    }
    fn after_total(&self) -> f64 { self.deaths.iter().map(|d| d.1).sum() }
    fn report_all(&self, ctx: &mut Ctx) {
        let after = self.after_total();
        ctx.report("deaths.total", "people", self.agents_dead.unwrap_or(self.quake_deaths) + self.hub_deaths + after);
        ctx.report("after.deaths", "people", after);
        ctx.report("after.deaths_low", "people", self.deaths_low); ctx.report("after.deaths_high", "people", self.deaths_high);
        for (id, v) in &self.deaths { ctx.report(&format!("after.deaths_{}", id.replace('-', "_")), "people", *v); }
        ctx.report("after.injuries_untreated", "people", self.untreated);
        ctx.report("after.person_days_without_power", "person-days", self.person_days_power);
        ctx.report("after.person_days_without_water", "person-days", self.person_days_water);
        ctx.report("after.people_over_month_without_power", "people", self.over_month_power);
        ctx.report("after.people_over_month_without_water", "people", self.over_month_water);
    }
}

impl Module for Aftermath {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.subscribe("quake.toll"); ctx.subscribe("agents.toll"); ctx.subscribe("lifeline.state");
        for c in &self.claims { self.deaths.push((c.id.clone(), 0.0)); }
        self.deaths.dedup_by(|a, b| a.0 == b.0);
        ctx.report("after.devices_without_power", "people", 0.0);
        self.report_all(ctx);
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        let f = |k: &str| ev.data.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
        match ev.topic.as_str() {
            "quake.toll" => {
                self.quake_deaths = f("deaths"); self.injuries_hospital = f("injuries_hospital"); self.injuries_life = f("injuries_life");
                self.untreated = self.injuries_life;
                self.report_all(ctx);
            }
            "agents.toll" => { self.agents_dead = Some(f("dead")); self.report_all(ctx); }
            "lifeline.state" => {
                self.day = f("day") as i64; self.hub_deaths = f("hub_deaths");
                let (hosp_open, power, water) = (f("hospitals_open"), f("power_share"), f("water_share"));
                let without_power = self.residents_total * (1.0 - power).max(0.0);
                let without_water = self.residents_total * (1.0 - water).max(0.0);
                self.person_days_power += without_power; self.person_days_water += without_water;
                let devices = without_power * self.power_dependent_share;
                self.devices_days += devices;
                ctx.report("after.devices_without_power", "people", devices);
                if self.day == self.month_days { self.over_month_power = without_power; self.over_month_water = without_water; }
                // the injured: treated the first day a hospital is open, else a share die
                let (mut lo, mut hi) = (0.0, 0.0);
                for c in self.claims.clone() {
                    if c.status != "on" { continue; }
                    let x = match c.on.as_str() {
                        "person_days_without_power" => without_power * c.rate,
                        "person_days_without_water" => without_water * c.rate,
                        "untreated_life_threatening" => { if hosp_open > 0.0 { self.untreated = 0.0; 0.0 } else { let d = self.untreated * c.rate.min(1.0); self.untreated -= d; d } }
                        "power_dependent_without_power" => devices * c.rate,
                        _ => 0.0,
                    };
                    if let Some(e) = self.deaths.iter_mut().find(|e| e.0 == c.id) { e.1 += x; }
                    lo += x * c.low; hi += x * c.high;
                }
                self.deaths_low += lo; self.deaths_high += hi;
                let after = self.after_total();
                if after >= 1.0 { let (a, l, h) = (after, self.deaths_low, self.deaths_high); self.say(ctx, "first-after", format!("Day {}: deaths after the shaking reach about {:.0}, with a range of {:.0} to {:.0}. The model adds them for medical care interrupted by the loss of electricity, and for life-threatening injuries on days when no hospital is open.", self.day, a, l, h), "Kishore and others 2018, on Puerto Rico after Hurricane Maria, for the death rate per person per day without electricity. The rate for untreated injuries is a guess. Both are rows of the claims table", "an analogy and a guess"); }
                if self.day == self.month_days { let (pw, ww) = (self.over_month_power, self.over_month_water); self.say(ctx, "one-month", format!("Day {}: a month after the earthquake, about {} people are still without electricity and {} without water. The Oregon Resilience Plan gives one month without services as the point past which a business tends to leave.", self.day, crate::lifelines::commas(pw), crate::lifelines::commas(ww)), "the Oregon Resilience Plan (2013) for the one-month rule; this run's electricity and water service for the counts", "a published rule applied to this run's restoration schedules"); }
                if self.day == 365 { let (a, l, h) = (after, self.deaths_low, self.deaths_high); self.say(ctx, "year", format!("One year after the earthquake: about {:.0} people died in the shaking and at the fuel hub, and about {:.0} more in the year that followed (range {:.0} to {:.0}). Residents spent {:.1} million person-days without electricity and {:.0} million without water.", self.agents_dead.unwrap_or(self.quake_deaths) + self.hub_deaths, a, l, h, self.person_days_power / 1e6, self.person_days_water / 1e6), "the rates switched on in the claims table: one from an analogous disaster and one guess", "a sum of an analogy and guesses"); }
                self.report_all(ctx);
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { *self = serde_json::from_value(v).unwrap(); }
}
