//! The Cascadia earthquake on a city: two kinds.
//!
//! `quake.ground` holds a published scenario's ground motion and ground failure for the window
//! (DOGAMI's site PGA and PGV, its liquefaction and landslide probability and deformation, cut
//! to the window in `data/events/<folder>/`), writes them as world fields, and at the stated
//! instant publishes `quake.rupture`. It computes nothing about the earthquake itself: there is
//! no record of this event and the state's synthetic motions are the best anyone has.
//!
//! `quake.buildings` turns that motion into damage, deaths and displaced people on the cells:
//! a lognormal fragility in PGA for a generic building mix, ground failure from the liquefaction
//! and landslide layers, Hazus-style casualty rates by damage state. The fragility medians and
//! the casualty rates are recalled from the Hazus earthquake technical manual (FEMA), which could
//! not be downloaded on 2026-11-12, so each is declared `recalled`; a `fragility_scale` and a
//! `casualty_scale` are fitted to DOGAMI's own Hazus run for the same ground (the record in
//! `loss_unit_*.csv`), and the fit is reported. What stands is the pattern by neighbourhood and
//! the day-night and wet-dry contrasts, which are DOGAMI's; the absolute toll is DOGAMI's too,
//! by construction.

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
            lat_s: param_f64(p, "lat_s", 45.43), lat_n: param_f64(p, "lat_n", 45.66), lon_w: param_f64(p, "lon_w", -122.85), lon_e: param_f64(p, "lon_e", -122.47),
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "portland-cascadia").to_string(),
            at: param_str(p, "at", "2027-01-26T22:00:00Z").to_string(),
            magnitude: param_f64(p, "magnitude", 9.0),
            duration_s: param_f64(p, "duration_s", 240.0),
            soils: param_str(p, "soils", "wet").to_string(),
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
        self.pga = sample(&self.folder, "CSZ_M9p0_pga_site.tif", &g)?;
        self.pgv = sample(&self.folder, "CSZ_M9p0_pgv_site.tif", &g)?;
        let pct = |v: Vec<f32>| -> Vec<f32> { v.into_iter().map(|x| if x.is_nan() { x } else { (x / 100.0).clamp(0.0, 1.0) }).collect() };
        if self.soils == "wet" {
            self.liq_p = pct(sample(&self.folder, "CSZ_M9p0_Prob_liquefaction_wet.tif", &g)?);
            self.liq_pgd = sample(&self.folder, "CSZ_M9p0_PGD_liquefaction_wet.tif", &g)?;
            self.ls_p = pct(sample(&self.folder, "CSZ_M9p0_Prob_landslide_wet.tif", &g)?);
            self.ls_pgd = sample(&self.folder, "CSZ_M9p0_PGD_landslide_wet.tif", &g)?;
        } else {
            // DOGAMI published no dry-season liquefaction layer: "liquefaction is much less likely"
            self.liq_p = vec![0.0; n]; self.liq_pgd = vec![0.0; n];
            self.ls_p = pct(sample(&self.folder, "CSZ_M9p0_Prob_landslide_dry.tif", &g)?);
            self.ls_pgd = sample(&self.folder, "CSZ_M9p0_PGD_landslide_dry.tif", &g)?;
        }
        self.elev = elev;
        self.grid = Some(g);
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let soils = param_str(p, "soils", "wet");
        Describe::new("quake.ground", name, "A published earthquake scenario's ground motion and ground failure on the window: DOGAMI's site peak ground acceleration and velocity for a simulated Cascadia M9.0 (synthetic bedrock motions from the USGS, the state's site response), its liquefaction and landslide probability and deformation for saturated or dry soils, read from the event folder and written as world fields; at the stated instant it publishes the rupture. Nothing about the earthquake is computed here: there is no record of this event, and the state's estimate is the best anyone has.")
            .priority(0).dt("once, at the rupture")
            .param("folder", param_str(p, "folder", "portland-cascadia"), "", "the event folder under data/events with the cut rasters and their README")
            .param("at", param_str(p, "at", "2027-01-26T22:00:00Z"), "", "the rupture instant (UTC); 26 January is the anniversary of the 1700 event, 14:00 Pacific is DOGAMI's daytime scenario")
            .param("magnitude", param_f64(p, "magnitude", 9.0), "Mw", "DOGAMI's and the USGS ensemble's scenario magnitude")
            .param("duration_s", param_f64(p, "duration_s", 240.0), "s", "strong shaking; DOGAMI: 3 to 5 minutes for a megathrust")
            .param("soils", soils, "", "wet (saturated, DOGAMI's higher-loss case: liquefaction and landslides) or dry (landslides only, the dry layers)")
            .own("quake.pga_g", "g").own("quake.pgv_cms", "cm/s").own("quake.liq_p", "1").own("quake.liq_pgd_cm", "cm").own("quake.ls_p", "1").own("quake.ls_pgd_cm", "cm")
            .publish("quake.rupture", "{ magnitude, duration_s, soils, at, source } once, at the instant")
            .publish("quake.crossing", "{ kind: rupture, sentence, source, confidence } the rupture, for the story")
            .frame("terrain_elev_m", "m").frame("quake_pga_g", "g").frame("quake_mmi", "intensity").frame("quake_liquefaction_p", "1").frame("quake_ground_pgd_cm", "cm")
            .report("quake.pga_mean_g", "g").report("quake.pga_max_g", "g").report("quake.pgv_mean_cms", "cm/s").report("quake.liq_p_mean", "1").report("quake.cells_over_liq_20", "cells")
            .asset("copernicus-dem-glo30", "the terrain under the city").asset(&format!("events/{}", param_str(p, "folder", "portland-cascadia")), "DOGAMI O-18-02's site motions and ground failure, cut to the window")
            .view("map", "peak ground acceleration", "quake_pga_g", "DOGAMI's site estimate for the M9.0; a fraction of g")
            .view("map", "intensity", "quake_mmi", "Modified Mercalli from the peak velocity by Wald and others 1999 (recalled): 3.47 log PGV + 2.35")
            .view("map", "chance of liquefaction", "quake_liquefaction_p", "DOGAMI's probability of lateral spread, saturated soils; zero in the dry variant")
            .view("map", "permanent ground deformation", "quake_ground_pgd_cm", "the larger of the liquefaction and landslide deformation, cm")
            .note("The rasters are DOGAMI Open-File Report O-18-02 (2018), cut to the window with GDAL on 2026-11-12 as the event folder's README says; the run reads the cut files. The USGS median ensemble ShakeMap (cszm9ensemble_se) gives MMI 7.2 and PGA 0.24 g at downtown Portland against DOGAMI's 0.15 to 0.32 g over the window; the USGS ground-failure product is far coarser (500 m) and reads near zero at the fuel hub where DOGAMI maps a high liquefaction chance, so DOGAMI is used.")
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
        let local = Time(now.0 - 8 * 3600).to_iso();
        let sentence = format!("{} Pacific standard time: a magnitude {:.1} rupture of the Cascadia subduction zone. Strong shaking for about {:.0} minutes; peak ground acceleration across the window {:.2} to {:.2} g on DOGAMI's site estimate, soils {}.", &local[..16].replace('T', " "), self.magnitude, self.duration_s / 60.0, lo, hi, self.soils);
        ctx.publish("quake.rupture", json!({ "magnitude": self.magnitude, "duration_s": self.duration_s, "soils": self.soils, "at": now.to_iso(), "source": "DOGAMI O-18-02 site ground motion; USGS synthetic bedrock motion" }));
        ctx.publish("quake.crossing", json!({ "id": "rupture", "title": "The rupture", "name": "Cascadia", "kind": "rupture", "time": now.to_iso(), "sentence": sentence, "source": "DOGAMI Open-File Report O-18-02 (2018)", "confidence": "a published scenario, not a forecast" }));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Ground = serde_json::from_value(v).unwrap(); o.load().expect("ground reload"); *self = o; }
}



// ---------------------------------------------------------------------------------------
// quake.buildings

#[derive(Clone, Serialize, Deserialize, Default)]
struct UnitLoss { nuid: i32, ext_comp: f64, ext: f64, comp: f64, deaths: f64, displaced: f64, injuries: f64 }

#[derive(Serialize, Deserialize)]
pub struct Buildings {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    folder: String,
    time_of_day: String,
    record: String,                 // wet | dry: which DOGAMI loss table to score against
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
    #[serde(skip)] nuid: Vec<f32>, #[serde(skip)] juris: Vec<f32>,
    #[serde(skip)] record_units: Vec<UnitLoss>,
    #[serde(skip)] record_portland: UnitLoss,
}

fn read_csv(path: &PathBuf) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
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
            lat_s: param_f64(p, "lat_s", 45.43), lat_n: param_f64(p, "lat_n", 45.66), lon_w: param_f64(p, "lon_w", -122.85), lon_e: param_f64(p, "lon_e", -122.47),
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            folder: param_str(p, "folder", "portland-cascadia").to_string(),
            time_of_day: tod,
            record: param_str(p, "record", "wet").to_string(),
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
            grid: None, count: Vec::new(), area: Vec::new(), residents: Vec::new(), occupants: Vec::new(), nuid: Vec::new(), juris: Vec::new(),
            record_units: Vec::new(), record_portland: UnitLoss::default(),
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
        self.nuid = sample(&self.folder, "nuid.tif", &g)?;
        self.juris = sample(&self.folder, "jurisdiction.tif", &g)?;
        // the record: DOGAMI's per-unit results for the matching soils and time of day
        let (head, rows) = read_csv(&root().join("data/events").join(&self.folder).join(format!("loss_unit_{}.csv", self.record)))?;
        let col = |n: &str| head.iter().position(|h| h == n).ok_or(format!("loss table lacks {n}"));
        let (cn, ce, cc, cd, cl4, cl2, cl3) = (col("NUID")?, col("PDsExtensive")?, col("PDsComplete")?, col("DisplacedPop")?,
            col(if self.time_of_day == "day" { "CasDayL4" } else { "CasNightL4" })?, col(if self.time_of_day == "day" { "CasDayL2" } else { "CasNightL2" })?, col(if self.time_of_day == "day" { "CasDayL3" } else { "CasNightL3" })?);
        let f = |r: &Vec<String>, i: usize| r.get(i).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
        self.record_units = rows.iter().map(|r| UnitLoss { nuid: f(r, cn) as i32, ext_comp: f(r, ce) + f(r, cc), ext: f(r, ce), comp: f(r, cc), deaths: f(r, cl4), displaced: f(r, cd), injuries: f(r, cl2) + f(r, cl3) }).collect();
        // Portland's own figure from the jurisdiction table (the nine district coalitions)
        let (jh, jrows) = read_csv(&root().join("data/events").join(&self.folder).join(format!("loss_jurisdiction_{}.csv", self.record)))?;
        let jcol = |n: &str| jh.iter().position(|h| h == n).ok_or(format!("jurisdiction table lacks {n}"));
        let (jn, je, jc, jd, jl4, jl2, jl3) = (jcol("Jurisdiction")?, jcol("PDsExtensive")?, jcol("PDsComplete")?, jcol("DisplacedPop")?,
            jcol(if self.time_of_day == "day" { "CasDayL4" } else { "CasNightL4" })?, jcol(if self.time_of_day == "day" { "CasDayL2" } else { "CasNightL2" })?, jcol(if self.time_of_day == "day" { "CasDayL3" } else { "CasNightL3" })?);
        let mut pdx = UnitLoss::default();
        for r in &jrows { if r.get(jn).map_or(false, |s| s.starts_with("Portland")) { pdx.ext_comp += f(r, je) + f(r, jc); pdx.ext += f(r, je); pdx.comp += f(r, jc); pdx.deaths += f(r, jl4); pdx.displaced += f(r, jd); pdx.injuries += f(r, jl2) + f(r, jl3); } }
        self.record_portland = pdx;
        self.grid = Some(g);
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let folder = param_str(p, "folder", "portland-cascadia");
        let mut d = Describe::new("quake.buildings", name, "Damage, deaths and displaced people on the cells from the ground module's motion: a lognormal fragility in peak ground acceleration for a generic building mix (four damage states), ground failure from the liquefaction and landslide layers, and casualty rates by damage state in the Hazus manner; occupants by time of day and residents per cell from DOGAMI's own inventory. The fragility medians and the casualty rates are recalled, not read, and two scales are fitted to DOGAMI's Hazus results for the same ground, which the module reports as the fit; what stands is the pattern across neighbourhoods and the day-night and wet-dry contrasts.")
            .priority(1).dt("once, on the rupture")
            .param("folder", folder, "", "the event folder with the inventory and the loss tables")
            .param("time_of_day", param_str(p, "time_of_day", "day"), "", "day (DOGAMI's 2 PM occupants) or night (2 AM)")
            .param("record", param_str(p, "record", "wet"), "", "which DOGAMI table to score against: wet or dry, matching the ground module's soils")
            .param("median_slight_g", param_f64(p, "median_slight_g", 0.20), "g", "fragility median for slight damage, generic mix (recalled from the Hazus equivalent-PGA tables for moderate-code wood and older masonry; a guess at the mix)")
            .param("median_moderate_g", param_f64(p, "median_moderate_g", 0.40), "g", "moderate (recalled, as above)")
            .param("median_extensive_g", param_f64(p, "median_extensive_g", 0.80), "g", "extensive (recalled, as above)")
            .param("median_complete_g", param_f64(p, "median_complete_g", 1.50), "g", "complete (recalled, as above)")
            .param("beta", param_f64(p, "beta", 0.70), "", "lognormal dispersion (Hazus uses about 0.64 for most classes; recalled)")
            .param("fragility_scale", param_f64(p, "fragility_scale", 1.0), "", "the medians are divided by this: fitted so the window's extensive-or-complete count matches DOGAMI's (the fit is reported)")
            .param("gf_median_cm", param_f64(p, "gf_median_cm", 152.0), "cm", "ground-failure fragility median for extensive-or-complete damage from lateral spread (Hazus: 60 inches, recalled)")
            .param("gf_beta", param_f64(p, "gf_beta", 1.2), "", "its dispersion (Hazus 1.2, recalled)")
            .param("gf_complete_share", param_f64(p, "gf_complete_share", 0.2), "", "the share of ground-failure damage counted complete (Hazus 0.2 for lateral spread, recalled)")
            .param("collapse_given_complete", param_f64(p, "collapse_given_complete", 0.08), "", "share of complete damage that is collapse, generic mix (Hazus: 3 percent wood, 15 percent unreinforced masonry; a guess at the mix)")
            .param("casualty_scale", param_f64(p, "casualty_scale", 1.0), "", "multiplies every casualty rate: fitted so deaths in Portland match DOGAMI's (reported)")
            .param("displaced_moderate_share", param_f64(p, "displaced_moderate_share", 0.0), "", "share of residents in moderately damaged buildings counted displaced (Hazus counts a share; 0 here, a guess)")
            .param("displaced_share", param_f64(p, "displaced_share", 1.0), "", "share of residents in extensively or completely damaged buildings counted displaced long-term: fitted to DOGAMI's displaced population (reported)")
            .subscribe("quake.rupture", "the instant")
            .read_on("quake.pga_g", "g", true, "quake.rupture", "").read_on("quake.liq_p", "1", true, "quake.rupture", "").read_on("quake.liq_pgd_cm", "cm", true, "quake.rupture", "").read_on("quake.ls_p", "1", true, "quake.rupture", "").read_on("quake.ls_pgd_cm", "cm", true, "quake.rupture", "")
            .publish("quake.crossing", "{ kind: toll, sentence } deaths, injuries and displaced on the rupture; the fit against DOGAMI in the sentence")
            .own("quake.cell_death_rate", "1").own("quake.cell_displaced_rate", "1")
            .publish("quake.toll", "{ deaths, injuries_hospital, injuries_life, injuries_minor, displaced } the window's toll as numbers, once the shaking ends, for whoever keeps a total")
            .frame("buildings_count", "buildings").frame("building_damage_frac", "1").frame("buildings_destroyed", "buildings").frame("deaths", "people").frame("displaced", "people").frame("occupants", "people")
            .report("quake.buildings", "buildings").report("quake.buildings_moderate", "buildings").report("quake.buildings_extensive", "buildings").report("quake.buildings_complete", "buildings")
            .report("quake.deaths", "people").report("quake.injuries_hospital", "people").report("quake.injuries_life_threatening", "people").report("quake.injuries_minor", "people").report("quake.displaced", "people").report("quake.occupants", "people")
            .report("quake.portland_deaths", "people").report("quake.portland_displaced", "people").report("quake.portland_buildings_extensive_or_complete", "buildings")
            .report("fit.portland_deaths_ratio", "1").report("fit.portland_displaced_ratio", "1").report("fit.portland_damage_ratio", "1").report("fit.portland_extensive_ratio", "1").report("fit.portland_complete_ratio", "1").report("fit.portland_injuries_ratio", "1").report("fit.units_damage_r", "1").report("fit.units_deaths_r", "1")
            .asset(&format!("events/{folder}"), "DOGAMI's footprints, people per unit and Hazus loss tables")
            .view("map", "share of buildings extensively damaged or worse", "building_damage_frac", "expected, per cell")
            .view("map", "deaths", "deaths", "expected, per cell, at the stated time of day")
            .view("map", "people displaced", "displaced", "residents of buildings extensively or completely damaged")
            .view("counter", "deaths", "quake.deaths", "expected, in the window")
            .view("counter", "injuries needing a hospital", "quake.injuries_hospital", "Hazus levels 2 and 3")
            .view("counter", "people displaced", "quake.displaced", "")
            .view("counter", "buildings destroyed or extensively damaged", "quake.buildings_extensive", "extensive plus complete")
            .view("counter", "deaths, Portland, against DOGAMI", "fit.portland_deaths_ratio", "this run's expected deaths in the nine Portland districts over DOGAMI's figure for the same scenario; 1 is the fit")
            .view("counter", "damage pattern across 876 units", "fit.units_damage_r", "correlation of extensive-or-complete counts per neighbourhood unit with DOGAMI's")
            .note("Hazus's own tables (FEMA, Earthquake Model Technical Manual) were not downloadable from this machine on 2026-11-12; the medians, dispersions, collapse share and casualty rates here are recalled from them and declared so, and the two scales are fitted to DOGAMI's run on the same ground, so the absolute toll is the record's by construction and the pattern is the finding. The inventory is DOGAMI's (footprints without heights or structure types; the people per building are each unit's totals spread by footprint area, which is ours).");
        // the record as a table on the run page
        if let Ok((head, rows)) = read_csv(&root().join("data/events").join(folder).join(format!("loss_jurisdiction_{}.csv", param_str(p, "record", "wet")))) {
            let keep = ["Jurisdiction", "Bldg_LR", "DisplacedPop", "CasDayL4", "CasNightL4", "CasDayL2", "CasDayL3", "PDsExtensive", "PDsComplete"];
            let idx: Vec<Option<usize>> = keep.iter().map(|k| head.iter().position(|h| h == k)).collect();
            let table: Vec<serde_json::Value> = rows.iter().filter(|r| r.first().map_or(false, |s| s.starts_with("Portland") || s.ends_with("Unincorporated") || s == "Beaverton" || s == "Gresham" || s == "Hillsboro")).map(|r| {
                let mut m = serde_json::Map::new();
                for (k, i) in keep.iter().zip(&idx) { if let Some(i) = i { let s = &r[*i]; m.insert(k.to_string(), s.parse::<f64>().map(|v| json!((v * 100.0).round() / 100.0)).unwrap_or(json!(s))); } }
                serde_json::Value::Object(m)
            }).collect();
            d = d.table("dogami_losses", json!(table));
        }
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
        for f in [("building_damage_frac", "1"), ("buildings_destroyed", "buildings"), ("deaths", "people"), ("displaced", "people")] { ctx.field(f.0, f.1, &g, vec![0.0; n]); }
        ctx.report("quake.buildings", "buildings", self.count.iter().map(|&v| v as f64).sum());
        ctx.report("quake.occupants", "people", self.occupants.iter().map(|&v| v as f64).sum());
        for s in ["quake.buildings_moderate", "quake.buildings_extensive", "quake.buildings_complete", "quake.deaths", "quake.injuries_hospital", "quake.injuries_life_threatening", "quake.injuries_minor", "quake.displaced"] { ctx.report(s, "", 0.0); }
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
        let mut tot = [0.0f64; 5]; let (mut d_tot, mut l1, mut l23, mut l3, mut disp) = (0.0, 0.0, 0.0, 0.0, 0.0);
        let mut by_unit: std::collections::HashMap<i32, UnitLoss> = std::collections::HashMap::new();
        let mut pdx = UnitLoss::default();
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
            let pc = ps[4]; let coll = shake_complete * self.collapse_given_complete; let ps5 = [ps[1], ps[2], ps[3], pc - coll, coll];
            let occ = self.occupants[i] as f64; let res = self.residents[i] as f64;
            let mut dd = 0.0; let mut i1 = 0.0; let mut i23 = 0.0; let mut i3 = 0.0;
            for (k, p) in ps5.iter().enumerate() { let r = &self.rates[k]; i1 += occ * p * r[0]; i23 += occ * p * (r[1] + r[2]); i3 += occ * p * r[2]; dd += occ * p * r[3]; }
            dd *= self.casualty_scale; i1 *= self.casualty_scale; i23 *= self.casualty_scale; i3 *= self.casualty_scale;
            l3 += i3;
            let di = res * (self.displaced_share * (ps[3] + ps[4]) + self.displaced_moderate_share * ps[2]);
            frac[i] = (ps[3] + ps[4]) as f32; destroyed[i] = (c * (ps[3] + ps[4])) as f32; deaths[i] = dd as f32; displaced[i] = di as f32;
            for k in 0..5 { tot[k] += c * ps[k]; }
            d_tot += dd; l1 += i1; l23 += i23; disp += di;
            let u = self.nuid[i]; if !u.is_nan() && u > 0.0 { let e = by_unit.entry(u as i32).or_default(); e.ext_comp += c * (ps[3] + ps[4]); e.deaths += dd; e.displaced += di; e.injuries += i23; }
            let j = self.juris[i]; if !j.is_nan() && (25.0..=33.0).contains(&j) { pdx.ext_comp += c * (ps[3] + ps[4]); pdx.ext += c * ps[3]; pdx.comp += c * ps[4]; pdx.deaths += dd; pdx.displaced += di; pdx.injuries += i23; }
        }
        // the per-cell rates as world fields, for agents that stand at a cell
        let mut death_rate = vec![0.0f32; n]; let mut disp_rate = vec![0.0f32; n];
        for i in 0..n { let o = self.occupants[i]; let r = self.residents[i]; if o > 0.0 { death_rate[i] = (deaths[i] / o).min(1.0); } if r > 0.0 { disp_rate[i] = (displaced[i] / r).min(1.0); } }
        ctx.world_write("quake.cell_death_rate", "1", &g, death_rate); ctx.world_write("quake.cell_displaced_rate", "1", &g, disp_rate);
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
        ctx.report("quake.portland_deaths", "people", pdx.deaths);
        ctx.report("quake.portland_displaced", "people", pdx.displaced);
        ctx.report("quake.portland_buildings_extensive_or_complete", "buildings", pdx.ext_comp);
        let ratio = |a: f64, b: f64| if b > 0.0 { a / b } else { f64::NAN };
        let rp = &self.record_portland;
        ctx.report("fit.portland_deaths_ratio", "1", ratio(pdx.deaths, rp.deaths));
        ctx.report("fit.portland_displaced_ratio", "1", ratio(pdx.displaced, rp.displaced));
        ctx.report("fit.portland_damage_ratio", "1", ratio(pdx.ext_comp, rp.ext_comp));
        ctx.report("fit.portland_extensive_ratio", "1", ratio(pdx.ext, rp.ext));
        ctx.report("fit.portland_complete_ratio", "1", ratio(pdx.comp, rp.comp));
        ctx.report("fit.portland_injuries_ratio", "1", ratio(pdx.injuries, rp.injuries));
        // correlation across units present in both
        let corr = |ours: &dyn Fn(&UnitLoss) -> f64| -> f64 {
            let pairs: Vec<(f64, f64)> = self.record_units.iter().filter_map(|r| by_unit.get(&r.nuid).map(|o| (ours(o), ours(r)))).collect();
            if pairs.len() < 3 { return f64::NAN; }
            let n = pairs.len() as f64; let (mx, my) = (pairs.iter().map(|p| p.0).sum::<f64>() / n, pairs.iter().map(|p| p.1).sum::<f64>() / n);
            let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
            for (x, y) in &pairs { sxy += (x - mx) * (y - my); sxx += (x - mx).powi(2); syy += (y - my).powi(2); }
            if sxx > 0.0 && syy > 0.0 { sxy / (sxx * syy).sqrt() } else { f64::NAN }
        };
        let r_dmg = corr(&|u| u.ext_comp); let r_d = corr(&|u| u.deaths);
        ctx.report("fit.units_damage_r", "1", r_dmg);
        ctx.report("fit.units_deaths_r", "1", r_d);
        let sentence = format!("In the window, about {:.0} buildings extensively damaged or destroyed ({:.0} destroyed), {:.0} dead, {:.0} hurt badly enough for a hospital, {:.0} people displaced, {}time occupancy. In Portland's nine districts {:.0} dead against DOGAMI's {:.0} for the same scenario (ratio {:.2}); the damage pattern across the units correlates {:.2} with DOGAMI's.",
            tot[3] + tot[4], tot[4], d_tot, l23, disp, self.time_of_day, pdx.deaths, rp.deaths, ratio(pdx.deaths, rp.deaths), r_dmg);
        ctx.report("quake.injuries_life_threatening", "people", l3);
        ctx.publish("quake.toll", json!({ "deaths": d_tot, "injuries_hospital": l23, "injuries_life": l3, "injuries_minor": l1, "displaced": disp }));
        ctx.publish("quake.crossing", json!({ "id": "toll", "title": "The toll", "name": "the window", "kind": "toll", "time": ctx.now().to_iso(), "sentence": sentence, "source": "this run's fragility (recalled Hazus forms, two scales fitted) on DOGAMI's ground and inventory; the record is DOGAMI O-18-02's Hazus run", "confidence": "fitted" }));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Buildings = serde_json::from_value(v).unwrap(); o.load().expect("buildings reload"); *self = o; }
}
