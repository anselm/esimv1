//! esim run <manifest.toml> [--out runs]   write a run folder and print the summary
//! esim verify <manifest.toml>             prove determinism and replay-from-snapshot
//! esim check <manifest.toml>              check the wiring from the modules' descriptions, fetching nothing
//! esim describe <manifest.toml>           the wired modules' descriptions and findings as JSON
//! esim kinds [--json]                     list module kinds, or describe each under default parameters
//! esim audit <run_dir>                    the independent auditor: spend-down, diversity, shocks, tears
//! esim project new <area/name> [--manifest f] [--title t] [--description d]   a project folder under projects/
//! esim project run <area/name> [--variant v] [--set module.param=value ...]   run a project's manifest into its runs/
//! esim project update <area/name> --manifest f   replace a project's manifest with a re-resolved one
//! esim projects [--json]                  list projects (and refresh projects/index.json)

use esim_kernel::{Kernel, Manifest, Module};
use std::fs;
use std::path::{Path, PathBuf};

fn factory(name: &str, params: &toml::Value) -> Result<Box<dyn Module>, String> {
    let kind = params.get("kind").and_then(|v| v.as_str()).ok_or("module without kind")?;
    esim_modules::build(kind, name, params)
}

/// The kernel's factory takes params only; fold each entry's kind into its params.
/// Every load checks the wiring first: errors refuse the run, warnings go to stderr.
fn load(path: &Path) -> Result<Manifest, String> {
    let mut m = Manifest::load(path)?;
    let r = check_report(&m);
    for w in &r.warnings { eprintln!("warning: {w}"); }
    if !r.errors.is_empty() {
        return Err(format!("manifest does not wire up:\n  {}", r.errors.join("\n  ")));
    }
    fold_params(&mut m);
    Ok(m)
}

fn esim_root() -> PathBuf { std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")) }

/// The kernel's wiring check plus the registry's: every asset a module names must be in
/// the catalog, every event table it names must exist.
fn check_report(m: &Manifest) -> esim_kernel::CheckReport {
    let mut r = esim_kernel::check(m, esim_modules::describe);
    let root = esim_root();
    let catalog = esim_registry::Catalog::load(&root.join("data/catalog.toml")).ok();
    for d in &r.modules {
        for a in &d.assets {
            if let Some(table) = a.key.strip_prefix("events/") {
                if !root.join("data/events").join(table).exists() { r.errors.push(format!("{} needs data/events/{table}, which is missing", d.name)); }
            } else {
                match &catalog {
                    None => r.warnings.push(format!("{} needs asset {}, and data/catalog.toml could not be read", d.name, a.key)),
                    Some(c) if c.asset(&a.key).is_none() => r.errors.push(format!("{} needs asset {}, which is not in data/catalog.toml", d.name, a.key)),
                    _ => {}
                }
            }
        }
    }
    r
}

/// esim check <manifest>: print the wiring report; exit 1 on errors.
fn check(path: &Path) -> Result<(), String> {
    let m = Manifest::load(path)?;
    let r = check_report(&m);
    println!("{}  {} -> {}  {} modules", m.run.name, m.run.start, m.run.end, r.modules.len());
    for d in &r.modules {
        println!("  {:<10} {:<16} priority {:>2}  dt {:<4} owns {}  reads {}  pub {}  sub {}  ledgers {}",
            d.name, d.kind, d.priority, d.dt,
            d.fields_owned.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(","),
            d.fields_read.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(","),
            d.publishes.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(","),
            d.subscribes.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(","),
            d.ledgers.iter().map(|l| l.name.as_str()).collect::<Vec<_>>().join(","));
    }
    for e in &r.errors { println!("error: {e}"); }
    for w in &r.warnings { println!("warning: {w}"); }
    for i in &r.info { println!("info: {i}"); }
    if r.errors.is_empty() { println!("ok: wiring checks"); Ok(()) } else { Err(format!("{} wiring error(s)", r.errors.len())) }
}

/// esim describe <manifest>: the wired modules' full descriptions and findings as JSON,
/// for a facade or a person who wants everything.
fn describe(path: &Path) -> Result<(), String> {
    let m = Manifest::load(path)?;
    let r = check_report(&m);
    println!("{}", serde_json::to_string_pretty(&r).unwrap());
    Ok(())
}

/// esim kinds [--json]: the kinds, or their full descriptions under default parameters.
fn kinds(json: bool) -> Result<(), String> {
    if !json { for k in esim_modules::kinds() { println!("{k}"); } return Ok(()); }
    let empty = toml::Value::Table(Default::default());
    let all: Vec<_> = esim_modules::kinds().iter().map(|k| esim_modules::describe(k, k, &empty).unwrap()).collect();
    println!("{}", serde_json::to_string_pretty(&all).unwrap());
    Ok(())
}

/// The kernel's factory sees params only, so the CLI folds each entry's kind and the
/// run seed into them. Modules that pick ensemble members read `seed`.
fn fold_params(m: &mut Manifest) {
    let seed = m.run.seed as i64;
    for e in &mut m.modules {
        if let toml::Value::Table(t) = &mut e.params {
            t.insert("kind".into(), toml::Value::String(e.kind.clone()));
            t.insert("seed".into(), toml::Value::Integer(seed));
        }
    }
}

/// A parameter override: "run.end" or "module.param", and a value given as text.
pub type Override = (String, String);

/// Apply overrides to a manifest document: `run.<key>` sets a run field, anything else
/// is `<module name>.<param>`. The value text is read as TOML (number, bool, string).
fn apply_overrides(text: &str, sets: &[Override]) -> Result<String, String> {
    if sets.is_empty() { return Ok(text.to_string()); }
    let mut doc: toml::Value = toml::from_str(text).map_err(|e| format!("manifest: {e}"))?;
    for (key, val) in sets {
        let v: toml::Value = toml::from_str::<toml::Value>(&format!("v = {val}")).ok().and_then(|t| t.get("v").cloned())
            .unwrap_or(toml::Value::String(val.clone()));
        let (head, param) = key.split_once('.').ok_or_else(|| format!("--set {key}: expected run.<key> or <module>.<param>"))?;
        if head == "run" {
            doc.get_mut("run").and_then(|r| r.as_table_mut()).ok_or("manifest has no [run]")?.insert(param.to_string(), v);
        } else {
            let mods = doc.get_mut("module").and_then(|m| m.as_array_mut()).ok_or("manifest has no [[module]]")?;
            let m = mods.iter_mut().find(|m| m.get("name").and_then(|n| n.as_str()) == Some(head)).ok_or_else(|| format!("--set {key}: no module named {head}"))?;
            let t = m.as_table_mut().ok_or("module entry is not a table")?;
            let params = t.entry("params").or_insert(toml::Value::Table(Default::default()));
            params.as_table_mut().ok_or("module.params is not a table")?.insert(param.to_string(), v);
        }
    }
    toml::to_string_pretty(&doc).map_err(|e| e.to_string())
}

fn run(path: &Path, out_root: &Path) -> Result<PathBuf, String> { run_with(path, out_root, &[], None) }

fn run_with(path: &Path, out_root: &Path, sets: &[Override], variant: Option<&str>) -> Result<PathBuf, String> {
    let original = Manifest::resolve_text(path)?;
    let mtext = apply_overrides(&original, sets)?;
    let mut manifest = Manifest::parse(&mtext)?;
    let r = check_report(&manifest);
    for w in &r.warnings { eprintln!("warning: {w}"); }
    if !r.errors.is_empty() { return Err(format!("manifest does not wire up:\n  {}", r.errors.join("\n  "))); }
    fold_params(&mut manifest);
    let mut k = Kernel::new(manifest.clone(), factory)?;
    let out = k.run();
    // The log digest alone does not see parameters that change only ledger and series
    // values, so the run id also hashes the resolved manifest text.
    let mhash = esim_kernel::hash::fnv1a(mtext.as_bytes());
    let id = format!("{}-{:08x}-{}", out.name, (mhash >> 32) as u32, &out.log_digest[..8]);
    let dir = out_root.join(&id);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join("manifest.toml"), &mtext).map_err(|e| e.to_string())?;
    // the modules' describes, in manifest order: what the viewer composes the page from
    fs::write(dir.join("describe.json"), serde_json::to_string(&r).unwrap()).map_err(|e| e.to_string())?;
    if variant.is_some() || !sets.is_empty() {
        let set: serde_json::Map<String, serde_json::Value> = sets.iter().map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone()))).collect();
        fs::write(dir.join("variant.json"), serde_json::to_string_pretty(&serde_json::json!({ "variant": variant, "set": set })).unwrap()).map_err(|e| e.to_string())?;
    }
    let f = fs::File::create(dir.join("log.jsonl")).map_err(|e| e.to_string())?;
    k.log.write_jsonl(std::io::BufWriter::new(f)).map_err(|e| e.to_string())?;
    fs::write(dir.join("run.json"), serde_json::to_string_pretty(&out).unwrap()).map_err(|e| e.to_string())?;
    for (name, f) in &k.fields {
        let fd = dir.join("fields").join(name);
        fs::create_dir_all(&fd).map_err(|e| e.to_string())?;
        for (i, (_, data)) in f.frames.iter().enumerate() {
            let bytes: Vec<u8> = data.iter().flat_map(|v| v.to_le_bytes()).collect();
            fs::write(fd.join(format!("{i:05}.f32")), bytes).map_err(|e| e.to_string())?;
        }
    }
    if !k.snapshots.is_empty() {
        fs::write(dir.join("snapshots.json"), serde_json::to_string(&k.snapshots).unwrap()).map_err(|e| e.to_string())?;
    }
    update_index(out_root)?;
    println!("run      {id}");
    println!("window   {} .. {}", out.start, out.end);
    println!("events   {} logged, {} delivered, {} ms", out.events, out.delivered, out.elapsed_ms);
    println!("digest   {}", out.log_digest);
    for l in &out.ledgers {
        let worst = l.closes.iter().map(|c| c.max_leak).fold(0.0, f64::max);
        println!("ledger   {} [{}]: {} transfers, {} closes, worst leak {:.3e} {}, {}",
            l.name, l.unit, l.transfers, l.closes.len(), worst, l.unit, if l.all_closed { "CLOSED" } else { "LEAK" });
    }
    // scoreboards the manifest names, then the auditor
    if let Ok(v) = toml::from_str::<toml::Value>(&mtext) {
        for pair in v.get("audit").and_then(|a| a.get("scores")).and_then(|x| x.as_array()).into_iter().flatten() {
            if let (Some(sim), Some(obs)) = (pair.get(0).and_then(|x| x.as_str()), pair.get(1).and_then(|x| x.as_str())) {
                if let Err(e) = score(&dir, sim, obs) { eprintln!("score    {sim} vs {obs}: {e}"); }
            }
        }
    }
    audit(&dir)?;
    println!("out      {}", dir.display());
    if !out.all_ledgers_closed { return Err("a ledger did not close".into()); }
    Ok(dir)
}

/// esim audit <run_dir>: write audit.json and print the verdict.
fn audit(dir: &Path) -> Result<(), String> {
    let a = esim_audit::audit_dir(dir)?;
    fs::write(dir.join("audit.json"), serde_json::to_string_pretty(&a).unwrap()).map_err(|e| e.to_string())?;
    println!("audit    {}: {}", a.verdict, a.reasons.join("; "));
    for s in &a.silent { println!("         silent: {s}"); }
    Ok(())
}

// ---- projects: a folder per project under projects/<area>/<name>, holding project.toml,
// manifest.toml and runs/. The folder is the project; the viewer and the server only read it.

fn projects_root() -> PathBuf { esim_root().join("projects") }

fn project_dir(path: &str) -> Result<PathBuf, String> {
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    if parts.len() != 2 || parts.iter().any(|p| p.is_empty() || p.starts_with('.') || !p.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')) {
        return Err(format!("project path must be <area>/<name> in [a-z0-9-_], got {path}"));
    }
    Ok(projects_root().join(parts[0]).join(parts[1]))
}

fn project_new(path: &str, manifest: Option<&Path>, title: Option<&str>, description: Option<&str>, by: &str) -> Result<(), String> {
    let dir = project_dir(path)?;
    if dir.exists() { return Err(format!("project {path} exists at {}", dir.display())); }
    let m = match manifest { Some(p) => load(p)?, None => return Err("a manifest is required (--manifest file)".into()) };
    fs::create_dir_all(dir.join("runs")).map_err(|e| e.to_string())?;
    // the project folder stands alone: includes are folded in before the copy
    fs::write(dir.join("manifest.toml"), Manifest::resolve_text(manifest.unwrap())?).map_err(|e| e.to_string())?;
    let (area, name) = path.trim_matches('/').split_once('/').unwrap();
    let meta = format!(
        "# A project: who asked for what, in their words, and the manifest beside it.
area = {}
name = {}
title = {}
description = {}
created = {}
by = {}
",
        toml::Value::String(area.into()), toml::Value::String(name.into()),
        toml::Value::String(title.unwrap_or(&m.run.name).into()), toml::Value::String(description.unwrap_or(&m.run.notes).into()),
        toml::Value::String(esim_registry::fetch::today_string()), toml::Value::String(by.into()));
    fs::write(dir.join("project.toml"), meta).map_err(|e| e.to_string())?;
    fs::write(dir.join("runs").join("index.json"), "[]").map_err(|e| e.to_string())?;
    projects_index()?;
    println!("project  {path} at {}", dir.display());
    Ok(())
}

/// Run a project: its manifest, with a named variant's overrides from project.toml
/// (`[[variant]] name = ..., [variant.set] "module.param" = value`) and any --set on top.
fn project_run(path: &str, variant: Option<&str>, extra: &[Override]) -> Result<(), String> {
    let dir = project_dir(path)?;
    if !dir.join("manifest.toml").exists() { return Err(format!("no project at {}", dir.display())); }
    let mut sets: Vec<Override> = Vec::new();
    if let Some(v) = variant {
        let meta: toml::Value = toml::from_str(&fs::read_to_string(dir.join("project.toml")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let found = meta.get("variant").and_then(|a| a.as_array()).into_iter().flatten()
            .find(|e| e.get("name").and_then(|n| n.as_str()) == Some(v)).ok_or_else(|| format!("project {path} has no variant {v}"))?;
        for (k, val) in found.get("set").and_then(|t| t.as_table()).into_iter().flatten() {
            let text = match val { toml::Value::String(s) => format!("{s:?}"), other => other.to_string() };
            sets.push((k.clone(), text));
        }
    }
    sets.extend(extra.iter().cloned());
    run_with(&dir.join("manifest.toml"), &dir.join("runs"), &sets, variant)?;
    projects_index()?;
    Ok(())
}

/// Replace a project's manifest with a (re)resolved one; project.toml and runs are kept.
/// A project folder stands alone, so a change to a bundle it included does not reach it
/// until this is run on purpose.
fn project_update(path: &str, manifest: &Path) -> Result<(), String> {
    let dir = project_dir(path)?;
    if !dir.join("project.toml").exists() { return Err(format!("no project at {}", dir.display())); }
    load(manifest)?;
    fs::write(dir.join("manifest.toml"), Manifest::resolve_text(manifest)?).map_err(|e| e.to_string())?;
    projects_index()?;
    println!("project  {path}: manifest replaced from {}", manifest.display());
    Ok(())
}

fn parse_overrides(args: &[String]) -> Vec<Override> {
    let mut out = Vec::new();
    for (i, a) in args.iter().enumerate() {
        if a == "--set" { if let Some((k, v)) = args.get(i + 1).and_then(|s| s.split_once('=')) { out.push((k.to_string(), v.to_string())); } }
    }
    out
}

/// projects/index.json: every project with its metadata and its runs' index rows.
fn projects_index() -> Result<Vec<serde_json::Value>, String> {
    let root = projects_root();
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    for area in fs::read_dir(&root).map_err(|e| e.to_string())?.flatten() {
        if !area.path().is_dir() { continue; }
        for proj in fs::read_dir(area.path()).map_err(|e| e.to_string())?.flatten() {
            let d = proj.path();
            let Ok(meta) = fs::read_to_string(d.join("project.toml")) else { continue };
            let meta: toml::Value = toml::from_str(&meta).unwrap_or(toml::Value::Table(Default::default()));
            let runs: serde_json::Value = fs::read_to_string(d.join("runs").join("index.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(serde_json::json!([]));
            let manifest = fs::read_to_string(d.join("manifest.toml")).unwrap_or_default();
            let kinds: Vec<String> = Manifest::parse(&manifest).map(|m| m.modules.iter().map(|e| e.kind.clone()).collect()).unwrap_or_default();
            let path = format!("{}/{}", area.file_name().to_string_lossy(), proj.file_name().to_string_lossy());
            let mut row = serde_json::to_value(&meta).unwrap_or(serde_json::json!({}));
            row["path"] = serde_json::json!(path);
            row["kinds"] = serde_json::json!(kinds);
            row["runs"] = runs;
            rows.push(row);
        }
    }
    rows.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    fs::write(root.join("index.json"), serde_json::to_string_pretty(&rows).unwrap()).map_err(|e| e.to_string())?;
    Ok(rows)
}

fn projects(json: bool) -> Result<(), String> {
    let rows = projects_index()?;
    if json { println!("{}", serde_json::to_string_pretty(&rows).unwrap()); return Ok(()); }
    for r in &rows {
        let variants: Vec<&str> = r["variant"].as_array().into_iter().flatten().filter_map(|v| v["name"].as_str()).collect();
        println!("{:40} {:3} runs  {}{}", r["path"].as_str().unwrap_or(""), r["runs"].as_array().map(|a| a.len()).unwrap_or(0), r["title"].as_str().unwrap_or(""), if variants.is_empty() { String::new() } else { format!("  [{}]", variants.join(", ")) });
    }
    Ok(())
}

/// runs/index.json: one line per run folder, for the viewer.
fn update_index(root: &Path) -> Result<(), String> {
    let mut rows = Vec::new();
    for e in fs::read_dir(root).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        let p = e.path().join("run.json");
        if let Ok(text) = fs::read_to_string(&p) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                let variant: serde_json::Value = fs::read_to_string(e.path().join("variant.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(serde_json::Value::Null);
                let audit: serde_json::Value = fs::read_to_string(e.path().join("audit.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(serde_json::Value::Null);
                rows.push(serde_json::json!({
                    "id": e.file_name().to_string_lossy(),
                    "name": v["name"], "start": v["start"], "end": v["end"], "seed": v["seed"],
                    "events": v["events"], "all_ledgers_closed": v["all_ledgers_closed"],
                    "digest": v["log_digest"],
                    "variant": variant["variant"], "set": variant["set"], "verdict": audit["verdict"],
                }));
            }
        }
    }
    rows.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    fs::write(root.join("index.json"), serde_json::to_string_pretty(&rows).unwrap()).map_err(|e| e.to_string())
}

fn verify(path: &Path) -> Result<(), String> {
    let manifest = load(path)?;
    let a = Kernel::new(manifest.clone(), factory)?.run();
    let b = Kernel::new(manifest.clone(), factory)?.run();
    if a.log_digest != b.log_digest { return Err(format!("two runs differ: {} vs {}", a.log_digest, b.log_digest)); }
    println!("determinism  two runs, same digest {}", a.log_digest);
    if !a.all_ledgers_closed { return Err("ledgers did not close".into()); }
    println!("ledgers      all closed");

    // Replay: run once collecting snapshots, resume from the middle one, compare tails.
    let mut m = manifest.clone();
    if m.run.snapshot_every.is_none() { m.run.snapshot_every = Some(m.run.close_every.clone()); }
    let mut full = Kernel::new(m.clone(), factory)?;
    let full_out = full.run();
    if full.snapshots.is_empty() { return Err("no snapshots taken; window too short for snapshot_every".into()); }
    let snap = full.snapshots[full.snapshots.len() / 2].clone();
    let cut = snap.t;
    let tail: Vec<&esim_kernel::Event> = full.log.events[snap.log_len..].iter().collect();
    let mut resumed = Kernel::from_snapshot(m.clone(), factory, snap)?;
    let resumed_out = resumed.run();
    let mismatch = tail.iter().zip(resumed.log.events.iter())
        .position(|(x, y)| serde_json::to_string(x).unwrap() != serde_json::to_string(y).unwrap());
    if tail.len() != resumed.log.events.len() || mismatch.is_some() {
        return Err(format!("replay differs: tail {} events vs resumed {} (first mismatch {:?})", tail.len(), resumed.log.events.len(), mismatch));
    }
    let lf = full_out.ledgers.iter().map(|l| l.closes.last().map(|c| c.balances.clone())).collect::<Vec<_>>();
    let lr = resumed_out.ledgers.iter().map(|l| l.closes.last().map(|c| c.balances.clone())).collect::<Vec<_>>();
    if lf != lr { return Err("replay: final ledger balances differ".into()); }
    println!("replay       resumed at {} and reproduced {} events and the final ledgers", cut, tail.len());
    println!("OK");
    Ok(())
}

/// esim score <run_dir> <sim_series> <obs_series>: skill of one series against another
/// at matching times. Writes scores.json in the run folder.
fn score(dir: &Path, sim: &str, obs: &str) -> Result<(), String> {
    let text = fs::read_to_string(dir.join("run.json")).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let get = |name: &str| -> Result<Vec<(i64, f64)>, String> {
        v["series"][name]["points"].as_array().ok_or(format!("no series {name}"))
            .map(|a| a.iter().filter_map(|p| Some((p[0].as_i64()?, p[1].as_f64()?))).collect())
    };
    let s = get(sim)?; let o = get(obs)?;
    let om: std::collections::BTreeMap<i64, f64> = o.into_iter().collect();
    let pairs: Vec<(f64, f64)> = s.iter().filter_map(|(t, x)| om.get(t).map(|y| (*x, *y))).collect();
    let n = pairs.len();
    if n < 2 { return Err(format!("only {n} matching times between {sim} and {obs}")); }
    let (ms, mo) = (pairs.iter().map(|p| p.0).sum::<f64>() / n as f64, pairs.iter().map(|p| p.1).sum::<f64>() / n as f64);
    let sse: f64 = pairs.iter().map(|(x, y)| (x - y).powi(2)).sum();
    let sso: f64 = pairs.iter().map(|(_, y)| (y - mo).powi(2)).sum();
    let nse = 1.0 - sse / sso;
    let rmse = (sse / n as f64).sqrt();
    let pbias = 100.0 * pairs.iter().map(|(x, y)| x - y).sum::<f64>() / pairs.iter().map(|p| p.1).sum::<f64>();
    let (ss, so) = ((pairs.iter().map(|(x, _)| (x - ms).powi(2)).sum::<f64>() / n as f64).sqrt(), (sso / n as f64).sqrt());
    let r = pairs.iter().map(|(x, y)| (x - ms) * (y - mo)).sum::<f64>() / (n as f64 * ss * so);
    let kge = 1.0 - ((r - 1.0).powi(2) + (ss / so - 1.0).powi(2) + (ms / mo - 1.0).powi(2)).sqrt();
    let (maxs, maxo) = (pairs.iter().map(|p| p.0).fold(f64::MIN, f64::max), pairs.iter().map(|p| p.1).fold(f64::MIN, f64::max));
    let row = serde_json::json!({ "sim": sim, "obs": obs, "n": n, "nse": nse, "kge": kge, "r": r, "pbias_pct": pbias, "rmse": rmse,
        "mean_sim": ms, "mean_obs": mo, "max_sim": maxs, "max_obs": maxo });
    let sp = dir.join("scores.json");
    let mut all: Vec<serde_json::Value> = fs::read_to_string(&sp).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
    all.retain(|x| !(x["sim"] == sim && x["obs"] == obs));
    all.push(row.clone());
    fs::write(&sp, serde_json::to_string_pretty(&all).unwrap()).map_err(|e| e.to_string())?;
    println!("score    {sim} vs {obs}: n={n}  NSE={nse:.3}  KGE={kge:.3}  r={r:.3}  PBIAS={pbias:+.1}%  RMSE={rmse:.2}");
    println!("         mean sim {ms:.2} obs {mo:.2}; max sim {maxs:.1} obs {maxo:.1}");
    Ok(())
}

/// esim ensemble <manifest> --branches N [--threads T] [--out runs]
/// Runs N branches with seeds base+1..base+N in parallel, keeps each branch's series,
/// and writes quantile bands per series. Branches never write field frames.
fn ensemble(path: &Path, branches: u64, threads: usize, out_root: &Path) -> Result<(), String> {
    let base = load(path)?;
    // Warm the cache in this thread so branches do not race on downloads and the lock.
    let _ = Kernel::new(base.clone(), factory)?;
    let t0 = std::time::Instant::now();
    let seeds: Vec<u64> = (1..=branches).map(|k| base.run.seed + k).collect();
    let queue = std::sync::Mutex::new(seeds.clone());
    let results: Vec<(u64, esim_kernel::RunOutput)> = std::thread::scope(|sc| {
        let (tx, rx) = std::sync::mpsc::channel();
        for _ in 0..threads.max(1) {
            let tx = tx.clone();
            let queue = &queue;
            let base = &base;
            sc.spawn(move || loop {
                let seed = { let mut q = queue.lock().unwrap(); if q.is_empty() { break; } q.remove(0) };
                let mut m = base.clone();
                m.run.seed = seed;
                fold_params(&mut m);
                let out = Kernel::new(m, factory).and_then(|mut k| Ok(k.run()));
                tx.send((seed, out)).unwrap();
            });
        }
        drop(tx);
        let mut v: Vec<(u64, esim_kernel::RunOutput)> = Vec::new();
        for (seed, out) in rx { match out { Ok(o) => { eprintln!("branch seed {seed}: {} events, digest {}, ledgers {}", o.events, o.log_digest, if o.all_ledgers_closed { "closed" } else { "LEAK" }); v.push((seed, o)); } Err(e) => eprintln!("branch seed {seed} failed: {e}") } }
        v.sort_by_key(|(s, _)| *s);
        v
    });
    if results.is_empty() { return Err("no branch finished".into()); }
    let mtext = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mhash = esim_kernel::hash::fnv1a(mtext.as_bytes());
    let id = format!("{}-ens{}-{:08x}", base.run.name, results.len(), (mhash >> 32) as u32);
    let dir = out_root.join(&id);
    fs::create_dir_all(dir.join("branches")).map_err(|e| e.to_string())?;
    fs::copy(path, dir.join("manifest.toml")).map_err(|e| e.to_string())?;
    // quantiles per series over branches at matching times
    let mut series_names: Vec<String> = results[0].1.series.keys().cloned().collect();
    series_names.sort();
    let mut bands = serde_json::Map::new();
    for name in &series_names {
        let unit = results[0].1.series[name].unit.clone();
        let times: Vec<i64> = results[0].1.series[name].points.iter().map(|p| p.0 .0).collect();
        let n = times.len();
        let mut cols: Vec<Vec<f64>> = vec![Vec::with_capacity(results.len()); n];
        for (_, o) in &results {
            if let Some(s) = o.series.get(name) { for (i, p) in s.points.iter().enumerate() { if i < n { cols[i].push(p.1); } } }
        }
        let q = |v: &mut Vec<f64>, f: f64| -> f64 { if v.is_empty() { return f64::NAN; } v.sort_by(|a, b| a.partial_cmp(b).unwrap()); let x = f * (v.len() - 1) as f64; let (i, w) = (x.floor() as usize, x.fract()); if i + 1 < v.len() { v[i] * (1.0 - w) + v[i + 1] * w } else { v[i] } };
        let mut band = serde_json::Map::new();
        band.insert("unit".into(), serde_json::json!(unit));
        band.insert("times".into(), serde_json::json!(times));
        for (label, f) in [("min", 0.0), ("p10", 0.1), ("p25", 0.25), ("p50", 0.5), ("p75", 0.75), ("p90", 0.9), ("max", 1.0)] {
            let vals: Vec<f64> = cols.iter_mut().map(|c| q(c, f)).collect();
            band.insert(label.into(), serde_json::json!(vals));
        }
        bands.insert(name.clone(), serde_json::Value::Object(band));
    }
    for (seed, o) in &results {
        let slim = serde_json::json!({ "seed": seed, "digest": o.log_digest, "events": o.events, "all_ledgers_closed": o.all_ledgers_closed, "series": o.series });
        fs::write(dir.join("branches").join(format!("{seed}.json")), serde_json::to_string(&slim).unwrap()).map_err(|e| e.to_string())?;
    }
    let distinct = { let mut d: Vec<&String> = results.iter().map(|(_, o)| &o.log_digest).collect(); d.sort(); d.dedup(); d.len() };
    let summary = serde_json::json!({
        "name": base.run.name, "kind": "ensemble", "branches": results.len(), "seeds": results.iter().map(|(s, _)| *s).collect::<Vec<_>>(),
        "start": results[0].1.start, "end": results[0].1.end, "kernel_version": results[0].1.kernel_version,
        "all_ledgers_closed": results.iter().all(|(_, o)| o.all_ledgers_closed),
        "distinct_digests": distinct,
        "elapsed_ms": t0.elapsed().as_millis(), "series": bands,
    });
    fs::write(dir.join("ensemble.json"), serde_json::to_string(&summary).unwrap()).map_err(|e| e.to_string())?;
    // an index row shaped like a run's, with kind
    fs::write(dir.join("run.json"), serde_json::to_string_pretty(&serde_json::json!({
        "name": base.run.name, "kind": "ensemble", "seed": base.run.seed, "start": results[0].1.start, "end": results[0].1.end,
        "kernel_version": results[0].1.kernel_version, "events": results.iter().map(|(_, o)| o.events).sum::<usize>(), "delivered": 0,
        "log_digest": format!("ens{}", results.len()), "all_ledgers_closed": summary["all_ledgers_closed"], "ledgers": [], "series": {}, "fields": {},
        "elapsed_ms": summary["elapsed_ms"], "branches": results.len(),
    })).unwrap()).map_err(|e| e.to_string())?;
    update_index(out_root)?;
    println!("ensemble {id}: {} branches, {} distinct digests, {} ms, ledgers {}", results.len(), summary["distinct_digests"], summary["elapsed_ms"], if summary["all_ledgers_closed"].as_bool().unwrap() { "all closed" } else { "LEAK" });
    println!("out      {}", dir.display());
    Ok(())
}

/// esim sweep <manifest> --score <sim> <obs> --samples N [--threads T] [--seed S]
///            --set <module>.<param>=<lo>:<hi> [--set ...]
/// Calibration by method, first form: Latin-hypercube samples of the named parameters,
/// one run each in parallel, scored at matching times, written to runs/<id>/sweep.json
/// with the best ten printed. Nothing is tuned by hand; the table is the record.
fn sweep(path: &Path, sim: &str, obs: &str, samples: usize, threads: usize, seed: u64, sets: &[(String, String, f64, f64)], out_root: &Path) -> Result<(), String> {
    let base = load(path)?;
    let _ = Kernel::new(base.clone(), factory)?; // warm the cache
    // Latin hypercube: each parameter's range cut into `samples` strata, one draw per stratum, strata shuffled per parameter
    let mut rng = esim_kernel::rng::Rng::from_seed(seed);
    let k = sets.len();
    let mut design = vec![vec![0.0f64; k]; samples];
    for (j, (_, _, lo, hi)) in sets.iter().enumerate() {
        let mut strata: Vec<usize> = (0..samples).collect();
        for i in (1..samples).rev() { let r = (rng.next_u64() % (i as u64 + 1)) as usize; strata.swap(i, r); }
        for i in 0..samples { let u = (strata[i] as f64 + rng.f64()) / samples as f64; design[i][j] = lo + u * (hi - lo); }
    }
    let t0 = std::time::Instant::now();
    let queue = std::sync::Mutex::new((0..samples).collect::<Vec<usize>>());
    let rows: Vec<serde_json::Value> = std::thread::scope(|sc| {
        let (tx, rx) = std::sync::mpsc::channel();
        for _ in 0..threads.max(1) {
            let tx = tx.clone(); let queue = &queue; let base = &base; let design = &design;
            sc.spawn(move || loop {
                let i = { let mut q = queue.lock().unwrap(); if q.is_empty() { break; } q.remove(0) };
                let mut m = base.clone();
                for (j, (module, param, _, _)) in sets.iter().enumerate() {
                    if let Some(e) = m.modules.iter_mut().find(|e| &e.name == module) {
                        if let toml::Value::Table(t) = &mut e.params { t.insert(param.clone(), toml::Value::Float(design[i][j])); }
                    }
                }
                fold_params(&mut m);
                let res = Kernel::new(m, factory).map(|mut k| k.run());
                let row = match res {
                    Ok(o) => {
                        let get = |name: &str| -> Vec<(i64, f64)> { o.series.get(name).map(|s| s.points.iter().map(|p| (p.0 .0, p.1)).collect()).unwrap_or_default() };
                        let (sv, ov) = (get(sim), get(obs));
                        let om: std::collections::BTreeMap<i64, f64> = ov.into_iter().collect();
                        let pairs: Vec<(f64, f64)> = sv.iter().filter_map(|(t, x)| om.get(t).map(|y| (*x, *y))).collect();
                        let n = pairs.len() as f64;
                        let (ms, mo) = (pairs.iter().map(|p| p.0).sum::<f64>() / n, pairs.iter().map(|p| p.1).sum::<f64>() / n);
                        let sse: f64 = pairs.iter().map(|(x, y)| (x - y).powi(2)).sum();
                        let sso: f64 = pairs.iter().map(|(_, y)| (y - mo).powi(2)).sum();
                        let nse = 1.0 - sse / sso;
                        let pbias = 100.0 * (ms - mo) / mo;
                        let (ss, so) = ((pairs.iter().map(|(x, _)| (x - ms).powi(2)).sum::<f64>() / n).sqrt(), (sso / n).sqrt());
                        let r = pairs.iter().map(|(x, y)| (x - ms) * (y - mo)).sum::<f64>() / (n * ss * so);
                        let kge = 1.0 - ((r - 1.0).powi(2) + (ss / so - 1.0).powi(2) + (ms / mo - 1.0).powi(2)).sqrt();
                        let mut pm = serde_json::Map::new();
                        for (j, (m_, p_, _, _)) in sets.iter().enumerate() { pm.insert(format!("{m_}.{p_}"), serde_json::Value::from(design[i][j])); }
                        serde_json::json!({ "i": i, "params": pm,
                            "nse": nse, "kge": kge, "r": r, "pbias_pct": pbias, "n": pairs.len(), "digest": o.log_digest, "ledgers_closed": o.all_ledgers_closed })
                    }
                    Err(e) => serde_json::json!({ "i": i, "error": e }),
                };
                eprintln!("sample {i}: NSE {}", row["nse"]);
                tx.send(row).unwrap();
            });
        }
        drop(tx);
        let mut v: Vec<serde_json::Value> = rx.into_iter().collect();
        v.sort_by(|a, b| b["nse"].as_f64().unwrap_or(f64::MIN).partial_cmp(&a["nse"].as_f64().unwrap_or(f64::MIN)).unwrap());
        v
    });
    let mtext = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let id = format!("{}-sweep{}-{:08x}", base.run.name, samples, (esim_kernel::hash::fnv1a(mtext.as_bytes()) >> 32) as u32);
    let dir = out_root.join(&id);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::copy(path, dir.join("manifest.toml")).map_err(|e| e.to_string())?;
    let out = serde_json::json!({ "name": base.run.name, "kind": "sweep", "sim": sim, "obs": obs, "samples": samples, "seed": seed,
        "ranges": sets.iter().map(|(m, p, lo, hi)| serde_json::json!({ "param": format!("{m}.{p}"), "lo": lo, "hi": hi })).collect::<Vec<_>>(),
        "elapsed_ms": t0.elapsed().as_millis(), "rows": rows });
    fs::write(dir.join("sweep.json"), serde_json::to_string_pretty(&out).unwrap()).map_err(|e| e.to_string())?;
    println!("sweep    {id}: {samples} samples in {} s", t0.elapsed().as_secs());
    println!("rank  NSE     KGE     bias%   params");
    for (rank, row) in out["rows"].as_array().unwrap().iter().take(10).enumerate() {
        if row.get("error").is_some() { println!("{rank:>4}  error {}", row["error"]); continue; }
        let ps: Vec<String> = row["params"].as_object().unwrap().iter().map(|(k, v)| format!("{}={:.4}", k.rsplit('.').next().unwrap(), v.as_f64().unwrap())).collect();
        println!("{rank:>4}  {:6.3}  {:6.3}  {:+6.1}  {}", row["nse"].as_f64().unwrap(), row["kge"].as_f64().unwrap(), row["pbias_pct"].as_f64().unwrap(), ps.join(" "));
    }
    println!("out      {}", dir.display());
    Ok(())
}

fn parse_sets(args: &[String]) -> Result<Vec<(String, String, f64, f64)>, String> {
    let mut sets = Vec::new();
    for (i, a) in args.iter().enumerate() {
        if a == "--set" {
            let spec = args.get(i + 1).ok_or("--set needs module.param=lo:hi")?;
            let (name, range) = spec.split_once('=').ok_or("--set module.param=lo:hi")?;
            let (module, param) = name.split_once('.').ok_or("--set module.param=lo:hi")?;
            let (lo, hi) = range.split_once(':').ok_or("--set module.param=lo:hi")?;
            sets.push((module.to_string(), param.to_string(), lo.parse::<f64>().map_err(|_| "lo")?, hi.parse::<f64>().map_err(|_| "hi")?));
        }
    }
    Ok(sets)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let res = match args.get(1).map(String::as_str) {
        Some("run") => {
            let path = args.get(2).map(PathBuf::from);
            let out = args.iter().position(|a| a == "--out").and_then(|i| args.get(i + 1)).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("runs"));
            match path { Some(p) => run(&p, &out).map(|_| ()), None => Err("usage: esim run <manifest.toml> [--out runs]".into()) }
        }
        Some("verify") => match args.get(2) { Some(p) => verify(Path::new(p)), None => Err("usage: esim verify <manifest.toml>".into()) },
        Some("ensemble") => {
            let path = args.get(2).map(PathBuf::from);
            let get = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1));
            let branches: u64 = get("--branches").and_then(|s| s.parse().ok()).unwrap_or(8);
            let threads: usize = get("--threads").and_then(|s| s.parse().ok()).unwrap_or(8);
            let out = get("--out").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("runs"));
            match path { Some(p) => ensemble(&p, branches, threads, &out), None => Err("usage: esim ensemble <manifest.toml> --branches N [--threads T] [--out runs]".into()) }
        }
        Some("sweep") => {
            let path = args.get(2).map(PathBuf::from);
            let get = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1));
            let sim = args.iter().position(|a| a == "--score").and_then(|i| args.get(i + 1)).cloned();
            let obs = args.iter().position(|a| a == "--score").and_then(|i| args.get(i + 2)).cloned();
            let samples: usize = get("--samples").and_then(|s| s.parse().ok()).unwrap_or(32);
            let threads: usize = get("--threads").and_then(|s| s.parse().ok()).unwrap_or(8);
            let seed: u64 = get("--seed").and_then(|s| s.parse().ok()).unwrap_or(1);
            let out = get("--out").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("runs"));
            match (path, sim, obs, parse_sets(&args)) { (Some(p), Some(s), Some(o), Ok(sets)) if !sets.is_empty() => sweep(&p, &s, &o, samples, threads, seed, &sets, &out), (_, _, _, Err(e)) => Err(e), _ => Err("usage: esim sweep <manifest> --score <sim> <obs> --samples N --set module.param=lo:hi ...".into()) }
        }
        Some("score") => match (args.get(2), args.get(3), args.get(4)) {
            (Some(d), Some(a), Some(b)) => score(Path::new(d), a, b),
            _ => Err("usage: esim score <run_dir> <sim_series> <obs_series>".into()),
        },
        Some("kinds") => kinds(args.iter().any(|a| a == "--json")),
        Some("audit") => match args.get(2) { Some(d) => audit(Path::new(d)), None => Err("usage: esim audit <run_dir>".into()) },
        Some("projects") => projects(args.iter().any(|a| a == "--json")),
        Some("project") => {
            let get = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).map(String::as_str);
            match (args.get(2).map(String::as_str), args.get(3)) {
                (Some("new"), Some(p)) => project_new(p, get("--manifest").map(Path::new), get("--title"), get("--description"), get("--by").unwrap_or("cli")),
                (Some("run"), Some(p)) => project_run(p, get("--variant"), &parse_overrides(&args)),
                (Some("update"), Some(p)) => match get("--manifest") { Some(m) => project_update(p, Path::new(m)), None => Err("usage: esim project update <area/name> --manifest <file>".into()) },
                _ => Err("usage: esim project new <area/name> --manifest <file> [--title t] [--description d] [--by who] | esim project run <area/name>".into()),
            }
        }
        Some("check") => match args.get(2) { Some(p) => check(Path::new(p)), None => Err("usage: esim check <manifest.toml>".into()) },
        Some("describe") => match args.get(2) { Some(p) => describe(Path::new(p)), None => Err("usage: esim describe <manifest.toml>".into()) },
        _ => Err("usage: esim run|verify|ensemble|sweep|score|audit|check|describe|kinds|project|projects".into()),
    };
    if let Err(e) = res { eprintln!("error: {e}"); std::process::exit(1); }
}
