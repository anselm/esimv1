//! M11 Nitrogen: fertilizer to river. Per basin cell: a mineral nitrogen pool fed by the
//! farm agents' fertilizer at planting and by mineralisation of a large soil organic
//! pool (Iowa's mollisols mineralise a lot, which is why its rivers run high in nitrate
//! even without fertilizer), drawn down by crop uptake in proportion to the crop module's
//! daily carbon gain, by denitrification when wet, and by leaching in proportion to the
//! water module's drainage. Soybean fixes half its nitrogen from the air. Harvest exports
//! grain nitrogen; residue returns to the organic pool. Leached nitrate enters the water
//! module's channel stores and rides its routing to the gauge, where the concentration is
//! scored against USGS 05481000's daily nitrate record (2012 to 2017).
//!
//! Nitrogen ledger (kg N): fertilizer -> soil_mineral; atmosphere -> crop_n (fixation);
//! soil_organic -> soil_mineral (mineralisation); soil_mineral -> crop_n | atmosphere
//! (denitrification) | channel_n (leaching); crop_n -> export | soil_organic (residue);
//! channel_n -> outlet (by the water module). Stocks: soil_mineral, soil_organic, crop_n
//! here; channel_n in the water module. Two modules, one ledger, as with energy in M2.
//! Every rate is a published order of magnitude named as a parameter.

use esim_kernel::manifest::{param_duration, param_f64};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module};
use esim_registry::hydro::delineate;
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

pub const LEDGER: &str = "nitrogen";

#[derive(Serialize, Deserialize)]
pub struct Nitrogen {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64, gauge_lat: f64, gauge_lon: f64, snap_radius: usize,
    dt: Duration,
    init_mineral_kg_ha: f64, init_organic_kg_ha: f64,
    mineralisation_per_day: f64, denitrification_per_day: f64,
    n_per_c_biomass: f64, n_per_c_grain: f64, soy_fixation_frac: f64,
    leach_efficiency: f64, soil_capacity_mm: f64,
    // state per basin cell (kg N per cell)
    mineral: Vec<f64>, organic: Vec<f64>, crop: Vec<f64>,
    last_biomass_gc: Vec<f32>,
    fert_total: f64, leach_total: f64, uptake_total: f64,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] cells: Vec<usize>,
    #[serde(skip)] area_ha: Vec<f64>,
}

impl Nitrogen {
    pub fn new(name: &str, p: &toml::Value) -> Result<Nitrogen, String> {
        let mut n = Nitrogen {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 42.3), lat_n: param_f64(p, "lat_n", 43.2), lon_w: param_f64(p, "lon_w", -94.2), lon_e: param_f64(p, "lon_e", -93.5),
            res_arcsec: param_f64(p, "res_arcsec", 30.0), gauge_lat: param_f64(p, "gauge_lat", 42.4319722), gauge_lon: param_f64(p, "gauge_lon", -93.8059444), snap_radius: param_f64(p, "snap_radius", 3.0) as usize,
            dt: param_duration(p, "dt", "1d")?,
            init_mineral_kg_ha: param_f64(p, "init_mineral_kg_ha", 40.0), init_organic_kg_ha: param_f64(p, "init_organic_kg_ha", 8000.0),
            mineralisation_per_day: param_f64(p, "mineralisation_per_day", 4e-5), denitrification_per_day: param_f64(p, "denitrification_per_day", 0.004),
            n_per_c_biomass: param_f64(p, "n_per_c_biomass", 0.025), n_per_c_grain: param_f64(p, "n_per_c_grain", 0.03), soy_fixation_frac: param_f64(p, "soy_fixation_frac", 0.5),
            leach_efficiency: param_f64(p, "leach_efficiency", 1.0), soil_capacity_mm: param_f64(p, "soil_capacity_mm", 150.0),
            mineral: Vec::new(), organic: Vec::new(), crop: Vec::new(), last_biomass_gc: Vec::new(), fert_total: 0.0, leach_total: 0.0, uptake_total: 0.0,
            grid: None, cells: Vec::new(), area_ha: Vec::new(),
        };
        n.load()?;
        Ok(n)
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
        self.area_ha = cells.iter().map(|&i| row_area[i / grid.nx] / 1e4).collect();
        let n = cells.len();
        if self.mineral.len() != n {
            self.mineral = self.area_ha.iter().map(|a| a * self.init_mineral_kg_ha).collect();
            self.organic = self.area_ha.iter().map(|a| a * self.init_organic_kg_ha).collect();
            self.crop = vec![0.0; n]; self.last_biomass_gc = vec![0.0; n];
        }
        self.grid = Some(grid); self.cells = cells;
        Ok(())
    }
    fn stocks(&self) -> (f64, f64, f64) { (self.mineral.iter().sum(), self.organic.iter().sum(), self.crop.iter().sum()) }
}

impl Module for Nitrogen {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.ledger(LEDGER, "kgN", 1e-3);
        let (m, o, c) = self.stocks();
        ctx.declare_stock(LEDGER, "soil_mineral", m); ctx.declare_stock(LEDGER, "soil_organic", o); ctx.declare_stock(LEDGER, "crop_n", c);
        ctx.subscribe("harvest.done");
        ctx.schedule_with_priority(self.dt, 7, "tick", json!(null)); // after water, crop, agents
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        let g = self.grid.clone().unwrap();
        match ev.topic.as_str() {
            "tick" => {
                let plant: Option<Vec<f32>> = ctx.world_read("agents.plant_now").map(|f| f.data.clone());
                let nrate: Option<Vec<f32>> = ctx.world_read("agents.n_rate_kg_ha").map(|f| f.data.clone());
                let kind: Option<Vec<f32>> = ctx.world_read("agents.crop_kind").map(|f| f.data.clone());
                let biomass: Option<Vec<f32>> = ctx.world_read("veg.biomass_gc_m2").map(|f| f.data.clone());
                let soil: Option<Vec<f32>> = ctx.world_read("water.soil_mm").map(|f| f.data.clone());
                let drain: Option<Vec<f32>> = ctx.world_read("water.drain_daily_mm").map(|f| f.data.clone());
                let t2m: Option<Vec<f32>> = ctx.world_read("wx.t2m_c").map(|f| f.data.clone());
                let (mut fert, mut fix, mut mineralised, mut uptake, mut denit, mut leached) = (0.0f64, 0.0, 0.0, 0.0, 0.0, 0.0);
                let mut leach_field = vec![f32::NAN; g.nx * g.ny];
                for (k, &i) in self.cells.iter().enumerate() {
                    let ha = self.area_ha[k];
                    // fertilizer on planting day
                    if plant.as_ref().map(|p| p[i] > 0.5).unwrap_or(false) {
                        let f = nrate.as_ref().map(|r| r[i].max(0.0) as f64).unwrap_or(0.0) * ha;
                        self.mineral[k] += f; fert += f;
                    }
                    // mineralisation with temperature
                    let tf = t2m.as_ref().map(|t| ((t[i] as f64 - 0.0) / 25.0).clamp(0.0, 1.5)).unwrap_or(0.5);
                    let m = self.organic[k] * self.mineralisation_per_day * tf;
                    self.organic[k] -= m; self.mineral[k] += m; mineralised += m;
                    // crop uptake follows the crop's carbon gain
                    if let Some(b) = &biomass {
                        let db = (b[i] - self.last_biomass_gc[k]).max(0.0) as f64; // gC/m2 gained today
                        self.last_biomass_gc[k] = if b[i].is_finite() { b[i] } else { 0.0 };
                        let need = db * 10.0 * ha * self.n_per_c_biomass; // gC/m2 -> kgC/ha -> kgN
                        let soy = kind.as_ref().map(|c| c[i] == 2.0).unwrap_or(false);
                        let from_air = if soy { need * self.soy_fixation_frac } else { 0.0 };
                        let from_soil = (need - from_air).min(self.mineral[k]);
                        self.mineral[k] -= from_soil; self.crop[k] += from_soil + from_air; uptake += from_soil; fix += from_air;
                    }
                    // denitrification when wet
                    let wet = soil.as_ref().map(|s| (s[i] as f64 / self.soil_capacity_mm).clamp(0.0, 1.0)).unwrap_or(0.5);
                    let d = self.mineral[k] * self.denitrification_per_day * wet * wet;
                    self.mineral[k] -= d; denit += d;
                    // leaching: the fraction of soil water that drained today carries that fraction of nitrate
                    if let (Some(dr), Some(s)) = (&drain, &soil) {
                        let frac = (dr[i] as f64 / (s[i] as f64 + dr[i] as f64).max(1.0)).clamp(0.0, 1.0) * self.leach_efficiency;
                        let l = self.mineral[k] * frac;
                        self.mineral[k] -= l; leached += l; leach_field[i] = l as f32;
                    } else { leach_field[i] = 0.0; }
                }
                self.fert_total += fert; self.leach_total += leached; self.uptake_total += uptake;
                ctx.transfer(LEDGER, "fertilizer", "soil_mineral", fert);
                ctx.transfer(LEDGER, "atmosphere", "crop_n", fix);
                ctx.transfer(LEDGER, "soil_organic", "soil_mineral", mineralised);
                ctx.transfer(LEDGER, "soil_mineral", "crop_n", uptake);
                ctx.transfer(LEDGER, "soil_mineral", "atmosphere", denit);
                ctx.transfer(LEDGER, "soil_mineral", "channel_n", leached);
                let (m, o, c) = self.stocks();
                ctx.declare_stock(LEDGER, "soil_mineral", m); ctx.declare_stock(LEDGER, "soil_organic", o); ctx.declare_stock(LEDGER, "crop_n", c);
                // the water module routes what leached
                ctx.world_write("nitrogen.leached_kg", "kgN", &g, leach_field);
                ctx.publish("nitrogen.leached", json!({ "kg": leached }));
                let area: f64 = self.area_ha.iter().sum();
                ctx.report("nitrogen.soil_mineral_kg_ha", "kgN/ha", m / area);
                ctx.report("nitrogen.leached_kg_day", "kgN/day", leached);
                ctx.report("nitrogen.fertilizer_kg_day", "kgN/day", fert);
                ctx.report("nitrogen.uptake_kg_day", "kgN/day", uptake);
                ctx.report("nitrogen.mineralised_kg_day", "kgN/day", mineralised);
                ctx.schedule_with_priority(self.dt, 7, "tick", json!(null));
            }
            "harvest.done" => {
                let Some(gr) = ctx.world_read("veg.grain_export_kgc").map(|f| f.data.clone()) else { return };
                let (mut export, mut residue) = (0.0, 0.0);
                for (k, &i) in self.cells.iter().enumerate() {
                    let kgc = gr[i].max(0.0) as f64; if kgc <= 0.0 { continue; }
                    let e = (kgc * self.n_per_c_grain).min(self.crop[k]);
                    let r = self.crop[k] - e;
                    self.crop[k] = 0.0; self.organic[k] += r; export += e; residue += r;
                    self.last_biomass_gc[k] = 0.0;
                }
                ctx.transfer(LEDGER, "crop_n", "export", export);
                ctx.transfer(LEDGER, "crop_n", "soil_organic", residue);
                let (m, o, c) = self.stocks();
                ctx.declare_stock(LEDGER, "soil_mineral", m); ctx.declare_stock(LEDGER, "soil_organic", o); ctx.declare_stock(LEDGER, "crop_n", c);
                ctx.report("nitrogen.harvest_export_kg", "kgN", export);
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut n: Nitrogen = serde_json::from_value(v).unwrap(); n.load().expect("nitrogen reload"); *self = n; }
}
