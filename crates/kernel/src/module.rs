//! The module trait and the context a module is handed while it runs. A module owns its
//! private state, reacts to events, schedules more, moves conserved quantities through
//! ledgers, and declares its stocks. It never touches another module.

use crate::event::{Event, Priority};
use crate::grid::LatLonGrid;
use crate::ledger::Ledger;
use crate::rng::Rng;
use crate::time::{Duration, Time};
use crate::world::{World, WorldField};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub trait Module {
    fn name(&self) -> &str;
    /// Subscribe, create ledgers and accounts, declare opening stocks, schedule the first events.
    fn init(&mut self, ctx: &mut Ctx);
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx);
    /// Private state, for snapshots. Must round-trip through `restore` exactly.
    fn snapshot(&self) -> serde_json::Value;
    fn restore(&mut self, state: serde_json::Value);
}

/// Builds a module from its manifest entry. Modules crates register one per `kind`.
pub type ModuleFactory = fn(name: &str, params: &toml::Value) -> Result<Box<dyn Module>, String>;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Series {
    pub unit: String,
    pub points: Vec<(Time, f64)>,
}

/// A field snapshot series: one grid, a unit, and frames at times. Output, not state:
/// a resumed run starts a new series.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldSeries {
    pub unit: String,
    pub grid: LatLonGrid,
    pub frames: Vec<(Time, Vec<f32>)>,
}

/// Requests a module makes during one `handle`, applied by the kernel afterwards.
pub struct Ctx<'a> {
    pub(crate) now: Time,
    pub(crate) me: String,
    pub(crate) out: Vec<(Time, Priority, Option<String>, String, Option<u32>, serde_json::Value)>,
    pub(crate) subs: Vec<String>,
    pub(crate) subs_at: Vec<(String, Vec<u32>)>,
    pub(crate) ledgers: &'a mut BTreeMap<String, Ledger>,
    pub(crate) series: &'a mut BTreeMap<String, Series>,
    pub(crate) fields: &'a mut BTreeMap<String, FieldSeries>,
    pub(crate) world: &'a mut World,
    pub(crate) world_writes: Vec<(String, u64, u64, usize)>,
    pub(crate) rng: &'a mut Rng,
}

impl<'a> Ctx<'a> {
    pub fn now(&self) -> Time { self.now }
    pub fn me(&self) -> &str { &self.me }

    /// Schedule an event to myself after `after`.
    pub fn schedule(&mut self, after: Duration, topic: &str, data: serde_json::Value) {
        self.schedule_with_priority(after, 0, topic, data);
    }
    /// Same, with a priority: lower runs first among events at the same instant. A
    /// module that reads what another writes at the same instant takes a higher number.
    pub fn schedule_with_priority(&mut self, after: Duration, priority: Priority, topic: &str, data: serde_json::Value) {
        let me = self.me.clone();
        self.out.push((self.now.add(after), priority, Some(me), topic.to_string(), None, data));
    }
    /// Schedule an event to a named module at an absolute time.
    pub fn send_at(&mut self, t: Time, to: &str, topic: &str, data: serde_json::Value) {
        self.out.push((t, 0, Some(to.to_string()), topic.to_string(), None, data));
    }
    /// Send to a named module now (delivered after the current instant's earlier events).
    pub fn send(&mut self, to: &str, topic: &str, data: serde_json::Value) {
        self.out.push((self.now, 0, Some(to.to_string()), topic.to_string(), None, data));
    }
    /// Broadcast to every subscriber of a topic, now.
    pub fn publish(&mut self, topic: &str, data: serde_json::Value) {
        self.out.push((self.now, 0, None, topic.to_string(), None, data));
    }
    /// A spatial event: about one grid cell. Delivered to the topic's subscribers and to
    /// modules subscribed to that topic at that cell.
    pub fn publish_at(&mut self, cell: usize, topic: &str, data: serde_json::Value) {
        self.out.push((self.now, 0, None, topic.to_string(), Some(cell as u32), data));
    }
    /// Send a spatial event to a named module.
    pub fn send_cell(&mut self, to: &str, cell: usize, topic: &str, data: serde_json::Value) {
        self.out.push((self.now, 0, Some(to.to_string()), topic.to_string(), Some(cell as u32), data));
    }
    pub fn subscribe(&mut self, topic: &str) { self.subs.push(topic.to_string()); }
    /// Subscribe to a topic only for events at these cells (the kernel's spatial index).
    pub fn subscribe_at(&mut self, topic: &str, cells: &[usize]) {
        self.subs_at.push((topic.to_string(), cells.iter().map(|&c| c as u32).collect()));
    }

    pub fn ledger(&mut self, name: &str, unit: &str, tolerance: f64) {
        self.ledgers.entry(name.to_string()).or_insert_with(|| Ledger::new(name, unit, tolerance));
    }
    pub fn transfer(&mut self, ledger: &str, from: &str, to: &str, amount: f64) {
        self.ledgers.get_mut(ledger).unwrap_or_else(|| panic!("no ledger {ledger}")).transfer(from, to, amount);
    }
    pub fn declare_stock(&mut self, ledger: &str, account: &str, value: f64) {
        self.ledgers.get_mut(ledger).unwrap_or_else(|| panic!("no ledger {ledger}")).declare_stock(account, value);
    }
    /// Record a named scalar at the current time, for scoreboards and the viewer.
    pub fn report(&mut self, series: &str, unit: &str, value: f64) {
        let s = self.series.entry(series.to_string()).or_default();
        if s.unit.is_empty() { s.unit = unit.to_string(); }
        s.points.push((self.now, value));
    }
    /// Record a whole field at the current time, for maps. Frames are kept in memory
    /// until the run writes them, so modules choose the cadence with care.
    pub fn field(&mut self, name: &str, unit: &str, grid: &LatLonGrid, data: Vec<f32>) {
        assert_eq!(data.len(), grid.nx * grid.ny, "field {name}: data does not match grid");
        let f = self.fields.entry(name.to_string()).or_insert_with(|| FieldSeries { unit: unit.to_string(), grid: grid.clone(), frames: Vec::new() });
        assert_eq!(&f.grid, grid, "field {name}: grid changed between frames");
        f.frames.push((self.now, data));
    }
    /// Write a shared field. The first writer owns it; nobody else may write it. The
    /// kernel logs the write (name, version, digest), not the data.
    pub fn world_write(&mut self, name: &str, unit: &str, grid: &LatLonGrid, data: Vec<f32>) {
        let len = data.len();
        let (version, digest) = self.world.write(&self.me, name, unit, grid, data, self.now).unwrap_or_else(|e| panic!("{e}"));
        self.world_writes.push((name.to_string(), version, digest, len));
    }
    /// Read a shared field as last written.
    pub fn world_read(&self, name: &str) -> Option<&WorldField> { self.world.get(name) }
    /// Read a shared field that must be on this module's grid. A mismatch is a manifest
    /// error (two modules given different windows or resolutions) and is reported as one
    /// instead of an index out of bounds later.
    pub fn world_read_on(&self, name: &str, grid: &LatLonGrid) -> Option<&WorldField> {
        let f = self.world.get(name)?;
        if &f.grid != grid {
            panic!("{} reads field {name} written by {} on a {}x{} grid at {:.4} arcsec, but runs on a {}x{} grid at {:.4} arcsec; give every module one window and resolution (a [place] table in the manifest)",
                self.me, f.owner, f.grid.nx, f.grid.ny, f.grid.dlon * 3600.0, grid.nx, grid.ny, grid.dlon * 3600.0);
        }
        Some(f)
    }
    pub fn rng(&mut self) -> &mut Rng { self.rng }
}
