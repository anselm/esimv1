//! esim-kernel: the coupler.
//!
//! One virtual clock. Events ordered by (time, priority, sequence). Ledgers that must
//! close. An append-only log that replays. A manifest that is the whole arrangement.
//! Modules are the only things that know any science.

pub mod time;
pub mod event;
pub mod rng;
pub mod ledger;
pub mod log;
pub mod module;
pub mod manifest;
pub mod grid;
pub mod kernel;
pub mod hash;
pub mod world;
pub mod describe;

pub use event::{Event, Priority};
pub use kernel::{FieldMeta, Kernel, RunOutput};
pub use grid::{Grid, LatLonGrid, LocalTile};
pub use ledger::{Ledger, LedgerReport};
pub use manifest::Manifest;
pub use module::{Ctx, FieldSeries, Module, ModuleFactory};
pub use time::{Duration, Time};
pub use world::{World, WorldField};
pub use describe::{check, CheckReport, Describe, Describer};
