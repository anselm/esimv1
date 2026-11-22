//! The run loop. Pops the smallest (time, priority, sequence) event, delivers it to the
//! addressed module or the topic's subscribers, queues whatever they scheduled, closes
//! ledgers on the cadence the manifest sets, snapshots on request. Single-threaded in M0;
//! the ordering is already the one a parallel version must reproduce.
//!
//! The kernel is an event bus and the manifest is played onto it (2026-11-07, after the
//! orbital pattern): each module entry is a `manifest.entity` event at the start, and a
//! built-in listener builds the module when the event comes by. A module may publish the
//! same topic later and a module is born mid-run; the log records it and a snapshot
//! carries it, so replay rebuilds the same list.

use crate::event::{Event, Queued};
use crate::ledger::{Ledger, LedgerReport};
use crate::log::EventLog;
use crate::manifest::Manifest;
use crate::module::{Ctx, FieldSeries, Module, ModuleFactory, Series};
use crate::rng::Rng;
use crate::time::{Duration, Time};
use crate::world::World;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BinaryHeap};

pub const KERNEL: &str = "kernel";
const CLOSE_PRIORITY: i16 = i16::MAX;
const SNAPSHOT_PRIORITY: i16 = i16::MAX - 1;
/// The topic a module is born on. The manifest's own entities arrive on it at the start,
/// before anything else at that instant; a module may publish one later and a module is
/// built, initialised and registered when it is delivered. Data: `{ id, kind, params }`.
pub const ENTITY: &str = "manifest.entity";
/// What the kernel logs when a birth is refused: an id already taken, a kind nobody has,
/// a factory that failed. The run goes on without it.
pub const REFUSED: &str = "manifest.refused";
const ENTITY_PRIORITY: i16 = i16::MIN;

pub struct Kernel {
    pub manifest: Manifest,
    factory: ModuleFactory,
    /// Modules born after the start, in order: `(id, kind, params)`, the birth event's data.
    /// A snapshot carries them so a resumed run rebuilds the same module list.
    pub born: Vec<serde_json::Value>,
    start: Time,
    end: Time,
    now: Time,
    seq: u64,
    queue: BinaryHeap<Queued>,
    modules: Vec<Box<dyn Module>>,
    rngs: Vec<Rng>,
    index: BTreeMap<String, usize>,
    subs: BTreeMap<String, Vec<usize>>,
    /// topic -> cell -> modules: the spatial index for cell-addressed events
    subs_at: BTreeMap<String, BTreeMap<u32, Vec<usize>>>,
    ledgers: BTreeMap<String, Ledger>,
    series: BTreeMap<String, Series>,
    pub fields: BTreeMap<String, FieldSeries>,
    pub world: World,
    pub log: EventLog,
    close_every: Duration,
    snapshot_every: Option<Duration>,
    pub snapshots: Vec<Snapshot>,
    delivered: u64,
}

/// Everything needed to continue a run from an instant.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub t: Time,
    pub seq: u64,
    pub delivered: u64,
    /// Length of the log when the snapshot was taken; a replay reproduces the log from here.
    pub log_len: usize,
    pub modules: Vec<(String, serde_json::Value)>,
    /// Modules born after the start, as their birth events' data, in order.
    #[serde(default)]
    pub born: Vec<serde_json::Value>,
    pub rngs: Vec<Rng>,
    pub queue: Vec<Event>,
    pub subs: BTreeMap<String, Vec<usize>>,
    #[serde(default)]
    pub subs_at: BTreeMap<String, BTreeMap<u32, Vec<usize>>>,
    pub ledgers: BTreeMap<String, Ledger>,
    pub series: BTreeMap<String, Series>,
    #[serde(default)]
    pub world: World,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunOutput {
    pub name: String,
    pub seed: u64,
    pub start: Time,
    pub end: Time,
    pub kernel_version: String,
    pub events: usize,
    pub delivered: u64,
    pub log_digest: String,
    pub ledgers: Vec<LedgerReport>,
    pub all_ledgers_closed: bool,
    pub series: BTreeMap<String, Series>,
    /// Field names with unit, grid and frame times; frame data is written separately.
    pub fields: BTreeMap<String, FieldMeta>,
    pub elapsed_ms: u128,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldMeta {
    pub unit: String,
    pub grid: crate::grid::LatLonGrid,
    pub times: Vec<Time>,
    pub min: f32,
    pub max: f32,
}

impl Kernel {
    pub fn new(manifest: Manifest, factory: ModuleFactory) -> Result<Kernel, String> {
        let start = manifest.start()?;
        let end = manifest.end()?;
        if end <= start { return Err("run.end must be after run.start".into()); }
        let close_every = manifest.close_every()?;
        let snapshot_every = manifest.snapshot_every()?;
        let mut k = Kernel {
            manifest, factory, born: Vec::new(), start, end, now: start, seq: 0, queue: BinaryHeap::new(),
            modules: Vec::new(), rngs: Vec::new(), index: BTreeMap::new(), subs: BTreeMap::new(), subs_at: BTreeMap::new(), ledgers: BTreeMap::new(),
            series: BTreeMap::new(), fields: BTreeMap::new(), world: World::default(), log: EventLog::new(), close_every, snapshot_every,
            snapshots: Vec::new(), delivered: 0,
        };
        k.init()?;
        Ok(k)
    }

    /// The manifest is played onto the bus: one `manifest.entity` event per module at the
    /// start instant, ahead of everything else, in the manifest's order. They are delivered
    /// here, before `run`, so a refused entity is an error of construction and every module
    /// of the manifest is initialised before the first event; they are not logged, since the
    /// run folder keeps the manifest itself. A birth after the start is logged like any event.
    fn init(&mut self) -> Result<(), String> {
        // The manifest's births take no sequence numbers of their own (they are not logged,
        // and the log's numbering must not move because of them): they are ordered among
        // themselves by position, and nothing else may queue at ENTITY_PRIORITY.
        let births: Vec<serde_json::Value> = self.manifest.modules.iter()
            .map(|m| serde_json::json!({ "id": m.name, "kind": m.kind, "params": crate::entities::toml_to_json(&m.params) })).collect();
        for (i, data) in births.into_iter().enumerate() {
            self.queue.push(Queued(Event { t: self.start, priority: ENTITY_PRIORITY, seq: i as u64 + 1, src: KERNEL.into(), to: None, topic: ENTITY.into(), at: None, data }));
        }
        while let Some(Queued(ev)) = self.queue.peek().cloned() {
            if !(ev.src == KERNEL && ev.topic == ENTITY && ev.t == self.start) { break; }
            self.queue.pop();
            self.birth(&ev)?;
        }
        // Kernel housekeeping: the first close and the first snapshot.
        self.push(self.start.add(self.close_every), CLOSE_PRIORITY, KERNEL, None, "kernel.close".into(), serde_json::Value::Null);
        if let Some(s) = self.snapshot_every {
            self.push(self.start.add(s), SNAPSHOT_PRIORITY, KERNEL, None, "kernel.snapshot".into(), serde_json::Value::Null);
        }
        Ok(())
    }

    fn push(&mut self, t: Time, priority: i16, src: &str, to: Option<String>, topic: String, data: serde_json::Value) {
        self.push_at(t, priority, src, to, topic, None, data);
    }
    fn push_at(&mut self, t: Time, priority: i16, src: &str, to: Option<String>, topic: String, at: Option<u32>, data: serde_json::Value) {
        let priority = if priority == ENTITY_PRIORITY { ENTITY_PRIORITY + 1 } else { priority };
        self.seq += 1;
        self.queue.push(Queued(Event { t, priority, seq: self.seq, src: src.to_string(), to, topic, at, data }));
    }

    /// Build, register and initialise the module an entity event names. Its `params` gain
    /// the kind and the run seed, which is what the factory reads. Errors are the reasons a
    /// birth is refused.
    fn birth(&mut self, ev: &Event) -> Result<(), String> {
        let id = ev.data.get("id").and_then(|v| v.as_str()).ok_or("entity without id")?.to_string();
        let kind = ev.data.get("kind").and_then(|v| v.as_str()).ok_or_else(|| format!("entity {id} without kind"))?.to_string();
        if self.index.contains_key(&id) { return Err(format!("entity {id}: a module with that id exists")); }
        let mut params = match ev.data.get("params") {
            Some(p) => crate::entities::json_to_toml(p).map_err(|e| format!("entity {id}: params: {e}"))?,
            None => toml::Value::Table(Default::default()),
        };
        let t = params.as_table_mut().ok_or_else(|| format!("entity {id}: params is not a table"))?;
        t.insert("kind".into(), toml::Value::String(kind.clone()));
        t.entry("seed").or_insert(toml::Value::Integer(self.manifest.run.seed as i64));
        let module = (self.factory)(&id, &params).map_err(|e| format!("module {id} ({kind}): {e}"))?;
        let i = self.modules.len();
        self.modules.push(module);
        self.rngs.push(Rng::for_module(self.manifest.run.seed, &id));
        self.index.insert(id.clone(), i);
        if ev.t > self.start || ev.src != KERNEL { self.born.push(ev.data.clone()); }
        let mut ctx = Ctx {
            now: self.now, me: id.clone(), out: Vec::new(), subs: Vec::new(), subs_at: Vec::new(),
            ledgers: &mut self.ledgers, series: &mut self.series, fields: &mut self.fields, world: &mut self.world, world_writes: Vec::new(), rng: &mut self.rngs[i],
        };
        self.modules[i].init(&mut ctx);
        let (out, subs, subs_at, writes) = (ctx.out, ctx.subs, ctx.subs_at, ctx.world_writes);
        for topic in subs { self.subs.entry(topic).or_default().push(i); }
        for (topic, cells) in subs_at { let m = self.subs_at.entry(topic).or_default(); for c in cells { m.entry(c).or_default().push(i); } }
        self.log_world_writes(&id, writes);
        for (t, p, to, topic, at, data) in out { self.push_at(t, p, &id, to, topic, at, data); }
        Ok(())
    }

    /// Run to the end of the manifest window.
    pub fn run(&mut self) -> RunOutput {
        let t0 = std::time::Instant::now();
        while let Some(Queued(ev)) = self.queue.peek().cloned() {
            if ev.t > self.end { break; }
            self.queue.pop();
            self.now = ev.t;
            self.log.append(&ev);
            if ev.topic == ENTITY {
                // a module born mid-run: built and initialised, then the event goes on to
                // whoever watches births; a refusal is logged and the run continues
                if let Err(reason) = self.birth(&ev) {
                    self.seq += 1;
                    let refused = Event { t: self.now, priority: ev.priority, seq: self.seq, src: KERNEL.into(), to: None, topic: REFUSED.into(), at: None,
                        data: serde_json::json!({ "id": ev.data.get("id").cloned().unwrap_or(serde_json::Value::Null), "reason": reason }) };
                    self.log.append(&refused);
                }
                self.deliver(&ev);
                continue;
            }
            if ev.src == KERNEL { self.housekeeping(&ev); continue; }
            self.deliver(&ev);
        }
        // Final close at the end instant so the last period is checked too.
        self.now = self.end;
        for l in self.ledgers.values_mut() { l.close(self.end); }
        self.output(t0.elapsed().as_millis())
    }

    fn housekeeping(&mut self, ev: &Event) {
        match ev.topic.as_str() {
            "kernel.close" => {
                for l in self.ledgers.values_mut() { l.close(ev.t); }
                let next = ev.t.add(self.close_every);
                if next < self.end { self.push(next, CLOSE_PRIORITY, KERNEL, None, "kernel.close".into(), serde_json::Value::Null); }
            }
            "kernel.snapshot" => {
                // Queue the next snapshot first so the snapshot's queue carries it and a
                // resumed run snapshots on the same cadence.
                let next = ev.t.add(self.snapshot_every.unwrap());
                if next < self.end { self.push(next, SNAPSHOT_PRIORITY, KERNEL, None, "kernel.snapshot".into(), serde_json::Value::Null); }
                self.snapshots.push(self.snapshot());
            }
            other => panic!("unknown kernel event {other}"),
        }
    }

    fn deliver(&mut self, ev: &Event) {
        let mut targets: Vec<usize> = match &ev.to {
            Some(name) => match self.index.get(name) { Some(&i) => vec![i], None => vec![] },
            None => {
                let mut v = self.subs.get(&ev.topic).cloned().unwrap_or_default();
                if let Some(c) = ev.at {
                    if let Some(m) = self.subs_at.get(&ev.topic).and_then(|m| m.get(&c)) { v.extend(m.iter().copied()); }
                }
                v
            }
        };
        targets.sort_unstable(); targets.dedup();
        for i in targets {
            let mut ctx = Ctx {
                now: self.now, me: self.modules[i].name().to_string(), out: Vec::new(), subs: Vec::new(), subs_at: Vec::new(),
                ledgers: &mut self.ledgers, series: &mut self.series, fields: &mut self.fields, world: &mut self.world, world_writes: Vec::new(), rng: &mut self.rngs[i],
            };
            self.modules[i].handle(ev, &mut ctx);
            self.delivered += 1;
            let (out, subs, subs_at, writes) = (ctx.out, ctx.subs, ctx.subs_at, ctx.world_writes);
            for topic in subs { self.subs.entry(topic).or_default().push(i); }
            for (topic, cells) in subs_at { let m = self.subs_at.entry(topic).or_default(); for c in cells { m.entry(c).or_default().push(i); } }
            let src = self.modules[i].name().to_string();
            self.log_world_writes(&src, writes);
            for (t, p, to, topic, at, data) in out { self.push_at(t, p, &src, to, topic, at, data); }
        }
    }

    /// World writes are logged as events that were never queued: they record that the
    /// state changed, with a digest, so a replay can be checked against the log.
    fn log_world_writes(&mut self, src: &str, writes: Vec<(String, u64, u64, usize)>) {
        for (name, version, digest, len) in writes {
            self.seq += 1;
            let ev = Event { t: self.now, priority: 0, seq: self.seq, src: src.to_string(), to: None, topic: "world.write".into(), at: None,
                data: serde_json::json!({ "field": name, "version": version, "digest": format!("{digest:016x}"), "len": len }) };
            self.log.append(&ev);
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            t: self.now, seq: self.seq, delivered: self.delivered, log_len: self.log.len(),
            modules: self.modules.iter().map(|m| (m.name().to_string(), m.snapshot())).collect(),
            born: self.born.clone(),
            rngs: self.rngs.clone(),
            queue: self.queue.iter().map(|q| q.0.clone()).collect(),
            subs: self.subs.clone(), subs_at: self.subs_at.clone(), ledgers: self.ledgers.clone(), series: self.series.clone(), world: self.world.clone(),
        }
    }

    /// Rebuild a kernel from a manifest and a snapshot, ready to `run` onward. The log
    /// starts empty here; a replay test compares it with the tail of the original log.
    pub fn from_snapshot(manifest: Manifest, factory: ModuleFactory, snap: Snapshot) -> Result<Kernel, String> {
        let start = manifest.start()?;
        let end = manifest.end()?;
        let close_every = manifest.close_every()?;
        let snapshot_every = manifest.snapshot_every()?;
        let mut modules = Vec::new();
        let mut index = BTreeMap::new();
        // the manifest's modules, then the ones born on the way, in the order they were built
        let mut entries: Vec<(String, toml::Value)> = manifest.modules.iter().map(|m| (m.name.clone(), m.params.clone())).collect();
        for b in &snap.born {
            let id = b.get("id").and_then(|v| v.as_str()).ok_or("snapshot: a born entity without id")?.to_string();
            let mut params = match b.get("params") { Some(p) => crate::entities::json_to_toml(p)?, None => toml::Value::Table(Default::default()) };
            let t = params.as_table_mut().ok_or("snapshot: born params is not a table")?;
            t.insert("kind".into(), b.get("kind").and_then(|v| v.as_str()).map(|k| toml::Value::String(k.into())).ok_or("snapshot: a born entity without kind")?);
            t.entry("seed").or_insert(toml::Value::Integer(manifest.run.seed as i64));
            entries.push((id, params));
        }
        for (i, (name, params)) in entries.iter().enumerate() {
            index.insert(name.clone(), i);
            let mut module = factory(name, params)?;
            let (saved, state) = snap.modules.get(i).ok_or("snapshot module count")?;
            if saved != name { return Err(format!("snapshot module order: {saved} vs {name}")); }
            module.restore(state.clone());
            modules.push(module);
        }
        let mut queue = BinaryHeap::new();
        for ev in snap.queue { queue.push(Queued(ev)); }
        Ok(Kernel {
            manifest, factory, born: snap.born, start, end, now: snap.t, seq: snap.seq, queue, modules, rngs: snap.rngs, index,
            subs: snap.subs, subs_at: snap.subs_at, ledgers: snap.ledgers, series: snap.series, fields: BTreeMap::new(), world: snap.world, log: EventLog::new(),
            close_every, snapshot_every, snapshots: Vec::new(), delivered: snap.delivered,
        })
    }

    fn output(&self, elapsed_ms: u128) -> RunOutput {
        let ledgers: Vec<LedgerReport> = self.ledgers.values().map(LedgerReport::from).collect();
        RunOutput {
            name: self.manifest.run.name.clone(), seed: self.manifest.run.seed, start: self.start, end: self.end,
            kernel_version: env!("CARGO_PKG_VERSION").to_string(),
            events: self.log.len(), delivered: self.delivered, log_digest: self.log.digest_hex(),
            all_ledgers_closed: ledgers.iter().all(|l| l.all_closed), ledgers,
            series: self.series.clone(),
            fields: self.fields.iter().map(|(k, f)| {
                let (mut lo, mut hi) = (f32::MAX, f32::MIN);
                for (_, d) in &f.frames { for &v in d { if v.is_finite() { lo = lo.min(v); hi = hi.max(v); } } }
                (k.clone(), FieldMeta { unit: f.unit.clone(), grid: f.grid.clone(), times: f.frames.iter().map(|(t, _)| *t).collect(), min: lo, max: hi })
            }).collect(),
            elapsed_ms,
        }
    }

    pub fn now(&self) -> Time { self.now }
    pub fn ledgers(&self) -> &BTreeMap<String, Ledger> { &self.ledgers }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::Ctx;

    /// Counts ticks; on its third tick, if it is the parent, it publishes an entity.
    struct Toy { name: String, parent: bool, ticks: u64, seen: Vec<String> }
    impl Module for Toy {
        fn name(&self) -> &str { &self.name }
        fn init(&mut self, ctx: &mut Ctx) {
            ctx.subscribe(ENTITY);
            ctx.schedule(Duration(3600), "toy.tick", serde_json::Value::Null);
        }
        fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
            if ev.topic == ENTITY { self.seen.push(ev.data["id"].as_str().unwrap_or("").to_string()); return; }
            self.ticks += 1;
            if self.parent && self.ticks == 3 {
                ctx.publish(ENTITY, serde_json::json!({ "id": "child", "kind": "toy", "params": { "parent": false } }));
                ctx.publish(ENTITY, serde_json::json!({ "id": "child", "kind": "toy", "params": { "parent": false } }));
                ctx.publish(ENTITY, serde_json::json!({ "id": "ghost", "kind": "nope" }));
            }
            ctx.schedule(Duration(3600), "toy.tick", serde_json::Value::Null);
        }
        fn snapshot(&self) -> serde_json::Value { serde_json::json!({ "ticks": self.ticks, "seen": self.seen }) }
        fn restore(&mut self, state: serde_json::Value) {
            self.ticks = state["ticks"].as_u64().unwrap();
            self.seen = state["seen"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
        }
    }
    fn toy_factory(name: &str, params: &toml::Value) -> Result<Box<dyn Module>, String> {
        match params.get("kind").and_then(|k| k.as_str()) {
            Some("toy") => Ok(Box::new(Toy { name: name.into(), parent: params.get("parent").and_then(|p| p.as_bool()).unwrap_or(true), ticks: 0, seen: Vec::new() })),
            other => Err(format!("no kind {other:?}")),
        }
    }
    fn manifest() -> Manifest {
        let mut m = Manifest::parse(r#"
[run]
name = "births"
start = "2026-01-01"
end = "2026-01-02"
close_every = "6h"
snapshot_every = "6h"
[[module]]
name = "parent"
kind = "toy"
"#).unwrap();
        for e in &mut m.modules { e.params.as_table_mut().unwrap().insert("kind".into(), toml::Value::String(e.kind.clone())); }
        m
    }

    #[test]
    fn a_module_is_born_mid_run_and_a_replay_rebuilds_it() {
        let mut k = Kernel::new(manifest(), toy_factory).unwrap();
        let out = k.run();
        assert_eq!(k.modules.len(), 2, "parent and child");
        assert_eq!(k.born.len(), 1);
        let topics: Vec<&str> = k.log.events.iter().map(|e| e.topic.as_str()).collect();
        assert_eq!(topics.iter().filter(|t| **t == ENTITY).count(), 3, "the three births the parent published are logged");
        assert_eq!(topics.iter().filter(|t| **t == REFUSED).count(), 2, "the duplicate and the unknown kind are refused");
        let parent = k.modules[0].snapshot();
        assert_eq!(parent["seen"], serde_json::json!(["child", "child", "ghost"]), "the parent watches births too");
        assert_eq!(parent["ticks"].as_u64().unwrap(), 24);
        assert_eq!(k.modules[1].snapshot()["ticks"].as_u64().unwrap(), 21, "the child ticks from its birth at hour 3");
        // resume from the snapshot taken after the birth and reproduce the tail of the log
        let snap = k.snapshots.iter().find(|s| !s.born.is_empty()).expect("a snapshot after the birth").clone();
        let mut k2 = Kernel::from_snapshot(manifest(), toy_factory, snap.clone()).unwrap();
        assert_eq!(k2.modules.len(), 2);
        let out2 = k2.run();
        let tail: Vec<&Event> = k.log.events.iter().skip(snap.log_len).collect();
        let replayed: Vec<&Event> = k2.log.events.iter().collect();
        assert_eq!(tail.len(), replayed.len());
        for (a, b) in tail.iter().zip(replayed.iter()) { assert_eq!((a.t, a.seq, &a.topic, &a.src), (b.t, b.seq, &b.topic, &b.src)); }
        assert_eq!(out.delivered, out2.delivered, "the resumed kernel counts on from the snapshot");
    }
}
