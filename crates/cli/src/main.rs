//! esim run <manifest.toml> [--out runs]   write a run folder and print the summary
//! esim verify <manifest.toml>             prove determinism and replay-from-snapshot
//! esim kinds                              list module kinds

use esim_kernel::{Kernel, Manifest, Module};
use std::fs;
use std::path::{Path, PathBuf};

fn factory(name: &str, params: &toml::Value) -> Result<Box<dyn Module>, String> {
    let kind = params.get("kind").and_then(|v| v.as_str()).ok_or("module without kind")?;
    esim_modules::build(kind, name, params)
}

/// The kernel's factory takes params only; fold each entry's kind into its params.
fn load(path: &Path) -> Result<Manifest, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut m = Manifest::parse(&text)?;
    for e in &mut m.modules {
        if let toml::Value::Table(t) = &mut e.params {
            t.insert("kind".into(), toml::Value::String(e.kind.clone()));
        }
    }
    Ok(m)
}

fn run(path: &Path, out_root: &Path) -> Result<(), String> {
    let manifest = load(path)?;
    let mut k = Kernel::new(manifest.clone(), factory)?;
    let out = k.run();
    // The log digest alone does not see parameters that change only ledger and series
    // values, so the run id also hashes the manifest text.
    let mtext = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mhash = esim_kernel::hash::fnv1a(mtext.as_bytes());
    let id = format!("{}-{:08x}-{}", out.name, (mhash >> 32) as u32, &out.log_digest[..8]);
    let dir = out_root.join(&id);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::copy(path, dir.join("manifest.toml")).map_err(|e| e.to_string())?;
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
    println!("out      {}", dir.display());
    if !out.all_ledgers_closed { return Err("a ledger did not close".into()); }
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
                rows.push(serde_json::json!({
                    "id": e.file_name().to_string_lossy(),
                    "name": v["name"], "start": v["start"], "end": v["end"], "seed": v["seed"],
                    "events": v["events"], "all_ledgers_closed": v["all_ledgers_closed"],
                    "digest": v["log_digest"],
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let res = match args.get(1).map(String::as_str) {
        Some("run") => {
            let path = args.get(2).map(PathBuf::from);
            let out = args.iter().position(|a| a == "--out").and_then(|i| args.get(i + 1)).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("runs"));
            match path { Some(p) => run(&p, &out), None => Err("usage: esim run <manifest.toml> [--out runs]".into()) }
        }
        Some("verify") => match args.get(2) { Some(p) => verify(Path::new(p)), None => Err("usage: esim verify <manifest.toml>".into()) },
        Some("score") => match (args.get(2), args.get(3), args.get(4)) {
            (Some(d), Some(a), Some(b)) => score(Path::new(d), a, b),
            _ => Err("usage: esim score <run_dir> <sim_series> <obs_series>".into()),
        },
        Some("kinds") => { for k in esim_modules::kinds() { println!("{k}"); } Ok(()) }
        _ => Err("usage: esim run|verify|score|kinds".into()),
    };
    if let Err(e) = res { eprintln!("error: {e}"); std::process::exit(1); }
}
