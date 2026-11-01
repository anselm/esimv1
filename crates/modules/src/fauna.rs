//! M6 Eaters: two animal cohorts attached to basin cells, in the manner of Madingley's
//! functional groups: a grazer that eats the crop module's biomass and a predator that
//! eats the grazer. Each cell holds a count of individuals of one body mass. Daily:
//! intake, assimilation, births limited by food, background and starvation mortality,
//! predation by a Holling type II response, and movement of a fraction of each cohort
//! to the richest neighbouring cell. Counts are conserved exactly through movement, so
//! the population ledger (individuals) closes, and body carbon rides on the carbon ledger.
//!
//! Protocol, all at the day boundary and in priority order: herbivore (4) writes its
//! demand field and publishes `graze.request`; the crop module answers by removing
//! biomass, writing `veg.grazed_kgc` and publishing `graze.done`; herbivore updates and
//! writes `fauna.herb_n`; predator (5) writes `fauna.pred_kills` and publishes
//! `predation.request`; herbivore removes prey, writes `fauna.prey_taken`, publishes
//! `predation.done`; predator updates. Bulk numbers travel in fields, events only signal.
//!
//! Ledgers. `animals` (individuals): births -> herbivores_n -> deaths_natural |
//! deaths_predation; births -> predators_n -> deaths_natural. `carbon` (kg C): biomass ->
//! herbivores (grazing); herbivores -> litter_dung | atmosphere | predators |
//! carcass_sink; predators -> atmosphere | carcass_sink. `carcass_sink` and `litter_dung`
//! are boundary accounts, not tracked pools: honest about what is not modelled.
//! Every parameter is an order of magnitude for a deer-sized grazer and a coyote-sized
//! predator, named so the hindcast harness can move it. No scoreboard yet.

use esim_kernel::manifest::{param_duration, param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module};
use esim_registry::hydro::delineate;
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

const ANIMALS: &str = "animals";
const CARBON: &str = "carbon";

/// Basin geometry shared by both cohorts.
struct Geo { grid: LatLonGrid, cells: Vec<usize>, area_km2: Vec<f64>, nbrs: Vec<Vec<usize>> }

fn load_geo(lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res: f64, glat: f64, glon: f64, snap: usize) -> Result<Geo, String> {
    let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let cat = Catalog::load(&root.join("data/catalog.toml"))?;
    let dem = cat.asset("copernicus-dem-glo30").ok_or("catalog has no copernicus-dem-glo30")?;
    let mut f = Fetcher::default_for(&root)?;
    let w = dem_window(&mut f, dem, lat_s, lat_n, lon_w, lon_e, res)?;
    let grid = LatLonGrid { nx: w.nx, ny: w.ny, dlon: w.dlon, dlat: w.dlat, lon_w: w.lon_w, lat_n: w.lat_n };
    let row_area: Vec<f64> = (0..grid.ny).map(|r| grid.row_area(r)).collect();
    let (_, _, cells, mask) = delineate(&f, &w, &row_area, glat, glon, snap);
    let mut pos = vec![usize::MAX; grid.nx * grid.ny];
    for (k, &i) in cells.iter().enumerate() { pos[i] = k; }
    let nbrs: Vec<Vec<usize>> = cells.iter().map(|&i| {
        let (c, r) = (i % grid.nx, i / grid.nx);
        let mut v = Vec::new();
        if c > 0 && mask[i - 1] { v.push(pos[i - 1]); }
        if c + 1 < grid.nx && mask[i + 1] { v.push(pos[i + 1]); }
        if r > 0 && mask[i - grid.nx] { v.push(pos[i - grid.nx]); }
        if r + 1 < grid.ny && mask[i + grid.nx] { v.push(pos[i + grid.nx]); }
        v
    }).collect();
    let area_km2 = cells.iter().map(|&i| row_area[i / grid.nx] / 1e6).collect();
    Ok(Geo { grid, cells, area_km2, nbrs })
}

/// Move `frac` of each cell's count to its best neighbour by `score` if better. Exact.
fn migrate(n: &mut [f64], score: &[f64], nbrs: &[Vec<usize>], frac: f64) -> f64 {
    let mut delta = vec![0.0; n.len()];
    let mut moved = 0.0;
    for k in 0..n.len() {
        if n[k] <= 0.0 { continue; }
        let mut best = k; let mut bs = score[k];
        for &j in &nbrs[k] { if score[j] > bs { bs = score[j]; best = j; } }
        if best != k { let m = n[k] * frac; delta[k] -= m; delta[best] += m; moved += m; }
    }
    for k in 0..n.len() { n[k] += delta[k]; }
    moved
}

// ---------------------------------------------------------------- herbivore

#[derive(Serialize, Deserialize)]
pub struct Herbivore {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64, gauge_lat: f64, gauge_lon: f64, snap_radius: usize,
    dt: Duration,
    veg_module: String,
    c_ind_kgc: f64, init_density_km2: f64, intake_max_kgc_day: f64, maintenance_kgc_day: f64,
    assimilation: f64, birth_per_day: f64, mortality_per_day: f64, starvation_per_day: f64, move_frac: f64,
    /// food outside the crop (residue, browse, pasture) per individual per day, from an
    /// untracked `browse` boundary account: Iowa deer do not live on standing corn alone
    browse_kgc_ind_day: f64,
    flee_frac: f64, fire_mortality: f64,
    n: Vec<f64>,
    intake_today: Vec<f64>,
    births_total: f64, deaths_total: f64, killed_total: f64,
    #[serde(skip)] geo: Option<Geo>,
}

impl Herbivore {
    pub fn new(name: &str, p: &toml::Value) -> Result<Herbivore, String> {
        let mut h = Herbivore {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 42.3), lat_n: param_f64(p, "lat_n", 43.2), lon_w: param_f64(p, "lon_w", -94.2), lon_e: param_f64(p, "lon_e", -93.5),
            res_arcsec: param_f64(p, "res_arcsec", 30.0), gauge_lat: param_f64(p, "gauge_lat", 42.4319722), gauge_lon: param_f64(p, "gauge_lon", -93.8059444), snap_radius: param_f64(p, "snap_radius", 3.0) as usize,
            dt: param_duration(p, "dt", "1d")?,
            veg_module: param_str(p, "veg_module", "veg").to_string(),
            c_ind_kgc: param_f64(p, "c_ind_kgc", 5.0), init_density_km2: param_f64(p, "init_density_km2", 6.0),
            intake_max_kgc_day: param_f64(p, "intake_max_kgc_day", 0.5), maintenance_kgc_day: param_f64(p, "maintenance_kgc_day", 0.3),
            assimilation: param_f64(p, "assimilation", 0.5), birth_per_day: param_f64(p, "birth_per_day", 0.002),
            mortality_per_day: param_f64(p, "mortality_per_day", 0.0007), starvation_per_day: param_f64(p, "starvation_per_day", 0.01), move_frac: param_f64(p, "move_frac", 0.1),
            browse_kgc_ind_day: param_f64(p, "browse_kgc_ind_day", 0.2),
            flee_frac: param_f64(p, "flee_frac", 0.8), fire_mortality: param_f64(p, "fire_mortality", 0.3),
            n: Vec::new(), intake_today: Vec::new(), births_total: 0.0, deaths_total: 0.0, killed_total: 0.0, geo: None,
        };
        h.load()?;
        Ok(h)
    }
    fn load(&mut self) -> Result<(), String> {
        let g = load_geo(self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec, self.gauge_lat, self.gauge_lon, self.snap_radius)?;
        if self.n.len() != g.cells.len() {
            self.n = g.area_km2.iter().map(|a| a * self.init_density_km2).collect();
            self.intake_today = vec![0.0; g.cells.len()];
        }
        self.geo = Some(g);
        Ok(())
    }
    fn total(&self) -> f64 { self.n.iter().sum() }
    fn write_n(&self, ctx: &mut Ctx) {
        let g = self.geo.as_ref().unwrap();
        let mut f = vec![f32::NAN; g.grid.nx * g.grid.ny];
        for (k, &i) in g.cells.iter().enumerate() { f[i] = self.n[k] as f32; }
        ctx.world_write("fauna.herb_n", "individuals", &g.grid, f);
    }
}

impl Module for Herbivore {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.ledger(ANIMALS, "individuals", 1e-6);
        ctx.ledger(CARBON, "kgC", 1e-3);
        ctx.declare_stock(ANIMALS, "herbivores_n", self.total());
        ctx.declare_stock(CARBON, "herbivores", self.total() * self.c_ind_kgc);
        ctx.subscribe("graze.done");
        ctx.subscribe("predation.request");
        ctx.subscribe("fire.burn");
        // the kernel's spatial index: hear ignitions only where this cohort lives
        let cells: Vec<usize> = self.geo.as_ref().unwrap().cells.clone();
        ctx.subscribe_at("fire.ignition", &cells);
        self.write_n(ctx);
        ctx.schedule_with_priority(self.dt, 4, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        let g = self.geo.as_ref().unwrap();
        match ev.topic.as_str() {
            "tick" => {
                // ask the crop for food: demand per cell
                let mut f = vec![f32::NAN; g.grid.nx * g.grid.ny];
                for (k, &i) in g.cells.iter().enumerate() { f[i] = (self.n[k] * self.intake_max_kgc_day) as f32; }
                ctx.world_write("fauna.herb_demand_kgc", "kgC", &g.grid, f);
                ctx.publish("graze.request", json!({ "by": self.name }));
                ctx.schedule_with_priority(self.dt, 4, "tick", json!(null));
            }
            "graze.done" => {
                let Some(gr) = ctx.world_read("veg.grazed_kgc").map(|f| f.data.clone()) else { return };
                let (mut dung, mut resp, mut births, mut deaths, mut eaten, mut browse) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
                let mut score = vec![0.0; g.cells.len()];
                let veg_b: Option<Vec<f32>> = ctx.world_read("veg.biomass_gc_m2").map(|f| f.data.clone());
                for (k, &i) in g.cells.iter().enumerate() {
                    let n = self.n[k];
                    let crop = gr[i].max(0.0) as f64;
                    let br = n * self.browse_kgc_ind_day;
                    let got = crop + br;
                    eaten += crop; browse += br;
                    let assim = got * self.assimilation;
                    dung += got - assim;
                    let per_ind = if n > 0.0 { got / n } else { 0.0 };
                    let fed = (per_ind / self.maintenance_kgc_day).clamp(0.0, 1.0);
                    let b = (n * self.birth_per_day * fed).min(assim / self.c_ind_kgc);
                    let starv = if per_ind < self.maintenance_kgc_day { self.starvation_per_day * (1.0 - fed) } else { 0.0 };
                    let d = n * (self.mortality_per_day + starv);
                    resp += assim - b * self.c_ind_kgc;
                    self.n[k] = (n + b - d).max(0.0);
                    births += b; deaths += d;
                    self.intake_today[k] = per_ind;
                    score[k] = veg_b.as_ref().map(|v| v[i] as f64).unwrap_or(0.0);
                }
                migrate(&mut self.n, &score, &g.nbrs, self.move_frac);
                self.births_total += births; self.deaths_total += deaths;
                ctx.transfer(CARBON, "browse", "herbivores", browse);
                ctx.transfer(CARBON, "herbivores", "litter_dung", dung);
                ctx.transfer(CARBON, "herbivores", "atmosphere", resp);
                ctx.transfer(CARBON, "herbivores", "carcass_sink", deaths * self.c_ind_kgc);
                ctx.transfer(ANIMALS, "births", "herbivores_n", births);
                ctx.transfer(ANIMALS, "herbivores_n", "deaths_natural", deaths);
                ctx.declare_stock(ANIMALS, "herbivores_n", self.total());
                ctx.declare_stock(CARBON, "herbivores", self.total() * self.c_ind_kgc);
                ctx.report("fauna.herb_total", "individuals", self.total());
                ctx.report("fauna.herb_grazed_kgc_day", "kgC/day", eaten);
                ctx.report("fauna.herb_births_day", "1/day", births);
                ctx.report("fauna.herb_deaths_day", "1/day", deaths);
                self.write_n(ctx);
                let day = (ctx.now().0 / 86_400) % 8;
                if day == 0 {
                    let mut f = vec![f32::NAN; g.grid.nx * g.grid.ny];
                    for (k, &i) in g.cells.iter().enumerate() { f[i] = (self.n[k] / g.area_km2[k]) as f32; }
                    ctx.field("herbivore_density_km2", "1/km2", &g.grid, f);
                }
            }
            "fire.ignition" => {
                // flight: most animals leave the ignited cell for its neighbours, exactly
                let Some(cell) = ev.at else { return };
                let Some(k) = g.cells.iter().position(|&c| c == cell as usize) else { return };
                if g.nbrs[k].is_empty() || self.n[k] <= 0.0 { return; }
                let leaving = self.n[k] * self.flee_frac;
                let share = leaving / g.nbrs[k].len() as f64;
                self.n[k] -= leaving;
                for &j in &g.nbrs[k] { self.n[j] += share; }
                ctx.report("fauna.herb_fled", "individuals", leaving);
            }
            "fire.burn" => {
                let Some(bf) = ctx.world_read("fire.burned_frac").map(|f| f.data.clone()) else { return };
                let mut dead = 0.0;
                for (k, &i) in g.cells.iter().enumerate() {
                    let f = bf[i]; if !f.is_finite() || f <= 0.0 { continue; }
                    let d = self.n[k] * f as f64 * self.fire_mortality; self.n[k] -= d; dead += d;
                }
                ctx.transfer(ANIMALS, "herbivores_n", "deaths_fire", dead);
                ctx.transfer(CARBON, "herbivores", "carcass_sink", dead * self.c_ind_kgc);
                ctx.declare_stock(ANIMALS, "herbivores_n", self.total());
                ctx.declare_stock(CARBON, "herbivores", self.total() * self.c_ind_kgc);
                ctx.report("fauna.herb_fire_deaths", "individuals", dead);
                self.write_n(ctx);
            }
            "predation.request" => {
                let Some(req) = ctx.world_read("fauna.pred_kills").map(|f| f.data.clone()) else { return };
                let mut taken = vec![f32::NAN; g.grid.nx * g.grid.ny];
                let mut kills = 0.0;
                for (k, &i) in g.cells.iter().enumerate() {
                    let want = req[i].max(0.0) as f64;
                    let k_ = want.min(self.n[k]);
                    self.n[k] -= k_;
                    taken[i] = k_ as f32;
                    kills += k_;
                }
                self.killed_total += kills;
                ctx.transfer(ANIMALS, "herbivores_n", "deaths_predation", kills);
                ctx.transfer(CARBON, "herbivores", "predators", kills * self.c_ind_kgc);
                ctx.declare_stock(ANIMALS, "herbivores_n", self.total());
                ctx.declare_stock(CARBON, "herbivores", self.total() * self.c_ind_kgc);
                ctx.report("fauna.herb_killed_day", "1/day", kills);
                ctx.world_write("fauna.prey_taken", "individuals", &g.grid, taken);
                ctx.world_write("fauna.prey_c_per_kill", "kgC", &g.grid, vec![self.c_ind_kgc as f32; g.grid.nx * g.grid.ny]);
                self.write_n(ctx);
                ctx.publish("predation.done", json!({ "by": self.name }));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut h: Herbivore = serde_json::from_value(v).unwrap(); h.load().expect("herbivore reload"); *self = h; }
}

// ---------------------------------------------------------------- predator

#[derive(Serialize, Deserialize)]
pub struct Predator {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64, gauge_lat: f64, gauge_lon: f64, snap_radius: usize,
    dt: Duration,
    c_ind_kgc: f64, init_density_km2: f64, attack_km2_day: f64, handling_days: f64, need_kgc_day: f64,
    assimilation: f64, birth_per_day: f64, mortality_per_day: f64, starvation_per_day: f64, move_frac: f64,
    n: Vec<f64>,
    #[serde(skip)] geo: Option<Geo>,
}

impl Predator {
    pub fn new(name: &str, p: &toml::Value) -> Result<Predator, String> {
        let mut d = Predator {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 42.3), lat_n: param_f64(p, "lat_n", 43.2), lon_w: param_f64(p, "lon_w", -94.2), lon_e: param_f64(p, "lon_e", -93.5),
            res_arcsec: param_f64(p, "res_arcsec", 30.0), gauge_lat: param_f64(p, "gauge_lat", 42.4319722), gauge_lon: param_f64(p, "gauge_lon", -93.8059444), snap_radius: param_f64(p, "snap_radius", 3.0) as usize,
            dt: param_duration(p, "dt", "1d")?,
            c_ind_kgc: param_f64(p, "c_ind_kgc", 1.5), init_density_km2: param_f64(p, "init_density_km2", 0.3),
            attack_km2_day: param_f64(p, "attack_km2_day", 0.02), handling_days: param_f64(p, "handling_days", 20.0), need_kgc_day: param_f64(p, "need_kgc_day", 0.1),
            assimilation: param_f64(p, "assimilation", 0.8), birth_per_day: param_f64(p, "birth_per_day", 0.002),
            mortality_per_day: param_f64(p, "mortality_per_day", 0.001), starvation_per_day: param_f64(p, "starvation_per_day", 0.01), move_frac: param_f64(p, "move_frac", 0.1),
            n: Vec::new(), geo: None,
        };
        d.load()?;
        Ok(d)
    }
    fn load(&mut self) -> Result<(), String> {
        let g = load_geo(self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec, self.gauge_lat, self.gauge_lon, self.snap_radius)?;
        if self.n.len() != g.cells.len() { self.n = g.area_km2.iter().map(|a| a * self.init_density_km2).collect(); }
        self.geo = Some(g);
        Ok(())
    }
    fn total(&self) -> f64 { self.n.iter().sum() }
}

impl Module for Predator {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.ledger(ANIMALS, "individuals", 1e-6);
        ctx.ledger(CARBON, "kgC", 1e-3);
        ctx.declare_stock(ANIMALS, "predators_n", self.total());
        ctx.declare_stock(CARBON, "predators", self.total() * self.c_ind_kgc);
        ctx.subscribe("predation.done");
        ctx.schedule_with_priority(self.dt, 5, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        let g = self.geo.as_ref().unwrap();
        match ev.topic.as_str() {
            "tick" => {
                let Some(hn) = ctx.world_read("fauna.herb_n").map(|f| f.data.clone()) else { ctx.schedule_with_priority(self.dt, 5, "tick", json!(null)); return };
                let mut f = vec![f32::NAN; g.grid.nx * g.grid.ny];
                let dt_days = self.dt.as_secs_f64() / 86_400.0;
                for (k, &i) in g.cells.iter().enumerate() {
                    let dh = hn[i].max(0.0) as f64 / g.area_km2[k];
                    let per_pred = self.attack_km2_day * dh / (1.0 + self.attack_km2_day * self.handling_days * dh);
                    f[i] = (self.n[k] * per_pred * dt_days) as f32;
                }
                ctx.world_write("fauna.pred_kills", "individuals", &g.grid, f);
                ctx.publish("predation.request", json!({ "by": self.name }));
                ctx.schedule_with_priority(self.dt, 5, "tick", json!(null));
            }
            "predation.done" => {
                let (Some(taken), Some(cpk)) = (ctx.world_read("fauna.prey_taken").map(|f| f.data.clone()), ctx.world_read("fauna.prey_c_per_kill").map(|f| f.data.clone())) else { return };
                let hn: Option<Vec<f32>> = ctx.world_read("fauna.herb_n").map(|f| f.data.clone());
                let (mut intake, mut resp, mut births, mut deaths) = (0.0, 0.0, 0.0, 0.0);
                let mut score = vec![0.0; g.cells.len()];
                for (k, &i) in g.cells.iter().enumerate() {
                    let n = self.n[k];
                    let got = taken[i].max(0.0) as f64 * cpk[i] as f64;
                    intake += got;
                    let assim = got * self.assimilation;
                    let per_ind = if n > 0.0 { got / n } else { 0.0 };
                    let fed = (per_ind / self.need_kgc_day).clamp(0.0, 1.0);
                    let b = (n * self.birth_per_day * fed).min(assim / self.c_ind_kgc);
                    let starv = if per_ind < self.need_kgc_day { self.starvation_per_day * (1.0 - fed) } else { 0.0 };
                    let d = n * (self.mortality_per_day + starv);
                    resp += assim - b * self.c_ind_kgc;
                    self.n[k] = (n + b - d).max(0.0);
                    births += b; deaths += d;
                    score[k] = hn.as_ref().map(|v| v[i] as f64 / g.area_km2[k]).unwrap_or(0.0);
                }
                migrate(&mut self.n, &score, &g.nbrs, self.move_frac);
                ctx.transfer(CARBON, "predators", "carcass_sink", intake * (1.0 - self.assimilation) + deaths * self.c_ind_kgc);
                ctx.transfer(CARBON, "predators", "atmosphere", resp);
                ctx.transfer(ANIMALS, "births", "predators_n", births);
                ctx.transfer(ANIMALS, "predators_n", "deaths_natural", deaths);
                ctx.declare_stock(ANIMALS, "predators_n", self.total());
                ctx.declare_stock(CARBON, "predators", self.total() * self.c_ind_kgc);
                ctx.report("fauna.pred_total", "individuals", self.total());
                ctx.report("fauna.pred_intake_kgc_day", "kgC/day", intake);
                let day = (ctx.now().0 / 86_400) % 8;
                if day == 0 {
                    let mut f = vec![f32::NAN; g.grid.nx * g.grid.ny];
                    for (k, &i) in g.cells.iter().enumerate() { f[i] = (self.n[k] / g.area_km2[k]) as f32; }
                    ctx.field("predator_density_km2", "1/km2", &g.grid, f);
                }
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut d: Predator = serde_json::from_value(v).unwrap(); d.load().expect("predator reload"); *self = d; }
}
