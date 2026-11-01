//! The describe verb. A module kind, given a name and parameters, says what its surface
//! is before anything is built or fetched: parameters with defaults, the topics it
//! publishes and subscribes, the fields it owns and reads, the ledgers it moves amounts
//! through, the frames and series it reports, and the data it needs. The kernel checks a
//! manifest's wiring from these alone, and a facade reads them instead of source.

use crate::manifest::Manifest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Describe {
    pub kind: String,
    pub name: String,
    pub summary: String,
    /// Ordering among events at the same instant; lower runs first.
    pub priority: i16,
    /// The module's own tick, as written in a manifest ("1h", "1d").
    pub dt: String,
    pub params: Vec<Param>,
    /// Broadcast topics, delivered to subscribers.
    pub publishes: Vec<Port>,
    /// Directed messages to a named module.
    pub sends: Vec<Send>,
    /// Directed topics this module handles when addressed by name.
    pub accepts: Vec<Port>,
    pub subscribes: Vec<Port>,
    /// Shared world fields this module writes. The first writer owns a field.
    pub fields_owned: Vec<Port>,
    /// Shared world fields this module reads. `required` means it does nothing useful without.
    pub fields_read: Vec<Port>,
    pub ledgers: Vec<LedgerUse>,
    /// Field frames written for maps (output, not shared state).
    pub frames: Vec<Port>,
    /// Scalar series reported for charts and scoring.
    pub reports: Vec<Port>,
    /// Data this module fetches through the registry: catalog asset ids, or `events/<table>`.
    pub assets: Vec<AssetUse>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Param {
    pub name: String,
    pub default: serde_json::Value,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub unit: String,
    /// Provenance or meaning. "guess" marks a number nobody has sourced.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Port {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub unit: String,
    #[serde(default)]
    pub required: bool,
    /// For a read: the topic whose arrival triggers it, when it is not read on the tick.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub on: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Send {
    pub to: String,
    pub topic: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LedgerUse {
    pub name: String,
    pub unit: String,
    pub tolerance: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AssetUse {
    pub key: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
}

fn port(name: &str, unit: &str, required: bool, note: &str) -> Port {
    Port { name: name.into(), unit: unit.into(), required, on: String::new(), note: note.into() }
}

impl Describe {
    pub fn new(kind: &str, name: &str, summary: &str) -> Describe {
        Describe { kind: kind.into(), name: name.into(), summary: summary.into(), ..Default::default() }
    }
    pub fn priority(mut self, p: i16) -> Self { self.priority = p; self }
    pub fn dt(mut self, dt: &str) -> Self { self.dt = dt.into(); self }
    pub fn param(mut self, name: &str, default: impl Into<serde_json::Value>, unit: &str, note: &str) -> Self {
        self.params.push(Param { name: name.into(), default: default.into(), unit: unit.into(), note: note.into() });
        self
    }
    pub fn publish(mut self, topic: &str, note: &str) -> Self { self.publishes.push(port(topic, "", false, note)); self }
    pub fn send(mut self, to: &str, topic: &str) -> Self { self.sends.push(Send { to: to.into(), topic: topic.into() }); self }
    pub fn accept(mut self, topic: &str, note: &str) -> Self { self.accepts.push(port(topic, "", false, note)); self }
    pub fn subscribe(mut self, topic: &str, note: &str) -> Self { self.subscribes.push(port(topic, "", false, note)); self }
    pub fn own(mut self, field: &str, unit: &str) -> Self { self.fields_owned.push(port(field, unit, false, "")); self }
    pub fn read(mut self, field: &str, unit: &str, required: bool, note: &str) -> Self { self.fields_read.push(port(field, unit, required, note)); self }
    /// A read made when `on` arrives rather than on the tick; ordering against the owner is by event, not priority.
    pub fn read_on(mut self, field: &str, unit: &str, required: bool, on: &str, note: &str) -> Self {
        let mut p = port(field, unit, required, note);
        p.on = on.into();
        self.fields_read.push(p);
        self
    }
    pub fn ledger(mut self, name: &str, unit: &str, tolerance: f64) -> Self { self.ledgers.push(LedgerUse { name: name.into(), unit: unit.into(), tolerance }); self }
    pub fn frame(mut self, name: &str, unit: &str) -> Self { self.frames.push(port(name, unit, false, "")); self }
    pub fn report(mut self, name: &str, unit: &str) -> Self { self.reports.push(port(name, unit, false, "")); self }
    pub fn asset(mut self, key: &str, note: &str) -> Self { self.assets.push(AssetUse { key: key.into(), note: note.into() }); self }
    pub fn note(mut self, s: &str) -> Self { self.notes.push(s.into()); self }
}

/// Describes a kind under a name and parameters, without building it.
pub type Describer = fn(kind: &str, name: &str, params: &toml::Value) -> Result<Describe, String>;

/// Topics the kernel itself emits; subscribing to them needs no publisher.
pub const KERNEL_TOPICS: &[&str] = &["kernel.close", "kernel.snapshot", "world.write"];

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CheckReport {
    pub modules: Vec<Describe>,
    /// The run cannot be right: unknown kinds, duplicate names, two owners of a field,
    /// a required read with no owner, a directed send to nobody, ledgers in two units.
    pub errors: Vec<String>,
    /// The run will go but something is probably not what was meant.
    pub warnings: Vec<String>,
    /// Couplings that are optional and absent, and outputs nobody reads.
    pub info: Vec<String>,
}

impl CheckReport {
    pub fn ok(&self) -> bool { self.errors.is_empty() }
}

/// Check a manifest's wiring from its modules' descriptions alone. No data is fetched.
pub fn check(m: &Manifest, describe: Describer) -> CheckReport {
    let mut r = CheckReport::default();
    let mut names = BTreeSet::new();
    for e in &m.modules {
        if !names.insert(e.name.clone()) { r.errors.push(format!("two modules named {}", e.name)); }
        match describe(&e.kind, &e.name, &e.params) {
            Ok(d) => r.modules.push(d),
            Err(err) => r.errors.push(format!("{}: {err}", e.name)),
        }
    }
    // Owners of fields, publishers of topics, acceptors of directed topics.
    let mut owner: BTreeMap<&str, Vec<&Describe>> = BTreeMap::new();
    let mut publishers: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut accepts: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut ledger_units: BTreeMap<&str, (&str, f64, &str)> = BTreeMap::new();
    for d in &r.modules {
        for f in &d.fields_owned { owner.entry(&f.name).or_default().push(d); }
        for t in &d.publishes { publishers.entry(&t.name).or_default().push(&d.name); }
        for t in &d.accepts { accepts.entry(&d.name).or_default().insert(&t.name); }
        for l in &d.ledgers {
            match ledger_units.get(l.name.as_str()) {
                None => { ledger_units.insert(&l.name, (&l.unit, l.tolerance, &d.name)); }
                Some((u, tol, first)) => {
                    if *u != l.unit { r.errors.push(format!("ledger {} is {} in {first} and {} in {}", l.name, u, l.unit, d.name)); }
                    else if (*tol - l.tolerance).abs() > f64::EPSILON { r.warnings.push(format!("ledger {}: tolerance {} in {first}, {} in {}; the first wins", l.name, tol, l.tolerance, d.name)); }
                }
            }
        }
    }
    for (f, ds) in &owner {
        if ds.len() > 1 {
            let who: Vec<&str> = ds.iter().map(|d| d.name.as_str()).collect();
            r.errors.push(format!("field {f} has two writers: {}; the first to write owns it and the second panics", who.join(", ")));
        }
    }
    let mut read_fields: BTreeSet<&str> = BTreeSet::new();
    for d in &r.modules {
        for f in &d.fields_read {
            read_fields.insert(&f.name);
            match owner.get(f.name.as_str()) {
                None => {
                    if f.required { r.errors.push(format!("{} reads {} and nothing writes it", d.name, f.name)); }
                    else { r.info.push(format!("{} would read {} but nothing writes it{}", d.name, f.name, if f.note.is_empty() { String::new() } else { format!(" ({})", f.note) })); }
                }
                Some(ds) => {
                    let o = ds[0];
                    if !f.unit.is_empty() && !o.fields_owned.iter().any(|p| p.name == f.name && (p.unit == f.unit || p.unit.is_empty())) {
                        let ou = o.fields_owned.iter().find(|p| p.name == f.name).map(|p| p.unit.as_str()).unwrap_or("");
                        r.errors.push(format!("{} reads {} in {} but {} writes it in {}", d.name, f.name, f.unit, o.name, ou));
                    }
                    if f.required && f.on.is_empty() && o.priority >= d.priority && o.name != d.name {
                        r.warnings.push(format!("{} (priority {}) reads {} written by {} (priority {}) at the same instant, so it sees the previous write", d.name, d.priority, f.name, o.name, o.priority));
                    }
                }
            }
        }
        for s in &d.subscribes {
            if KERNEL_TOPICS.contains(&s.name.as_str()) { continue; }
            if !publishers.contains_key(s.name.as_str()) {
                r.info.push(format!("{} subscribes to {} and nothing publishes it{}", d.name, s.name, if s.note.is_empty() { String::new() } else { format!(" ({})", s.note) }));
            }
        }
        for s in &d.sends {
            match accepts.get(s.to.as_str()) {
                Some(set) if set.contains(s.topic.as_str()) => {}
                Some(_) => r.errors.push(format!("{} sends {} to {}, which does not accept it", d.name, s.topic, s.to)),
                None => r.errors.push(format!("{} sends {} to {}, which is not in the manifest", d.name, s.topic, s.to)),
            }
        }
    }
    for (f, ds) in &owner {
        if !read_fields.contains(f) { r.info.push(format!("{} writes {f} and nothing reads it", ds[0].name)); }
    }
    r
}
