//! M9 Takers: farm agents. The basin's cells are grouped into farm blocks (a square of
//! cells each, about a neighbourhood of Iowa farms). Each block is an agent with cash,
//! a rotation memory and simple rules: each spring it chooses corn or soybean by expected
//! margin with a rotation bonus, decides a nitrogen rate, and plants when the soil is
//! workable and warm. It writes those decisions as fields the crop module reads
//! (`agents.crop_kind`, `agents.plant_now`, `agents.n_rate_kg_ha`), which is the brief's
//! seam "population to agents, and agents write a field". At harvest the crop module
//! publishes `harvest.done` with the grain exported per cell; the agent sells it at the
//! season's price and pays its input costs, on a money ledger in dollars.
//!
//! Prices and costs are parameters with the agent's recollection of typical recent US
//! levels, marked as such; no market data is fetched yet. A rule module (M10) can write
//! `rules.no_crop` to forbid cropping in cells, which the agents obey.

use esim_kernel::manifest::{param_duration, param_f64};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module};
use esim_registry::hydro::delineate;
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

const MONEY: &str = "money";
pub const CROP_NONE: f32 = 0.0;
pub const CROP_CORN: f32 = 1.0;
pub const CROP_SOY: f32 = 2.0;
pub const CROP_GRASS: f32 = 3.0;

#[derive(Serialize, Deserialize)]
pub struct Farms {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64, gauge_lat: f64, gauge_lon: f64, snap_radius: usize,
    dt: Duration,
    block_cells: usize,
    // economics (per hectare unless said), parameters with typical recent US levels
    price_corn_usd_bu: f64, price_soy_usd_bu: f64,
    cost_corn_usd_ha: f64, cost_soy_usd_ha: f64, land_cost_usd_ha: f64,
    expect_corn_bu_ac: f64, expect_soy_bu_ac: f64,
    rotation_bonus_bu_ac: f64, corn_on_corn_penalty_bu_ac: f64,
    n_rate_corn_kg_ha: f64, n_rate_soy_kg_ha: f64,
    n_price_usd_kg: f64,
    kgc_per_bu_corn: f64, kgc_per_bu_soy: f64,
    plant_after_doy: u32, plant_t_c: f64, workable_soil_frac: f64, soil_capacity_mm: f64,
    // state
    cash: Vec<f64>,           // per block
    last_crop: Vec<f32>,      // per block
    crop_kind: Vec<f32>,      // per cell (basin order)
    planted_year: Vec<i64>,   // per block
    warm_days: u32, day_t_sum: f64, day_t_n: u32,
    revenue_total: f64, cost_total: f64,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] cells: Vec<usize>,
    #[serde(skip)] area_ha: Vec<f64>,
    #[serde(skip)] block_of: Vec<usize>,
    #[serde(skip)] n_blocks: usize,
}

impl Farms {
    pub fn new(name: &str, p: &toml::Value) -> Result<Farms, String> {
        let mut f = Farms {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 42.3), lat_n: param_f64(p, "lat_n", 43.2), lon_w: param_f64(p, "lon_w", -94.2), lon_e: param_f64(p, "lon_e", -93.5),
            res_arcsec: param_f64(p, "res_arcsec", 30.0), gauge_lat: param_f64(p, "gauge_lat", 42.4319722), gauge_lon: param_f64(p, "gauge_lon", -93.8059444), snap_radius: param_f64(p, "snap_radius", 3.0) as usize,
            dt: param_duration(p, "dt", "1d")?,
            block_cells: param_f64(p, "block_cells", 4.0) as usize,
            price_corn_usd_bu: param_f64(p, "price_corn_usd_bu", 4.3), price_soy_usd_bu: param_f64(p, "price_soy_usd_bu", 10.2),
            cost_corn_usd_ha: param_f64(p, "cost_corn_usd_ha", 1100.0), cost_soy_usd_ha: param_f64(p, "cost_soy_usd_ha", 700.0), land_cost_usd_ha: param_f64(p, "land_cost_usd_ha", 650.0),
            expect_corn_bu_ac: param_f64(p, "expect_corn_bu_ac", 200.0), expect_soy_bu_ac: param_f64(p, "expect_soy_bu_ac", 58.0),
            rotation_bonus_bu_ac: param_f64(p, "rotation_bonus_bu_ac", 12.0), corn_on_corn_penalty_bu_ac: param_f64(p, "corn_on_corn_penalty_bu_ac", 20.0),
            n_rate_corn_kg_ha: param_f64(p, "n_rate_corn_kg_ha", 170.0), n_rate_soy_kg_ha: param_f64(p, "n_rate_soy_kg_ha", 0.0),
            n_price_usd_kg: param_f64(p, "n_price_usd_kg", 1.1),
            kgc_per_bu_corn: param_f64(p, "kgc_per_bu_corn", 9.0), kgc_per_bu_soy: param_f64(p, "kgc_per_bu_soy", 12.0),
            plant_after_doy: param_f64(p, "plant_after_doy", 105.0) as u32, plant_t_c: param_f64(p, "plant_t_c", 10.0), workable_soil_frac: param_f64(p, "workable_soil_frac", 0.85), soil_capacity_mm: param_f64(p, "soil_capacity_mm", 150.0),
            cash: Vec::new(), last_crop: Vec::new(), crop_kind: Vec::new(), planted_year: Vec::new(), warm_days: 0, day_t_sum: 0.0, day_t_n: 0, revenue_total: 0.0, cost_total: 0.0,
            grid: None, cells: Vec::new(), area_ha: Vec::new(), block_of: Vec::new(), n_blocks: 0,
        };
        f.load()?;
        Ok(f)
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
        // blocks: square tiles of block_cells x block_cells grid cells
        let bx = grid.nx.div_ceil(self.block_cells);
        let mut ids = std::collections::BTreeMap::new();
        self.block_of = cells.iter().map(|&i| {
            let (c, r) = (i % grid.nx, i / grid.nx);
            let key = (r / self.block_cells) * bx + c / self.block_cells;
            let n = ids.len();
            *ids.entry(key).or_insert(n)
        }).collect();
        self.n_blocks = ids.len();
        if self.cash.len() != self.n_blocks {
            self.cash = vec![0.0; self.n_blocks];
            self.last_crop = vec![CROP_SOY; self.n_blocks];
            self.planted_year = vec![0; self.n_blocks];
            // alternate the starting rotation by block so the basin is half and half
            for (b, lc) in self.last_crop.iter_mut().enumerate() { if b % 2 == 0 { *lc = CROP_CORN; } }
        }
        if self.crop_kind.len() != cells.len() { self.crop_kind = vec![CROP_NONE; cells.len()]; }
        self.grid = Some(grid); self.cells = cells;
        Ok(())
    }
    fn year_of(day_secs: i64) -> i64 {
        let z = day_secs.div_euclid(86_400) + 719_468; let era = z.div_euclid(146_097); let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); let mp = (5 * doy + 2) / 153; let m = if mp < 10 { mp + 3 } else { mp - 9 };
        if m <= 2 { y + 1 } else { y }
    }
    fn doy_of(day_secs: i64) -> u32 {
        let y = Self::year_of(day_secs);
        let jan1 = { let y0 = y - 1; let era = y0.div_euclid(400); let yoe = y0 - era * 400; let doy = (153 * 10 + 2) / 5; let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; era * 146_097 + doe - 719_468 };
        (day_secs.div_euclid(86_400) - jan1) as u32 + 1
    }
    fn write_fields(&self, ctx: &mut Ctx, plant_now: &[f32]) {
        let g = self.grid.as_ref().unwrap();
        let n = g.nx * g.ny;
        let (mut kind, mut plant, mut nrate) = (vec![f32::NAN; n], vec![f32::NAN; n], vec![f32::NAN; n]);
        for (k, &i) in self.cells.iter().enumerate() {
            kind[i] = self.crop_kind[k];
            plant[i] = plant_now[k];
            nrate[i] = match self.crop_kind[k] { x if x == CROP_CORN => self.n_rate_corn_kg_ha as f32, x if x == CROP_SOY => self.n_rate_soy_kg_ha as f32, _ => 0.0 };
        }
        ctx.world_write("agents.crop_kind", "0 none 1 corn 2 soy 3 grass", g, kind);
        ctx.world_write("agents.plant_now", "1", g, plant);
        ctx.world_write("agents.n_rate_kg_ha", "kgN/ha", g, nrate);
    }
}

impl Module for Farms {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.ledger(MONEY, "USD", 1e-3);
        ctx.declare_stock(MONEY, "farms", self.cash.iter().sum());
        ctx.subscribe("harvest.done");
        ctx.report("agents.farm_blocks", "1", self.n_blocks as f64);
        let zero = vec![0f32; self.cells.len()];
        self.write_fields(ctx, &zero);
        ctx.schedule_with_priority(self.dt, 2, "tick", json!(null)); // before the crop (3) reads
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "tick" => {
                let now = ctx.now().0;
                let day = (now - 1).div_euclid(86_400) * 86_400;
                let doy = Self::doy_of(day); let year = Self::year_of(day);
                // temperature over the basin for the planting rule
                if let Some(t) = ctx.world_read("wx.t2m_c") {
                    let mut s = 0.0; let mut n = 0u32;
                    for &i in &self.cells { let v = t.data[i] as f64; if v.is_finite() { s += v; n += 1; } }
                    if n > 0 { self.day_t_sum += s / n as f64; self.day_t_n += 1; }
                }
                let tmean = if self.day_t_n > 0 { self.day_t_sum / self.day_t_n as f64 } else { 10.0 };
                self.day_t_sum = 0.0; self.day_t_n = 0;
                if tmean >= self.plant_t_c { self.warm_days += 1; } else { self.warm_days = 0; }
                let soil: Option<Vec<f32>> = ctx.world_read("water.soil_mm").map(|f| f.data.clone());
                let no_crop: Option<Vec<f32>> = ctx.world_read("rules.no_crop").map(|f| f.data.clone());
                let mut plant_now = vec![0f32; self.cells.len()];
                let mut planted_blocks = 0;
                let mut costs = 0.0;
                // 1. the decision, once a year on the first day of the window: corn or soy by margin and rotation
                if doy == self.plant_after_doy {
                    for b in 0..self.n_blocks {
                        let rot_corn = if self.last_crop[b] == CROP_SOY { self.rotation_bonus_bu_ac } else { -self.corn_on_corn_penalty_bu_ac };
                        let m_corn = (self.expect_corn_bu_ac + rot_corn) * 2.471 * self.price_corn_usd_bu - self.cost_corn_usd_ha - self.n_rate_corn_kg_ha * self.n_price_usd_kg;
                        let m_soy = self.expect_soy_bu_ac * 2.471 * self.price_soy_usd_bu - self.cost_soy_usd_ha;
                        let choice = if m_corn >= m_soy { CROP_CORN } else { CROP_SOY };
                        for (k, &bb) in self.block_of.iter().enumerate() { if bb == b { self.crop_kind[k] = choice; } }
                    }
                    // rules: cells forbidden to crop become perennial grass
                    if let Some(nc) = &no_crop { for (k, &i) in self.cells.iter().enumerate() { if nc[i] > 0.5 { self.crop_kind[k] = CROP_GRASS; } } }
                }
                // 2. planting: a block plants when warm enough and its soil is workable, once per year
                if doy >= self.plant_after_doy && doy < 190 && self.warm_days >= 5 {
                    for b in 0..self.n_blocks {
                        if self.planted_year[b] == year { continue; }
                        let (mut wet, mut cnt) = (0.0, 0);
                        for (k, &bb) in self.block_of.iter().enumerate() { if bb == b { if let Some(s) = &soil { wet += s[self.cells[k]] as f64; cnt += 1; } } }
                        let workable = cnt == 0 || wet / cnt as f64 <= self.workable_soil_frac * self.soil_capacity_mm;
                        if !workable { continue; }
                        self.planted_year[b] = year; planted_blocks += 1;
                        for (k, &bb) in self.block_of.iter().enumerate() {
                            if bb != b || self.crop_kind[k] == CROP_GRASS || self.crop_kind[k] == CROP_NONE { continue; }
                            plant_now[k] = 1.0;
                            let ha = self.area_ha[k];
                            costs += ha * (if self.crop_kind[k] == CROP_CORN { self.cost_corn_usd_ha + self.n_rate_corn_kg_ha * self.n_price_usd_kg } else { self.cost_soy_usd_ha } + self.land_cost_usd_ha);
                            self.cash[b] -= ha * (if self.crop_kind[k] == CROP_CORN { self.cost_corn_usd_ha + self.n_rate_corn_kg_ha * self.n_price_usd_kg } else { self.cost_soy_usd_ha } + self.land_cost_usd_ha);
                        }
                        self.last_crop[b] = {
                            // the block's dominant crop this year
                            let mut c = 0; let mut s = 0;
                            for (k, &bb) in self.block_of.iter().enumerate() { if bb == b { if self.crop_kind[k] == CROP_CORN { c += 1; } else if self.crop_kind[k] == CROP_SOY { s += 1; } } }
                            if c >= s { CROP_CORN } else { CROP_SOY }
                        };
                    }
                }
                if costs > 0.0 { ctx.transfer(MONEY, "farms", "suppliers_and_land", costs); self.cost_total += costs; }
                ctx.declare_stock(MONEY, "farms", self.cash.iter().sum());
                if planted_blocks > 0 { ctx.report("agents.blocks_planted_day", "1/day", planted_blocks as f64); }
                self.write_fields(ctx, &plant_now);
                ctx.report("agents.cash_total_usd", "USD", self.cash.iter().sum());
                if doy == self.plant_after_doy {
                    let corn = self.crop_kind.iter().zip(&self.area_ha).filter(|(k, _)| **k == CROP_CORN).map(|(_, a)| a).sum::<f64>();
                    let soy = self.crop_kind.iter().zip(&self.area_ha).filter(|(k, _)| **k == CROP_SOY).map(|(_, a)| a).sum::<f64>();
                    ctx.report("agents.corn_ha", "ha", corn); ctx.report("agents.soy_ha", "ha", soy);
                }
                ctx.schedule_with_priority(self.dt, 2, "tick", json!(null));
            }
            "harvest.done" => {
                // grain exported per cell, in kgC, from the crop module's field
                let Some(gr) = ctx.world_read("veg.grain_export_kgc").map(|f| f.data.clone()) else { return };
                let mut revenue = 0.0; let mut bu_corn = 0.0; let mut bu_soy = 0.0;
                for (k, &i) in self.cells.iter().enumerate() {
                    let kgc = gr[i].max(0.0) as f64; if kgc <= 0.0 { continue; }
                    let (bu, price) = if self.crop_kind[k] == CROP_SOY { (kgc / self.kgc_per_bu_soy, self.price_soy_usd_bu) } else { (kgc / self.kgc_per_bu_corn, self.price_corn_usd_bu) };
                    if self.crop_kind[k] == CROP_SOY { bu_soy += bu; } else { bu_corn += bu; }
                    let usd = bu * price;
                    self.cash[self.block_of[k]] += usd; revenue += usd;
                }
                if revenue > 0.0 { ctx.transfer(MONEY, "market", "farms", revenue); self.revenue_total += revenue; }
                ctx.declare_stock(MONEY, "farms", self.cash.iter().sum());
                ctx.report("agents.revenue_usd_day", "USD/day", revenue);
                if bu_corn > 0.0 { ctx.report("agents.corn_bu_day", "bu/day", bu_corn); }
                if bu_soy > 0.0 { ctx.report("agents.soy_bu_day", "bu/day", bu_soy); }
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut f: Farms = serde_json::from_value(v).unwrap(); f.load().expect("farms reload"); *self = f; }
}

impl Farms {
    pub fn describe(name: &str, p: &toml::Value) -> esim_kernel::describe::Describe {
        esim_kernel::describe::Describe::new("agents.farms", name, "Farm agents on blocks of basin cells: choose corn or soy each spring by expected margin and rotation, plant when the soil is warm and workable and no rule forbids, pay costs, sell the harvest, keep a money ledger; write their orders as fields the crop and the nitrogen module read.")
            .priority(2).dt(esim_kernel::manifest::param_str(p, "dt", "1d"))
            .param("lat_s", 42.3, "deg", "").param("lat_n", 43.2, "deg", "").param("lon_w", -94.2, "deg", "").param("lon_e", -93.5, "deg", "")
            .param("res_arcsec", 30.0, "arcsec", "").param("gauge_lat", 42.4319722, "deg", "").param("gauge_lon", -93.8059444, "deg", "").param("snap_radius", 3, "cells", "")
            .param("dt", "1d", "", "").param("block_cells", 4, "cells", "cells on a side of one farm block")
            .param("price_corn_usd_bu", 4.3, "USD/bu", "").param("price_soy_usd_bu", 10.2, "USD/bu", "")
            .param("cost_corn_usd_ha", 1100.0, "USD/ha", "").param("cost_soy_usd_ha", 700.0, "USD/ha", "").param("land_cost_usd_ha", 650.0, "USD/ha", "")
            .param("expect_corn_bu_ac", 200.0, "bu/ac", "").param("expect_soy_bu_ac", 58.0, "bu/ac", "")
            .param("rotation_bonus_bu_ac", 12.0, "bu/ac", "").param("corn_on_corn_penalty_bu_ac", 20.0, "bu/ac", "so the basin rotates about half and half")
            .param("n_rate_corn_kg_ha", 170.0, "kgN/ha", "").param("n_rate_soy_kg_ha", 0.0, "kgN/ha", "").param("n_price_usd_kg", 1.1, "USD/kg", "")
            .param("kgc_per_bu_corn", 9.0, "kgC/bu", "").param("kgc_per_bu_soy", 12.0, "kgC/bu", "")
            .param("plant_after_doy", 105, "doy", "").param("plant_t_c", 10.0, "degC", "").param("workable_soil_frac", 0.85, "", "of soil capacity").param("soil_capacity_mm", 150.0, "mm", "")
            .subscribe("harvest.done", "the crop has exported grain")
            .own("agents.crop_kind", "0 none 1 corn 2 soy 3 grass").own("agents.plant_now", "1").own("agents.n_rate_kg_ha", "kgN/ha")
            .read("wx.t2m_c", "degC", true, "warm days before planting").read("water.soil_mm", "mm", false, "workable soil")
            .read("rules.no_crop", "", false, "cells a rule closes to crops").read_on("veg.grain_export_kgc", "kgC", false, "harvest.done", "revenue")
            .ledger("money", "USD", 1e-3)
            .report("agents.farm_blocks", "1").report("agents.blocks_planted_day", "1/day").report("agents.cash_total_usd", "USD")
            .report("agents.corn_ha", "ha").report("agents.soy_ha", "ha").report("agents.revenue_usd_day", "USD/day").report("agents.corn_bu_day", "bu/day").report("agents.soy_bu_day", "bu/day")
            .asset("copernicus-dem-glo30", "basin cells")
            .note("Parameter provenance: devlog/20261101-m9-takers.md; prices and costs are round numbers, not quotes")
    }
}
