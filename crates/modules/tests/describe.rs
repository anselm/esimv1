//! The describe verb must tell the truth. Every manifest in ../../manifests checks clean,
//! and what a run actually publishes and writes matches what its modules declared.

use esim_kernel::{Kernel, Manifest, Module};
use std::collections::BTreeSet;
use std::path::PathBuf;

fn factory(name: &str, params: &toml::Value) -> Result<Box<dyn Module>, String> {
    let kind = params.get("kind").and_then(|v| v.as_str()).ok_or("module without kind")?;
    esim_modules::build(kind, name, params)
}

fn root() -> PathBuf { [env!("CARGO_MANIFEST_DIR"), "..", ".."].iter().collect() }

fn load(file: &str) -> Manifest {
    let path = root().join("manifests").join(file);
    let mut m = Manifest::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    for e in &mut m.modules {
        if let toml::Value::Table(t) = &mut e.params { t.insert("kind".into(), toml::Value::String(e.kind.clone())); }
    }
    m
}

fn have_dem() -> bool { root().join("data/cache").exists() && root().join("data/catalog.lock.toml").exists() }

fn with_root<T>(f: impl FnOnce() -> T) -> T {
    std::env::set_var("ESIM_ROOT", root());
    f()
}

#[test]
fn every_manifest_checks_clean() {
    let dir = root().join("manifests");
    let mut n = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().and_then(|e| e.to_str()) != Some("toml") { continue; }
        let m = Manifest::parse(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let r = esim_kernel::check(&m, esim_modules::describe);
        assert!(r.errors.is_empty(), "{}: {:?}", p.display(), r.errors);
        for w in &r.warnings { eprintln!("{}: warning: {w}", p.file_name().unwrap().to_string_lossy()); }
        n += 1;
    }
    assert!(n >= 20, "only {n} manifests found");
}

#[test]
fn check_catches_a_read_with_no_writer_and_a_send_to_nobody() {
    let m = Manifest::parse(r#"
[run]
name = "broken"
start = "2026-01-01"
end = "2026-01-02"
[[module]]
name = "slab"
kind = "heat.slab"
[[module]]
name = "tap"
kind = "water.source"
[module.params]
into = "nowhere"
"#).unwrap();
    let r = esim_kernel::check(&m, esim_modules::describe);
    assert!(r.errors.iter().any(|e| e.contains("slab reads sun.incident_j and nothing writes it")), "{:?}", r.errors);
    assert!(r.errors.iter().any(|e| e.contains("tap sends inflow to nowhere")), "{:?}", r.errors);
}

#[test]
fn check_rejects_an_unknown_kind_and_a_duplicate_name() {
    let m = Manifest::parse(r#"
[run]
name = "broken"
start = "2026-01-01"
end = "2026-01-02"
[[module]]
name = "a"
kind = "water.bucket"
[[module]]
name = "a"
kind = "no.such"
"#).unwrap();
    let r = esim_kernel::check(&m, esim_modules::describe);
    assert!(r.errors.iter().any(|e| e.contains("two modules named a")), "{:?}", r.errors);
    assert!(r.errors.iter().any(|e| e.contains("unknown module kind no.such")), "{:?}", r.errors);
}

/// Run a manifest and compare the log against the declarations: every broadcast topic
/// a module published is in its `publishes`, every directed message is in its `sends`,
/// every world field it wrote is in its `fields_owned`, every subscription is declared.
fn surface_matches_log(m: Manifest) {
    let decl: Vec<_> = m.modules.iter().map(|e| esim_modules::describe(&e.kind, &e.name, &e.params).unwrap()).collect();
    let by_name = |n: &str| decl.iter().find(|d| d.name == n).unwrap_or_else(|| panic!("no module {n}"));
    let mut k = Kernel::new(m.clone(), factory).unwrap();
    k.run();
    let mut undeclared: BTreeSet<String> = BTreeSet::new();
    for ev in &k.log.events {
        if ev.src == "kernel" { continue; }
        let d = by_name(&ev.src);
        if ev.topic == "world.write" {
            let field = ev.data.get("field").and_then(|v| v.as_str()).unwrap_or("");
            if !d.fields_owned.iter().any(|p| p.name == field) { undeclared.insert(format!("{} writes field {field}", ev.src)); }
        } else if let Some(to) = &ev.to {
            if to == &ev.src { continue; } // its own tick or a note to itself
            if !d.sends.iter().any(|s| &s.to == to && s.topic == ev.topic) { undeclared.insert(format!("{} sends {} to {to}", ev.src, ev.topic)); }
        } else if !d.publishes.iter().any(|p| p.name == ev.topic) {
            undeclared.insert(format!("{} publishes {}", ev.src, ev.topic));
        }
    }
    let snap = k.snapshot();
    for (topic, idx) in snap.subs.iter() {
        for &i in idx {
            let d = &decl[i];
            if !d.subscribes.iter().any(|p| p.name == *topic) { undeclared.insert(format!("{} subscribes to {topic}", d.name)); }
        }
    }
    for (topic, cells) in snap.subs_at.iter() {
        let mut idxs = BTreeSet::new();
        for v in cells.values() { idxs.extend(v.iter().copied()); }
        for i in idxs {
            let d = &decl[i];
            if !d.subscribes.iter().any(|p| p.name == *topic) { undeclared.insert(format!("{} subscribes at cells to {topic}", d.name)); }
        }
    }
    assert!(undeclared.is_empty(), "undeclared surface: {undeclared:#?}");
}

#[test]
fn m0_surface_matches_the_log() { surface_matches_log(load("m0-buckets.toml")); }

#[test]
fn m10_surface_matches_the_log() {
    if !have_dem() { eprintln!("skipped: no data cache"); return; }
    with_root(|| surface_matches_log(load("m10-rules-boone-season.toml")));
}

#[test]
fn m7_fire_surface_matches_the_log() {
    if !have_dem() { eprintln!("skipped: no data cache"); return; }
    with_root(|| surface_matches_log(load("m7-fire-boone-sept.toml")));
}

#[test]
fn kinds_describe_under_default_params() {
    let empty = toml::Value::Table(Default::default());
    for k in esim_modules::kinds() {
        let d = esim_modules::describe(k, k, &empty).unwrap();
        assert_eq!(d.kind, k);
        assert!(!d.summary.is_empty(), "{k} has no summary");
        assert!(!d.dt.is_empty(), "{k} has no dt");
        let json = serde_json::to_string(&d).unwrap();
        assert!(json.contains("\"params\""));
    }
}
