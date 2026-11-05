//! Wild cards: speculative events drawn by a seeded coin each year from a list with rough
//! annual chances, and applied to the modules that listen. A placeholder by design (Anselm,
//! 2026-11-05: "i don't care if the module is accurate - i just want a placeholder, it is ok
//! to have a black swan event happen more than once ... think of this like a video game with
//! wild card factors optionally"). The list is `data/events/future-speculative.toml`, whose
//! probabilities are recalled orders of magnitude and say so; events may repeat; the effects
//! are three levers: a forcing pulse on the energy balance for some years (`forcing.extra`,
//! the same port the gases and the assessed forcings use), a scaling of emissions for some
//! years (`emissions.scale`), and a fraction of the world's people (`people.shock`). Every
//! event drawn is a `shock.crossing` with a sentence, so it stands in the story beside the
//! thresholds. `mode = "named"` fires one chosen event in a chosen year for a scenario.
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, Module, Time};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize, Default)]
struct Row {
    id: String, title: String, kind: String, magnitude: String, source: String, confidence: String,
    p_per_year: Option<f64>,
    forcing_w_m2: Option<f64>, forcing_years: Option<f64>,
    emissions_scale: Option<f64>, emissions_years: Option<f64>,
    mortality: Option<f64>, region: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Shock {
    name: String,
    file: String, mode: String, event: String, at: String, scale: f64, from: String, max: i64,
    fired: i64,
    /// active pulses and scalings: (last year inclusive, value)
    pulses: Vec<(i64, f64)>, scalings: Vec<(i64, f64)>,
    #[serde(skip)] rows: Vec<Row>,
}

fn year_of(t: Time) -> i64 { t.to_iso()[0..4].parse().unwrap() }
fn to_next_year(now: Time) -> Duration { Time::parse(&format!("{:04}-01-01", year_of(now) + 1)).unwrap().since(now) }
fn year_of_str(s: &str) -> Option<i64> { s.get(0..4).and_then(|y| y.parse().ok()) }

fn load_rows(file: &str) -> Result<Vec<Row>, String> {
    let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let path = root.join("data/events").join(file);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: toml::Value = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let s = |t: &toml::Value, k: &str| t.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    let f = |t: &toml::Value, k: &str| t.get(k).and_then(|x| x.as_float().or_else(|| x.as_integer().map(|i| i as f64)));
    let mut rows = Vec::new();
    for e in v.get("event").and_then(|x| x.as_array()).ok_or("no [[event]] tables")? {
        let eff = e.get("effect");
        rows.push(Row {
            id: s(e, "id"), title: s(e, "title"), kind: s(e, "kind"), magnitude: s(e, "magnitude"), source: s(e, "source"), confidence: s(e, "confidence"),
            p_per_year: f(e, "p_per_year"),
            forcing_w_m2: eff.and_then(|t| f(t, "forcing_w_m2")), forcing_years: eff.and_then(|t| f(t, "forcing_years")),
            emissions_scale: eff.and_then(|t| f(t, "emissions_scale")), emissions_years: eff.and_then(|t| f(t, "emissions_years")),
            mortality: eff.and_then(|t| f(t, "mortality")), region: eff.and_then(|t| t.get("region")).and_then(|x| x.as_str()).map(str::to_string),
        });
    }
    Ok(rows)
}

impl Shock {
    pub fn new(name: &str, p: &toml::Value) -> Result<Shock, String> {
        let file = param_str(p, "file", "future-speculative.toml").to_string();
        let rows = load_rows(&file)?;
        let mode = param_str(p, "mode", "off").to_string();
        if !["off", "draw", "named"].contains(&mode.as_str()) { return Err(format!("{name}: mode must be off, draw or named")); }
        let event = param_str(p, "event", "").to_string();
        if mode == "named" && !rows.iter().any(|r| r.id == event) { return Err(format!("{name}: no event {event:?} in {file}")); }
        Ok(Shock {
            name: name.to_string(), file, mode, event,
            at: param_str(p, "at", "").to_string(), scale: param_f64(p, "scale", 1.0), from: param_str(p, "from", "2025-01-01").to_string(),
            max: param_f64(p, "max", 0.0) as i64,
            fired: 0, pulses: Vec::new(), scalings: Vec::new(), rows,
        })
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let file = param_str(p, "file", "future-speculative.toml").to_string();
        let mode = param_str(p, "mode", "off").to_string();
        let mut d = Describe::new("events.shock", name, "Wild cards: speculative events (eruptions, pandemics, wars, storms, crashes) drawn by a seeded coin each year from a list with rough annual chances, or one named event in a chosen year, applied through three levers the planet modules expose: a forcing pulse for some years, a scaling of emissions for some years, a fraction of the world's people. A placeholder by design: the list's chances are recalled orders of magnitude and say so, events may repeat, and nothing here is a forecast of anything. Off unless a manifest or variant turns it on.")
            .priority(1).dt("1 year (1 January)")
            .param("file", file.clone(), "", "the list, under data/events; rows carry p_per_year and an effect table")
            .param("mode", mode.clone(), "", "off: nothing happens; draw: each year from `from`, every row with a chance is tried once against the module's seeded stream; named: the event named fires once in the year `at`, or the first year from `from` if `at` is empty")
            .param("event", param_str(p, "event", ""), "", "the row id for mode = named")
            .param("at", param_str(p, "at", ""), "", "the year (or date) the named event fires")
            .param("scale", param_f64(p, "scale", 1.0), "", "multiplies every annual chance: 2 is a wilder world, 0.5 a calmer one")
            .param("from", param_str(p, "from", "2025-01-01"), "", "no draws before this date, so the hindcast is untouched")
            .param("max", param_f64(p, "max", 0.0), "", "at most this many events in a run; 0 means no limit, and the same event may come twice")
            .note("Effects: forcing_w_m2 for forcing_years goes to the energy balance as forcing.extra named shock (a cooling eruption is negative); emissions_scale for emissions_years multiplies the emissions rate (people.shock does the same to the count of people, once, by the row's mortality). Deaths are a single fraction of everyone, with no place and no age: the population module applies it to every country alike. The region field is carried in the event and not applied.");
        if let Ok(rows) = load_rows(&file) {
            d = d.table("events", json!(rows.iter().map(|r| json!({ "id": r.id, "title": r.title, "kind": r.kind, "p_per_year": r.p_per_year, "forcing_w_m2": r.forcing_w_m2, "forcing_years": r.forcing_years, "emissions_scale": r.emissions_scale, "emissions_years": r.emissions_years, "mortality": r.mortality, "source": r.source, "confidence": r.confidence })).collect::<Vec<_>>()));
        }
        if mode != "off" {
            d = d.publish("shock.crossing", "{ id, title, name, kind, year, magnitude, source, confidence, forcing_w_m2, forcing_years, emissions_scale, emissions_years, mortality, region, sentence } for each event drawn")
                .publish("forcing.extra", "{ name: shock, w_m2 } each January: the sum of the active pulses, zero when none")
                .publish("emissions.scale", "{ name: shock, factor } each January: the product of the active scalings, one when none")
                .publish("people.shock", "{ id, title, year, fraction } once per event with a mortality")
                .report("shock.count", "").report("shock.forcing_w_m2", "W/m2").report("shock.emissions_factor", "1")
                .view("crossings", "wild cards drawn", "shock.crossing", "each one a row of the list, with the chance it was drawn at and what the run did with it")
                .view("chart", "wild-card forcing", "shock.forcing_w_m2", "pulses on the energy balance, negative for a dust veil")
                .view("chart", "wild-card emissions factor", "shock.emissions_factor", "");
        }
        d
    }
    fn fire(&mut self, i: usize, y: i64, ctx: &mut Ctx) {
        let r = self.rows[i].clone();
        self.fired += 1;
        let mut did = Vec::new();
        if let Some(w) = r.forcing_w_m2 {
            let yrs = r.forcing_years.unwrap_or(1.0).ceil() as i64;
            self.pulses.push((y + yrs - 1, w));
            did.push(format!("{} W/m² on the energy balance for {} year{}", w, yrs, if yrs == 1 { "" } else { "s" }));
        }
        if let Some(s) = r.emissions_scale {
            let yrs = r.emissions_years.unwrap_or(1.0).ceil() as i64;
            self.scalings.push((y + yrs - 1, s));
            did.push(format!("emissions × {} for {} year{}", s, yrs, if yrs == 1 { "" } else { "s" }));
        }
        if let Some(m) = r.mortality {
            if m > 0.0 {
                ctx.publish("people.shock", json!({ "id": r.id, "title": r.title, "year": y, "fraction": m }));
                did.push(format!("{:.2}% of everyone dies", 100.0 * m));
            }
        }
        let applied = if did.is_empty() { "nothing applied: a region named and no lever for it".to_string() } else { did.join("; ") };
        let chance = if self.mode == "named" { "named for this year by the manifest".to_string() } else { match r.p_per_year { Some(p) => format!("drawn at {} a year", pct(p * self.scale)), None => "named".to_string() } };
        let sentence = format!("{}: wild card. {}. {}. ({}; {}.)", y, r.title, r.magnitude, chance, applied);
        ctx.publish("shock.crossing", json!({ "id": r.id, "title": r.title, "name": r.title, "kind": r.kind, "year": y, "magnitude": r.magnitude,
            "source": r.source, "confidence": r.confidence, "p_per_year": r.p_per_year, "scale": self.scale,
            "forcing_w_m2": r.forcing_w_m2, "forcing_years": r.forcing_years, "emissions_scale": r.emissions_scale, "emissions_years": r.emissions_years,
            "mortality": r.mortality, "region": r.region, "sentence": sentence }));
    }
}

fn pct(p: f64) -> String {
    if p >= 0.01 { format!("{:.0}%", 100.0 * p) } else if p >= 0.0001 { format!("{:.2}%", 100.0 * p) } else { format!("{:e}", p) }
}

use esim_kernel::describe::Describe;

impl Module for Shock {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        if self.mode == "off" { return; }
        ctx.schedule_with_priority(to_next_year(ctx.now()), 1, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" || self.mode == "off" { return; }
        let y = year_of(ctx.now());
        let from_y = year_of_str(&self.from).unwrap_or(i64::MIN);
        if y >= from_y {
            let mut fire = Vec::new();
            if self.mode == "named" {
                let at_y = year_of_str(&self.at);
                let due = match at_y { Some(a) => a == y, None => self.fired == 0 };
                if due { if let Some(i) = self.rows.iter().position(|r| r.id == self.event) { fire.push(i); } }
            } else {
                for i in 0..self.rows.len() {
                    if self.max > 0 && self.fired + fire.len() as i64 >= self.max { break; }
                    if let Some(p) = self.rows[i].p_per_year {
                        if p <= 0.0 { continue; }
                        let u = ctx.rng().f64();
                        if u < p * self.scale { fire.push(i); }
                    }
                }
            }
            for i in fire { self.fire(i, y, ctx); }
        }
        self.pulses.retain(|(until, _)| *until >= y);
        self.scalings.retain(|(until, _)| *until >= y);
        let w: f64 = self.pulses.iter().map(|(_, w)| w).sum();
        let f: f64 = self.scalings.iter().map(|(_, f)| f).product();
        ctx.publish("forcing.extra", json!({ "name": "shock", "w_m2": w }));
        ctx.publish("emissions.scale", json!({ "name": "shock", "factor": f }));
        ctx.report("shock.count", "", self.fired as f64);
        ctx.report("shock.forcing_w_m2", "W/m2", w);
        ctx.report("shock.emissions_factor", "1", f);
        ctx.schedule_with_priority(to_next_year(ctx.now()), 1, "tick", json!(null));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) {
        let mut s: Shock = serde_json::from_value(v).unwrap();
        s.rows = load_rows(&s.file).expect("wild cards reload");
        *self = s;
    }
}
