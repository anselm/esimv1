//! M5 Green: vegetation as a field. One plant functional type, a summer annual crop (the
//! Corn Belt's corn and soybean treated as one), on every basin cell: planting by
//! temperature and date, growing-degree-day phenology, light-use-efficiency growth from
//! the sun's energy, water stress from the soil the water module publishes, senescence,
//! harvest with grain exported and residue to litter, litter respiring. Leaf area is
//! published for the water module's evaporation (the two-way feedback of the brief) and
//! converted to NDVI for scoring against MODIS MOD13A1.
//!
//! Carbon ledger (kg C): atmosphere -> biomass (GPP); biomass -> atmosphere (autotrophic
//! respiration); biomass -> litter (senescence, residue); biomass -> export (grain);
//! litter -> atmosphere (heterotrophic respiration). Stocks declared: biomass, litter.
//! Every number here is a published order of magnitude, not a calibrated value, and is
//! named as a parameter so the hindcast harness can move it.

use esim_kernel::manifest::{param_duration, param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module};
use esim_registry::hydro::delineate;
use esim_registry::modis::ndvi_frames;
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

const LEDGER: &str = "carbon";

#[derive(Serialize, Deserialize)]
pub struct Veg {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64,
    res_arcsec: f64,
    gauge_lat: f64, gauge_lon: f64, snap_radius: usize,
    dt: Duration,
    energy_source: String,
    soil_source: String,
    soil_capacity_mm: f64,
    modis_start: String, modis_end: String,
    // parameters
    plant_after_doy: u32, plant_t_c: f64, gdd_base_c: f64,
    gdd_emerge: f64, gdd_senesce: f64, gdd_harvest: f64, frost_c: f64,
    lue_gc_per_mj: f64, par_fraction: f64, k_light: f64, sla_m2_per_gc: f64, leaf_fraction: f64, lai_max: f64, lai_emerge: f64, snow_ndvi_mm: f64,
    ra_fraction: f64, grain_fraction: f64, k_litter_per_day: f64,
    ndvi_soil: f64, ndvi_max: f64, k_ndvi: f64,
    graze_max_frac: f64,
    burn_biomass_frac: f64, burn_litter_frac: f64,
    /// soybean relative to corn: light-use efficiency and season length, and grass as perennial cover
    soy_lue_factor: f64, soy_gdd_factor: f64, soy_grain_fraction: f64, grass_lai: f64,
    // state per basin cell
    kind: Vec<f32>,
    lai: Vec<f64>, biomass_gc: Vec<f64>, litter_gc: Vec<f64>, gdd: Vec<f64>, stage: Vec<u8>, lai_at_senesce: Vec<f64>,
    seen_energy: u64,
    day_energy: Vec<f64>, day_t_sum: f64, day_t_n: u32, day_tmin: f64, warm_days: u32,
    planted_doy: u32, harvested: bool, last_day: i64,
    biomass_kg: f64, litter_kg: f64,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] cells: Vec<usize>,
    #[serde(skip)] area: Vec<f64>,
    #[serde(skip)] modis: BTreeMap<i64, Vec<f32>>,
    #[serde(skip)] basin_km2: f64,
}

impl Veg {
    pub fn new(name: &str, p: &toml::Value) -> Result<Veg, String> {
        let mut v = Veg {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 42.3), lat_n: param_f64(p, "lat_n", 43.2), lon_w: param_f64(p, "lon_w", -94.2), lon_e: param_f64(p, "lon_e", -93.5),
            res_arcsec: param_f64(p, "res_arcsec", 30.0),
            gauge_lat: param_f64(p, "gauge_lat", 42.4319722), gauge_lon: param_f64(p, "gauge_lon", -93.8059444), snap_radius: param_f64(p, "snap_radius", 3.0) as usize,
            dt: param_duration(p, "dt", "1h")?,
            energy_source: param_str(p, "energy_source", "sun.incident_j").to_string(),
            soil_source: param_str(p, "soil_source", "water.soil_mm").to_string(),
            soil_capacity_mm: param_f64(p, "soil_capacity_mm", 150.0),
            modis_start: param_str(p, "modis_start", "2025-01-01").to_string(), modis_end: param_str(p, "modis_end", "2025-12-31").to_string(),
            plant_after_doy: param_f64(p, "plant_after_doy", 110.0) as u32, plant_t_c: param_f64(p, "plant_t_c", 10.0), gdd_base_c: param_f64(p, "gdd_base_c", 10.0),
            gdd_emerge: param_f64(p, "gdd_emerge", 80.0), gdd_senesce: param_f64(p, "gdd_senesce", 1300.0), gdd_harvest: param_f64(p, "gdd_harvest", 1650.0), frost_c: param_f64(p, "frost_c", -2.0),
            lue_gc_per_mj: param_f64(p, "lue_gc_per_mj", 1.6), par_fraction: param_f64(p, "par_fraction", 0.48), k_light: param_f64(p, "k_light", 0.5),
            sla_m2_per_gc: param_f64(p, "sla_m2_per_gc", 0.025), leaf_fraction: param_f64(p, "leaf_fraction", 0.45), lai_max: param_f64(p, "lai_max", 5.5),
            lai_emerge: param_f64(p, "lai_emerge", 0.3), snow_ndvi_mm: param_f64(p, "snow_ndvi_mm", 5.0),
            ra_fraction: param_f64(p, "ra_fraction", 0.5), grain_fraction: param_f64(p, "grain_fraction", 0.45), k_litter_per_day: param_f64(p, "k_litter_per_day", 0.003),
            ndvi_soil: param_f64(p, "ndvi_soil", 0.18), ndvi_max: param_f64(p, "ndvi_max", 0.88), k_ndvi: param_f64(p, "k_ndvi", 0.7),
            graze_max_frac: param_f64(p, "graze_max_frac", 0.3),
            burn_biomass_frac: param_f64(p, "burn_biomass_frac", 0.9), burn_litter_frac: param_f64(p, "burn_litter_frac", 0.7),
            soy_lue_factor: param_f64(p, "soy_lue_factor", 0.75), soy_gdd_factor: param_f64(p, "soy_gdd_factor", 0.85), soy_grain_fraction: param_f64(p, "soy_grain_fraction", 0.4), grass_lai: param_f64(p, "grass_lai", 2.0),
            kind: Vec::new(),
            lai: Vec::new(), biomass_gc: Vec::new(), litter_gc: Vec::new(), gdd: Vec::new(), stage: Vec::new(), lai_at_senesce: Vec::new(),
            seen_energy: 0, day_energy: Vec::new(), day_t_sum: 0.0, day_t_n: 0, day_tmin: 99.0, warm_days: 0, planted_doy: 0, harvested: false, last_day: i64::MIN,
            biomass_kg: 0.0, litter_kg: 0.0,
            grid: None, cells: Vec::new(), area: Vec::new(), modis: BTreeMap::new(), basin_km2: 0.0,
        };
        v.load()?;
        Ok(v)
    }

    fn load(&mut self) -> Result<(), String> {
        let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let dem = cat.asset("copernicus-dem-glo30").ok_or("catalog has no copernicus-dem-glo30")?;
        let mut f = Fetcher::default_for(&root)?;
        let w = dem_window(&mut f, dem, self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let grid = LatLonGrid { nx: w.nx, ny: w.ny, dlon: w.dlon, dlat: w.dlat, lon_w: w.lon_w, lat_n: w.lat_n };
        let row_area: Vec<f64> = (0..grid.ny).map(|r| grid.row_area(r)).collect();
        let (_, _, cells, _) = delineate(&f, &w, &row_area, self.gauge_lat, self.gauge_lon, self.snap_radius);
        self.area = cells.iter().map(|&i| row_area[i / grid.nx]).collect();
        self.basin_km2 = self.area.iter().sum::<f64>() / 1e6;
        let n = cells.len();
        if self.lai.len() != n {
            self.lai = vec![0.0; n]; self.biomass_gc = vec![0.0; n]; self.litter_gc = vec![200.0; n]; self.gdd = vec![0.0; n];
            self.stage = vec![0; n]; self.lai_at_senesce = vec![0.0; n]; self.day_energy = vec![0.0; n];
            self.kind = vec![1.0; n];
        }
        let frames = ndvi_frames(&mut f, (grid.nx, grid.ny, grid.dlon, grid.dlat, grid.lon_w, grid.lat_n), [self.lon_w, self.lat_s, self.lon_e, self.lat_n], &self.modis_start, &self.modis_end)?;
        self.modis = frames.into_iter().collect();
        self.grid = Some(grid); self.cells = cells;
        Ok(())
    }

    fn ndvi(&self, lai: f64) -> f64 { self.ndvi_soil + (self.ndvi_max - self.ndvi_soil) * (1.0 - (-self.k_ndvi * lai).exp()) }
    fn stocks(&self) -> (f64, f64) {
        let b: f64 = self.biomass_gc.iter().zip(&self.area).map(|(g, a)| g / 1000.0 * a).sum();
        let l: f64 = self.litter_gc.iter().zip(&self.area).map(|(g, a)| g / 1000.0 * a).sum();
        (b, l)
    }
}

fn doy_of(day_secs: i64) -> u32 {
    let z = day_secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097); let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy_mar = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy_mar + 2) / 153;
    let d = doy_mar - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let cum = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    (cum[(m - 1) as usize] + d + if leap && m > 2 { 1 } else { 0 }) as u32
}

impl Module for Veg {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.ledger(LEDGER, "kgC", 1e-3);
        let (b, l) = self.stocks();
        self.biomass_kg = b; self.litter_kg = l;
        ctx.declare_stock(LEDGER, "biomass", b);
        ctx.declare_stock(LEDGER, "litter", l);
        ctx.report("veg.modis_composites", "1", self.modis.len() as f64);
        let grid = self.grid.clone().unwrap();
        let mut lai = vec![f32::NAN; grid.nx * grid.ny];
        for &i in &self.cells { lai[i] = 0.0; }
        ctx.world_write("veg.lai", "m2/m2", &grid, lai);
        ctx.subscribe("graze.request");
        ctx.subscribe("fire.burn");
        ctx.schedule_with_priority(self.dt, 3, "tick", json!(null));
    }

    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic == "fire.burn" {
            // standing biomass and most litter go to the atmosphere in burned cells
            let Some(bf) = ctx.world_read("fire.burned_frac").map(|f| f.data.clone()) else { return };
            let (mut em_b, mut em_l) = (0.0, 0.0);
            for (k, &i) in self.cells.iter().enumerate() {
                let f = bf[i]; if !f.is_finite() || f <= 0.0 { continue; }
                let a = self.area[k];
                let b = self.biomass_gc[k] * f as f64 * self.burn_biomass_frac; let l = self.litter_gc[k] * f as f64 * self.burn_litter_frac;
                self.biomass_gc[k] -= b; self.litter_gc[k] -= l; self.lai[k] *= 1.0 - f as f64;
                if f >= 0.99 && self.stage[k] == 1 { self.stage[k] = 3; }
                em_b += b / 1000.0 * a; em_l += l / 1000.0 * a;
            }
            ctx.transfer(LEDGER, "biomass", "atmosphere", em_b);
            ctx.transfer(LEDGER, "litter", "atmosphere", em_l);
            let (b, l) = self.stocks();
            ctx.declare_stock(LEDGER, "biomass", b);
            ctx.declare_stock(LEDGER, "litter", l);
            ctx.report("veg.fire_emissions_kgc_day", "kgC/day", em_b + em_l);
            return;
        }
        if ev.topic == "graze.request" {
            // a grazer asks for carbon per cell; give up to a fraction of standing biomass
            let Some(demand) = ctx.world_read("fauna.herb_demand_kgc").map(|f| f.data.clone()) else { return };
            let grid = self.grid.clone().unwrap();
            let mut given = vec![f32::NAN; grid.nx * grid.ny];
            let mut total = 0.0;
            for (k, &i) in self.cells.iter().enumerate() {
                let a = self.area[k];
                let have_kgc = self.biomass_gc[k] / 1000.0 * a;
                let g = (demand[i].max(0.0) as f64).min(self.graze_max_frac * have_kgc);
                if g > 0.0 && have_kgc > 0.0 {
                    let frac = g / have_kgc;
                    self.biomass_gc[k] -= g * 1000.0 / a;
                    self.lai[k] *= 1.0 - frac;
                }
                given[i] = g as f32; total += g;
            }
            ctx.transfer(LEDGER, "biomass", "herbivores", total);
            let (b, l) = self.stocks();
            ctx.declare_stock(LEDGER, "biomass", b);
            ctx.declare_stock(LEDGER, "litter", l);
            ctx.world_write("veg.grazed_kgc", "kgC", &grid, given);
            ctx.report("veg.grazed_kgc_day", "kgC/day", total);
            ctx.publish("graze.done", json!({ "by": self.name }));
            return;
        }
        if ev.topic != "tick" { return; }
        let now = ctx.now();
        // accumulate the hour: energy per cell, air temperature over the basin
        if let Some(e) = ctx.world_read(&self.energy_source) {
            if e.version != self.seen_energy {
                for (k, &i) in self.cells.iter().enumerate() { self.day_energy[k] += e.data[i] as f64; }
                self.seen_energy = e.version;
            }
        }
        if let Some(t) = ctx.world_read("wx.t2m_c") {
            let mut s = 0.0; let mut n = 0u32; let mut mn = 99.0f64;
            for &i in &self.cells { let v = t.data[i] as f64; if v.is_finite() { s += v; n += 1; mn = mn.min(v); } }
            if n > 0 { self.day_t_sum += s / n as f64; self.day_t_n += 1; self.day_tmin = self.day_tmin.min(mn); }
        }
        let day = (now.0 - 1).div_euclid(86_400) * 86_400;
        let next_day = (now.0 + self.dt.0 - 1).div_euclid(86_400) * 86_400;
        if next_day != day {
            // ---- daily update ----
            let tmean = if self.day_t_n > 0 { self.day_t_sum / self.day_t_n as f64 } else { 10.0 };
            let doy = doy_of(day);
            if doy == 1 { self.harvested = false; self.planted_doy = 0; }
            if tmean >= self.plant_t_c { self.warm_days += 1; } else { self.warm_days = 0; }
            let agent_plant: Option<Vec<f32>> = ctx.world_read("agents.plant_now").map(|f| f.data.clone());
            let agent_kind: Option<Vec<f32>> = ctx.world_read("agents.crop_kind").map(|f| f.data.clone());
            let plant_today = agent_plant.is_none() && self.planted_doy == 0 && !self.harvested && doy >= self.plant_after_doy && doy < 200 && self.warm_days >= 5;
            if plant_today { self.planted_doy = doy; }
            if let Some(ak) = &agent_kind { for (k, &i) in self.cells.iter().enumerate() { if ak[i].is_finite() && ak[i] > 0.0 { self.kind[k] = ak[i]; } } }
            let mut grain_export = vec![f32::NAN; self.grid.as_ref().unwrap().nx * self.grid.as_ref().unwrap().ny];
            for &i in &self.cells { grain_export[i] = 0.0; }
            let mut any_harvest = false;
            let soil: Option<Vec<f32>> = ctx.world_read(&self.soil_source).map(|f| f.data.clone());
            let (mut gpp, mut ra, mut to_litter, mut export, mut rh, mut seed_import) = (0.0f64, 0.0, 0.0, 0.0, 0.0, 0.0);
            let snow: Option<Vec<f32>> = ctx.world_read("water.snow_mm").map(|f| f.data.clone());
            let tf = ((tmean - 8.0) / 22.0).clamp(0.0, 1.0).min(((42.0 - tmean) / 12.0).clamp(0.0, 1.0));
            for (k, &i) in self.cells.iter().enumerate() {
                let a = self.area[k];
                let soy = self.kind[k] == 2.0;
                let grass = self.kind[k] == 3.0;
                let lue = if soy { self.lue_gc_per_mj * self.soy_lue_factor } else { self.lue_gc_per_mj };
                let (gdd_sen, gdd_harv) = if soy { (self.gdd_senesce * self.soy_gdd_factor, self.gdd_harvest * self.soy_gdd_factor) } else { (self.gdd_senesce, self.gdd_harvest) };
                let grain_frac = if soy { self.soy_grain_fraction } else { self.grain_fraction };
                // perennial grass (a rule's buffer strip): a standing canopy that is never harvested
                if grass {
                    if self.stage[k] != 4 { self.stage[k] = 4; self.lai[k] = self.lai[k].max(0.3); }
                    let par_mj = self.day_energy[k] / a * self.par_fraction / 1e6;
                    let fapar = 1.0 - (-self.k_light * self.lai[k].max(0.05)).exp();
                    let wf = soil.as_ref().map(|s| (s[i] as f64 / (0.5 * self.soil_capacity_mm)).clamp(0.0, 1.0)).unwrap_or(1.0);
                    let g = lue * 0.7 * par_mj * fapar * wf * tf; let r = self.ra_fraction * g; let npp = g - r;
                    let turnover = self.biomass_gc[k] * 0.01; // leaves and roots to litter
                    self.biomass_gc[k] += npp - turnover; self.litter_gc[k] += turnover; to_litter += turnover / 1000.0 * a;
                    self.lai[k] = (self.lai[k] + self.sla_m2_per_gc * self.leaf_fraction * npp - 0.01 * self.lai[k]).clamp(0.0, self.grass_lai);
                    if self.day_tmin <= self.frost_c { self.lai[k] *= 0.5; }
                    gpp += g / 1000.0 * a; ra += r / 1000.0 * a;
                    let dl = self.litter_gc[k] * self.k_litter_per_day * tf.max(0.05);
                    self.litter_gc[k] -= dl; rh += dl / 1000.0 * a;
                    continue;
                }
                let plant_here = plant_today || agent_plant.as_ref().map(|p| p[i] > 0.5).unwrap_or(false);
                if plant_here && self.stage[k] != 1 && self.stage[k] != 2 { self.stage[k] = 1; self.gdd[k] = 0.0; if self.planted_doy == 0 { self.planted_doy = doy; } }
                if self.stage[k] >= 1 && self.stage[k] <= 2 { self.gdd[k] += (tmean - self.gdd_base_c).max(0.0); }
                // emergence: a seedling canopy from imported seed carbon, once
                if self.stage[k] == 1 && self.gdd[k] >= self.gdd_emerge && self.lai[k] < self.lai_emerge {
                    let leaf_c = self.lai_emerge / self.sla_m2_per_gc;
                    self.biomass_gc[k] += leaf_c; self.lai[k] = self.lai_emerge; seed_import += leaf_c / 1000.0 * a;
                }
                // growth
                if self.stage[k] == 1 && self.gdd[k] >= self.gdd_emerge {
                    let par_mj = self.day_energy[k] / a * self.par_fraction / 1e6;
                    let fapar = 1.0 - (-self.k_light * self.lai[k].max(0.05)).exp();
                    let wf = soil.as_ref().map(|s| (s[i] as f64 / (0.5 * self.soil_capacity_mm)).clamp(0.0, 1.0)).unwrap_or(1.0);
                    let g = lue * par_mj * fapar * wf * tf; // g C/m2/day
                    let r = self.ra_fraction * g;
                    let npp = g - r;
                    self.biomass_gc[k] += npp;
                    let dlai = (self.sla_m2_per_gc * self.leaf_fraction * npp).max(0.0);
                    self.lai[k] = (self.lai[k] + dlai).min(self.lai_max);
                    gpp += g / 1000.0 * a; ra += r / 1000.0 * a;
                    if self.gdd[k] >= gdd_sen { self.stage[k] = 2; self.lai_at_senesce[k] = self.lai[k]; }
                }
                // senescence: leaf area falls linearly with gdd to harvest; carbon stays in biomass until harvest
                if self.stage[k] == 2 {
                    let f = ((gdd_harv - self.gdd[k]) / (gdd_harv - gdd_sen)).clamp(0.0, 1.0);
                    self.lai[k] = self.lai_at_senesce[k] * f;
                    let frost = self.day_tmin <= self.frost_c;
                    if self.gdd[k] >= gdd_harv || frost {
                        let b = self.biomass_gc[k];
                        let grain = grain_frac * b;
                        grain_export[i] = (grain / 1000.0 * a) as f32; any_harvest = true;
                        export += grain / 1000.0 * a;
                        to_litter += (b - grain) / 1000.0 * a;
                        self.litter_gc[k] += b - grain;
                        self.biomass_gc[k] = 0.0; self.lai[k] = 0.0; self.stage[k] = 3;
                    }
                }
                if self.stage[k] == 3 && doy > 300 { self.stage[k] = 0; }
                // litter respiration
                let dl = self.litter_gc[k] * self.k_litter_per_day * tf.max(0.05);
                self.litter_gc[k] -= dl; rh += dl / 1000.0 * a;
            }
            if self.cells.iter().enumerate().all(|(k, _)| self.stage[k] == 3 || self.stage[k] == 0 || self.stage[k] == 4) && self.planted_doy > 0 && doy > 250 { self.harvested = true; }
            if any_harvest {
                let grid = self.grid.clone().unwrap();
                ctx.world_write("veg.grain_export_kgc", "kgC", &grid, grain_export);
                ctx.publish("harvest.done", json!({ "by": self.name }));
            }
            ctx.transfer(LEDGER, "import", "biomass", seed_import);
            ctx.transfer(LEDGER, "atmosphere", "biomass", gpp);
            ctx.transfer(LEDGER, "biomass", "atmosphere", ra);
            ctx.transfer(LEDGER, "biomass", "litter", to_litter);
            ctx.transfer(LEDGER, "biomass", "export", export);
            ctx.transfer(LEDGER, "litter", "atmosphere", rh);
            let (b, l) = self.stocks();
            ctx.declare_stock(LEDGER, "biomass", b);
            ctx.declare_stock(LEDGER, "litter", l);
            // publish leaf area and report
            let grid = self.grid.clone().unwrap();
            let mut lai_f = vec![f32::NAN; grid.nx * grid.ny];
            let mut ndvi_f = vec![f32::NAN; grid.nx * grid.ny];
            let (mut lai_sum, mut ndvi_sum) = (0.0, 0.0);
            let snowy = |i: usize| snow.as_ref().map(|s| s[i] as f64 > self.snow_ndvi_mm).unwrap_or(false);
            for (k, &i) in self.cells.iter().enumerate() {
                lai_f[i] = self.lai[k] as f32;
                let nd = if snowy(i) { 0.0 } else { self.ndvi(self.lai[k]) };
                ndvi_f[i] = nd as f32;
                lai_sum += self.lai[k] * self.area[k]; ndvi_sum += nd * self.area[k];
            }
            ctx.world_write("veg.lai", "m2/m2", &grid, lai_f.clone());
            let mut bf = vec![f32::NAN; grid.nx * grid.ny];
            for (k, &i) in self.cells.iter().enumerate() { bf[i] = self.biomass_gc[k] as f32; }
            ctx.world_write("veg.biomass_gc_m2", "gC/m2", &grid, bf);
            let area_tot = self.basin_km2 * 1e6;
            ctx.report("veg.lai_mean", "m2/m2", lai_sum / area_tot);
            ctx.report("veg.ndvi_sim_daily", "1", ndvi_sum / area_tot);
            ctx.report("veg.biomass_mean_gc_m2", "gC/m2", b * 1000.0 / area_tot);
            ctx.report("veg.gpp_basin_tc_day", "tC/day", gpp / 1000.0);
            if doy % 8 == 0 { ctx.field("lai", "m2/m2", &grid, lai_f); }
            // MODIS composite starting today: score pair and frames
            if let Some(obs) = self.modis.get(&day) {
                let (mut s, mut n) = (0.0f64, 0usize);
                let mut sim_at = 0.0f64;
                for (k, &i) in self.cells.iter().enumerate() { let o = obs[i]; if o.is_finite() { s += o as f64 * self.area[k]; sim_at += ndvi_f[i] as f64 * self.area[k]; n += 1; } }
                if n > 0 {
                    let atot: f64 = self.cells.iter().enumerate().filter(|(_, &i)| obs[i].is_finite()).map(|(k, _)| self.area[k]).sum();
                    ctx.report("veg.ndvi_obs_16d", "1", s / atot);
                    ctx.report("veg.ndvi_sim_16d", "1", sim_at / atot);
                }
                let mut of = vec![f32::NAN; grid.nx * grid.ny];
                for &i in &self.cells { of[i] = obs[i]; }
                ctx.field("ndvi_modis", "1", &grid, of);
                ctx.field("ndvi_sim", "1", &grid, ndvi_f);
            }
            self.day_energy.iter_mut().for_each(|v| *v = 0.0);
            self.day_t_sum = 0.0; self.day_t_n = 0; self.day_tmin = 99.0;
        }
        ctx.schedule_with_priority(self.dt, 3, "tick", json!(null));
    }

    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) {
        let mut g: Veg = serde_json::from_value(v).unwrap();
        g.load().expect("veg reload on restore");
        *self = g;
    }
}
