//! The manifest is the whole arrangement: run window, seed, closing cadence, and the
//! modules with their parameters. Heavy code lives in modules; this is a document.

use crate::time::{Duration, Time};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub run: Run,
    #[serde(default, rename = "module")]
    pub modules: Vec<ModuleEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Run {
    pub name: String,
    pub start: String,
    pub end: String,
    #[serde(default)]
    pub seed: u64,
    /// Ledger closing period, e.g. "1d", "30d". Default one day.
    #[serde(default = "default_close")]
    pub close_every: String,
    /// Snapshot period, e.g. "7d". Absent means no snapshots.
    #[serde(default)]
    pub snapshot_every: Option<String>,
    #[serde(default)]
    pub notes: String,
}

fn default_close() -> String { "1d".to_string() }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleEntry {
    pub name: String,
    pub kind: String,
    #[serde(default = "empty_table")]
    pub params: toml::Value,
}

fn empty_table() -> toml::Value { toml::Value::Table(Default::default()) }

impl Manifest {
    pub fn parse(text: &str) -> Result<Manifest, String> {
        toml::from_str(text).map_err(|e| format!("manifest: {e}"))
    }
    pub fn start(&self) -> Result<Time, String> { Time::parse(&self.run.start) }
    pub fn end(&self) -> Result<Time, String> { Time::parse(&self.run.end) }
    pub fn close_every(&self) -> Result<Duration, String> { parse_duration(&self.run.close_every) }
    pub fn snapshot_every(&self) -> Result<Option<Duration>, String> {
        self.run.snapshot_every.as_deref().map(parse_duration).transpose()
    }
}

/// "30s", "15m", "6h", "1d", or a bare number of seconds.
pub fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    let (num, unit) = s.split_at(s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len()));
    let n: i64 = num.parse().map_err(|_| format!("duration: {s}"))?;
    let mult = match unit {
        "" | "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86_400,
        "w" => 7 * 86_400,
        _ => return Err(format!("duration unit: {s}")),
    };
    Ok(Duration(n * mult))
}

/// Read a numeric parameter with a default.
pub fn param_f64(p: &toml::Value, key: &str, default: f64) -> f64 {
    match p.get(key) {
        Some(toml::Value::Float(f)) => *f,
        Some(toml::Value::Integer(i)) => *i as f64,
        _ => default,
    }
}
pub fn param_str<'a>(p: &'a toml::Value, key: &str, default: &'a str) -> &'a str {
    p.get(key).and_then(|v| v.as_str()).unwrap_or(default)
}
pub fn param_duration(p: &toml::Value, key: &str, default: &str) -> Result<Duration, String> {
    parse_duration(param_str(p, key, default))
}
