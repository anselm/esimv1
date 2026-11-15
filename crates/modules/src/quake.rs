//! The Cascadia earthquake on a city: two kinds.
//!
//! `quake.ground` holds a published scenario's ground motion and ground failure for the window
//! (DOGAMI's site PGA and PGV, its liquefaction and landslide probability and deformation, cut
//! to the window in `data/events/<folder>/`), writes them as world fields, and at the stated
//! instant publishes `quake.rupture`. It computes nothing about the earthquake itself: there is
//! no record of this event, and the state's synthetic motions are the published estimate used.
//!
//! `quake.buildings` turns that motion into damage, deaths and displaced people on the cells:
//! a lognormal fragility in PGA for a generic building mix, ground failure from the liquefaction
//! and landslide layers, Hazus-style casualty rates by damage state. The fragility medians and
//! the casualty rates are recalled from the Hazus earthquake technical manual (FEMA), which could
//! not be downloaded on 2026-11-12, so each is declared `recalled`; six parameters are fitted to
//! DOGAMI's own Hazus run for the same ground on saturated soil by day (the record in
//! `loss_unit_*.csv`), and the fit is reported. The daytime toll on saturated soil is close to
//! DOGAMI's by construction. The pattern of damage by neighbourhood follows DOGAMI's. The
//! night-time and dry-soil tolls do not, because only shaking makes a building collapse here.

use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Describe, Duration, Event, LatLonGrid, Module, Time};
use esim_registry::{cog, dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

pub(crate) fn root() -> PathBuf { std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")) }

/// A window raster from the event folder sampled onto the run grid at each cell's centre
/// (nearest pixel). NaN where the file has no data.
pub(crate) fn sample(folder: &str, file: &str, g: &LatLonGrid) -> Result<Vec<f32>, String> {
    let path = root().join("data/events").join(folder).join(file);
    let r = cog::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = vec![f32::NAN; g.nx * g.ny];
    for row in 0..g.ny {
        let lat = g.lat(row);
        let rr = ((r.y0 - lat) / r.dy).floor();
        if rr < 0.0 || rr >= r.height as f64 { continue; }
        for col in 0..g.nx {
            let lon = g.lon(col);
            let cc = ((lon - r.x0) / r.dx).floor();
            if cc < 0.0 || cc >= r.width as f64 { continue; }
            let v = r.get(cc as usize, rr as usize);
            let bad = v.is_nan() || v.abs() > 1e30 || r.nodata.map_or(false, |nd| (v - nd).abs() < 1e-6 * nd.abs().max(1.0));
            out[row * g.nx + col] = if bad { f32::NAN } else { v };
        }
    }
    Ok(out)
}

pub(crate) fn window_grid(lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64) -> Result<(LatLonGrid, Vec<f32>), String> {
    let root = root();
    let cat = Catalog::load(&root.join("data/catalog.toml"))?;
    let dem = cat.asset("copernicus-dem-glo30").ok_or("catalog has no copernicus-dem-glo30")?;
    let mut fe = Fetcher::default_for(&root)?;
    let w = dem_window(&mut fe, dem, lat_s, lat_n, lon_w, lon_e, res_arcsec)?;
    Ok((LatLonGrid { nx: w.nx, ny: w.ny, dlon: w.dlon, dlat: w.dlat, lon_w: w.lon_w, lat_n: w.lat_n }, w.elev))
}

/// A binomial draw: how many of n things fail at chance p, by the given uniforms (exact under 64, normal above).
pub(crate) fn binom(n: f64, p: f64, u: &mut dyn FnMut() -> f64) -> f64 {
    if n <= 0.0 || p <= 0.0 { return 0.0; }
    if p >= 1.0 { return n.round(); }
    let ni = n.round() as i64;
    if ni <= 64 { let mut k = 0; for _ in 0..ni { if u() < p { k += 1; } } return k as f64; }
    let (m, sd) = (n * p, (n * p * (1.0 - p)).sqrt());
    let (a, b) = (u().max(1e-12), u());
    let z = (-2.0 * a.ln()).sqrt() * (2.0 * std::f64::consts::PI * b).cos();
    (m + sd * z).round().clamp(0.0, n.round())
}
pub(crate) fn cell_of(g: &LatLonGrid, lat: f64, lon: f64) -> usize {
    let c = (((lon - g.lon_w) / g.dlon).floor().max(0.0) as usize).min(g.nx - 1);
    let r = (((g.lat_n - lat) / g.dlat).floor().max(0.0) as usize).min(g.ny - 1);
    r * g.nx + c
}

/// Standard normal CDF (Abramowitz and Stegun 26.2.17, recalled; error under 1e-7).
pub(crate) fn phi(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.2316419 * x.abs());
    let d = 0.3989422804014327 * (-x * x / 2.0).exp();
    let p = d * t * (0.319381530 + t * (-0.356563782 + t * (1.781477937 + t * (-1.821255978 + t * 1.330274429))));
    if x >= 0.0 { 1.0 - p } else { p }
}

// ---------------------------------------------------------------------------------------
// quake.ground

#[derive(Serialize, Deserialize)]
pub struct Ground {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    folder: String,
    at: String,
    magnitude: f64,
    duration_s: f64,
    soils: String,
    tz_offset_h: f64, tz_name: String,
    #[serde(default)] event: String, #[serde(default)] agency: String, #[serde(default)] source: String, #[serde(default)] motion_source: String,
    fired: bool,
    scheduled: bool,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] elev: Vec<f32>,
    #[serde(skip)] pga: Vec<f32>, #[serde(skip)] pgv: Vec<f32>,
    #[serde(skip)] liq_p: Vec<f32>, #[serde(skip)] liq_pgd: Vec<f32>,
    #[serde(skip)] ls_p: Vec<f32>, #[serde(skip)] ls_pgd: Vec<f32>,
}

impl Ground {
    pub fn new(name: &str, p: &toml::Value) -> Result<Ground, String> {
        let mut o = Ground {
            name: name.to_string(),
            lat_s: crate::fragility::need_f64(p, "lat_s", name)?, lat_n: crate::fragility::need_f64(p, "lat_n", name)?, lon_w: crate::fragility::need_f64(p, "lon_w", name)?, lon_e: crate::fragility::need_f64(p, "lon_e", name)?,
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "").to_string(),
            at: param_str(p, "at", "2027-01-26T22:00:00Z").to_string(),
            event: String::new(), agency: String::new(), source: String::new(), motion_source: String::new(),
            magnitude: param_f64(p, "magnitude", 9.0),
            duration_s: param_f64(p, "duration_s", 240.0),
            soils: param_str(p, "soils", "wet").to_string(),
            tz_offset_h: param_f64(p, "tz_offset_h", -8.0), tz_name: param_str(p, "tz_name", "PST").to_string(),
            fired: false, scheduled: false,
            grid: None, elev: Vec::new(), pga: Vec::new(), pgv: Vec::new(), liq_p: Vec::new(), liq_pgd: Vec::new(), ls_p: Vec::new(), ls_pgd: Vec::new(),
        };
        if o.soils != "wet" && o.soils != "dry" { return Err("quake.ground: soils must be wet or dry".into()); }
        Time::parse(&o.at)?;
        o.load()?;
        Ok(o)
    }
    fn load(&mut self) -> Result<(), String> {
        let (g, elev) = window_grid(self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let n = g.nx * g.ny;
        // which file holds which map, and whose maps they are, is the event folder's to say
        let meta = Self::meta(&self.folder)?;
        let st = |k: &str| meta.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
        self.event = st("event"); self.agency = st("agency"); self.source = st("source"); self.motion_source = st("motion_source");
        let files = meta.get("files").ok_or("ground.toml has no [files]")?;
        let file = |t: &toml::Value, k: &str| t.get(k).and_then(|v| v.as_str()).map(String::from);
        let need = |k: &str| file(files, k).ok_or(format!("ground.toml [files] has no {k}"));
        self.pga = sample(&self.folder, &need("pga")?, &g)?;
        self.pgv = sample(&self.folder, &need("pgv")?, &g)?;
        let pct = |v: Vec<f32>| -> Vec<f32> { v.into_iter().map(|x| if x.is_nan() { x } else { (x / 100.0).clamp(0.0, 1.0) }).collect() };
        // a map the folder does not name for these soils is zero (a study may publish no dry liquefaction layer)
        let soil = files.get(self.soils.as_str()).cloned().unwrap_or(toml::Value::Table(Default::default()));
        let map = |k: &str| -> Result<Vec<f32>, String> { match file(&soil, k) { Some(f) => sample(&self.folder, &f, &g), None => Ok(vec![0.0; n]) } };
        let (lp, lpgd, sp, spgd) = (map("liq_p")?, map("liq_pgd")?, map("ls_p")?, map("ls_pgd")?);
        self.liq_p = pct(lp); self.liq_pgd = lpgd; self.ls_p = pct(sp); self.ls_pgd = spgd;
        self.elev = elev;
        self.grid = Some(g);
        Ok(())
    }
    fn meta(folder: &str) -> Result<toml::Value, String> {
        let path = root().join("data/events").join(folder).join("ground.toml");
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        text.parse::<toml::Value>().map_err(|e| format!("ground.toml: {e}"))
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let soils = param_str(p, "soils", "wet");
        let folder = param_str(p, "folder", "");
        let meta = Self::meta(folder).ok();
        let st = |k: &str| meta.as_ref().and_then(|m| m.get(k)).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let (source, method, about) = (st("source"), st("method"), st("note"));
        let mut summary = String::from("This module supplies the ground motion for a scenario earthquake. It does not simulate the fault or the shaking. It reads published maps from an event folder: the peak acceleration and velocity of the ground at the surface, and the probability and extent of liquefaction and landslides, for saturated or dry soil. A file in the folder, ground.toml, names the maps and their source. At the moment set for the earthquake, the module makes the maps available to the other modules and announces the rupture. A scenario earthquake has not happened, so there are no measurements to compare the maps with.");
        if !source.is_empty() { summary.push_str(&format!(" In this run the maps are from {source}.")); }
        if !method.is_empty() { summary.push(' '); summary.push_str(&method); }
        let d = Describe::new("quake.ground", name, &summary)
            .priority(0).dt("once, at the rupture")
            .param("folder", folder, "", "the folder under data/events that holds this scenario's maps, a ground.toml naming them, and a README that says how they were made")
            .param("tz_offset_h", param_f64(p, "tz_offset_h", -8.0), "h", "local time's offset from UTC in hours (PST -8, PDT -7), used only to print local times")
            .param("tz_name", param_str(p, "tz_name", "PST"), "", "the name of the local time zone, used only in printed times")
            .param("at", param_str(p, "at", "2027-01-26T22:00:00Z"), "", "the moment of the earthquake in UTC, a scenario choice and not a prediction")
            .param("magnitude", param_f64(p, "magnitude", 9.0), "Mw", "the magnitude of the scenario the maps were made for")
            .param("duration_s", param_f64(p, "duration_s", 240.0), "s", "seconds of strong shaking, as the source study gives it")
            .param("soils", soils, "", "wet: the maps for saturated soil, usually the higher-loss case, with liquefaction and landslides. dry: the maps for dry soil")
            .own("quake.pga_g", "g").own("quake.pgv_cms", "cm/s").own("quake.liq_p", "1").own("quake.liq_pgd_cm", "cm").own("quake.ls_p", "1").own("quake.ls_pgd_cm", "cm")
            .publish("quake.rupture", "{ magnitude, duration_s, soils, at, source } once, at the instant")
            .publish("quake.crossing", "{ kind: rupture, sentence, source, confidence } the rupture, for the story")
            .frame("terrain_elev_m", "m").frame("quake_pga_g", "g").frame("quake_mmi", "intensity").frame("quake_liquefaction_p", "1").frame("quake_ground_pgd_cm", "cm")
            .report("quake.pga_mean_g", "g").report("quake.pga_max_g", "g").report("quake.pgv_mean_cms", "cm/s").report("quake.liq_p_mean", "1").report("quake.cells_over_liq_20", "cells")
            .asset("copernicus-dem-glo30", "the terrain under the city").asset(&format!("events/{folder}"), "the scenario's ground motion and ground failure maps, cut to the modelled area")
            .view("map", "peak ground acceleration", "quake_pga_g", "the source study's estimate at the surface, as a fraction of g, the acceleration of gravity")
            .view("map", "intensity", "quake_mmi", "Modified Mercalli intensity, computed from peak ground velocity (PGV) with the relation of Wald and others 1999, recalled from memory: 3.47 log PGV + 2.35")
            .view("map", "chance of liquefaction", "quake_liquefaction_p", "the source study's probability that the ground spreads sideways as saturated soil liquefies; zero where the study gives no map for these soils")
            .view("map", "permanent ground deformation", "quake_ground_pgd_cm", "the larger of the permanent ground movement from liquefaction and from landslides, in centimetres");
        if about.is_empty() { d } else { d.note(&about) }
    }
    fn mmi(pgv: f32) -> f32 { if pgv.is_nan() || pgv <= 0.0 { f32::NAN } else { (3.47 * (pgv as f64).log10() + 2.35) as f32 } }
}

impl Module for Ground {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        let g = self.grid.clone().expect("grid");
        ctx.world_write("quake.pga_g", "g", &g, self.pga.clone());
        ctx.world_write("quake.pgv_cms", "cm/s", &g, self.pgv.clone());
        ctx.world_write("quake.liq_p", "1", &g, self.liq_p.clone());
        ctx.world_write("quake.liq_pgd_cm", "cm", &g, self.liq_pgd.clone());
        ctx.world_write("quake.ls_p", "1", &g, self.ls_p.clone());
        ctx.world_write("quake.ls_pgd_cm", "cm", &g, self.ls_pgd.clone());
        ctx.field("terrain_elev_m", "m", &g, self.elev.clone());
        ctx.field("quake_liquefaction_p", "1", &g, self.liq_p.clone());
        ctx.field("quake_ground_pgd_cm", "cm", &g, self.liq_pgd.iter().zip(&self.ls_pgd).map(|(&a, &b)| if a.is_nan() { b } else if b.is_nan() { a } else { a.max(b) }).collect());
        let ok: Vec<f32> = self.pga.iter().copied().filter(|v| !v.is_nan()).collect();
        let mean = |v: &[f32]| if v.is_empty() { 0.0 } else { v.iter().map(|&x| x as f64).sum::<f64>() / v.len() as f64 };
        ctx.report("quake.pga_mean_g", "g", mean(&ok));
        ctx.report("quake.pga_max_g", "g", ok.iter().copied().fold(0.0f32, f32::max) as f64);
        let pv: Vec<f32> = self.pgv.iter().copied().filter(|v| !v.is_nan()).collect();
        ctx.report("quake.pgv_mean_cms", "cm/s", mean(&pv));
        let lp: Vec<f32> = self.liq_p.iter().copied().filter(|v| !v.is_nan()).collect();
        ctx.report("quake.liq_p_mean", "1", mean(&lp));
        ctx.report("quake.cells_over_liq_20", "cells", lp.iter().filter(|&&v| v >= 0.2).count() as f64);
        if !self.scheduled {
            let at = Time::parse(&self.at).expect("at");
            let d = at.since(ctx.now());
            ctx.schedule_with_priority(Duration::secs(d.0.max(0)), 0, "rupture", json!(null));
            self.scheduled = true;
        }
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "rupture" || self.fired { return; }
        self.fired = true;
        let now = ctx.now();
        // the motion maps are written now, not at the start: before this instant there is nothing to map
        let g = self.grid.clone().expect("grid");
        ctx.field("quake_pga_g", "g", &g, self.pga.clone());
        ctx.field("quake_mmi", "intensity", &g, self.pgv.iter().map(|&v| Self::mmi(v)).collect());
        let ok: Vec<f32> = self.pga.iter().copied().filter(|v| !v.is_nan()).collect();
        let (lo, hi) = (ok.iter().copied().fold(f32::INFINITY, f32::min), ok.iter().copied().fold(0.0f32, f32::max));
        let local = Time(now.0 + (self.tz_offset_h * 3600.0).round() as i64).to_iso();
        let local_words = format!("{} {}", &local[..16].replace('T', " "), self.tz_name);
        let sentence = format!("{} {}: a magnitude {:.1} earthquake ruptures {}. Strong shaking lasts about {:.0} minutes. Peak ground acceleration across the modelled area ranges from {:.2} to {:.2} g in {} estimate for {} soil.", &local[..16].replace('T', " "), self.tz_name, self.magnitude, if self.event.is_empty() { "the fault" } else { self.event.as_str() }, self.duration_s / 60.0, lo, hi, if self.agency.is_empty() { "the source study's".to_string() } else { format!("{}'s", self.agency) }, if self.soils == "dry" { "dry" } else { "saturated" });
        ctx.publish("quake.rupture", json!({ "magnitude": self.magnitude, "duration_s": self.duration_s, "soils": self.soils, "at": now.to_iso(), "source": self.motion_source }));
        ctx.publish("quake.crossing", json!({ "id": "rupture", "title": "The rupture", "name": "the earthquake", "kind": "rupture", "time": now.to_iso(), "local": local_words, "sentence": sentence, "source": self.source, "confidence": "a published scenario, not a forecast" }));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Ground = serde_json::from_value(v).unwrap(); o.load().expect("ground reload"); *self = o; }
}



// ---------------------------------------------------------------------------------------
// quake.buildings

#[derive(Serialize, Deserialize)]
pub struct Buildings {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    folder: String,
    time_of_day: String,
    medians_g: [f64; 4],            // slight, moderate, extensive, complete
    beta: f64,
    fragility_scale: f64,
    gf_median_cm: f64, gf_beta: f64, gf_complete_share: f64,
    collapse_given_complete: f64,
    casualty_scale: f64,
    rates: [[f64; 4]; 5],           // per damage state (slight, moderate, extensive, complete, collapse): L1..L4 as fractions of occupants
    displaced_moderate_share: f64,
    displaced_share: f64,
    done: bool,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] count: Vec<f32>, #[serde(skip)] area: Vec<f32>,
    #[serde(skip)] residents: Vec<f32>, #[serde(skip)] occupants: Vec<f32>,
}

pub(crate) fn read_csv(path: &PathBuf) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut lines = text.lines();
    let head: Vec<String> = lines.next().ok_or("empty csv")?.split(',').map(|s| s.trim().to_string()).collect();
    let rows = lines.filter(|l| !l.trim().is_empty()).map(|l| l.split(',').map(|s| s.trim().to_string()).collect()).collect();
    Ok((head, rows))
}

impl Buildings {
    pub fn new(name: &str, p: &toml::Value) -> Result<Buildings, String> {
        let tod = param_str(p, "time_of_day", "day").to_string();
        if tod != "day" && tod != "night" { return Err("quake.buildings: time_of_day must be day or night".into()); }
        let mut o = Buildings {
            name: name.to_string(),
            lat_s: crate::fragility::need_f64(p, "lat_s", name)?, lat_n: crate::fragility::need_f64(p, "lat_n", name)?, lon_w: crate::fragility::need_f64(p, "lon_w", name)?, lon_e: crate::fragility::need_f64(p, "lon_e", name)?,
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "").to_string(),
            time_of_day: tod,
            medians_g: [param_f64(p, "median_slight_g", 0.20), param_f64(p, "median_moderate_g", 0.40), param_f64(p, "median_extensive_g", 0.80), param_f64(p, "median_complete_g", 1.50)],
            beta: param_f64(p, "beta", 0.70),
            fragility_scale: param_f64(p, "fragility_scale", 1.0),
            gf_median_cm: param_f64(p, "gf_median_cm", 152.0), gf_beta: param_f64(p, "gf_beta", 1.2), gf_complete_share: param_f64(p, "gf_complete_share", 0.2),
            collapse_given_complete: param_f64(p, "collapse_given_complete", 0.08),
            casualty_scale: param_f64(p, "casualty_scale", 1.0),
            rates: [[0.0005, 0.0, 0.0, 0.0], [0.002, 0.0002, 0.0, 0.0], [0.01, 0.001, 0.00001, 0.00001], [0.05, 0.01, 0.0001, 0.0001], [0.40, 0.20, 0.03, 0.05]],
            displaced_moderate_share: param_f64(p, "displaced_moderate_share", 0.0),
            displaced_share: param_f64(p, "displaced_share", 1.0),
            done: false,
            grid: None, count: Vec::new(), area: Vec::new(), residents: Vec::new(), occupants: Vec::new(),
        };
        o.load()?;
        Ok(o)
    }
    fn load(&mut self) -> Result<(), String> {
        let (g, _) = window_grid(self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let z = |v: Vec<f32>| -> Vec<f32> { v.into_iter().map(|x| if x.is_nan() { 0.0 } else { x }).collect() };
        self.count = z(sample(&self.folder, "buildings_count.tif", &g)?);
        self.area = z(sample(&self.folder, "building_area_m2.tif", &g)?);
        self.residents = z(sample(&self.folder, "people_resident.tif", &g)?);
        self.occupants = z(sample(&self.folder, if self.time_of_day == "day" { "people_day.tif" } else { "people_night.tif" }, &g)?);
        self.grid = Some(g);
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let folder = param_str(p, "folder", "");
        let d = Describe::new("quake.buildings", name, "This module estimates building damage, deaths, injuries and displaced people in each cell of the map, from the ground module's shaking. For each cell it uses a fragility curve, which gives the probability that a building reaches each of four damage states at a given peak ground acceleration. One set of curves stands for a generic mix of building types. The module adds the buildings lost where the ground liquefies or slides, then applies a casualty rate to each damage state. This is the approach of Hazus, the loss-estimation software of the Federal Emergency Management Agency. The buildings, the residents and the occupants by time of day are maps in the event folder. The default values of the curves and of the casualty rates are recalled from memory and have not been checked against the Hazus manual. A scenario may adjust the scales below to fit a published loss study, and a run that carries an obs.losses module reports how close the result is to that study. In this model only shaking makes a building collapse, and nearly all deaths are in collapsed buildings. The toll therefore changes less between day and night, and between saturated and dry soil, than a Hazus run's does.")
            .priority(1).dt("once, on the rupture")
            .param("folder", folder, "", "the folder under data/events that holds this scenario's building inventory: buildings per cell, floor area, residents, and occupants by day and by night")
            .param("time_of_day", param_str(p, "time_of_day", "day"), "", "day: people where the inventory places them by day. night: where it places them at night")
            .param("median_slight_g", param_f64(p, "median_slight_g", 0.20), "g", "the peak ground acceleration at which half the buildings reach slight damage, for the generic building mix. Recalled from the Hazus tables for moderate-code wood and older masonry; the mix is a guess")
            .param("median_moderate_g", param_f64(p, "median_moderate_g", 0.40), "g", "the same for moderate damage (recalled)")
            .param("median_extensive_g", param_f64(p, "median_extensive_g", 0.80), "g", "the same for extensive damage (recalled)")
            .param("median_complete_g", param_f64(p, "median_complete_g", 1.50), "g", "the same for complete damage (recalled)")
            .param("beta", param_f64(p, "beta", 0.70), "", "the spread of the fragility curves, a lognormal dispersion. Hazus uses about 0.64 for most building classes (recalled)")
            .param("fragility_scale", param_f64(p, "fragility_scale", 1.0), "", "divides the four accelerations above. A scenario may fit it so that the count of buildings with extensive or complete damage matches a published study")
            .param("gf_median_cm", param_f64(p, "gf_median_cm", 152.0), "cm", "the permanent ground movement at which half the buildings on failed ground take extensive or complete damage. Hazus gives 60 inches for lateral spreading (recalled)")
            .param("gf_beta", param_f64(p, "gf_beta", 1.2), "", "the spread of that curve (Hazus 1.2, recalled)")
            .param("gf_complete_share", param_f64(p, "gf_complete_share", 0.2), "", "the share of ground-failure damage counted as complete (Hazus 0.2 for lateral spreading, recalled)")
            .param("collapse_given_complete", param_f64(p, "collapse_given_complete", 0.08), "", "the share of completely damaged buildings that collapse, for the generic mix. Hazus gives 3 percent for wood and 15 percent for unreinforced masonry; the mix is a guess")
            .param("casualty_scale", param_f64(p, "casualty_scale", 1.0), "", "multiplies every casualty rate. A scenario may fit it so that deaths match a published study")
            .param("displaced_moderate_share", param_f64(p, "displaced_moderate_share", 0.0), "", "the share of residents of moderately damaged buildings counted as displaced. Hazus counts some of them; 0 here is a guess")
            .param("displaced_share", param_f64(p, "displaced_share", 1.0), "", "the share of residents of buildings with extensive or complete damage counted as displaced for the long term. A scenario may fit it to a published study")
            .subscribe("quake.rupture", "the instant")
            .read_on("quake.pga_g", "g", true, "quake.rupture", "").read_on("quake.liq_p", "1", true, "quake.rupture", "").read_on("quake.liq_pgd_cm", "cm", true, "quake.rupture", "").read_on("quake.ls_p", "1", true, "quake.rupture", "").read_on("quake.ls_pgd_cm", "cm", true, "quake.rupture", "")
            .publish("quake.crossing", "{ kind: toll, sentence } the sentence shown on the run page: deaths, injuries and displaced people from the shaking")
            .own("quake.cell_death_rate", "1").own("quake.cell_displaced_rate", "1").own("quake.cell_collapse_frac", "1").own("quake.cell_death_rate_collapse", "1").own("quake.cell_death_rate_other", "1")
            .own("quake.cell_extensive", "buildings").own("quake.cell_complete", "buildings").own("quake.cell_deaths", "people").own("quake.cell_displaced", "people").own("quake.cell_injuries_hospital", "people")
            .publish("quake.toll", "{ deaths, injuries_hospital, injuries_life, injuries_minor, displaced } the toll in the modelled area as numbers, once the shaking ends, for the module that keeps the total")
            .frame("buildings_count", "buildings").frame("building_damage_frac", "1").frame("buildings_destroyed", "buildings").frame("buildings_collapsed", "buildings").frame("deaths", "people").frame("displaced", "people").frame("occupants", "people")
            .report("quake.buildings", "buildings").report("quake.buildings_moderate", "buildings").report("quake.buildings_extensive", "buildings").report("quake.buildings_complete", "buildings")
            .report("quake.buildings_collapsed", "buildings").report("quake.deaths_in_collapses", "people").report("quake.deaths", "people").report("quake.injuries_hospital", "people").report("quake.injuries_life_threatening", "people").report("quake.injuries_minor", "people").report("quake.displaced", "people").report("quake.occupants", "people")
            .asset(&format!("events/{folder}"), "the building inventory: buildings per cell, floor area, residents, occupants by day and by night")
            .view("map", "share of buildings with extensive or complete damage", "building_damage_frac", "the expected share in each cell")
            .view("map", "buildings collapsed", "buildings_collapsed", "counted building by building: in each cell a random draw decides how many buildings collapse, given the cell's probability of collapse")
            .view("map", "deaths", "deaths", "per cell, for people where they are at the time of the earthquake. Deaths in collapsed buildings follow the drawn collapses; the others are expected values")
            .view("map", "people displaced", "displaced", "a fitted share of the residents of buildings with extensive or complete damage")
            .view("counter", "buildings collapsed", "quake.buildings_collapsed", "from the random draw in each cell")
            .view("counter", "dead in collapsed buildings", "quake.deaths_in_collapses", "at the casualty rate for collapsed buildings")
            .view("counter", "deaths", "quake.deaths", "in the modelled area. Deaths in collapsed buildings follow the drawn collapses; the others are expected values")
            .view("counter", "injuries needing a hospital", "quake.injuries_hospital", "Hazus severity levels 2 and 3")
            .view("counter", "people displaced", "quake.displaced", "")
            .view("counter", "buildings with extensive or complete damage", "quake.buildings_extensive", "the two worst of the four damage states")
            .note("The Hazus tables (FEMA, Earthquake Model Technical Manual) could not be downloaded to the development machine on 12 November 2026. The default fragility accelerations, their spread, the collapse share and the casualty rates are therefore recalled from memory and not checked. Where a scenario has fitted the scales to a published loss study, its toll is close to that study because it was made to be. The inventory gives a count of buildings and a number of people per cell, with no heights or structure types.");
        d
    }
}

impl Module for Buildings {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.subscribe("quake.rupture");
        let g = self.grid.clone().expect("grid");
        ctx.field("buildings_count", "buildings", &g, self.count.clone());
        ctx.field("occupants", "people", &g, self.occupants.clone());
        let n = g.nx * g.ny;
        for f in [("building_damage_frac", "1"), ("buildings_destroyed", "buildings"), ("buildings_collapsed", "buildings"), ("deaths", "people"), ("displaced", "people")] { ctx.field(f.0, f.1, &g, vec![0.0; n]); }
        ctx.report("quake.buildings", "buildings", self.count.iter().map(|&v| v as f64).sum());
        ctx.report("quake.occupants", "people", self.occupants.iter().map(|&v| v as f64).sum());
        for s in ["quake.buildings_moderate", "quake.buildings_extensive", "quake.buildings_complete", "quake.buildings_collapsed", "quake.deaths_in_collapses", "quake.deaths", "quake.injuries_hospital", "quake.injuries_life_threatening", "quake.injuries_minor", "quake.displaced"] { ctx.report(s, "", 0.0); }
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic == "quake.rupture" && !self.done {
            // the reckoning waits for the shaking to end; the frames before it are the zeros written at the start
            let d = ev.data.get("duration_s").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0) as i64;
            ctx.schedule_with_priority(Duration::secs(d), 1, "settle", json!(null));
            return;
        }
        if ev.topic != "settle" || self.done { return; }
        self.done = true;
        let g = self.grid.clone().expect("grid");
        let n = g.nx * g.ny;
        let take = |name: &str| -> Vec<f32> { ctx.world_read(name).map(|f| f.data.clone()).unwrap_or_else(|| vec![0.0; n]) };
        let (pga, liq_p, liq_pgd, ls_p, ls_pgd) = (take("quake.pga_g"), take("quake.liq_p"), take("quake.liq_pgd_cm"), take("quake.ls_p"), take("quake.ls_pgd_cm"));
        let mut frac = vec![f32::NAN; n]; let mut destroyed = vec![0.0f32; n]; let mut deaths = vec![0.0f32; n]; let mut displaced = vec![0.0f32; n];
        // the collapses realised building by building, and the rates an agent in the cell needs
        let mut collapsed = vec![0.0f32; n]; let mut cf_f = vec![0.0f32; n]; let mut rc_f = vec![0.0f32; n]; let mut ro_f = vec![0.0f32; n];
        let (mut n_coll_tot, mut dead_in_coll) = (0.0f64, 0.0f64);
        let rc = (self.rates[4][3] * self.casualty_scale).min(1.0);
        let mut tot = [0.0f64; 5]; let (mut d_tot, mut l1, mut l23, mut l3, mut disp) = (0.0, 0.0, 0.0, 0.0, 0.0);
        let mut ext_f = vec![0.0f32; n]; let mut comp_f = vec![0.0f32; n]; let mut inj_f = vec![0.0f32; n];
        for i in 0..n {
            let c = self.count[i] as f64;
            if c <= 0.0 { continue; }
            let a = pga[i] as f64;
            if a.is_nan() || a <= 0.0 { continue; }
            // shaking: exceedance of each state, then the state probabilities
            let ex: Vec<f64> = self.medians_g.iter().map(|m| phi((a / (m / self.fragility_scale)).ln() / self.beta)).collect();
            let mut ps = [1.0 - ex[0], ex[0] - ex[1], ex[1] - ex[2], ex[2] - ex[3], ex[3]];
            // ground failure: liquefaction and landslide each give an independent chance of extensive-or-complete
            let gf = |p: f32, pgd: f32| -> f64 { if p.is_nan() || pgd.is_nan() || p <= 0.0 || pgd <= 0.0 { 0.0 } else { p as f64 * phi((pgd as f64 / self.gf_median_cm).ln() / self.gf_beta) } };
            let pg = 1.0 - (1.0 - gf(liq_p[i], liq_pgd[i])) * (1.0 - gf(ls_p[i], ls_pgd[i]));
            // collapse, and its casualty rates, belong to shaking: a building the ground carried
            // away is complete without the pancake (Hazus applies no collapse rate to ground failure)
            let shake_complete = ps[4];
            if pg > 0.0 {
                // Hazus: shaking and ground failure independent; ground failure moves any state up to extensive or complete
                let moved: f64 = pg * (ps[0] + ps[1] + ps[2]);
                for k in 0..3 { ps[k] *= 1.0 - pg; }
                ps[4] += moved * self.gf_complete_share; ps[3] += moved * (1.0 - self.gf_complete_share);
            }
            let pc = ps[4]; let coll = shake_complete * self.collapse_given_complete;
            // the collapses realised: a binomial draw of this cell's buildings at the collapse share, by the seed,
            // so the dead come out of the buildings that fall and differ by seed; the expectation is unchanged
            let n_coll = binom(c, coll, &mut || ctx.rng().f64());
            let cf = if c > 0.0 { n_coll / c } else { 0.0 };
            let (mut p3, mut pcx) = (ps[3], pc.max(cf));
            if cf > pc { p3 = (p3 - (cf - pc)).max(0.0); }
            let ps5 = [ps[1], ps[2], p3, pcx - cf, cf];
            pcx = pcx.max(0.0); let _ = pcx;
            let occ = self.occupants[i] as f64; let res = self.residents[i] as f64;
            let mut dd = 0.0; let mut i1 = 0.0; let mut i23 = 0.0; let mut i3 = 0.0;
            for (k, p) in ps5.iter().enumerate() { let r = &self.rates[k]; i1 += occ * p * r[0]; i23 += occ * p * (r[1] + r[2]); i3 += occ * p * r[2]; dd += occ * p * r[3]; }
            dd *= self.casualty_scale; i1 *= self.casualty_scale; i23 *= self.casualty_scale; i3 *= self.casualty_scale;
            l3 += i3;
            let dc = (occ * cf * rc).min(occ); let ro = if occ > 0.0 && cf < 1.0 { ((dd - dc).max(0.0) / (occ * (1.0 - cf))).min(1.0) } else { 0.0 };
            collapsed[i] = n_coll as f32; cf_f[i] = cf as f32; rc_f[i] = rc as f32; ro_f[i] = ro as f32; n_coll_tot += n_coll; dead_in_coll += dc;
            let di = res * (self.displaced_share * (ps[3] + ps[4]) + self.displaced_moderate_share * ps[2]);
            frac[i] = (ps[3] + ps[4]) as f32; destroyed[i] = (c * (ps[3] + ps[4])) as f32; deaths[i] = dd as f32; displaced[i] = di as f32;
            for k in 0..5 { tot[k] += c * ps[k]; }
            d_tot += dd; l1 += i1; l23 += i23; disp += di;
            ext_f[i] = (c * ps[3]) as f32; comp_f[i] = (c * ps[4]) as f32; inj_f[i] = i23 as f32;
        }
        // the per-cell rates as world fields, for agents that stand at a cell
        let mut death_rate = vec![0.0f32; n]; let mut disp_rate = vec![0.0f32; n];
        for i in 0..n { let o = self.occupants[i]; let r = self.residents[i]; if o > 0.0 { death_rate[i] = (deaths[i] / o).min(1.0); } if r > 0.0 { disp_rate[i] = (displaced[i] / r).min(1.0); } }
        ctx.world_write("quake.cell_death_rate", "1", &g, death_rate); ctx.world_write("quake.cell_displaced_rate", "1", &g, disp_rate);
        ctx.world_write("quake.cell_collapse_frac", "1", &g, cf_f); ctx.world_write("quake.cell_death_rate_collapse", "1", &g, rc_f); ctx.world_write("quake.cell_death_rate_other", "1", &g, ro_f);
        // the per-cell results a scoring module sums over a study's areas
        ctx.world_write("quake.cell_extensive", "buildings", &g, ext_f); ctx.world_write("quake.cell_complete", "buildings", &g, comp_f);
        ctx.world_write("quake.cell_deaths", "people", &g, deaths.clone()); ctx.world_write("quake.cell_displaced", "people", &g, displaced.clone()); ctx.world_write("quake.cell_injuries_hospital", "people", &g, inj_f);
        ctx.field("buildings_collapsed", "buildings", &g, collapsed);
        ctx.report("quake.buildings_collapsed", "buildings", n_coll_tot); ctx.report("quake.deaths_in_collapses", "people", dead_in_coll);
        ctx.field("building_damage_frac", "1", &g, frac);
        ctx.field("buildings_destroyed", "buildings", &g, destroyed);
        ctx.field("deaths", "people", &g, deaths);
        ctx.field("displaced", "people", &g, displaced);
        ctx.report("quake.buildings_moderate", "buildings", tot[2]);
        ctx.report("quake.buildings_extensive", "buildings", tot[3] + tot[4]);
        ctx.report("quake.buildings_complete", "buildings", tot[4]);
        ctx.report("quake.deaths", "people", d_tot);
        ctx.report("quake.injuries_hospital", "people", l23);
        ctx.report("quake.injuries_minor", "people", l1);
        ctx.report("quake.displaced", "people", disp);
        let when = if self.time_of_day == "night" { "at night" } else { "by day" };
        let c = |v: f64| crate::lifelines::commas(v);
        let sentence = format!("About {} buildings in the modelled area take extensive or complete damage, {} of them complete, and {} collapse. About {} people die, {} of them in collapsed buildings. Another {} are injured badly enough to need a hospital and {} are displaced. The casualties are for people where they are {}. This is the buildings module's count by map cell. Where a run also has agents, the page's death total uses the agents' count.",
            c((tot[3] + tot[4]) as f64), c(tot[4] as f64), c(n_coll_tot as f64), c(d_tot as f64), c(dead_in_coll as f64), c(l23 as f64), c(disp as f64), when);
        ctx.report("quake.injuries_life_threatening", "people", l3);
        ctx.publish("quake.toll", json!({ "deaths": d_tot, "injuries_hospital": l23, "injuries_life": l3, "injuries_minor": l1, "displaced": disp }));
        ctx.publish("quake.crossing", json!({ "id": "toll", "title": "The toll", "name": "the modelled area", "kind": "toll", "time": ctx.now().to_iso(), "sentence": sentence, "source": "This model's fragility curves, which have the form Hazus uses, with default values recalled from memory and scales set in the manifest, applied to the ground module's motion and the event folder's building inventory", "confidence": "a model on unverified curves" }));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Buildings = serde_json::from_value(v).unwrap(); o.load().expect("buildings reload"); *self = o; }
}
