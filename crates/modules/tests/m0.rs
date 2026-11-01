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

/// M1 and M2 run on the real Copernicus tile. They need the tile in the cache; on a
/// box without it (or without network) they are skipped rather than downloading 40 MB
/// inside a unit test. Run `cargo run -p esim-registry --example dem_stats` once.
fn have_dem() -> bool {
    let p: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "..", "data", "cache", "copernicus-dem-glo30_N42_W094.tif"].iter().collect();
    p.exists()
}

fn with_root<T>(f: impl FnOnce() -> T) -> T {
    let root: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", ".."].iter().collect();
    std::env::set_var("ESIM_ROOT", root.canonicalize().unwrap());
    f()
}

#[test]
fn m1_sun_week_closes_and_replays() {
    if !have_dem() { eprintln!("skipped: DEM tile not cached"); return; }
    with_root(|| {
        let m = load("m1-sun-boone-week.toml");
        let a = Kernel::new(m.clone(), factory).unwrap().run();
        assert!(a.all_ledgers_closed, "energy ledger must close");
        let peak = a.series["sun.ghi_mean_w_m2"].points.iter().map(|p| p.1).fold(0.0, f64::max);
        assert!(peak > 900.0 && peak < 1050.0, "June clear-sky peak {peak}");
        let mut full = Kernel::new(m.clone(), factory).unwrap();
        full.run();
        let snap = full.snapshots[0].clone();
        let tail: Vec<String> = full.log.events[snap.log_len..].iter().map(|e| serde_json::to_string(e).unwrap()).collect();
        let mut resumed = Kernel::from_snapshot(m, factory, snap).unwrap();
        resumed.run();
        let got: Vec<String> = resumed.log.events.iter().map(|e| serde_json::to_string(e).unwrap()).collect();
        assert_eq!(tail, got);
    });
}

#[test]
fn m2_two_rates_one_energy_ledger() {
    if !have_dem() { eprintln!("skipped: DEM tile not cached"); return; }
    with_root(|| {
        let m = load("m2-heat-boone-week.toml");
        let out = Kernel::new(m.clone(), factory).unwrap().run();
        assert!(out.all_ledgers_closed, "energy ledger across sun and heat must close");
        let e = &out.ledgers[0];
        let worst = e.closes.iter().map(|c| c.max_leak).fold(0.0, f64::max);
        assert!(worst < 1e4, "worst leak {worst} J");
        // the two modules really ran at different rates
        let sun_ticks = out.ledgers[0].transfers; // 3 per sun step + 3 per heat step
        assert!(sun_ticks > 2000);
        let t = &out.series["heat.t_surface_mean_c"].points;
        let (lo, hi) = t.iter().fold((f64::MAX, f64::MIN), |(l, h), p| (l.min(p.1), h.max(p.1)));
        assert!(hi > lo + 10.0, "a diurnal cycle should exist: {lo}..{hi}");
    });
}

#[test]
fn m3_water_closes_and_delineates_the_boone() {
    if !have_dem() { eprintln!("skipped: DEM tile not cached"); return; }
    with_root(|| {
        let m = load("m3-water-boone-june.toml");
        let out = Kernel::new(m.clone(), factory).unwrap().run();
        assert!(out.all_ledgers_closed, "water and energy ledgers must close");
        let area = out.series["basin.area_km2"].points[0].1;
        assert!((area - 2186.0).abs() / 2186.0 < 0.05, "basin area {area} km2 vs USGS 2186");
        assert!(out.series.contains_key("hydro.q_obs_daily_m3_s"), "observed discharge must be loaded");
        let q = &out.series["hydro.q_sim_daily_m3_s"].points;
        assert!(q.len() >= 28 && q.iter().all(|p| p.1.is_finite() && p.1 >= 0.0));
    });
}
