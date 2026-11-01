//! The event tables: calendar and hazards, read from data/events/*.toml. Modules that
//! implement a hazard or honour a calendar date look their row up here, so the run's
//! provenance for "why did this happen" is the table row, not a constant in code.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EventRow {
    pub id: String,
    pub title: String,
    #[serde(default)] pub category: String,
    #[serde(default)] pub date: Option<String>,
    #[serde(default)] pub window: Option<Vec<String>>,
    #[serde(default)] pub rate_per_year: Option<f64>,
    #[serde(default)] pub depends_on: Vec<String>,
    #[serde(default)] pub consequence: String,
    #[serde(default)] pub module: Option<String>,
    #[serde(default)] pub valence: String,
    #[serde(default)] pub confidence: String,
    #[serde(default)] pub source: String,
    #[serde(default)] pub known_since: String,
    #[serde(default)] pub setting: Option<String>,
    #[serde(default)] pub notes: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EventTable {
    #[serde(default, rename = "event")]
    pub events: Vec<EventRow>,
}

impl EventTable {
    pub fn load(path: &Path) -> Result<EventTable, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        toml::from_str(&text).map_err(|e| format!("event table {}: {e}", path.display()))
    }
    pub fn get(&self, id: &str) -> Option<&EventRow> { self.events.iter().find(|e| e.id == id) }
    /// Rows known by a date (YYYY-MM-DD), for hindcasts with honest knowledge.
    pub fn known_by<'a>(&'a self, date: &'a str) -> impl Iterator<Item = &'a EventRow> + 'a {
        self.events.iter().filter(move |e| e.known_since.is_empty() || e.known_since.as_str() <= date)
    }
}
