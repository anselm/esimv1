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

pub use event::{Event, Priority};
pub use kernel::{Kernel, RunOutput};
pub use ledger::{Ledger, LedgerReport};
pub use manifest::Manifest;
pub use module::{Ctx, Module, ModuleFactory};
pub use time::{Duration, Time};
