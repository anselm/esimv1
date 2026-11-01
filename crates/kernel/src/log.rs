//! Append-only event log with a running digest. The digest of two runs being equal is
//! the test of determinism; the log itself is the output format.

use crate::event::Event;
use crate::hash;
use std::io::Write;

#[derive(Default)]
pub struct EventLog {
    pub events: Vec<Event>,
    digest: u64,
}

impl EventLog {
    pub fn new() -> EventLog { EventLog { events: Vec::new(), digest: hash::OFFSET } }
    pub fn append(&mut self, ev: &Event) {
        let line = serde_json::to_string(ev).expect("event serializes");
        self.digest = hash::fnv1a_extend(self.digest, line.as_bytes());
        self.digest = hash::fnv1a_extend(self.digest, b"\n");
        self.events.push(ev.clone());
    }
    pub fn digest(&self) -> u64 { self.digest }
    pub fn digest_hex(&self) -> String { format!("{:016x}", self.digest) }
    pub fn len(&self) -> usize { self.events.len() }
    pub fn is_empty(&self) -> bool { self.events.is_empty() }
    pub fn write_jsonl<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        for ev in &self.events {
            serde_json::to_writer(&mut w, ev)?;
            w.write_all(b"\n")?;
        }
        Ok(())
    }
}
