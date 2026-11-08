//! The entity form of a manifest: the object form that RFC 0001 (`docs/rfc/0001-manifest.md`)
//! proposes as the transport between engines. A manifest is a list of entities, each an id
//! and a bag of components that are plain data. Presence is type: an entity with a `run`
//! component is the run, with a `kind` a module instance, with a `place` the place, with an
//! `audit` the scoreboards, with a `variant` a variant. A component nobody here reads is
//! carried and reported, never rejected.
//!
//! This file converts between that form and the TOML-shaped `Manifest` the kernel has always
//! used, writes the canonical text (RFC 8785 style: sorted keys, no whitespace, ES number
//! formatting) and computes the manifest id, the sha256 of that text.

use crate::manifest::{Manifest, ModuleEntry, Run, Variant};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

pub const VERSION: &str = "manifest/1";

/// Where a resolved manifest came from: the `from` block. Provenance, not a dependency; a
/// manifest without it is still complete.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Source {
    /// "toml", "json" or "js"
    pub language: String,
    /// the file as it was named
    pub entry: String,
    /// `sha256:<hex>` of the entry file's bytes
    pub hash: String,
    /// every other file folded in, by path, with its hash
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub sources: std::collections::BTreeMap<String, String>,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("sha256:{:x}", h.finalize())
}

fn valid_id(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some(a) if a.is_ascii_lowercase()) && c.all(|x| x.is_ascii_lowercase() || x.is_ascii_digit() || x == '_' || x == '-')
}

impl Manifest {
    /// Read the entity form. `doc` is the envelope: `{ "manifest": "manifest/1", "entities": [...] }`.
    pub fn from_entities(doc: &Value) -> Result<Manifest, String> {
        let env = doc.as_object().ok_or("manifest: not an object")?;
        match env.get("manifest").and_then(|v| v.as_str()) {
            Some(VERSION) => {}
            Some(other) => return Err(format!("manifest: version {other} is not {VERSION}")),
            None => return Err(format!("manifest: no \"manifest\": \"{VERSION}\" at the top")),
        }
        let entities = env.get("entities").and_then(|v| v.as_array()).ok_or("manifest: no entities array")?;
        let mut run: Option<Run> = None;
        let mut place: Option<toml::Value> = None;
        let mut audit: Option<toml::Value> = None;
        let mut modules: Vec<ModuleEntry> = Vec::new();
        let mut variants: Vec<Variant> = Vec::new();
        let mut unknown: Vec<(String, String)> = Vec::new();
        for (n, e) in entities.iter().enumerate() {
            let obj = e.as_object().ok_or_else(|| format!("entity {n}: not an object"))?;
            let id = obj.get("id").and_then(|v| v.as_str()).ok_or_else(|| format!("entity {n}: no id"))?.to_string();
            if !valid_id(&id) { return Err(format!("entity {id}: an id is lower-case letters, digits, _ and -, starting with a letter")); }
            let known = |k: &str| matches!(k, "id" | "run" | "place" | "kind" | "params" | "ref" | "audit" | "variant" | "notes");
            for k in obj.keys() { if !known(k) { unknown.push((id.clone(), k.clone())); } }
            if let Some(r) = obj.get("run") {
                let r = r.as_object().ok_or_else(|| format!("{id}: run is not an object"))?;
                let s = |k: &str| r.get(k).and_then(|v| v.as_str()).map(String::from);
                let start = s("start").ok_or_else(|| format!("{id}: run.start is required"))?;
                let end = s("end").ok_or_else(|| format!("{id}: run.end is required"))?;
                run = Some(Run {
                    name: s("name").unwrap_or_else(|| id.clone()), start, end,
                    seed: r.get("seed").and_then(|v| v.as_u64()).unwrap_or(0),
                    close_every: s("close_every").unwrap_or_else(|| "1d".into()),
                    snapshot_every: s("snapshot_every"),
                    notes: s("notes").unwrap_or_default(),
                });
            }
            if let Some(p) = obj.get("place") { place = Some(json_to_toml(p).map_err(|e| format!("{id}: place: {e}"))?); }
            if let Some(a) = obj.get("audit") { audit = Some(json_to_toml(a).map_err(|e| format!("{id}: audit: {e}"))?); }
            if let Some(k) = obj.get("kind") {
                let kind = k.as_str().ok_or_else(|| format!("{id}: kind is not a string"))?.to_string();
                let mut params = match obj.get("params") {
                    Some(p) => json_to_toml(p).map_err(|e| format!("{id}: params: {e}"))?,
                    None => toml::Value::Table(Default::default()),
                };
                if !params.is_table() { return Err(format!("{id}: params is not an object")); }
                if let Some(r) = obj.get("ref") {
                    params.as_table_mut().unwrap().insert("ref".into(), json_to_toml(r).map_err(|e| format!("{id}: ref: {e}"))?);
                }
                // replacement by id: a later entity with an earlier id replaces it
                if let Some(slot) = modules.iter_mut().find(|m| m.name == id) { slot.kind = kind; slot.params = params; }
                else { modules.push(ModuleEntry { name: id.clone(), kind, params }); }
            } else if obj.contains_key("params") || obj.contains_key("ref") {
                return Err(format!("{id}: params and ref need a kind"));
            }
            if let Some(v) = obj.get("variant") {
                let v = v.as_object().ok_or_else(|| format!("{id}: variant is not an object"))?;
                let title = v.get("title").and_then(|x| x.as_str()).unwrap_or("").to_string();
                let notes = v.get("notes").and_then(|x| x.as_str()).unwrap_or("").to_string();
                let mut out: Vec<Variant> = Vec::new();
                if let Some(set) = v.get("set") {
                    let set = set.as_object().ok_or_else(|| format!("{id}: variant.set is not an object"))?;
                    let mut t = toml::value::Table::new();
                    for (k, val) in set { t.insert(k.clone(), json_to_toml(val).map_err(|e| format!("{id}: variant.set.{k}: {e}"))?); }
                    out.push(Variant { name: id.clone(), title: title.clone(), notes: notes.clone(), set: t });
                } else if let Some(range) = v.get("range") {
                    // one loop in the transport: a range unrolls to one variant per member
                    let range = range.as_object().ok_or_else(|| format!("{id}: variant.range is not an object"))?;
                    let mut axes: Vec<(String, Vec<Value>)> = Vec::new();
                    for (k, spec) in range {
                        let spec = spec.as_object().ok_or_else(|| format!("{id}: range.{k} is not an object"))?;
                        let values: Vec<Value> = if let Some(vs) = spec.get("values").and_then(|x| x.as_array()) { vs.clone() } else {
                            let from = spec.get("from").and_then(|x| x.as_f64()).ok_or_else(|| format!("{id}: range.{k} needs from, to and steps, or values"))?;
                            let to = spec.get("to").and_then(|x| x.as_f64()).ok_or_else(|| format!("{id}: range.{k} needs to"))?;
                            let steps = spec.get("steps").and_then(|x| x.as_u64()).ok_or_else(|| format!("{id}: range.{k} needs steps"))? as usize;
                            if steps < 1 { return Err(format!("{id}: range.{k}: steps must be at least 1")); }
                            let integral = spec.get("from").map(|x| x.is_i64() || x.is_u64()).unwrap_or(false) && spec.get("to").map(|x| x.is_i64() || x.is_u64()).unwrap_or(false);
                            (0..steps).map(|i| {
                                let x = if steps == 1 { from } else { from + (to - from) * i as f64 / (steps - 1) as f64 };
                                if integral && x.fract() == 0.0 { json!(x as i64) } else { json!(x) }
                            }).collect()
                        };
                        axes.push((k.clone(), values));
                    }
                    if axes.is_empty() { return Err(format!("{id}: variant.range is empty")); }
                    // the cartesian product of the axes, first axis slowest
                    let mut combos: Vec<Vec<(String, Value)>> = vec![Vec::new()];
                    for (k, values) in &axes {
                        let mut next = Vec::new();
                        for c in &combos { for val in values { let mut c2 = c.clone(); c2.push((k.clone(), val.clone())); next.push(c2); } }
                        combos = next;
                    }
                    let width = combos.len().to_string().len();
                    for (i, c) in combos.iter().enumerate() {
                        let mut t = toml::value::Table::new();
                        for (k, val) in c { t.insert(k.clone(), json_to_toml(val).map_err(|e| format!("{id}: range.{k}: {e}"))?); }
                        let member = format!("{id}-{:0width$}", i + 1, width = width);
                        let value_text: Vec<String> = c.iter().map(|(k, val)| format!("{k} = {val}")).collect();
                        out.push(Variant { name: member, title: if title.is_empty() { value_text.join(", ") } else { format!("{title} ({})", value_text.join(", ")) }, notes: notes.clone(), set: t });
                    }
                } else {
                    return Err(format!("{id}: a variant needs set or range"));
                }
                for v in out {
                    if let Some(slot) = variants.iter_mut().find(|x| x.name == v.name) { *slot = v; } else { variants.push(v); }
                }
            }
        }
        let run = run.ok_or("manifest: no entity with a run component")?;
        let mut m = Manifest { run, place, modules, audit, variants, vocabulary: env.get("vocabulary").and_then(|v| v.as_str()).map(String::from), from: None, unknown };
        m.fold_place();
        Ok(m)
    }

    /// The entity form: the envelope with every entity, in page order. Includes are already
    /// folded, place is already copied into params, so this is a resolved manifest.
    pub fn to_entities(&self) -> Value {
        let mut entities: Vec<Value> = Vec::new();
        let mut run = Map::new();
        run.insert("name".into(), json!(self.run.name));
        run.insert("start".into(), json!(self.run.start));
        run.insert("end".into(), json!(self.run.end));
        run.insert("seed".into(), json!(self.run.seed));
        run.insert("close_every".into(), json!(self.run.close_every));
        if let Some(s) = &self.run.snapshot_every { run.insert("snapshot_every".into(), json!(s)); }
        if !self.run.notes.is_empty() { run.insert("notes".into(), json!(self.run.notes)); }
        entities.push(json!({ "id": "run", "run": Value::Object(run) }));
        if let Some(p) = &self.place { entities.push(json!({ "id": "place", "place": toml_to_json(p) })); }
        for m in &self.modules {
            let mut e = Map::new();
            e.insert("id".into(), json!(m.name));
            e.insert("kind".into(), json!(m.kind));
            let mut params = toml_to_json(&m.params);
            if let Some(obj) = params.as_object_mut() {
                // `kind` and `seed` are folded into params for the factory, not part of the document
                obj.remove("kind");
                if let Some(r) = obj.remove("ref") { e.insert("ref".into(), r); }
                if !obj.is_empty() { e.insert("params".into(), params); }
            } else { e.insert("params".into(), params); }
            entities.push(Value::Object(e));
        }
        if let Some(a) = &self.audit { entities.push(json!({ "id": "audit", "audit": toml_to_json(a) })); }
        for v in &self.variants {
            let mut set = Map::new();
            for (k, val) in &v.set { set.insert(k.clone(), toml_to_json(val)); }
            let mut body = Map::new();
            if !v.title.is_empty() { body.insert("title".into(), json!(v.title)); }
            if !v.notes.is_empty() { body.insert("notes".into(), json!(v.notes)); }
            body.insert("set".into(), Value::Object(set));
            entities.push(json!({ "id": v.name, "variant": Value::Object(body) }));
        }
        let mut env = Map::new();
        env.insert("manifest".into(), json!(VERSION));
        if let Some(v) = &self.vocabulary { env.insert("vocabulary".into(), json!(v)); }
        env.insert("entities".into(), Value::Array(entities));
        if let Some(f) = &self.from { env.insert("from".into(), serde_json::to_value(f).unwrap()); }
        Value::Object(env)
    }

    /// The canonical text: the entity form without `from`, keys sorted, no whitespace,
    /// numbers as ECMAScript prints them. Two manifests with the same canonical text are the
    /// same manifest whatever wrote them.
    pub fn canonical(&self) -> String {
        let mut v = self.to_entities();
        if let Some(o) = v.as_object_mut() { o.remove("from"); }
        canonical_json(&v)
    }

    /// `sha256:<hex>` of the canonical text.
    pub fn id(&self) -> String { sha256_hex(self.canonical().as_bytes()) }

    /// The TOML text of this manifest, for the tools and pages that read that form.
    pub fn to_toml_text(&self) -> Result<String, String> {
        let mut m = self.clone();
        for e in &mut m.modules {
            if let toml::Value::Table(t) = &mut e.params { t.remove("kind"); }
        }
        toml::to_string_pretty(&m).map_err(|e| format!("manifest to toml: {e}"))
    }
}

/// A TOML value as JSON. Datetimes become strings; everything else maps one to one.
pub fn toml_to_json(v: &toml::Value) -> Value {
    match v {
        toml::Value::String(s) => json!(s),
        toml::Value::Integer(i) => json!(i),
        toml::Value::Float(f) => json!(f),
        toml::Value::Boolean(b) => json!(b),
        toml::Value::Datetime(d) => json!(d.to_string()),
        toml::Value::Array(a) => Value::Array(a.iter().map(toml_to_json).collect()),
        toml::Value::Table(t) => Value::Object(t.iter().map(|(k, v)| (k.clone(), toml_to_json(v))).collect()),
    }
}

/// A JSON value as TOML. `null` has no TOML form and is refused; a number that is a whole
/// number within i64 is an integer, any other an f64.
pub fn json_to_toml(v: &Value) -> Result<toml::Value, String> {
    Ok(match v {
        Value::Null => return Err("null has no place in a manifest; leave the key out".into()),
        Value::Bool(b) => toml::Value::Boolean(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() { toml::Value::Integer(i) }
            else if let Some(f) = n.as_f64() { toml::Value::Float(f) }
            else { return Err(format!("number {n} is out of range")); }
        }
        Value::String(s) => toml::Value::String(s.clone()),
        Value::Array(a) => toml::Value::Array(a.iter().map(json_to_toml).collect::<Result<_, _>>()?),
        Value::Object(o) => toml::Value::Table(o.iter().map(|(k, v)| json_to_toml(v).map(|t| (k.clone(), t))).collect::<Result<_, _>>()?),
    })
}

/// RFC 8785 (JSON Canonicalization Scheme) as far as this document needs it: object keys in
/// code-point order, no whitespace, strings escaped minimally, numbers as ECMAScript's
/// Number::toString prints them.
pub fn canonical_json(v: &Value) -> String {
    let mut out = String::new();
    write_canonical(v, &mut out);
    out
}

fn write_canonical(v: &Value, out: &mut String) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() { out.push_str(&i.to_string()); }
            else if let Some(u) = n.as_u64() { out.push_str(&u.to_string()); }
            else { out.push_str(&es_number(n.as_f64().unwrap_or(0.0))); }
        }
        Value::String(s) => write_string(s, out),
        Value::Array(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() { if i > 0 { out.push(','); } write_canonical(x, out); }
            out.push(']');
        }
        Value::Object(o) => {
            // serde_json's Map is ordered by key already; sort by UTF-16 code units as JCS asks,
            // which differs from byte order only outside the basic multilingual plane
            let mut keys: Vec<&String> = o.keys().collect();
            keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 { out.push(','); }
                write_string(k, out);
                out.push(':');
                write_canonical(&o[*k], out);
            }
            out.push('}');
        }
    }
}

fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// ECMAScript Number::toString for a finite f64: the shortest digits that round trip, laid
/// out by the standard's rules (plain up to 1e21, fixed down to 1e-6, exponent beyond).
pub fn es_number(x: f64) -> String {
    if x == 0.0 || !x.is_finite() { return if x.is_finite() { "0".into() } else { "null".into() }; }
    if x < 0.0 { return format!("-{}", es_number(-x)); }
    // Rust prints the shortest round-trip digits in `{:e}` form: d[.ddd]e[-]n
    let sci = format!("{:e}", x);
    let (mant, exp) = sci.split_once('e').unwrap();
    let e: i32 = exp.parse().unwrap();
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    let digits = digits.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let k = digits.len() as i32;
    let n = e + 1;
    if k <= n && n <= 21 {
        format!("{digits}{}", "0".repeat((n - k) as usize))
    } else if 0 < n && n <= 21 {
        format!("{}.{}", &digits[..n as usize], &digits[n as usize..])
    } else if -6 < n && n <= 0 {
        format!("0.{}{digits}", "0".repeat((-n) as usize))
    } else {
        let sign = if n - 1 < 0 { "-" } else { "+" };
        if k == 1 { format!("{digits}e{sign}{}", (n - 1).abs()) }
        else { format!("{}.{}e{sign}{}", &digits[..1], &digits[1..], (n - 1).abs()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_print_as_ecmascript_does() {
        for (x, s) in [(1.0, "1"), (0.5, "0.5"), (100.0, "100"), (1e21, "1e+21"), (1e-7, "1e-7"), (0.000001, "0.000001"),
                       (123456789012345680000.0, "123456789012345680000"), (0.1 + 0.2, "0.30000000000000004"), (-1.5, "-1.5"),
                       (1.5e300, "1.5e+300"), (2.5e-10, "2.5e-10"), (42.3, "42.3"), (-94.2, "-94.2"), (1e20, "100000000000000000000")] {
            assert_eq!(es_number(x), s, "{x}");
        }
    }

    #[test]
    fn canonical_sorts_keys_and_escapes() {
        let v = json!({ "b": [1, 2.5, "x\"y\n"], "a": { "z": true, "y": null } });
        assert_eq!(canonical_json(&v), r#"{"a":{"y":null,"z":true},"b":[1,2.5,"x\"y\n"]}"#);
    }

    #[test]
    fn a_manifest_round_trips_through_entities() {
        let m = Manifest::parse(r#"
[run]
name = "rt"
start = "2026-01-01"
end = "2026-01-02"
seed = 3
[place]
lat_s = 42.3
[[module]]
name = "a"
kind = "k.one"
[module.params]
x = 1
[[module]]
name = "b"
kind = "k.two"
[audit]
scores = [["s", "o"]]
"#).unwrap();
        let doc = m.to_entities();
        let back = Manifest::from_entities(&doc).unwrap();
        assert_eq!(back.to_entities(), doc);
        assert_eq!(back.id(), m.id());
        assert_eq!(back.modules[1].params.get("lat_s").and_then(|v| v.as_float()), Some(42.3));
        assert!(doc["entities"][2]["params"]["lat_s"].is_number(), "place folded into params in the resolved form");
    }

    #[test]
    fn replacement_by_id_and_ranges() {
        let doc = json!({ "manifest": "manifest/1", "entities": [
            { "id": "run", "run": { "start": "2026-01-01", "end": "2026-01-02" } },
            { "id": "a", "kind": "k.one", "params": { "x": 1 } },
            { "id": "a", "kind": "k.one", "params": { "x": 2 } },
            { "id": "seeds", "variant": { "title": "Three members", "range": { "run.seed": { "from": 1, "to": 3, "steps": 3 } } } },
            { "id": "odd", "code": { "language": "js" } }
        ]});
        let m = Manifest::from_entities(&doc).unwrap();
        assert_eq!(m.modules.len(), 1);
        assert_eq!(m.modules[0].params.get("x").and_then(|v| v.as_integer()), Some(2));
        assert_eq!(m.variants.len(), 3);
        assert_eq!(m.variants[2].name, "seeds-3");
        assert_eq!(m.variants[2].set.get("run.seed").and_then(|v| v.as_integer()), Some(3));
        assert_eq!(m.unknown, vec![("odd".to_string(), "code".to_string())]);
        assert!(Manifest::from_entities(&json!({ "manifest": "manifest/2", "entities": [] })).is_err());
        assert!(Manifest::from_entities(&json!({ "manifest": "manifest/1", "entities": [ { "id": "run", "run": { "start": "2026-01-01", "end": "2026-01-02" } }, { "id": "x", "params": {} } ] })).is_err());
    }
}
