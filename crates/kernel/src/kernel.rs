//! The run loop. Pops the smallest (time, priority, sequence) event, delivers it to the
//! addressed module or the topic's subscribers, queues whatever they scheduled, closes
//! ledgers on the cadence the manifest sets, snapshots on request. Single-threaded in M0;
//! the ordering is already the one a parallel version must reproduce.

use crate::event::{Event, Queued};
use crate::ledger::{Ledger, LedgerReport};
use crate::log::EventLog;
use crate::manifest::Manifest;
use crate::module::{Ctx, Module, ModuleFactory, Series};
use crate::rng::Rng;
use crate::time::{Duration, Time};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BinaryHeap};

pub const KERNEL: &str = "kernel";
const CLOSE_PRIORITY: i16 = i16::MAX;
const SNAPSHOT_PRIORITY: i16 = i16::MAX - 1;

pub struct Kernel {
    pub manifest: Manifest,
    start: Time,
    end: Time,
    now: Time,
    seq: u64,
    queue: BinaryHeap<Queued>,
    modules: Vec<Box<dyn Module>>,
    rngs: Vec<Rng>,
    index: BTreeMap<String, usize>,
    subs: BTreeMap<String, Vec<usize>>,
    ledgers: BTreeMap<String, Ledger>,
    series: BTreeMap<String, Series>,
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
    pub rngs: Vec<Rng>,
    pub queue: Vec<Event>,
    pub subs: BTreeMap<String, Vec<usize>>,
    pub ledgers: BTreeMap<String, Ledger>,
    pub series: BTreeMap<String, Series>,
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
    pub elapsed_ms: u128,
}

impl Kernel {
    pub fn new(manifest: Manifest, factory: ModuleFactory) -> Result<Kernel, String> {
        let start = manifest.start()?;
        let end = manifest.end()?;
        if end <= start { return Err("run.end must be after run.start".into()); }
        let close_every = manifest.close_every()?;
        let snapshot_every = manifest.snapshot_every()?;
        let mut modules = Vec::new();
        let mut rngs = Vec::new();
        let mut index = BTreeMap::new();
        for (i, m) in manifest.modules.iter().enumerate() {
            if index.insert(m.name.clone(), i).is_some() { return Err(format!("duplicate module name {}", m.name)); }
            modules.push(factory(&m.name, &m.params).map_err(|e| format!("module {} ({}): {e}", m.name, m.kind))?);
            rngs.push(Rng::for_module(manifest.run.seed, &m.name));
        }
        let mut k = Kernel {
            manifest, start, end, now: start, seq: 0, queue: BinaryHeap::new(),
            modules, rngs, index, subs: BTreeMap::new(), ledgers: BTreeMap::new(),
            series: BTreeMap::new(), log: EventLog::new(), close_every, snapshot_every,
            snapshots: Vec::new(), delivered: 0,
        };
        k.init();
        Ok(k)
    }

    fn init(&mut self) {
        for i in 0..self.modules.len() {
            let mut ctx = Ctx {
                now: self.now, me: self.modules[i].name().to_string(), out: Vec::new(), subs: Vec::new(),
                ledgers: &mut self.ledgers, series: &mut self.series, rng: &mut self.rngs[i],
            };
            self.modules[i].init(&mut ctx);
            let (out, subs) = (ctx.out, ctx.subs);
            for topic in subs { self.subs.entry(topic).or_default().push(i); }
            let src = self.modules[i].name().to_string();
            for (t, p, to, topic, data) in out { self.push(t, p, &src, to, topic, data); }
        }
        // Kernel housekeeping: the first close and the first snapshot.
        self.push(self.start.add(self.close_every), CLOSE_PRIORITY, KERNEL, None, "kernel.close".into(), serde_json::Value::Null);
        if let Some(s) = self.snapshot_every {
            self.push(self.start.add(s), SNAPSHOT_PRIORITY, KERNEL, None, "kernel.snapshot".into(), serde_json::Value::Null);
        }
    }

    fn push(&mut self, t: Time, priority: i16, src: &str, to: Option<String>, topic: String, data: serde_json::Value) {
        self.seq += 1;
        self.queue.push(Queued(Event { t, priority, seq: self.seq, src: src.to_string(), to, topic, data }));
    }

    /// Run to the end of the manifest window.
    pub fn run(&mut self) -> RunOutput {
        let t0 = std::time::Instant::now();
        while let Some(Queued(ev)) = self.queue.peek().cloned() {
            if ev.t > self.end { break; }
            self.queue.pop();
            self.now = ev.t;
            self.log.append(&ev);
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
        let targets: Vec<usize> = match &ev.to {
            Some(name) => match self.index.get(name) { Some(&i) => vec![i], None => vec![] },
            None => self.subs.get(&ev.topic).cloned().unwrap_or_default(),
        };
        for i in targets {
            let mut ctx = Ctx {
                now: self.now, me: self.modules[i].name().to_string(), out: Vec::new(), subs: Vec::new(),
                ledgers: &mut self.ledgers, series: &mut self.series, rng: &mut self.rngs[i],
            };
            self.modules[i].handle(ev, &mut ctx);
            self.delivered += 1;
            let (out, subs) = (ctx.out, ctx.subs);
            for topic in subs { self.subs.entry(topic).or_default().push(i); }
            let src = self.modules[i].name().to_string();
            for (t, p, to, topic, data) in out { self.push(t, p, &src, to, topic, data); }
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            t: self.now, seq: self.seq, delivered: self.delivered, log_len: self.log.len(),
            modules: self.modules.iter().map(|m| (m.name().to_string(), m.snapshot())).collect(),
            rngs: self.rngs.clone(),
            queue: self.queue.iter().map(|q| q.0.clone()).collect(),
            subs: self.subs.clone(), ledgers: self.ledgers.clone(), series: self.series.clone(),
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
        for (i, m) in manifest.modules.iter().enumerate() {
            index.insert(m.name.clone(), i);
            let mut module = factory(&m.name, &m.params)?;
            let (name, state) = snap.modules.get(i).ok_or("snapshot module count")?;
            if name != &m.name { return Err(format!("snapshot module order: {name} vs {}", m.name)); }
            module.restore(state.clone());
            modules.push(module);
        }
        let mut queue = BinaryHeap::new();
        for ev in snap.queue { queue.push(Queued(ev)); }
        Ok(Kernel {
            manifest, start, end, now: snap.t, seq: snap.seq, queue, modules, rngs: snap.rngs, index,
            subs: snap.subs, ledgers: snap.ledgers, series: snap.series, log: EventLog::new(),
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
            series: self.series.clone(), elapsed_ms,
        }
    }

    pub fn now(&self) -> Time { self.now }
    pub fn ledgers(&self) -> &BTreeMap<String, Ledger> { &self.ledgers }
}
