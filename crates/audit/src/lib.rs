//! The auditor reads a finished run folder and says what the numbers mean. It is
//! independent of the kernel and the modules on purpose: it audits the artifact (run.json,
//! log.jsonl, scores.json, manifest.toml), never the live run, so no module can arrange
//! its ordering to look good. It holds no theory of any system. It measures four things
//! that every run with the right ingredients exposes, and says when an ingredient is
//! missing rather than inventing a score.
//!
//! - **Spend-down.** Every ledger stock account has a balance at each close. A stock that
//!   falls across the run is being spent; one that rises is accumulating. Reported as the
//!   change over the run and the trend over its last quarter, per account.
//! - **Diversity.** Population series (unit `individuals`) give a Shannon evenness at the
//!   start and the end. With fewer than two populations there is nothing to say, and the
//!   audit says so.
//! - **Shocks and recovery.** Shock events in the log (topics the manifest's `[audit]`
//!   table names, by default `fire.burn` and anything under `hazard.` except
//!   `hazard.loaded`) are grouped into episodes. For each watched series (populations and
//!   biomass by default) the audit measures the drop after the shock and the time it
//!   takes to return within a band of its pre-shock level, and compares that recovery
//!   time with the interval to the next shock. Recovery shorter than recurrence is
//!   stretching; no recovery before the next shock, or before the end with time to spare,
//!   is a tear. This is Anselm's definition from the far-field entry and needs no theory.
//! - **Tears.** Extinctions (a population that reaches zero and stays there), failed
//!   ledger closes, and unrecovered shocks. A verdict for the run: holds, stretching,
//!   tearing, with the reasons listed.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Audit {
    pub run: String,
    pub start: i64,
    pub end: i64,
    pub verdict: String,
    pub reasons: Vec<String>,
    pub stocks: Vec<Stock>,
    pub populations: Vec<Population>,
    pub diversity: Option<Diversity>,
    pub shocks: Vec<Shock>,
    pub tears: Vec<Tear>,
    pub scores: Vec<Value>,
    pub silent: Vec<String>,
    pub settings: Settings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stock { pub ledger: String, pub unit: String, pub account: String, pub start: f64, pub end: f64, pub change_pct: Option<f64>, pub last_quarter_pct: Option<f64>, pub verdict: String }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Population { pub series: String, pub start: f64, pub end: f64, pub min: f64, pub min_at: i64, pub extinct_at: Option<i64> }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Diversity { pub series: Vec<String>, pub evenness_start: f64, pub evenness_end: f64, pub effective_start: f64, pub effective_end: f64 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shock {
    pub t: i64, pub t_end: i64, pub topic: String, pub events: usize,
    pub next_shock_days: Option<f64>,
    pub responses: Vec<Response>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response { pub series: String, pub baseline: f64, pub min: f64, pub min_at: i64, pub drop_pct: f64, pub recovered_at: Option<i64>, pub recovery_days: Option<f64>, pub verdict: String }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tear { pub kind: String, pub subject: String, pub t: i64, pub note: String }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub shock_topics: Vec<String>,
    pub watch: Vec<String>,
    pub band_pct: f64,
    pub baseline_days: f64,
    pub episode_gap_days: f64,
    pub steady_pct: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { shock_topics: vec!["fire.burn".into(), "hazard.".into()], watch: vec![], band_pct: 5.0, baseline_days: 30.0, episode_gap_days: 7.0, steady_pct: 5.0 }
    }
}

const DAY: f64 = 86_400.0;

fn f(v: &Value) -> Option<f64> { v.as_f64() }

fn points(s: &Value) -> Vec<(i64, f64)> {
    s["points"].as_array().map(|a| a.iter().filter_map(|p| Some((p[0].as_i64()?, p[1].as_f64()?))).collect()).unwrap_or_default()
}

fn value_at(pts: &[(i64, f64)], t: i64) -> Option<f64> {
    // last point at or before t
    let i = pts.partition_point(|p| p.0 <= t);
    if i == 0 { None } else { Some(pts[i - 1].1) }
}

fn settings_from(manifest: &str) -> Settings {
    let mut s = Settings::default();
    if let Ok(v) = toml::from_str::<toml::Value>(manifest) {
        if let Some(a) = v.get("audit") {
            if let Some(t) = a.get("shock_topics").and_then(|x| x.as_array()) { s.shock_topics = t.iter().filter_map(|x| x.as_str().map(String::from)).collect(); }
            if let Some(t) = a.get("watch").and_then(|x| x.as_array()) { s.watch = t.iter().filter_map(|x| x.as_str().map(String::from)).collect(); }
            if let Some(x) = a.get("band_pct").and_then(|x| x.as_float()) { s.band_pct = x; }
            if let Some(x) = a.get("baseline_days").and_then(|x| x.as_float()) { s.baseline_days = x; }
            if let Some(x) = a.get("episode_gap_days").and_then(|x| x.as_float()) { s.episode_gap_days = x; }
            if let Some(x) = a.get("steady_pct").and_then(|x| x.as_float()) { s.steady_pct = x; }
        }
    }
    s
}

fn is_shock(topic: &str, settings: &Settings) -> bool {
    if topic == "hazard.loaded" { return false; }
    settings.shock_topics.iter().any(|p| if p.ends_with('.') { topic.starts_with(p.as_str()) } else { topic == p })
}

/// Audit a run folder. Missing optional files (scores, log) narrow the audit and are
/// reported under `silent`, never fabricated.
pub fn audit_dir(dir: &Path) -> Result<Audit, String> {
    let run: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("run.json")).map_err(|e| format!("run.json: {e}"))?).map_err(|e| e.to_string())?;
    let manifest = std::fs::read_to_string(dir.join("manifest.toml")).unwrap_or_default();
    let scores: Vec<Value> = std::fs::read_to_string(dir.join("scores.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
    let log = std::fs::read_to_string(dir.join("log.jsonl")).ok();
    Ok(audit(&run, &manifest, scores, log.as_deref()))
}

pub fn audit(run: &Value, manifest: &str, scores: Vec<Value>, log: Option<&str>) -> Audit {
    let settings = settings_from(manifest);
    let start = run["start"].as_i64().unwrap_or(0);
    let end = run["end"].as_i64().unwrap_or(start);
    let span = (end - start).max(1) as f64;
    let mut a = Audit { run: run["name"].as_str().unwrap_or("").to_string(), start, end, scores, settings: settings.clone(), ..Default::default() };

    // ---- spend-down from ledger closes
    for l in run["ledgers"].as_array().into_iter().flatten() {
        let (name, unit) = (l["name"].as_str().unwrap_or(""), l["unit"].as_str().unwrap_or(""));
        let closes = l["closes"].as_array().cloned().unwrap_or_default();
        let failed = closes.iter().filter(|c| c["ok"].as_bool() == Some(false)).count();
        if failed > 0 {
            let t = closes.iter().find(|c| c["ok"].as_bool() == Some(false)).and_then(|c| c["t"].as_i64()).unwrap_or(start);
            a.tears.push(Tear { kind: "ledger".into(), subject: name.into(), t, note: format!("{failed} of {} closes failed", closes.len()) });
        }
        if closes.len() < 2 { continue; }
        let q = closes.len() * 3 / 4;
        for acct in l["stocks"].as_array().into_iter().flatten().filter_map(|s| s.as_str()) {
            let bal = |c: &Value| f(&c["balances"][acct]).unwrap_or(0.0);
            let (b0, b1, bq) = (bal(&closes[0]), bal(&closes[closes.len() - 1]), bal(&closes[q]));
            let pct = |from: f64, to: f64| if from.abs() > 1e-12 { Some((to - from) / from.abs() * 100.0) } else { None };
            let change = pct(b0, b1);
            let lastq = pct(bq, b1);
            let from_nothing = b0.abs() < 1e-6 * b1.abs().max(1.0);
            let verdict = match change {
                _ if from_nothing && b1 > 0.0 => "accumulating from nothing",
                Some(c) if c < -settings.steady_pct => "spent down",
                Some(c) if c > settings.steady_pct => "accumulating",
                Some(_) => "steady",
                None => if b1 > 0.0 { "accumulating from nothing" } else { "empty" },
            };
            a.stocks.push(Stock { ledger: name.into(), unit: unit.into(), account: acct.into(), start: b0, end: b1, change_pct: change, last_quarter_pct: lastq, verdict: verdict.into() });
        }
    }

    // ---- populations and diversity, from the stocks the ledgers declare
    let series = run["series"].as_object().cloned().unwrap_or_default();
    let mut stock_series: Vec<(String, String, Vec<(i64, f64)>)> = Vec::new(); // (label, unit, points)
    for l in run["ledgers"].as_array().into_iter().flatten() {
        let (name, unit) = (l["name"].as_str().unwrap_or(""), l["unit"].as_str().unwrap_or(""));
        let closes = l["closes"].as_array().cloned().unwrap_or_default();
        for acct in l["stocks"].as_array().into_iter().flatten().filter_map(|s| s.as_str()) {
            let pts: Vec<(i64, f64)> = closes.iter().filter_map(|c| Some((c["t"].as_i64()?, f(&c["balances"][acct]).unwrap_or(0.0)))).collect();
            if pts.len() >= 2 { stock_series.push((format!("{name}/{acct}"), unit.to_string(), pts)); }
        }
    }
    let pops: Vec<(String, Vec<(i64, f64)>)> = stock_series.iter().filter(|(_, u, _)| u == "individuals").map(|(n, _, p)| (n.clone(), p.clone())).collect();
    for (name, pts) in &pops {
        let (mut min, mut min_at) = (f64::INFINITY, pts[0].0);
        for &(t, v) in pts { if v < min { min = v; min_at = t; } }
        let last = pts[pts.len() - 1].1;
        let extinct_at = if last <= 0.0 && pts[0].1 > 0.0 { pts.iter().find(|p| p.1 <= 0.0).map(|p| p.0) } else { None };
        if let Some(t) = extinct_at { a.tears.push(Tear { kind: "extinction".into(), subject: name.clone(), t, note: format!("{name} fell from {:.0} to zero and did not return", pts[0].1) }); }
        a.populations.push(Population { series: name.clone(), start: pts[0].1, end: last, min, min_at, extinct_at });
    }
    if pops.len() >= 2 {
        let even = |t: i64| -> (f64, f64) {
            let vals: Vec<f64> = pops.iter().filter_map(|(_, p)| value_at(p, t)).filter(|v| *v > 0.0).collect();
            let total: f64 = vals.iter().sum();
            if vals.len() < 2 || total <= 0.0 { return (0.0, if vals.is_empty() { 0.0 } else { 1.0 }); }
            let h: f64 = -vals.iter().map(|v| { let p = v / total; p * p.ln() }).sum::<f64>();
            (h / (pops.len() as f64).ln(), h.exp())
        };
        let first_t = pops.iter().map(|p| p.1[0].0).max().unwrap_or(start);
        let (e0, n0) = even(first_t);
        let (e1, n1) = even(end);
        a.diversity = Some(Diversity { series: pops.iter().map(|p| p.0.clone()).collect(), evenness_start: e0, evenness_end: e1, effective_start: n0, effective_end: n1 });
    } else {
        a.silent.push(format!("diversity needs two or more population stocks (a ledger in individuals); this run has {}", pops.len()));
    }

    // ---- shocks from the log
    // what recovery is measured on: every ledger stock, plus any series [audit] watch names
    let mut watched: Vec<(String, Vec<(i64, f64)>)> = stock_series.iter().map(|(n, _, p)| (n.clone(), p.clone())).collect();
    for w in &settings.watch { if let Some(s) = series.get(w) { watched.push((w.clone(), points(s))); } }
    let watch: Vec<String> = watched.iter().map(|w| w.0.clone()).collect();
    match log {
        None => a.silent.push("no log.jsonl, so shocks and recovery were not measured".into()),
        Some(text) => {
            let mut hits: Vec<(i64, String)> = Vec::new();
            for line in text.lines() {
                let Ok(ev) = serde_json::from_str::<Value>(line) else { continue };
                let topic = ev["topic"].as_str().unwrap_or("");
                if is_shock(topic, &settings) { if let Some(t) = ev["t"].as_i64() { hits.push((t, topic.to_string())); } }
            }
            hits.sort();
            // group into episodes
            let gap = (settings.episode_gap_days * DAY) as i64;
            let mut episodes: Vec<(i64, i64, String, usize)> = Vec::new();
            for (t, topic) in hits {
                match episodes.last_mut() {
                    Some(e) if t - e.1 <= gap && e.2 == topic => { e.1 = t; e.3 += 1; }
                    _ => episodes.push((t, t, topic, 1)),
                }
            }
            if episodes.is_empty() && !settings.shock_topics.is_empty() { a.silent.push("no shock events in the log (topics: ".to_string() + &settings.shock_topics.join(", ") + "), so recovery was not measured"); }
            for (i, (t0, t1, topic, n)) in episodes.iter().enumerate() {
                let next = episodes.get(i + 1).map(|e| (e.0 - t0) as f64 / DAY);
                let horizon = episodes.get(i + 1).map(|e| e.0).unwrap_or(end);
                let mut responses = Vec::new();
                for (w, pts) in &watched {
                    let pts = pts.clone();
                    let pre: Vec<f64> = pts.iter().filter(|p| p.0 < *t0 && p.0 >= t0 - (settings.baseline_days * DAY) as i64).map(|p| p.1).collect();
                    let Some(&(_, first)) = pts.iter().find(|p| p.0 < *t0) else { continue };
                    let baseline = if pre.is_empty() { first } else { pre.iter().sum::<f64>() / pre.len() as f64 };
                    if !baseline.is_finite() || baseline.abs() < 1e-12 { continue; }
                    let after: Vec<&(i64, f64)> = pts.iter().filter(|p| p.0 >= *t0 && p.0 <= horizon).collect();
                    if after.is_empty() { continue; }
                    let (mut min, mut min_at) = (f64::INFINITY, *t0);
                    for p in &after { if p.1 < min { min = p.1; min_at = p.0; } }
                    let drop_pct = (baseline - min) / baseline.abs() * 100.0;
                    let band = baseline - baseline.abs() * settings.band_pct / 100.0;
                    let recovered_at = if drop_pct <= settings.band_pct { Some(*t1) } else { after.iter().filter(|p| p.0 > min_at && p.1 >= band).map(|p| p.0).next() };
                    let recovery_days = recovered_at.map(|r| (r - t0) as f64 / DAY);
                    let verdict = match (drop_pct <= settings.band_pct, recovery_days, next) {
                        (true, _, _) => "unmoved".to_string(),
                        (false, Some(r), Some(nx)) if r < nx => "stretched and recovered".into(),
                        (false, Some(_), _) => "recovered".into(),
                        (false, None, Some(_)) => "not recovered before the next shock".into(),
                        (false, None, None) => if (end - t0) as f64 / DAY > 365.0 { "not recovered a year on".into() } else { "unresolved at the end of the run".into() },
                    };
                    if verdict.starts_with("not recovered") {
                        a.tears.push(Tear { kind: "no recovery".into(), subject: w.clone(), t: *t0, note: format!("{w} fell {drop_pct:.0}% after {topic} and was {verdict}") });
                    }
                    responses.push(Response { series: w.clone(), baseline, min, min_at, drop_pct, recovered_at, recovery_days, verdict });
                }
                a.shocks.push(Shock { t: *t0, t_end: *t1, topic: topic.clone(), events: *n, next_shock_days: next, responses });
            }
        }
    }
    if watch.is_empty() { a.silent.push("no ledger stocks to watch for recovery".into()); }
    let _ = (span, &series);

    // ---- verdict
    let stretched = a.shocks.iter().flat_map(|s| &s.responses).filter(|r| r.verdict.starts_with("stretched") || r.verdict == "recovered").count();
    if !a.tears.is_empty() {
        a.verdict = "tearing".into();
        for t in &a.tears { a.reasons.push(format!("{}: {}", t.kind, t.note)); }
    } else if stretched > 0 {
        a.verdict = "stretching".into();
        a.reasons.push(format!("{stretched} shock responses recovered before the next shock"));
    } else {
        a.verdict = "holds".into();
        if a.shocks.is_empty() { a.reasons.push("no shocks in the record".into()); }
    }
    let spent: Vec<&Stock> = a.stocks.iter().filter(|s| s.verdict == "spent down").collect();
    if !spent.is_empty() { a.reasons.push(format!("{} stock{} being spent down: {}", spent.len(), if spent.len() == 1 { "" } else { "s" }, spent.iter().map(|s| format!("{}/{} {:+.0}%", s.ledger, s.account, s.change_pct.unwrap_or(0.0))).collect::<Vec<_>>().join(", "))); }
    a
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn run_with(series: Value, ledgers: Value) -> Value {
        json!({ "name": "t", "start": 0, "end": 365 * 86400, "series": series, "ledgers": ledgers })
    }

    #[test]
    fn extinction_is_a_tear_and_spend_down_is_seen() {
        let closes: Vec<Value> = (0..365).map(|d| json!({ "t": d * 86400, "ok": true, "balances": { "deer": if d < 100 { 50.0 } else { 0.0 }, "pool": 100.0 - d as f64 / 10.0 } })).collect();
        let run = run_with(json!({}), json!([
            { "name": "animals", "unit": "individuals", "stocks": ["deer"], "closes": closes },
            { "name": "soil", "unit": "kg", "stocks": ["pool"], "closes": closes }]));
        let a = audit(&run, "", vec![], None);
        assert_eq!(a.verdict, "tearing", "{:?}", a.reasons);
        assert_eq!(a.tears[0].kind, "extinction");
        assert!(a.stocks.iter().any(|s| s.account == "pool" && s.verdict == "spent down"), "{:?}", a.stocks);
    }

    #[test]
    fn a_shock_that_recovers_before_the_next_is_stretching() {
        // biomass drops at day 100 and 200, recovers in 30 days each time
        let pts: Vec<Value> = (0..365).map(|d| { let v = if (100..130).contains(&d) || (200..230).contains(&d) { 50.0 } else { 100.0 }; json!([d * 86400, v]) }).collect();
        let run = run_with(json!({ "veg.biomass": { "unit": "g", "points": pts } }), json!([]));
        let log = format!("{}\n{}", json!({ "t": 100 * 86400, "topic": "fire.burn" }), json!({ "t": 200 * 86400, "topic": "fire.burn" }));
        let a = audit(&run, "[audit]\nwatch = [\"veg.biomass\"]", vec![], Some(&log));
        assert_eq!(a.shocks.len(), 2, "{:?}", a.shocks);
        assert_eq!(a.shocks[0].responses[0].verdict, "stretched and recovered");
        assert_eq!(a.verdict, "stretching");
        assert!(a.silent.iter().any(|s| s.contains("diversity")));
    }

    #[test]
    fn shocks_too_close_to_recover_from_are_a_tear() {
        let pts: Vec<Value> = (0..365).map(|d| { let v = if d >= 100 { 50.0 } else { 100.0 }; json!([d * 86400, v]) }).collect();
        let run = run_with(json!({ "veg.biomass": { "unit": "g", "points": pts } }), json!([]));
        let log = format!("{}\n{}", json!({ "t": 100 * 86400, "topic": "fire.burn" }), json!({ "t": 120 * 86400, "topic": "fire.burn" }));
        let a = audit(&run, "[audit]\nwatch = [\"veg.biomass\"]", vec![], Some(&log));
        assert_eq!(a.verdict, "tearing");
        assert!(a.tears[0].note.contains("not recovered before the next shock"));
    }
}
