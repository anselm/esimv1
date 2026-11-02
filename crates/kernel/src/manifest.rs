//! The manifest is the whole arrangement: run window, seed, closing cadence, and the
//! modules with their parameters. Heavy code lives in modules; this is a document.

use crate::time::{Duration, Time};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub run: Run,
    /// Optional `[place]` table: a window, resolution and gauge stated once and inherited
    /// by every module that does not set the same keys itself.
    #[serde(default)]
    pub place: Option<toml::Value>,
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
        let mut m: Manifest = toml::from_str(text).map_err(|e| format!("manifest: {e}"))?;
        m.fold_place();
        Ok(m)
    }
    /// Read a manifest file, expanding `include = ["other.toml", ...]` (paths relative to
    /// the file) into one document first. See `expand_includes`.
    pub fn load(path: &std::path::Path) -> Result<Manifest, String> {
        Manifest::parse(&Manifest::resolve_text(path)?)
    }
    /// The resolved TOML text of a manifest file with its includes folded in. This is what
    /// a run folder keeps as `manifest.toml`, so a run never depends on another file.
    pub fn resolve_text(path: &std::path::Path) -> Result<String, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let doc: toml::Value = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let module_includes = doc.get("module").and_then(|m| m.as_array()).map(|a| a.iter().any(|m| m.get("include").is_some())).unwrap_or(false);
        if doc.get("include").is_none() && !module_includes { return Ok(text); }
        let merged = expand_includes(doc, path.parent().unwrap_or(std::path::Path::new(".")), 0)?;
        toml::to_string_pretty(&merged).map_err(|e| e.to_string())
    }
    /// Copy every key of `[place]` into each module's params where the module has not
    /// set that key. Done at parse time so the check and the build see the same params.
    pub fn fold_place(&mut self) {
        let Some(toml::Value::Table(place)) = &self.place else { return };
        for e in &mut self.modules {
            if let toml::Value::Table(t) = &mut e.params {
                for (k, v) in place { t.entry(k.clone()).or_insert_with(|| v.clone()); }
            }
        }
    }
    pub fn start(&self) -> Result<Time, String> { Time::parse(&self.run.start) }
    pub fn end(&self) -> Result<Time, String> { Time::parse(&self.run.end) }
    pub fn close_every(&self) -> Result<Duration, String> { parse_duration(&self.run.close_every) }
    pub fn snapshot_every(&self) -> Result<Option<Duration>, String> {
        self.run.snapshot_every.as_deref().map(parse_duration).transpose()
    }
}

/// Nested manifests. A document may name `include = ["a.toml", "b.toml"]`; each included
/// file (itself expandable) contributes its `[place]`, `[audit]` and `[[module]]` entries
/// first, in order, then the including document's own. A `[[module]]` entry that is only
/// `include = "file"` splices that file's modules in at that position instead, which
/// matters because module order is page order. The including document's `[run]`
/// stands; an included `[run]` is used only when the includer has none. A module whose
/// name repeats an included one replaces it, which is how a short manifest restates one
/// module's parameters on top of a bundle. Keys of `[place]` and `[audit]` merge, the
/// includer's winning.
pub fn expand_includes(doc: toml::Value, dir: &std::path::Path, depth: usize) -> Result<toml::Value, String> {
    if depth > 8 { return Err("include: nested deeper than 8".into()); }
    let mut table = match doc { toml::Value::Table(t) => t, _ => return Err("manifest is not a table".into()) };
    let includes: Vec<String> = match table.remove("include") {
        Some(toml::Value::Array(a)) => a.iter().filter_map(|v| v.as_str().map(String::from)).collect(),
        Some(toml::Value::String(s)) => vec![s],
        Some(_) => return Err("include must be a string or an array of strings".into()),
        None => Vec::new(),
    };
    let mut modules: Vec<toml::Value> = Vec::new();
    let mut place = toml::value::Table::new();
    let mut audit = toml::value::Table::new();
    let mut run: Option<toml::Value> = None;
    for inc in includes {
        let p = dir.join(&inc);
        let text = std::fs::read_to_string(&p).map_err(|e| format!("include {}: {e}", p.display()))?;
        let d: toml::Value = toml::from_str(&text).map_err(|e| format!("include {}: {e}", p.display()))?;
        let d = expand_includes(d, p.parent().unwrap_or(dir), depth + 1)?;
        let t = d.as_table().unwrap();
        if let Some(toml::Value::Array(ms)) = t.get("module") { for m in ms { upsert_module(&mut modules, m.clone()); } }
        if let Some(toml::Value::Table(pl)) = t.get("place") { for (k, v) in pl { place.insert(k.clone(), v.clone()); } }
        if let Some(toml::Value::Table(au)) = t.get("audit") { for (k, v) in au { audit.insert(k.clone(), v.clone()); } }
        if run.is_none() { run = t.get("run").cloned(); }
    }
    // the includer's own modules; a module entry that is only `include = "file"` splices
    // that file's modules in at its position, so a manifest can order itself around a bundle
    if let Some(toml::Value::Array(ms)) = table.remove("module") {
        for m in ms {
            if let Some(inc) = m.get("include").and_then(|v| v.as_str()) {
                let p = dir.join(inc);
                let text = std::fs::read_to_string(&p).map_err(|e| format!("include {}: {e}", p.display()))?;
                let d: toml::Value = toml::from_str(&text).map_err(|e| format!("include {}: {e}", p.display()))?;
                let d = expand_includes(d, p.parent().unwrap_or(dir), depth + 1)?;
                let t = d.as_table().unwrap();
                if let Some(toml::Value::Array(inner)) = t.get("module") { for im in inner { upsert_module(&mut modules, im.clone()); } }
                if let Some(toml::Value::Table(pl)) = t.get("place") { for (k, v) in pl { place.entry(k.clone()).or_insert(v.clone()); } }
                if let Some(toml::Value::Table(au)) = t.get("audit") { for (k, v) in au { audit.entry(k.clone()).or_insert(v.clone()); } }
                if run.is_none() { run = t.get("run").cloned(); }
                continue;
            }
            upsert_module(&mut modules, m);
        }
    }
    if let Some(toml::Value::Table(pl)) = table.remove("place") { for (k, v) in pl { place.insert(k, v); } }
    if let Some(toml::Value::Table(au)) = table.remove("audit") { for (k, v) in au { audit.insert(k, v); } }
    if let Some(r) = table.remove("run") { run = Some(r); }
    if let Some(r) = run { table.insert("run".into(), r); }
    if !place.is_empty() { table.insert("place".into(), toml::Value::Table(place)); }
    if !audit.is_empty() { table.insert("audit".into(), toml::Value::Table(audit)); }
    table.insert("module".into(), toml::Value::Array(modules));
    Ok(toml::Value::Table(table))
}

fn upsert_module(list: &mut Vec<toml::Value>, m: toml::Value) {
    let name = m.get("name").and_then(|n| n.as_str()).map(String::from);
    if let Some(n) = &name {
        if let Some(slot) = list.iter_mut().find(|e| e.get("name").and_then(|x| x.as_str()) == Some(n.as_str())) { *slot = m; return; }
    }
    list.push(m);
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
