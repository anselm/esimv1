//! Events: the only way state moves between modules. Ordered by (time, priority,
//! sequence) so a run is a function of its manifest and seed, never of thread timing.

use crate::time::Time;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Lower runs first within the same instant. Kernel housekeeping uses the extremes.
pub type Priority = i16;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub t: Time,
    pub priority: Priority,
    pub seq: u64,
    /// Module that scheduled it, or "kernel".
    pub src: String,
    /// Addressed module, or None to broadcast to the topic's subscribers.
    pub to: Option<String>,
    pub topic: String,
    #[serde(default)]
    pub data: serde_json::Value,
}

impl Event {
    pub fn key(&self) -> (Time, Priority, u64) { (self.t, self.priority, self.seq) }
}

/// Heap wrapper: BinaryHeap is a max-heap, we want the smallest key first.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Queued(pub Event);

impl PartialEq for Queued { fn eq(&self, o: &Self) -> bool { self.0.key() == o.0.key() } }
impl Eq for Queued {}
impl PartialOrd for Queued { fn partial_cmp(&self, o: &Self) -> Option<Ordering> { Some(self.cmp(o)) } }
impl Ord for Queued { fn cmp(&self, o: &Self) -> Ordering { o.0.key().cmp(&self.0.key()) } }
