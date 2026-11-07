//! Derailment: the capacity to cut emissions, eroded by what the warming does. Anselm,
//! 2026-11-07: "I feel we are still failing to really consider knock on effects", with James
//! Dyke's piece in The Conversation of 2026-11-05 ("the worse climate change gets, the harder
//! it becomes to stop it"), whose argument is that the consequences of climate change erode
//! the capacity to address its causes: a sovereign debt loop, conflict, political backlash,
//! and the asymmetry that collective action is easier to destroy than to build. Until this
//! module nothing in a run could move the emissions after 2024; the assumption was the
//! variant's and held whatever the heat did to people. This module closes that loop with
//! declared placeholders: a world capacity between 0 and 1, read from the run's own pressures
//! (the share of people under heat stress, the voice and fairness bands weighted by people,
//! the wild cards drawn, conflicts it draws itself where the stress is), falling fast and
//! recovering slowly, published to the emissions module as `emissions.capacity`, which slows
//! every planned cut by it. The numbers are guesses and say so; the shape is the article's.
//! Beside the capacity it keeps the options: the budget left to a warming level and the cut
//! now needed to hold it, so a page can show what must be done rising while what can be done
//! falls. `mode = "watch"` computes and reports all of that and changes nothing.
use esim_kernel::describe::Describe;
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, Module, Time};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Clone, Serialize, Deserialize, Default)]
struct CountryState { name: String, people: f64, stress: f64, voice: Option<f64>, fairness: Option<f64> }

#[derive(Serialize, Deserialize)]
pub struct Capacity {
    name: String,
    mode: String,
    gain: f64, inequality: f64, fall: f64, recover: f64, floor: f64,
    knock_economy: f64, knock_war: f64, knock_disease: f64, knock_eruption: f64,
    conflict_gain: f64, conflict_mortality: f64, conflict_knock: f64, conflict_years: i64,
    level_c: f64, level2_c: f64, tcre_c_per_gtc: f64,
    from: String,
    /// the world capacity, 0 to 1
    capacity: f64,
    /// knocks from wild cards and conflicts: (multiplier, last year inclusive, what)
    knocks: Vec<(f64, i64, String)>,
    /// the latest pressures from the regions module: people-weighted shares and bands
    stress: f64, voice: f64, fairness: f64, people: f64,
    /// the latest per-country table, for the conflict draw and the capacity table
    countries: BTreeMap<String, CountryState>,
    conflicts: i64,
    gmst: f64, emitted_this_year: f64, emitted_last_year: f64, last_year: i64,
}

fn year_of(t: Time) -> i64 { t.to_iso()[0..4].parse().unwrap() }
fn to_next_year(now: Time) -> Duration { Time::parse(&format!("{:04}-01-01", year_of(now) + 1)).unwrap().since(now) }
fn year_of_str(s: &str) -> Option<i64> { s.get(0..4).and_then(|y| y.parse().ok()) }

impl Capacity {
    pub fn new(name: &str, p: &toml::Value) -> Result<Capacity, String> {
        let mode = param_str(p, "mode", "watch").to_string();
        if !["off", "watch", "on"].contains(&mode.as_str()) { return Err(format!("{name}: mode must be off, watch or on")); }
        Ok(Capacity {
            name: name.to_string(), mode,
            gain: param_f64(p, "gain", 4.0), inequality: param_f64(p, "inequality", 1.0), fall: param_f64(p, "fall", 0.5), recover: param_f64(p, "recover", 0.1), floor: param_f64(p, "floor", 0.0),
            knock_economy: param_f64(p, "knock_economy", 0.15), knock_war: param_f64(p, "knock_war", 0.2), knock_disease: param_f64(p, "knock_disease", 0.1), knock_eruption: param_f64(p, "knock_eruption", 0.05),
            conflict_gain: param_f64(p, "conflict_gain", 0.02), conflict_mortality: param_f64(p, "conflict_mortality", 0.01), conflict_knock: param_f64(p, "conflict_knock", 0.02), conflict_years: param_f64(p, "conflict_years", 5.0) as i64,
            level_c: param_f64(p, "level_c", 2.0), level2_c: param_f64(p, "level2_c", 3.0), tcre_c_per_gtc: param_f64(p, "tcre_c_per_gtc", 0.00165),
            from: param_str(p, "from", "2025-01-01").to_string(),
            capacity: 1.0, knocks: Vec::new(), stress: 0.0, voice: 0.5, fairness: 0.5, people: 0.0, countries: BTreeMap::new(), conflicts: 0,
            gmst: 0.0, emitted_this_year: 0.0, emitted_last_year: 0.0, last_year: i64::MIN,
        })
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let mode = param_str(p, "mode", "watch").to_string();
        let (level, level2) = (param_f64(p, "level_c", 2.0), param_f64(p, "level2_c", 3.0));
        let mut d = Describe::new("policy.capacity", name, "Derailment: the world's capacity to cut emissions, 0 to 1, eroded by what the run's own warming does to people and recovering slowly, after Dyke (The Conversation, 2026-11-05): the consequences of climate change erode the capacity to address its causes, through debt, conflict and backlash, and collective action is easier to destroy than to build. Each January the capacity moves toward a target set by the share of people under heat stress (from the regions module), amplified where the people-weighted fairness band is low; it falls fast and recovers slowly, the recovery scaled by the voice band; a wild card knocks it by its kind for some years; and where a country's people are under stress a conflict may be drawn, which takes a fraction of that country's people and knocks the capacity by the country's share of the world. With mode = on the capacity goes to the emissions module, which slows every planned cut by it: a plan to zero by 2050 becomes zero by later or never. Beside it, the options: the carbon budget left to a stated warming level under a stated TCRE and the cut rate now needed to hold it, from the run's own series. Every number here is a placeholder and says so; the shape is the article's and the inputs are the run's.")
            .priority(9).dt("1 year (1 January), after the regions module")
            .param("mode", mode.clone(), "", "off: nothing happens; watch: the capacity and the options are computed and reported, nothing is changed; on: the capacity is published to the emissions module and conflicts are drawn")
            .param("gain", param_f64(p, "gain", 4.0), "", "the target capacity is one minus gain times the share of the world's people under heat stress (a guess: at today's 2% that is 0.92, at a fifth of everyone 0.2)")
            .param("inequality", param_f64(p, "inequality", 1.0), "", "the pressure is multiplied by one plus this times one minus the people-weighted fairness band (the article: conflict and backlash are stronger where inequality is high; a guess)")
            .param("fall", param_f64(p, "fall", 0.5), "1/yr", "the share of the gap to a lower target closed each year (fast)")
            .param("recover", param_f64(p, "recover", 0.1), "1/yr", "the share of the gap to a higher target closed each year, times the people-weighted voice band (slow; the article's asymmetry)")
            .param("floor", param_f64(p, "floor", 0.0), "", "the capacity never falls below this")
            .param("knock_economy", param_f64(p, "knock_economy", 0.15), "", "a financial or technology wild card multiplies the capacity by one minus this for its emissions_years (a guess)")
            .param("knock_war", param_f64(p, "knock_war", 0.2), "", "a war wild card, for five years (a guess)")
            .param("knock_disease", param_f64(p, "knock_disease", 0.1), "", "a pandemic wild card, for three years (a guess)")
            .param("knock_eruption", param_f64(p, "knock_eruption", 0.05), "", "a cooling eruption or impact, for its forcing_years (a guess)")
            .param("conflict_gain", param_f64(p, "conflict_gain", 0.02), "1/yr", "a country's annual chance of conflict is this times the share of its people under heat stress, times one plus inequality times one minus its fairness band, times one minus half its voice band (a guess; mode = on only)")
            .param("conflict_mortality", param_f64(p, "conflict_mortality", 0.01), "", "the fraction of that country's people a conflict takes, once (a guess)")
            .param("conflict_knock", param_f64(p, "conflict_knock", 0.02), "", "the world capacity is multiplied by one minus this for every percent of the world's people in the country, capped at 0.2, for conflict_years (a guess)")
            .param("conflict_years", param_f64(p, "conflict_years", 5.0), "yr", "")
            .param("level_c", param_f64(p, "level_c", 2.0), "degC", "the warming level the options are measured against")
            .param("level2_c", param_f64(p, "level2_c", 3.0), "degC", "a second, higher level, for when the first is spent")
            .param("tcre_c_per_gtc", param_f64(p, "tcre_c_per_gtc", 0.00165), "degC/GtC", "transient climate response to cumulative emissions: AR6's best estimate of 0.45 °C per 1000 GtCO2, which is 1.65e-3 °C per GtC (recalled, not checked against the report)")
            .param("from", param_str(p, "from", "2025-01-01"), "", "no change before this date, so the hindcast is untouched")
            .note("The budget left is (level_c minus the run's warming) over the TCRE, in GtC, and zero once the level is passed; the years left are that over last year's emissions; the cut now needed is the constant percentage fall from today that spends exactly the budget on a straight line to zero, which is half the current rate over the budget, capped at 100 when nothing is left. The capacity applies to the planned fall only: a held or growing path is not helped by a weak capacity, and a lost year of cutting is not recovered when the capacity returns, which is the article's point. Conflicts here are a draw from the run's seed where the heat stress is, not a forecast of any war.");
        if mode != "off" {
            d = d.subscribe("region.pressure", "{ year, people, people_stress, voice, fairness, means } each January from the regions module")
                .subscribe("region.scored", "the per-country table, for the conflict draw and the capacity table")
                .subscribe("shock.crossing", "{ kind, forcing_w_m2, forcing_years, emissions_years } wild cards knock the capacity by kind")
                .subscribe("energy.state", "{ gmst_c } for the budget").subscribe("carbon.emitted", "{ gtc } summed over the year for the cut now needed")
                .report("capacity.world", "1").report("capacity.target", "1").report("capacity.pressure", "1")
                .report("options.budget_gtc", "GtC").report("options.years_left", "yr").report("options.cut_needed_pct_yr", "%/yr")
                .report("options.budget2_gtc", "GtC").report("options.years_left2", "yr").report("options.cut_needed2_pct_yr", "%/yr")
                .view("chart", "capacity to cut emissions", "capacity.world", "one is the plan as written; what the heat, the wild cards and the conflicts leave of it")
                .view("chart", &format!("the cut now needed to hold {} °C", level), "options.cut_needed_pct_yr", "percent a year from this year on, on a straight line to zero that spends the budget exactly; 100 when nothing is left")
                .view("chart", &format!("the cut now needed to hold {} °C", level2), "options.cut_needed2_pct_yr", "the same, for the higher level")
                .view("chart", &format!("carbon budget left to {} °C", level), "options.budget_gtc", "under the stated TCRE, from the run's own warming")
                .view("chart", &format!("carbon budget left to {} °C", level2), "options.budget2_gtc", "");
        }
        if mode == "on" {
            d = d.publish("emissions.capacity", "{ name: policy, capacity } each January from `from`, to the emissions module")
                .publish("people.shock", "{ id, title, year, fraction, iso3, name } a conflict drawn in a country: a fraction of that country's people")
                .publish("capacity.crossing", "{ id, title, name, iso3, year, stress, chance, fraction, sentence } for every conflict drawn")
                .publish("capacity.scored", "{ year, by_iso3: { ISO3: capacity } } whenever the regions module publishes its table: each country's own capacity by the same rule")
                .report("capacity.conflicts", "")
                .view("crossings", "conflicts drawn", "capacity.crossing", "where the stress is, by the run's seed; a placeholder for the article's conflict mechanism");
        }
        d
    }
    fn target(&self, stress: f64, fairness: Option<f64>) -> f64 {
        let ineq = 1.0 + self.inequality * (1.0 - fairness.unwrap_or(0.5));
        (1.0 - (self.gain * stress * ineq).min(1.0)).max(self.floor)
    }
    fn knock_factor(&self, y: i64) -> f64 { self.knocks.iter().filter(|(_, until, _)| *until >= y).map(|(m, _, _)| m).product() }
}

impl Module for Capacity {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        if self.mode == "off" { return; }
        for t in ["region.pressure", "region.scored", "shock.crossing", "energy.state", "carbon.emitted"] { ctx.subscribe(t); }
        ctx.schedule_with_priority(to_next_year(ctx.now()), 9, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if self.mode == "off" { return; }
        match ev.topic.as_str() {
            "energy.state" => { if let Some(g) = ev.data["gmst_c"].as_f64() { self.gmst = g; } }
            "carbon.emitted" => { self.emitted_this_year += ev.data["gtc"].as_f64().unwrap_or(0.0); }
            "region.pressure" => {
                self.stress = ev.data["people_stress"].as_f64().unwrap_or(self.stress);
                self.voice = ev.data["voice"].as_f64().unwrap_or(self.voice);
                self.fairness = ev.data["fairness"].as_f64().unwrap_or(self.fairness);
                self.people = ev.data["people"].as_f64().unwrap_or(self.people);
            }
            "region.scored" => {
                let y = ev.data["year"].as_i64().unwrap_or(year_of(ctx.now()));
                let mut table = BTreeMap::new();
                if let Some(by) = ev.data["by_iso3"].as_object() {
                    for (iso, c) in by {
                        let s = c["s"].as_array();
                        let band = |i: usize| s.and_then(|a| a.get(i)).and_then(|v| v.as_f64());
                        table.insert(iso.clone(), CountryState { name: c["name"].as_str().unwrap_or(iso).to_string(), people: c["people"].as_f64().unwrap_or(0.0),
                            stress: c["stress"].as_f64().unwrap_or(0.0), voice: band(3), fairness: band(2) });
                    }
                }
                self.countries = table;
                if self.mode == "on" && year_of_str(&self.from).map(|f| y >= f).unwrap_or(true) {
                    let by: serde_json::Map<String, serde_json::Value> = self.countries.iter().map(|(iso, c)| {
                        let own = self.target(c.stress, c.fairness) * self.knock_factor(y);
                        (iso.clone(), json!((own * 1000.0).round() / 1000.0))
                    }).collect();
                    ctx.publish("capacity.scored", json!({ "year": y, "by_iso3": by }));
                }
            }
            "shock.crossing" => {
                let y = year_of(ctx.now());
                let kind = ev.data["kind"].as_str().unwrap_or("");
                let yrs = |k: &str, d: f64| ev.data[k].as_f64().unwrap_or(d).ceil() as i64;
                let cooling = ev.data["forcing_w_m2"].as_f64().unwrap_or(0.0) < 0.0;
                let title = ev.data["title"].as_str().unwrap_or(kind).to_string();
                match kind {
                    "economy" | "technology" | "solar" => self.knocks.push((1.0 - self.knock_economy, y + yrs("emissions_years", 2.0) - 1, title)),
                    "war" => self.knocks.push((1.0 - self.knock_war, y + 4, title)),
                    "disease" => self.knocks.push((1.0 - self.knock_disease, y + 2, title)),
                    "geology" | "astronomy" if cooling => self.knocks.push((1.0 - self.knock_eruption, y + yrs("forcing_years", 1.0) - 1, title)),
                    _ => {}
                }
            }
            "tick" => {
                let y = year_of(ctx.now());
                let active = y >= year_of_str(&self.from).unwrap_or(i64::MIN);
                self.knocks.retain(|(_, until, _)| *until >= y);
                // the options, from last year's emissions and this year's warming
                self.emitted_last_year = self.emitted_this_year; self.emitted_this_year = 0.0;
                let e = self.emitted_last_year;
                for (level, b, yl, cut) in [(self.level_c, "options.budget_gtc", "options.years_left", "options.cut_needed_pct_yr"), (self.level2_c, "options.budget2_gtc", "options.years_left2", "options.cut_needed2_pct_yr")] {
                    let budget = ((level - self.gmst) / self.tcre_c_per_gtc).max(0.0);
                    ctx.report(b, "GtC", budget);
                    ctx.report(yl, "yr", if e > 0.0 { budget / e } else { 0.0 });
                    ctx.report(cut, "%/yr", if budget > 0.0 && e > 0.0 { (100.0 * e / (2.0 * budget)).min(100.0) } else if e > 0.0 { 100.0 } else { 0.0 });
                }
                // conflicts, where the stress is
                if active && self.mode == "on" && self.conflict_gain > 0.0 {
                    let mut drawn = Vec::new();
                    for (iso, c) in &self.countries {
                        if c.stress <= 0.0 { continue }
                        let chance = (self.conflict_gain * c.stress * (1.0 + self.inequality * (1.0 - c.fairness.unwrap_or(0.5))) * (1.0 - 0.5 * c.voice.unwrap_or(0.5))).clamp(0.0, 1.0);
                        if ctx.rng().f64() < chance { drawn.push((iso.clone(), c.clone(), chance)); }
                    }
                    for (iso, c, chance) in drawn {
                        self.conflicts += 1;
                        let share = if self.people > 0.0 { c.people / self.people } else { 0.0 };
                        let knock = (self.conflict_knock * share * 100.0).min(0.2);
                        self.knocks.push((1.0 - knock, y + self.conflict_years - 1, format!("conflict in {}", c.name)));
                        let title = format!("Conflict in {}", c.name);
                        let sentence = format!("{}: conflict in {}, drawn at {:.1}% a year with {:.0}% of its people under heat stress: {:.1}% of them die and the world's capacity to cut loses {:.1}% for {} years.", y, c.name, 100.0 * chance, 100.0 * c.stress, 100.0 * self.conflict_mortality, 100.0 * knock, self.conflict_years);
                        ctx.publish("people.shock", json!({ "id": format!("conflict-{}", iso.to_lowercase()), "title": title, "year": y, "fraction": self.conflict_mortality, "iso3": iso, "name": c.name }));
                        ctx.publish("capacity.crossing", json!({ "id": format!("conflict-{}-{}", iso.to_lowercase(), y), "title": title, "name": c.name, "iso3": iso, "year": y, "stress": c.stress, "chance": chance, "fraction": self.conflict_mortality, "knock": knock, "sentence": sentence, "source": "a draw from the run's seed under the module's declared placeholder rule", "confidence": "guess" }));
                    }
                    ctx.report("capacity.conflicts", "", self.conflicts as f64);
                }
                // the capacity: toward the target, fast down and slow up
                let target = self.target(self.stress, Some(self.fairness));
                if active {
                    let gap = target - self.capacity;
                    self.capacity += if gap < 0.0 { self.fall * gap } else { self.recover * self.voice * gap };
                    self.capacity = self.capacity.clamp(self.floor, 1.0);
                }
                let effective = (self.capacity * self.knock_factor(y)).clamp(self.floor, 1.0);
                ctx.report("capacity.target", "1", target);
                ctx.report("capacity.pressure", "1", 1.0 - target);
                ctx.report("capacity.world", "1", effective);
                if active && self.mode == "on" { ctx.publish("emissions.capacity", json!({ "name": "policy", "capacity": effective })); }
                self.last_year = y;
                ctx.schedule_with_priority(to_next_year(ctx.now()), 9, "tick", json!(null));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { *self = serde_json::from_value(v).unwrap(); }
}
