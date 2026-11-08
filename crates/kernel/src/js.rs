//! JavaScript manifests: the source form. A manifest may be a JavaScript module whose
//! exports are the entities (the orbital loader's rule: the export name is the id, an
//! exported array is spliced in, and `export default` may be the whole list or the
//! envelope). The program runs on QuickJS with nothing but the language: no clock
//! (`Date` is removed), no randomness (`Math.random` throws), no network, no process,
//! and `import` only of files beside the entry, each hashed into the `from` block. Given
//! the same files it produces the same document, which is what makes the object form,
//! not the program, the thing two engines compare.
//!
//! Functions do not survive: the exports are read through JSON.stringify, so a component
//! is data or it is dropped, as RFC 0001 requires.

use rquickjs::loader::{ImportAttributes, Loader, Resolver};
use rquickjs::{Context, Ctx, Module, Runtime, Value};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const PRELUDE: &str = r#"
delete globalThis.Date;
Object.defineProperty(Math, "random", { value: function () { throw new Error("Math.random is not available in a manifest: randomness is the run's seed"); }, writable: false, configurable: false });
Object.freeze(Math);
"#;
const TIME_BUDGET_MS: u128 = 10_000;
const MEMORY_LIMIT: usize = 256 << 20;

/// Imports resolve only to files under the entry's directory, by relative path.
struct Hermetic { root: PathBuf, sources: Rc<RefCell<BTreeMap<String, String>>> }

impl Resolver for Hermetic {
    fn resolve<'js>(&mut self, ctx: &Ctx<'js>, base: &str, name: &str, _attributes: Option<ImportAttributes<'js>>) -> rquickjs::Result<String> {
        if !(name.starts_with("./") || name.starts_with("../")) {
            return Err(rquickjs::Exception::throw_message(ctx, &format!("import \"{name}\": a manifest imports only files beside it, by relative path (./ or ../); no packages, no URLs")));
        }
        let base_dir = Path::new(base).parent().unwrap_or(&self.root).to_path_buf();
        let joined = base_dir.join(name);
        let full = joined.canonicalize().map_err(|e| rquickjs::Exception::throw_message(ctx, &format!("import \"{name}\" from {base}: {e}")))?;
        if !full.starts_with(&self.root) {
            return Err(rquickjs::Exception::throw_message(ctx, &format!("import \"{name}\" from {base}: {} is outside the manifest's directory {}", full.display(), self.root.display())));
        }
        Ok(full.display().to_string())
    }
}

impl Loader for Hermetic {
    fn load<'js>(&mut self, ctx: &Ctx<'js>, name: &str, _attributes: Option<ImportAttributes<'js>>) -> rquickjs::Result<Module<'js, rquickjs::module::Declared>> {
        let path = Path::new(name);
        let bytes = std::fs::read(path).map_err(|e| rquickjs::Exception::throw_message(ctx, &format!("import {name}: {e}")))?;
        let rel = path.strip_prefix(&self.root).map(|p| p.display().to_string()).unwrap_or_else(|_| name.to_string());
        self.sources.borrow_mut().insert(rel, crate::entities::sha256_hex(&bytes));
        let text = String::from_utf8(bytes).map_err(|e| rquickjs::Exception::throw_message(ctx, &format!("import {name}: {e}")))?;
        let source = match path.extension().and_then(|e| e.to_str()) {
            Some("json") => format!("export default {text};"),
            // a TOML manifest imports as its entity list, includes folded, so a program can
            // start from any existing manifest and restate what it wants
            Some("toml") => {
                let m = crate::Manifest::load(path).map_err(|e| rquickjs::Exception::throw_message(ctx, &format!("import {name}: {e}")))?;
                if let Some(from) = &m.from {
                    for (p, h) in &from.sources {
                        let p = Path::new(p).canonicalize().ok().and_then(|f| f.strip_prefix(&self.root).ok().map(|r| r.display().to_string())).unwrap_or_else(|| p.clone());
                        self.sources.borrow_mut().insert(p, h.clone());
                    }
                }
                format!("export default {};", m.to_entities()["entities"])
            }
            _ => text,
        };
        Module::declare(ctx.clone(), name, source)
    }
}

/// Evaluate a JavaScript manifest. Returns the envelope (`manifest`, `entities`, maybe
/// `vocabulary`) and the files the program read, by path relative to the entry's directory,
/// with their hashes. The entry itself is not in that map; the caller records it as `entry`.
pub fn evaluate(path: &Path) -> Result<(serde_json::Value, BTreeMap<String, String>), String> {
    let full = path.canonicalize().map_err(|e| format!("{}: {e}", path.display()))?;
    let root = full.parent().ok_or("manifest has no directory")?.to_path_buf();
    let text = std::fs::read_to_string(&full).map_err(|e| format!("{}: {e}", path.display()))?;
    let order = export_order(&text);
    let sources = Rc::new(RefCell::new(BTreeMap::new()));

    let rt = Runtime::new().map_err(|e| format!("quickjs: {e}"))?;
    rt.set_memory_limit(MEMORY_LIMIT);
    rt.set_max_stack_size(1 << 20);
    let started = std::time::Instant::now();
    rt.set_interrupt_handler(Some(Box::new(move || started.elapsed().as_millis() > TIME_BUDGET_MS)));
    rt.set_loader(Hermetic { root: root.clone(), sources: sources.clone() }, Hermetic { root: root.clone(), sources: sources.clone() });
    let context = Context::full(&rt).map_err(|e| format!("quickjs: {e}"))?;

    let exports: Vec<(String, serde_json::Value)> = context.with(|ctx| -> Result<Vec<(String, serde_json::Value)>, String> {
        ctx.eval::<(), _>(PRELUDE).map_err(|e| describe(&ctx, e, "prelude"))?;
        let name = full.display().to_string();
        let declared = Module::declare(ctx.clone(), name.as_str(), text.as_str()).map_err(|e| describe(&ctx, e, &name))?;
        let (module, promise) = declared.eval().map_err(|e| describe(&ctx, e, &name))?;
        while ctx.execute_pending_job() {}
        promise.finish::<()>().map_err(|e| describe(&ctx, e, &name))?;
        let ns = module.namespace().map_err(|e| describe(&ctx, e, &name))?;
        let mut found: Vec<(String, serde_json::Value)> = Vec::new();
        for item in ns.props::<String, Value>() {
            let (k, v) = item.map_err(|e| describe(&ctx, e, &name))?;
            let json = match ctx.json_stringify(v).map_err(|e| describe(&ctx, e, &name))? {
                Some(s) => s.to_string().map_err(|e| describe(&ctx, e, &name))?,
                None => continue, // a function or undefined: not data, not carried
            };
            let value: serde_json::Value = serde_json::from_str(&json).map_err(|e| format!("export {k}: {e}"))?;
            found.push((k, value));
        }
        // the module namespace lists exports alphabetically; the source's order is the layout
        found.sort_by_key(|(k, _)| order.iter().position(|o| o == k).unwrap_or(usize::MAX));
        Ok(found)
    })?;
    if started.elapsed().as_millis() > TIME_BUDGET_MS { return Err(format!("{}: the program ran past its {} s budget", path.display(), TIME_BUDGET_MS / 1000)); }

    let (doc, _) = entities_from_exports(exports).map_err(|e| format!("{}: {e}", path.display()))?;
    let read = sources.borrow().clone();
    Ok((doc, read))
}

/// Exports to an envelope. `default` may be the list of entities or the envelope itself; a
/// named export that is an object is an entity whose id is the export name unless it says
/// otherwise; a named export that is an array is spliced; `vocabulary` as a string names the
/// vocabulary; `manifest` as a string must be the version this reader knows.
fn entities_from_exports(exports: Vec<(String, serde_json::Value)>) -> Result<(serde_json::Value, usize), String> {
    use serde_json::Value;
    let mut entities: Vec<Value> = Vec::new();
    let mut vocabulary: Option<String> = None;
    let push = |entities: &mut Vec<Value>, v: Value, what: &str| -> Result<(), String> {
        match v {
            Value::Object(o) => { if !o.contains_key("id") { return Err(format!("{what}: an entity in an array needs an id")); } entities.push(Value::Object(o)); Ok(()) }
            other => Err(format!("{what}: {} is not an entity", short(&other))),
        }
    };
    for (name, value) in exports {
        match (name.as_str(), value) {
            ("manifest", Value::String(s)) => { if s != crate::entities::VERSION { return Err(format!("manifest: version {s} is not {}", crate::entities::VERSION)); } }
            ("vocabulary", Value::String(s)) => vocabulary = Some(s),
            ("default", Value::Array(a)) => { for v in a { push(&mut entities, v, "default")?; } }
            ("default", Value::Object(mut o)) if o.contains_key("entities") => {
                if let Some(Value::String(s)) = o.get("manifest") { if s != crate::entities::VERSION { return Err(format!("manifest: version {s} is not {}", crate::entities::VERSION)); } }
                if let Some(Value::String(s)) = o.get("vocabulary") { vocabulary = Some(s.clone()); }
                match o.remove("entities") { Some(Value::Array(a)) => { for v in a { push(&mut entities, v, "default.entities")?; } } _ => return Err("default.entities is not an array".into()) }
            }
            (_, Value::Array(a)) => { for v in a { push(&mut entities, v, &name)?; } }
            (_, Value::Object(o)) => {
                // the short form: an export named for a singleton component whose value has no
                // component of its own is that component (`export const run = { start, end }`)
                let components = ["run", "place", "audit", "variant", "kind", "params", "ref"];
                let has_component = o.keys().any(|k| components.contains(&k.as_str()));
                let mut o = if !has_component && ["run", "place", "audit"].contains(&name.as_str()) {
                    let mut wrapped = serde_json::Map::new();
                    wrapped.insert(name.clone(), Value::Object(o));
                    wrapped
                } else if !has_component && o.contains_key("set") || !has_component && o.contains_key("range") {
                    let mut wrapped = serde_json::Map::new();
                    wrapped.insert("variant".into(), Value::Object(o));
                    wrapped
                } else { o };
                o.entry("id").or_insert(Value::String(name.clone()));
                entities.push(Value::Object(o));
            }
            (_, other) => return Err(format!("export {name}: {} is not an entity", short(&other))),
        }
    }
    let n = entities.len();
    let mut env = serde_json::Map::new();
    env.insert("manifest".into(), Value::String(crate::entities::VERSION.into()));
    if let Some(v) = vocabulary { env.insert("vocabulary".into(), Value::String(v)); }
    env.insert("entities".into(), Value::Array(entities));
    Ok((Value::Object(env), n))
}

fn short(v: &serde_json::Value) -> String {
    let s = v.to_string();
    if s.len() > 60 { format!("{}…", &s[..60]) } else { s }
}

/// The names a module exports, in the order they appear in its text. The module namespace
/// is alphabetical by the language's rules; the author's order is the page's.
fn export_order(src: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let bytes = src.as_bytes();
    let mut i = 0;
    while let Some(pos) = src[i..].find("export") {
        let at = i + pos;
        i = at + 6;
        if at > 0 && (bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'_' || bytes[at - 1] == b'$') { continue; }
        let rest = src[i..].trim_start();
        if let Some(r) = rest.strip_prefix('{') {
            if let Some(end) = r.find('}') {
                for part in r[..end].split(',') {
                    let part = part.trim();
                    if part.is_empty() { continue; }
                    let name = match part.split_once(" as ") { Some((_, alias)) => alias.trim(), None => part };
                    names.push(name.to_string());
                }
            }
            continue;
        }
        if rest.starts_with("default") { names.push("default".into()); continue; }
        for kw in ["const", "let", "var", "async function", "function", "class"] {
            if let Some(r) = rest.strip_prefix(kw) {
                let r = r.trim_start();
                if let Some(pat) = r.strip_prefix('{') {
                    // export const { a, b: c } = ...: the bound names, in order
                    if let Some(end) = pat.find('}') {
                        for part in pat[..end].split(',') {
                            let part = part.trim();
                            if part.is_empty() { continue; }
                            let name = match part.split_once(':') { Some((_, bound)) => bound.trim(), None => part };
                            let name = name.split('=').next().unwrap_or("").trim();
                            if !name.is_empty() { names.push(name.to_string()); }
                        }
                    }
                    break;
                }
                let name: String = r.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$').collect();
                if !name.is_empty() { names.push(name); }
                break;
            }
        }
    }
    names
}

fn describe(ctx: &Ctx<'_>, e: rquickjs::Error, name: &str) -> String {
    if e.is_exception() {
        let v = ctx.catch();
        if let Some(ex) = v.as_exception() {
            let msg = ex.message().unwrap_or_default();
            let stack = ex.stack().unwrap_or_default();
            return format!("{name}: {msg}{}", if stack.is_empty() { String::new() } else { format!("\n{stack}") });
        }
        return format!("{name}: {}", ctx.json_stringify(v).ok().flatten().and_then(|s| s.to_string().ok()).unwrap_or_else(|| "exception".into()));
    }
    format!("{name}: {e}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_order_follows_the_text() {
        let src = "const b = 1; const a = 2;\nexport { b, a as c };\nexport const zed = 3;\nexport default [];\nfunction exported() {}\nexport const { p, q: r } = o;";
        assert_eq!(export_order(src), vec!["b", "c", "zed", "default", "p", "r"]);
    }

    #[test]
    fn a_program_writes_entities_and_nothing_else_reaches_it() {
        let dir = std::env::temp_dir().join(format!("esim-js-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("shared.js"), "export const horizon = 2100;\nexport const core = [{ id: 'carbon', kind: 'carbon.pools' }];\n").unwrap();
        std::fs::write(dir.join("numbers.json"), "{ \"ppm\": 286.8 }").unwrap();
        std::fs::write(dir.join("m.js"), r#"
import { horizon, core } from "./shared.js";
import numbers from "./numbers.json";
const farms = Array.from({ length: 3 }, (_, i) => ({ id: `farm-${i}`, kind: "farm.agent", params: { acres: 160 + 40 * i } }));
export const run = { start: "1850-01-01", end: `${horizon}-01-01`, seed: 1 };
export const audit = { scores: [["a", "b"]] };
export const emissions = { kind: "carbon.emissions", params: { after_record: "hold", ppm: numbers.ppm, when: typeof Date } };
export { farms, core };
export const seeds = { variant: { title: "Members", range: { "run.seed": { from: 1, to: 3, steps: 3 } } } };
export function helper() { return 1 }
"#).unwrap();
        let (doc, sources) = evaluate(&dir.join("m.js")).unwrap();
        let ids: Vec<&str> = doc["entities"].as_array().unwrap().iter().map(|e| e["id"].as_str().unwrap()).collect();
        assert_eq!(ids, vec!["run", "audit", "emissions", "farm-0", "farm-1", "farm-2", "carbon", "seeds"], "export order, arrays spliced, functions dropped");
        assert!(doc["entities"][0]["run"]["start"].is_string() && doc["entities"][1]["audit"]["scores"].is_array(), "the short form wraps a singleton component");
        assert_eq!(doc["entities"][2]["params"]["when"], "undefined", "no Date");
        assert_eq!(doc["entities"][2]["params"]["ppm"], 286.8);
        assert_eq!(sources.keys().cloned().collect::<Vec<_>>(), vec!["numbers.json", "shared.js"]);
        let m = crate::Manifest::from_entities(&doc).unwrap();
        assert_eq!(m.modules.len(), 5);
        assert_eq!(m.variants.len(), 3);
        // what is refused
        std::fs::write(dir.join("bad1.js"), "export const x = { kind: 'k', params: { r: Math.random() } };").unwrap();
        assert!(evaluate(&dir.join("bad1.js")).unwrap_err().contains("Math.random"));
        std::fs::write(dir.join("bad2.js"), "import fs from 'fs'; export const x = {};").unwrap();
        assert!(evaluate(&dir.join("bad2.js")).unwrap_err().contains("relative path"));
        std::fs::write(dir.join("bad3.js"), "import x from '../../../etc/hostname'; export const y = {};").unwrap();
        assert!(evaluate(&dir.join("bad3.js")).unwrap_err().contains("outside"));
        std::fs::write(dir.join("bad4.js"), "while (true) {}").unwrap();
        let t0 = std::time::Instant::now();
        assert!(evaluate(&dir.join("bad4.js")).is_err());
        assert!(t0.elapsed().as_secs() < 30, "the budget stops a loop");
        std::fs::remove_dir_all(&dir).ok();
    }
}
