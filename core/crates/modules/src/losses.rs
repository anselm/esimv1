//! `obs.losses`: a published loss study as the record for a run's building damage and casualties.
//! The buildings module computes; this one scores. It reads the buildings' per-cell results from
//! the world on `quake.toll`, sums them over the study's small areas and over one named group of
//! its larger areas, and reports the ratios and the correlation across units. Which rasters,
//! tables and columns the study has is the event folder's `losses.toml` to say; the code names no
//! place and no agency. Until 2026-11-13 this comparison lived inside `quake.buildings`, written
//! for Portland and DOGAMI by name.
use crate::quake::{read_csv, sample, window_grid};
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Describe, Event, LatLonGrid, Module};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, Default)]
struct Loss { id: i32, ext: f64, comp: f64, deaths: f64, displaced: f64, injuries: f64 }
impl Loss { fn ext_comp(&self) -> f64 { self.ext + self.comp } }

#[derive(Serialize, Deserialize)]
pub struct Losses {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    folder: String, record: String, time_of_day: String,
    key: String, label: String, units_label: String, agency: String, study: String, source: String,
    code_min: f64, code_max: f64,
    done: bool,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] unit: Vec<f32>, #[serde(skip)] code: Vec<f32>,
    #[serde(skip)] record_units: Vec<Loss>, #[serde(skip)] record_group: Loss,
}

fn ts(t: &toml::Value, k: &str) -> String { t.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string() }
fn tf(t: &toml::Value, k: &str) -> f64 { t.get(k).and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64))).unwrap_or(0.0) }
fn list(t: &toml::Value, k: &str) -> Vec<String> { match t.get(k) { Some(toml::Value::Array(a)) => a.iter().filter_map(|x| x.as_str().map(String::from)).collect(), Some(toml::Value::String(s)) => vec![s.clone()], _ => Vec::new() } }

impl Losses {
    pub fn new(name: &str, p: &toml::Value) -> Result<Losses, String> {
        let tod = param_str(p, "time_of_day", "day").to_string();
        if tod != "day" && tod != "night" { return Err("obs.losses: time_of_day must be day or night".into()); }
        let mut o = Losses {
            name: name.to_string(),
            lat_s: crate::fragility::need_f64(p, "lat_s", name)?, lat_n: crate::fragility::need_f64(p, "lat_n", name)?, lon_w: crate::fragility::need_f64(p, "lon_w", name)?, lon_e: crate::fragility::need_f64(p, "lon_e", name)?,
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "").to_string(), record: param_str(p, "record", "wet").to_string(), time_of_day: tod,
            key: String::new(), label: String::new(), units_label: String::new(), agency: String::new(), study: String::new(), source: String::new(),
            code_min: 0.0, code_max: 0.0, done: false,
            grid: None, unit: Vec::new(), code: Vec::new(), record_units: Vec::new(), record_group: Loss::default(),
        };
        o.load()?;
        Ok(o)
    }
    fn meta(folder: &str) -> Result<toml::Value, String> {
        let path = esim_registry::paths::event(folder).join("losses.toml");
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        text.parse::<toml::Value>().map_err(|e| format!("losses.toml: {e}"))
    }
    fn fill(s: &str, record: &str, tod: &str) -> String { s.replace("{record}", record).replace("{Time}", if tod == "night" { "Night" } else { "Day" }) }
    fn load(&mut self) -> Result<(), String> {
        let (g, _) = window_grid(self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let m = Self::meta(&self.folder)?;
        let (units, group, cols) = (m.get("units").ok_or("losses.toml has no [units]")?, m.get("group").ok_or("losses.toml has no [group]")?, m.get("columns").ok_or("losses.toml has no [columns]")?);
        self.agency = ts(&m, "agency"); self.study = ts(&m, "study"); self.source = ts(&m, "source");
        self.key = ts(group, "key"); self.label = ts(group, "label"); self.units_label = ts(units, "label");
        self.code_min = tf(group, "code_min"); self.code_max = tf(group, "code_max");
        self.unit = sample(&self.folder, &ts(units, "raster"), &g)?;
        self.code = sample(&self.folder, &ts(group, "raster"), &g)?;
        let fill = |s: &str| Self::fill(s, &self.record, &self.time_of_day);
        let dir = esim_registry::paths::event(&self.folder);
        let f = |r: &Vec<String>, i: usize| r.get(i).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
        let read = |table: &str, first: &str| -> Result<(Vec<Vec<String>>, usize, [usize; 4], Vec<usize>), String> {
            let (head, rows) = read_csv(&dir.join(fill(table)))?;
            let col = |n: &str| { let n = fill(n); head.iter().position(|h| *h == n).ok_or(format!("{table} lacks the column {n}")) };
            let inj = list(cols, "injuries").iter().map(|n| col(n)).collect::<Result<Vec<_>, _>>()?;
            Ok((rows, col(first)?, [col(&ts(cols, "extensive"))?, col(&ts(cols, "complete"))?, col(&ts(cols, "displaced"))?, col(&ts(cols, "deaths"))?], inj))
        };
        let (rows, id, c, inj) = read(&ts(units, "table"), &ts(units, "id"))?;
        self.record_units = rows.iter().map(|r| Loss { id: f(r, id) as i32, ext: f(r, c[0]), comp: f(r, c[1]), displaced: f(r, c[2]), deaths: f(r, c[3]), injuries: inj.iter().map(|&i| f(r, i)).sum() }).collect();
        let (rows, nm, c, inj) = read(&ts(group, "table"), &ts(group, "name"))?;
        let prefix = ts(group, "rows_prefix");
        let mut tot = Loss::default();
        for r in &rows { if r.get(nm).map_or(false, |s| s.starts_with(&prefix)) { tot.ext += f(r, c[0]); tot.comp += f(r, c[1]); tot.displaced += f(r, c[2]); tot.deaths += f(r, c[3]); tot.injuries += inj.iter().map(|&i| f(r, i)).sum::<f64>(); } }
        self.record_group = tot;
        self.grid = Some(g);
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let folder = param_str(p, "folder", "");
        let meta = Self::meta(folder).ok();
        let sub = |t: &str, k: &str| meta.as_ref().and_then(|m| m.get(t)).map(|x| ts(x, k)).unwrap_or_default();
        let top = |k: &str| meta.as_ref().map(|m| ts(m, k)).unwrap_or_default();
        let key = { let k = sub("group", "key"); if k.is_empty() { "group".to_string() } else { k } };
        let label = { let l = sub("group", "label"); if l.is_empty() { "the study's named group of areas".to_string() } else { l } };
        let study = { let s = top("study"); if s.is_empty() { "the published study".to_string() } else { s } };
        let mut summary = String::from("This module holds a run's building damage and casualties against a published loss study. It computes nothing about the earthquake. Once the buildings module has finished, it sums that module's results for each cell over the small areas the study reports by and over one named group of larger areas, and reports how the two sets of numbers compare: a ratio for the group's deaths, displaced people, injuries and damaged buildings, and a correlation across the small areas. A file in the event folder, losses.toml, names the study, its tables and its columns. A ratio near 1 does not make the run an independent estimate if the buildings module's parameters were fitted to the same study, and the buildings module says where they were.");
        let src = top("source");
        if !src.is_empty() { summary.push_str(&format!(" In this run the study is {src}, and the group is {label}.")); }
        let mut d = Describe::new("obs.losses", name, &summary)
            .priority(7).dt("once, after the buildings module's reckoning")
            .param("folder", folder, "", "the folder under data/events that holds the study's tables, the rasters of its areas and a losses.toml naming them")
            .param("record", param_str(p, "record", "wet"), "", "which of the study's cases the run is compared with; it fills {record} in the table names and should match the ground module's soil setting")
            .param("time_of_day", param_str(p, "time_of_day", "day"), "", "day or night: which of the study's casualty columns to use; it should match the buildings module's setting")
            .subscribe("quake.toll", "the buildings module has finished")
            .read_on("quake.cell_extensive", "buildings", true, "quake.toll", "buildings with extensive damage, per cell").read_on("quake.cell_complete", "buildings", true, "quake.toll", "buildings with complete damage, per cell")
            .read_on("quake.cell_deaths", "people", true, "quake.toll", "").read_on("quake.cell_displaced", "people", true, "quake.toll", "").read_on("quake.cell_injuries_hospital", "people", true, "quake.toll", "")
            .publish("fit.crossing", "{ kind: fit, sentence, source, confidence } the comparison as a sentence on the run page")
            .report(&format!("quake.{key}_deaths"), "people").report(&format!("quake.{key}_displaced"), "people").report(&format!("quake.{key}_buildings_extensive_or_complete"), "buildings");
        for k in ["deaths", "displaced", "damage", "extensive", "complete", "injuries"] { d = d.report(&format!("fit.{key}_{k}_ratio"), "1"); }
        d = d.report("fit.units_damage_r", "1").report("fit.units_deaths_r", "1")
            .asset(&format!("events/{folder}"), "the study's loss tables and the rasters of its areas")
            .view("counter", "deaths in the compared area, relative to the study", &format!("fit.{key}_deaths_ratio"), &format!("this run's deaths in {label} divided by the estimate of {study} for the same case; 1 would be an exact match"))
            .view("counter", "damage pattern, correlation with the study", "fit.units_damage_r", &format!("the correlation between this run's counts of buildings with extensive or complete damage and those of {study}, across the study's {}", { let l = sub("units", "label"); if l.is_empty() { "small areas".to_string() } else { l } }));
        // the record as a table on the run page
        if let Some(m) = &meta { if let Some(group) = m.get("group") {
            let table_name = Self::fill(&ts(group, "table"), param_str(p, "record", "wet"), param_str(p, "time_of_day", "day"));
            if let Ok((head, rows)) = read_csv(&esim_registry::paths::event(folder).join(table_name)) {
                let keep = list(group, "show_columns");
                let (pre, suf, eq) = (list(group, "show_prefix"), list(group, "show_suffix"), list(group, "show_equal"));
                let idx: Vec<Option<usize>> = keep.iter().map(|k| head.iter().position(|h| h == k)).collect();
                let table: Vec<serde_json::Value> = rows.iter().filter(|r| r.first().map_or(false, |s| pre.iter().any(|x| s.starts_with(x.as_str())) || suf.iter().any(|x| s.ends_with(x.as_str())) || eq.iter().any(|x| s == x))).map(|r| {
                    let mut o = serde_json::Map::new();
                    for (k, i) in keep.iter().zip(&idx) { if let Some(i) = i { let s = &r[*i]; o.insert(k.to_string(), s.parse::<f64>().map(|v| json!((v * 100.0).round() / 100.0)).unwrap_or(json!(s))); } }
                    serde_json::Value::Object(o)
                }).collect();
                d = d.table("published_losses", json!(table));
            }
        } }
        d
    }
}

impl Module for Losses {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.subscribe("quake.toll");
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "quake.toll" || self.done { return; }
        self.done = true;
        let g = self.grid.clone().expect("grid");
        let n = g.nx * g.ny;
        let take = |name: &str| -> Vec<f32> { ctx.world_read(name).map(|f| f.data.clone()).unwrap_or_else(|| vec![0.0; n]) };
        let (ext, comp, deaths, disp, inj) = (take("quake.cell_extensive"), take("quake.cell_complete"), take("quake.cell_deaths"), take("quake.cell_displaced"), take("quake.cell_injuries_hospital"));
        let mut by_unit: std::collections::HashMap<i32, Loss> = std::collections::HashMap::new();
        let mut grp = Loss::default();
        for i in 0..n {
            let add = |e: &mut Loss| { e.ext += ext[i] as f64; e.comp += comp[i] as f64; e.deaths += deaths[i] as f64; e.displaced += disp[i] as f64; e.injuries += inj[i] as f64; };
            // a unit enters the comparison only through a cell that has buildings, as it did when the buildings module scored itself
            let u = self.unit[i]; if !u.is_nan() && u > 0.0 && ext[i] + comp[i] > 0.0 { add(by_unit.entry(u as i32).or_default()); }
            let j = self.code[i] as f64; if !j.is_nan() && j >= self.code_min && j <= self.code_max { add(&mut grp); }
        }
        let k = self.key.clone();
        ctx.report(&format!("quake.{k}_deaths"), "people", grp.deaths);
        ctx.report(&format!("quake.{k}_displaced"), "people", grp.displaced);
        ctx.report(&format!("quake.{k}_buildings_extensive_or_complete"), "buildings", grp.ext_comp());
        let ratio = |a: f64, b: f64| if b > 0.0 { a / b } else { f64::NAN };
        let rp = self.record_group.clone();
        ctx.report(&format!("fit.{k}_deaths_ratio"), "1", ratio(grp.deaths, rp.deaths));
        ctx.report(&format!("fit.{k}_displaced_ratio"), "1", ratio(grp.displaced, rp.displaced));
        ctx.report(&format!("fit.{k}_damage_ratio"), "1", ratio(grp.ext_comp(), rp.ext_comp()));
        ctx.report(&format!("fit.{k}_extensive_ratio"), "1", ratio(grp.ext, rp.ext));
        ctx.report(&format!("fit.{k}_complete_ratio"), "1", ratio(grp.comp, rp.comp));
        ctx.report(&format!("fit.{k}_injuries_ratio"), "1", ratio(grp.injuries, rp.injuries));
        // correlation across the units present in both
        let corr = |of: &dyn Fn(&Loss) -> f64| -> f64 {
            let pairs: Vec<(f64, f64)> = self.record_units.iter().filter_map(|r| by_unit.get(&r.id).map(|o| (of(o), of(r)))).collect();
            if pairs.len() < 3 { return f64::NAN; }
            let n = pairs.len() as f64; let (mx, my) = (pairs.iter().map(|p| p.0).sum::<f64>() / n, pairs.iter().map(|p| p.1).sum::<f64>() / n);
            let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
            for (x, y) in &pairs { sxy += (x - mx) * (y - my); sxx += (x - mx).powi(2); syy += (y - my).powi(2); }
            if sxx > 0.0 && syy > 0.0 { sxy / (sxx * syy).sqrt() } else { f64::NAN }
        };
        let (r_dmg, r_d) = (corr(&|u| u.ext_comp()), corr(&|u| u.deaths));
        ctx.report("fit.units_damage_r", "1", r_dmg);
        ctx.report("fit.units_deaths_r", "1", r_d);
        let c = |v: f64| crate::lifelines::commas(v);
        let study = if self.study.is_empty() { "the published study".to_string() } else { self.study.clone() };
        let whose = if self.agency.is_empty() { "the study's".to_string() } else { format!("{}'s", self.agency) };
        let sentence = format!("In {} the model counts {} dead where {} estimates {} for the same case, a ratio of {:.2}. Across the study's {}, the model's counts of buildings with extensive or complete damage correlate {:.2} with {}.",
            self.label, c(grp.deaths), study, c(rp.deaths), ratio(grp.deaths, rp.deaths), if self.units_label.is_empty() { "small areas" } else { self.units_label.as_str() }, r_dmg, whose);
        ctx.publish("fit.crossing", json!({ "id": "fit", "title": "The comparison", "name": "the study", "kind": "fit", "time": ctx.now().to_iso(), "sentence": sentence, "source": self.source, "confidence": "a comparison with a published study, not with an event" }));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Losses = serde_json::from_value(v).unwrap(); o.load().expect("losses reload"); *self = o; }
}
