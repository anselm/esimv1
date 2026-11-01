//! M0 proofs: determinism, ledger closure, leak detection, replay from snapshot.
//! These run the manifests in ../../manifests, so the manifests are the fixtures.

use esim_kernel::{Kernel, Manifest, Module};
use std::path::PathBuf;

fn factory(name: &str, params: &toml::Value) -> Result<Box<dyn Module>, String> {
    let kind = params.get("kind").and_then(|v| v.as_str()).ok_or("module without kind")?;
    esim_modules::build(kind, name, params)
}

fn load(file: &str) -> Manifest {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "..", "manifests", file].iter().collect();
    let mut m = Manifest::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    for e in &mut m.modules {
        if let toml::Value::Table(t) = &mut e.params { t.insert("kind".into(), toml::Value::String(e.kind.clone())); }
    }
    m
}

#[test]
fn same_manifest_same_seed_same_log() {
    let a = Kernel::new(load("m0-buckets.toml"), factory).unwrap().run();
    let b = Kernel::new(load("m0-buckets.toml"), factory).unwrap().run();
    assert_eq!(a.log_digest, b.log_digest);
    assert!(a.events > 1000);
}

#[test]
fn different_seed_same_log_when_nothing_is_random() {
    // The bucket modules draw no randomness, so the seed must not matter. If it ever
    // does, a module is using its RNG where it should not.
    let mut m = load("m0-buckets.toml");
    let a = Kernel::new(m.clone(), factory).unwrap().run();
    m.run.seed = 7;
    let b = Kernel::new(m, factory).unwrap().run();
    assert_eq!(a.log_digest, b.log_digest);
}

#[test]
fn water_ledger_closes_across_two_rates() {
    let out = Kernel::new(load("m0-buckets.toml"), factory).unwrap().run();
    assert!(out.all_ledgers_closed);
    let water = &out.ledgers[0];
    assert_eq!(water.unit, "m3");
    assert!(water.closes.len() >= 30);
    assert!(water.closes.iter().all(|c| c.max_leak <= 1e-6));
}

#[test]
fn leaky_bucket_fails_to_close() {
    let out = Kernel::new(load("m0-leaky.toml"), factory).unwrap().run();
    assert!(!out.all_ledgers_closed);
    let water = &out.ledgers[0];
    assert!(water.closes.iter().any(|c| c.max_leak > 1.0));
}

#[test]
fn replay_from_snapshot_reproduces_the_tail() {
    let m = load("m0-buckets.toml");
    let mut full = Kernel::new(m.clone(), factory).unwrap();
    full.run();
    assert!(!full.snapshots.is_empty());
    for snap in full.snapshots.clone() {
        let tail: Vec<String> = full.log.events[snap.log_len..].iter().map(|e| serde_json::to_string(e).unwrap()).collect();
        let mut resumed = Kernel::from_snapshot(m.clone(), factory, snap).unwrap();
        resumed.run();
        let got: Vec<String> = resumed.log.events.iter().map(|e| serde_json::to_string(e).unwrap()).collect();
        assert_eq!(tail, got);
    }
}
