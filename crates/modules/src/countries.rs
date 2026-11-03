//! The planetary stack above the climate, with the country as the coarse unit. Three
//! kinds, each a layer with its own record and a stated response, and one that reads a
//! table of published thresholds:
//!
//! - `people.countries`: outlines (Natural Earth) rasterised to a coarse world grid, and
//!   people per country per year (UN World Population Prospects: estimates to 2023, the
//!   UN's medium projection after, labelled as such). A data layer; nothing modelled.
//! - `climate.countries`: each country's mean temperature from its own record (CRU TS
//!   through the World Bank portal) regressed on the global anomaly over 1901 to 2022.
//!   The slope is the country's observed share of global warming; the run's global
//!   anomaly times the slope gives the country's future. Pattern scaling from the record,
//!   with the stated assumption that the pattern persists.
//! - `people.heat`: people living in countries whose annual mean has left the human
//!   climate niche (Xu et al. 2020: mean annual temperature above 29 °C), and a second
//!   marker on the hottest month that has no paper behind it and says so. Publishes a
//!   `people.crossing` event the first year a country crosses, which the auditor lists.
//! - `tipping.elements`: `data/events/tipping.toml`, the climate tipping elements with the
//!   warming at which each is assessed likely to tip (central and range), checked
//!   against the run's global anomaly; a `tipping.crossing` event when the range's low end
//!   and the central estimate are passed.
//!
//! Countries are a crude unit and the modules say so: a whole country is in or out of
//! the niche at once, and a half-degree record averaged over a country hides its deserts
//! and its mountains. The unit was chosen because borders are where people's options
//! change, and because the records exist at this unit today.
//!
//! Since 2026-11-02 each of these kinds also has a cells mode, chosen by a parameter and
//! leaving the country mode as the default: people per cell from WorldPop's 1 km count
//! (scaled to the UN totals), a baseline temperature and daily temperature and dew point
//! per cell from the MERRA-2 reanalysis through NASA POWER, and exposure computed cell by
//! cell before it is summed by country. A country then crosses a limit when a stated
//! share of its people live where the limit is passed, not when its capital's station
//! does. The warming applied to a cell is still its country's, from the country record.

use esim_kernel::describe::Describe;
use esim_kernel::manifest::param_f64;
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module, Time};
use esim_registry::countries::{self, Country};
use esim_registry::cells;
use esim_registry::{Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// "1.7 billion", "95 million", "600,000".
fn people_words(p: f64) -> String {
    if p >= 1e9 { format!("{:.1} billion", p / 1e9) } else if p >= 1e6 { format!("{:.0} million", p / 1e6) } else { format!("{}", (p as i64).to_string().as_bytes().rchunks(3).rev().map(|c| std::str::from_utf8(c).unwrap()).collect::<Vec<_>>().join(",")) }
}

fn root() -> PathBuf { std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")) }
fn asset_url(cat: &Catalog, id: &str) -> Result<String, String> { Ok(cat.asset(id).ok_or_else(|| format!("catalog has no {id}"))?.url_template.clone()) }
fn year_of(t: Time) -> i64 { t.to_iso()[0..4].parse().unwrap() }
fn to_next_year(now: Time) -> Duration { Time::parse(&format!("{:04}-01-01", year_of(now) + 1)).unwrap().since(now) }
fn world_grid(res_deg: f64) -> LatLonGrid {
    LatLonGrid { nx: (360.0 / res_deg).round() as usize, ny: (180.0 / res_deg).round() as usize, dlon: res_deg, dlat: res_deg, lon_w: -180.0, lat_n: 90.0 }
}
fn scale_asset(scale: &str) -> &'static str { if scale == "50m" { "natural-earth-50m-countries" } else { "natural-earth-110m-countries" } }
fn load_countries(scale: &str) -> Result<Vec<Country>, String> {
    let root = root();
    let cat = Catalog::load(&root.join("data/catalog.toml"))?;
    let mut f = Fetcher::default_for(&root)?;
    countries::natural_earth_countries(&mut f, &asset_url(&cat, scale_asset(scale))?)
}
fn param_str(p: &toml::Value, key: &str, default: &str) -> String { esim_kernel::manifest::param_str(p, key, default).to_string() }

/// How a fine field is folded into a coarser frame for the page.
#[derive(Clone, Copy)]
enum Fold { Sum, Mean, Max, Min }

/// A frame on a grid `factor` times coarser than `g`, NaN where every fine cell is NaN.
fn coarsen(g: &LatLonGrid, data: &[f32], factor: usize, how: Fold) -> (LatLonGrid, Vec<f32>) {
    if factor <= 1 { return (g.clone(), data.to_vec()); }
    let cg = LatLonGrid { nx: g.nx / factor, ny: g.ny / factor, dlon: g.dlon * factor as f64, dlat: g.dlat * factor as f64, lon_w: g.lon_w, lat_n: g.lat_n };
    let mut out = vec![f32::NAN; cg.nx * cg.ny];
    for row in 0..cg.ny {
        for col in 0..cg.nx {
            let (mut acc, mut n) = (match how { Fold::Max => f32::NEG_INFINITY, Fold::Min => f32::INFINITY, _ => 0.0 }, 0usize);
            for r in 0..factor { for c in 0..factor {
                let v = data[(row * factor + r) * g.nx + col * factor + c];
                if v.is_nan() { continue; }
                n += 1;
                acc = match how { Fold::Sum | Fold::Mean => acc + v, Fold::Max => acc.max(v), Fold::Min => acc.min(v) };
            } }
            if n > 0 { out[row * cg.nx + col] = if let Fold::Mean = how { acc / n as f32 } else { acc }; }
        }
    }
    (cg, out)
}
fn frame_factor(res_deg: f64, frame_res_deg: f64) -> usize { ((frame_res_deg / res_deg).round() as usize).max(1) }

/// The 10-degree POWER box a cell centre falls in.
fn box_of(lat: f64, lon: f64) -> (i32, i32) { (((lat / 10.0).floor() * 10.0) as i32, ((lon / 10.0).floor() * 10.0) as i32) }

/// For a POWER box answer, the index of the point nearest each listed cell centre.
fn points_for(b: &cells::PowerBox, g: &LatLonGrid, cells_in: &[usize]) -> Vec<Option<usize>> {
    let key = |la: f32, lo: f32| ((la * 2.0).round() as i32, (lo * 1000.0).round() as i32);
    let map: std::collections::HashMap<(i32, i32), usize> = b.points.iter().enumerate().map(|(i, &(la, lo))| (key(la, lo), i)).collect();
    cells_in.iter().map(|&i| {
        let (col, row) = g.col_row(i);
        let (la, lo) = cells::nearest_power_point(g.lat(row), g.lon(col));
        map.get(&key(la, lo)).or_else(|| map.get(&key(la, lo + 360.0))).copied()
    }).collect()
}

// ---------------------------------------------------------------------------------------
// people.countries

#[derive(Serialize, Deserialize)]
pub struct PeopleCountries {
    name: String,
    res_deg: f64, frame_res_deg: f64, frame_every_years: i64, hold_before_record: bool,
    scale: String, distribution: String,
    #[serde(skip)] countries: Vec<Country>,
    #[serde(skip)] index: Vec<f32>,        // country index per cell, NaN for sea
    #[serde(skip)] land_km2: Vec<f64>,     // per country, from the raster
    #[serde(skip)] weights: Vec<f32>,      // WorldPop people per cell (cells mode), else empty
    #[serde(skip)] wsum: Vec<f64>,         // per country, the weights' sum
    #[serde(skip)] unassigned: usize,      // populated cells no country claimed
    #[serde(skip)] pop: BTreeMap<String, BTreeMap<i64, f64>>,
    last_frame_year: i64,
}

impl PeopleCountries {
    pub fn new(name: &str, p: &toml::Value) -> Result<PeopleCountries, String> {
        let res = param_f64(p, "res_deg", 2.0);
        let mut m = PeopleCountries {
            name: name.to_string(), res_deg: res, frame_res_deg: param_f64(p, "frame_res_deg", res), frame_every_years: param_f64(p, "frame_every_years", 5.0) as i64,
            hold_before_record: p.get("hold_before_record").and_then(|v| v.as_bool()).unwrap_or(true),
            scale: param_str(p, "scale", "110m"), distribution: param_str(p, "distribution", "even"),
            countries: Vec::new(), index: Vec::new(), land_km2: Vec::new(), weights: Vec::new(), wsum: Vec::new(), unassigned: 0, pop: BTreeMap::new(), last_frame_year: -1_000_000,
        };
        m.load()?;
        Ok(m)
    }
    fn load(&mut self) -> Result<(), String> {
        let root = root();
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let mut f = Fetcher::default_for(&root)?;
        self.countries = load_countries(&self.scale)?;
        self.pop = countries::un_wpp_population(&mut f, &asset_url(&cat, "un-wpp-2024-total-population")?)?;
        let g = world_grid(self.res_deg);
        self.index = vec![f32::NAN; g.nx * g.ny];
        self.land_km2 = vec![0.0; self.countries.len()];
        for row in 0..g.ny {
            let lat = g.lat(row);
            let area_km2 = g.row_area(row) / 1e6;
            for col in 0..g.nx {
                let lon = g.lon(col);
                if let Some(k) = self.countries.iter().position(|c| c.contains(lon, lat)) {
                    self.index[row * g.nx + col] = k as f32;
                    self.land_km2[k] += area_km2;
                }
            }
        }
        if self.distribution == "worldpop" {
            self.weights = cells::worldpop_count(&mut f, &asset_url(&cat, "worldpop-2020-1km")?, self.res_deg)?;
            // a populated cell no outline claimed (coasts, small islands at this scale) goes to
            // the nearest claimed cell within three cells; beyond that it is counted and left
            let before = self.index.clone();
            self.unassigned = 0;
            for i in 0..self.index.len() {
                if !(self.weights[i] > 0.0) || !before[i].is_nan() { continue; }
                let (col, row) = g.col_row(i);
                let mut found = None;
                'ring: for r in 1..=3i64 {
                    for dr in -r..=r { for dc in -r..=r {
                        if dr.abs() != r && dc.abs() != r { continue; }
                        let rr = row as i64 + dr; if rr < 0 || rr >= g.ny as i64 { continue; }
                        let cc = (col as i64 + dc).rem_euclid(g.nx as i64);
                        let v = before[rr as usize * g.nx + cc as usize];
                        if !v.is_nan() { found = Some(v); break 'ring; }
                    } }
                }
                match found { Some(k) => { self.index[i] = k; self.land_km2[k as usize] += g.row_area(row) / 1e6; } None => self.unassigned += 1 }
            }
            self.wsum = vec![0.0; self.countries.len()];
            for (i, &k) in self.index.iter().enumerate() { if !k.is_nan() { self.wsum[k as usize] += self.weights[i] as f64; } }
        }
        Ok(())
    }
    fn population(&self, iso3: &str, y: i64) -> Option<f64> {
        let s = self.pop.get(iso3)?;
        if let Some(v) = s.get(&y) { return Some(*v); }
        let (&first, &fv) = s.iter().next()?;
        if y < first && self.hold_before_record { return Some(fv); }
        s.range(..y).next_back().map(|(_, v)| *v)
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let res = param_f64(p, "res_deg", 2.0);
        let worldpop = param_str(p, "distribution", "even") == "worldpop";
        let scale = param_str(p, "scale", "110m");
        let mut d = Describe::new("people.countries", name, "Countries as a world raster (Natural Earth outlines) and people per country per year (UN World Population Prospects 2024: estimates to 2023, the UN medium projection after, which is the UN's assumption and labelled so), placed on the cells of each country either evenly by area or, in the worldpop distribution, in proportion to WorldPop's 1 km count of 2020. A data layer: nothing is modelled, people are read; WorldPop gives the pattern, the UN the count.")
            .priority(0).dt("1 year (1 January)")
            .param("res_deg", res, "deg", "world grid cell; 2 degrees is 16,200 cells, 0.5 is 259,200")
            .param("frame_res_deg", param_f64(p, "frame_res_deg", res), "deg", "map frames are folded to this coarser grid for the page")
            .param("frame_every_years", param_f64(p, "frame_every_years", 5.0), "yr", "map frames are written this often")
            .param("hold_before_record", p.get("hold_before_record").and_then(|v| v.as_bool()).unwrap_or(true), "", "before 1950 the UN has no numbers; hold 1950 flat (a placeholder, not history; HYDE would be the record)")
            .param("scale", scale.clone(), "", "Natural Earth scale for the outlines: 110m or 50m; use 50m below 2 degrees or small island states vanish")
            .param("distribution", param_str(p, "distribution", "even"), "", "even: a country's people spread evenly over its land cells; worldpop: in proportion to WorldPop's 1 km count")
            .own("country.index", "index").own("people.count", "persons").own("people.density_km2", "1/km2")
            .publish("people.population", "{ year, iso3: persons } each January")
            .report("people.world_total", "persons").report("people.countries_matched", "")
            .view("counter", "people on Earth", "people.world_total", "the UN's estimates to 2023 and its medium projection after")
            .view("map", "people per square kilometre", "people_density_km2", if worldpop { "WorldPop's 2020 pattern, each country scaled to the UN count for the year" } else { "a country's people spread evenly over its land" })
            .asset(scale_asset(&scale), "outlines, ISO3 codes").asset("un-wpp-2024-total-population", "people per country per year");
        if worldpop { d = d.report("people.cells_unassigned", "").asset("worldpop-2020-1km", "people per 1 km pixel in 2020, summed to the grid").note("A populated cell the outlines do not claim is given to the nearest claimed cell within three cells; the rest are counted in people.cells_unassigned. The 2020 pattern is held for every year: cities do not move or grow relative to their countryside here."); }
        else { d = d.note("Population is spread evenly over a country's land cells for the density map; distribution = worldpop places it by the 1 km count."); }
        d
    }
}

impl Module for PeopleCountries {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        let g = world_grid(self.res_deg);
        ctx.world_write("country.index", "index", &g, self.index.clone());
        let (cg, ci) = coarsen(&g, &self.index, frame_factor(self.res_deg, self.frame_res_deg), Fold::Max);
        ctx.field("country_index", "index", &cg, ci);
        if self.distribution == "worldpop" { ctx.report("people.cells_unassigned", "", self.unassigned as f64); }
        self.emit(ctx);
        ctx.schedule_with_priority(to_next_year(ctx.now()), 0, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" { return; }
        self.emit(ctx);
        ctx.schedule_with_priority(to_next_year(ctx.now()), 0, "tick", json!(null));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut m: PeopleCountries = serde_json::from_value(v).unwrap(); m.load().expect("people reload"); *self = m; }
}

impl PeopleCountries {
    fn emit(&mut self, ctx: &mut Ctx) {
        let y = year_of(ctx.now());
        let g = world_grid(self.res_deg);
        let mut table = serde_json::Map::new();
        let mut per_country = vec![0.0f64; self.countries.len()];
        let (mut total, mut matched) = (0.0, 0usize);
        for (k, c) in self.countries.iter().enumerate() {
            if let Some(p) = self.population(&c.iso3, y) { per_country[k] = p; total += p; matched += 1; table.insert(c.iso3.clone(), json!(p)); }
        }
        let worldpop = self.distribution == "worldpop";
        let mut count = vec![f32::NAN; g.nx * g.ny];
        let mut density = vec![f32::NAN; g.nx * g.ny];
        for (i, &k) in self.index.iter().enumerate() {
            if k.is_nan() { continue; }
            let k = k as usize;
            let area = g.row_area(g.col_row(i).1) / 1e6;
            let share = if worldpop && self.wsum[k] > 0.0 { self.weights[i] as f64 / self.wsum[k] } else if self.land_km2[k] > 0.0 { area / self.land_km2[k] } else { 0.0 };
            count[i] = (per_country[k] * share) as f32;
            density[i] = if area > 0.0 { (per_country[k] * share / area) as f32 } else { 0.0 };
        }
        ctx.world_write("people.count", "persons", &g, count.clone());
        ctx.world_write("people.density_km2", "1/km2", &g, density);
        if y - self.last_frame_year >= self.frame_every_years {
            let (cg, cc) = coarsen(&g, &count, frame_factor(self.res_deg, self.frame_res_deg), Fold::Sum);
            let dens: Vec<f32> = cc.iter().enumerate().map(|(i, v)| if v.is_nan() { f32::NAN } else { v / (cg.row_area(cg.col_row(i).1) / 1e6) as f32 }).collect();
            ctx.field("people_density_km2", "1/km2", &cg, dens);
            self.last_frame_year = y;
        }
        ctx.publish("people.population", json!({ "year": y, "by_iso3": table }));
        ctx.report("people.world_total", "persons", total);
        ctx.report("people.countries_matched", "", matched as f64);
    }
}

// ---------------------------------------------------------------------------------------
// climate.countries

#[derive(Serialize, Deserialize)]
pub struct ClimateCountries {
    name: String,
    res_deg: f64, frame_res_deg: f64, frame_every_years: i64, fit_from: i64, fit_to: i64, clim_from: i64, clim_to: i64,
    anom_from: i64, anom_to: i64, cells_clim_from: i64, cells_clim_to: i64, scale: String,
    gmst: f64,
    #[serde(skip)] g_anom: f64,                           // global anomaly's observed mean over the anomaly window
    #[serde(skip)] g_cells: f64,                          // and over the cells' climatology window (POWER's)
    #[serde(skip)] countries: Vec<Country>,
    #[serde(skip)] fit: Vec<Option<(f64, f64, usize)>>,   // slope, intercept, n per country
    #[serde(skip)] hot_clim: Vec<Option<f64>>,            // hottest month's climatological mean
    #[serde(skip)] g_clim: f64,                           // global anomaly's mean over the climatology window
    last_frame_year: i64,
}

impl ClimateCountries {
    pub fn new(name: &str, p: &toml::Value) -> Result<ClimateCountries, String> {
        let res = param_f64(p, "res_deg", 2.0);
        let mut m = ClimateCountries {
            name: name.to_string(), res_deg: res, frame_res_deg: param_f64(p, "frame_res_deg", res), frame_every_years: param_f64(p, "frame_every_years", 5.0) as i64,
            fit_from: param_f64(p, "fit_from", 1901.0) as i64, fit_to: param_f64(p, "fit_to", 2022.0) as i64,
            clim_from: param_f64(p, "clim_from", 1991.0) as i64, clim_to: param_f64(p, "clim_to", 2020.0) as i64,
            anom_from: param_f64(p, "anom_from", 2014.0) as i64, anom_to: param_f64(p, "anom_to", 2023.0) as i64,
            cells_clim_from: param_f64(p, "cells_clim_from", 2001.0) as i64, cells_clim_to: param_f64(p, "cells_clim_to", 2025.0) as i64, scale: param_str(p, "scale", "110m"),
            gmst: 0.0, g_anom: 0.0, g_cells: 0.0, countries: Vec::new(), fit: Vec::new(), hot_clim: Vec::new(), g_clim: 0.0, last_frame_year: -1_000_000,
        };
        m.load()?;
        Ok(m)
    }
    fn load(&mut self) -> Result<(), String> {
        let root = root();
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let mut f = Fetcher::default_for(&root)?;
        self.countries = load_countries(&self.scale)?;
        let annual = countries::cckp_country_annual_tas(&mut f, &asset_url(&cat, "worldbank-cckp-cru-tas-annual")?)?;
        let monthly = countries::cckp_country_monthly_tas(&mut f, &asset_url(&cat, "worldbank-cckp-cru-tas-monthly")?)?;
        let had = esim_registry::planet::hadcrut5_annual(&mut f, &asset_url(&cat, "hadcrut5-global-annual")?)?;
        let base = esim_registry::planet::window_mean(&had, 1850, 1900).unwrap_or(0.0);
        let g: BTreeMap<i64, f64> = had.iter().filter(|(y, _)| **y >= self.fit_from && **y <= self.fit_to).map(|(y, v)| (*y, v - base)).collect();
        self.g_clim = esim_registry::planet::window_mean(&g, self.clim_from, self.clim_to).unwrap_or(0.0);
        let g_all: BTreeMap<i64, f64> = had.iter().map(|(y, v)| (*y, v - base)).collect();
        self.g_anom = esim_registry::planet::window_mean(&g_all, self.anom_from, self.anom_to).unwrap_or(0.0);
        // POWER's climatology runs past the global record's end; the window mean uses the years both have
        self.g_cells = esim_registry::planet::window_mean(&g_all, self.cells_clim_from, self.cells_clim_to).unwrap_or(self.g_anom);
        // both series smoothed over eleven years before the fit: year-to-year noise in the
        // regressor would otherwise pull every slope toward zero
        let gs = countries::smooth(&g, 11);
        self.fit = self.countries.iter().map(|c| annual.get(&c.iso3).and_then(|s| countries::regress(&gs, &countries::smooth(s, 11)))).collect();
        self.hot_clim = self.countries.iter().map(|c| {
            let m = monthly.get(&c.iso3)?;
            // climatological mean per calendar month over the window, then the hottest
            let mut sum = [0.0f64; 12]; let mut n = [0usize; 12];
            for ((y, mo), t) in m { if *y >= self.clim_from && *y <= self.clim_to { sum[(*mo - 1) as usize] += t; n[(*mo - 1) as usize] += 1; } }
            (0..12).filter(|i| n[*i] > 0).map(|i| sum[i] / n[i] as f64).fold(None, |acc: Option<f64>, v| Some(acc.map_or(v, |a| a.max(v))))
        }).collect();
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        Describe::new("climate.countries", name, "Each country's mean temperature from its own record (CRU TS 4.07, 1901 to 2022, via the World Bank portal) regressed on the global anomaly (HadCRUT5, re-zeroed to 1850 to 1900): the slope is the country's observed share of global warming, the intercept its level. The run's global anomaly from the energy module times the slope gives the country's annual mean each year; the hottest month moves by the same slope from its 1991 to 2020 mean. Pattern scaling from the record; the stated assumption is that the pattern persists.")
            .priority(3).dt("1 year (1 January), on the latest energy.state")
            .param("res_deg", param_f64(p, "res_deg", 2.0), "deg", "must match people.countries").param("frame_res_deg", param_f64(p, "frame_res_deg", param_f64(p, "res_deg", 2.0)), "deg", "frames folded to this grid").param("frame_every_years", param_f64(p, "frame_every_years", 5.0), "yr", "")
            .param("scale", param_str(p, "scale", "110m"), "", "must match people.countries")
            .param("fit_from", param_f64(p, "fit_from", 1901.0), "yr", "regression window").param("fit_to", param_f64(p, "fit_to", 2022.0), "yr", "")
            .param("clim_from", param_f64(p, "clim_from", 1991.0), "yr", "climatology window for the hottest month").param("clim_to", param_f64(p, "clim_to", 2020.0), "yr", "")
            .param("anom_from", param_f64(p, "anom_from", 2014.0), "yr", "window the per-country anomaly (tas_anom_c) is measured against; the wet-bulb layer's observed years").param("anom_to", param_f64(p, "anom_to", 2023.0), "yr", "")
            .param("cells_clim_from", param_f64(p, "cells_clim_from", 2001.0), "yr", "window of the cells' baseline climatology (NASA POWER's, 2001 to 2025); tas_anom_cells_c is measured against it").param("cells_clim_to", param_f64(p, "cells_clim_to", 2025.0), "yr", "")
            .subscribe("energy.state", "{ gmst_c } the global anomaly")
            .read("country.index", "index", true, "which cell is which country")
            .own("climate.tas_c", "degC").own("climate.hot_month_c", "degC")
            .publish("climate.country_tas", "{ year, by_iso3: { tas_c, hot_month_c, slope, tas_anom_c, tas_anom_cells_c } } each January; tas_anom_c is the country's warming since the anom window, tas_anom_cells_c since the cells' climatology window")
            .report("climate.slope_median", "degC/degC").report("climate.countries_fitted", "").report("climate.hottest_country_tas_c", "degC")
            .view("map", "annual mean temperature by country", "climate_tas_c", "each country's record scaled by the run's warming")
            .view("map", "hottest month by country", "climate_hot_month_c", "")
            .view("chart", "hottest country's annual mean", "climate.hottest_country_tas_c", "")
            .asset("worldbank-cckp-cru-tas-annual", "country annual means 1901 to 2022").asset("worldbank-cckp-cru-tas-monthly", "country monthly means for the hottest-month climatology").asset("hadcrut5-global-annual", "the global anomaly the records are regressed on").asset("natural-earth-110m-countries", "the country list")
            .note("A country's variance about the fit is lost; years are given their fitted value, not their weather. Countries missing from the record (Antarctica, a few territories) get no temperature and are counted in climate.countries_fitted.")
    }
}

impl Module for ClimateCountries {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.subscribe("energy.state");
        ctx.schedule_with_priority(to_next_year(ctx.now()), 3, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "energy.state" => { if let Some(g) = ev.data.get("gmst_c").and_then(|v| v.as_f64()) { self.gmst = g; } }
            "tick" => {
                let y = year_of(ctx.now());
                let g = world_grid(self.res_deg);
                let Some(index) = ctx.world_read_on("country.index", &g).map(|f| f.data.clone()) else { return };
                let mut tas = vec![f32::NAN; g.nx * g.ny];
                let mut hot = vec![f32::NAN; g.nx * g.ny];
                let mut by = serde_json::Map::new();
                let mut per_tas = vec![f32::NAN; self.countries.len()];
                let mut per_hot = vec![f32::NAN; self.countries.len()];
                let mut slopes = Vec::new();
                let mut hottest = f64::NEG_INFINITY;
                for (k, c) in self.countries.iter().enumerate() {
                    let Some((slope, intercept, _)) = self.fit[k] else { continue };
                    let t = intercept + slope * self.gmst;
                    let h = self.hot_clim[k].map(|hc| hc + slope * (self.gmst - self.g_clim));
                    per_tas[k] = t as f32; per_hot[k] = h.map(|v| v as f32).unwrap_or(f32::NAN);
                    slopes.push(slope); hottest = hottest.max(t);
                    by.insert(c.iso3.clone(), json!({ "tas_c": t, "hot_month_c": h, "slope": slope, "tas_anom_c": slope * (self.gmst - self.g_anom), "tas_anom_cells_c": slope * (self.gmst - self.g_cells) }));
                }
                for (i, &k) in index.iter().enumerate() { if !k.is_nan() { tas[i] = per_tas[k as usize]; hot[i] = per_hot[k as usize]; } }
                ctx.world_write("climate.tas_c", "degC", &g, tas.clone());
                ctx.world_write("climate.hot_month_c", "degC", &g, hot.clone());
                if y - self.last_frame_year >= self.frame_every_years {
                    let fac = frame_factor(self.res_deg, self.frame_res_deg);
                    let (cg, t) = coarsen(&g, &tas, fac, Fold::Mean); let (_, h) = coarsen(&g, &hot, fac, Fold::Mean);
                    ctx.field("climate_tas_c", "degC", &cg, t); ctx.field("climate_hot_month_c", "degC", &cg, h); self.last_frame_year = y;
                }
                ctx.publish("climate.country_tas", json!({ "year": y, "gmst_c": self.gmst, "by_iso3": by }));
                slopes.sort_by(|a, b| a.partial_cmp(b).unwrap());
                if !slopes.is_empty() { ctx.report("climate.slope_median", "degC/degC", slopes[slopes.len() / 2]); }
                ctx.report("climate.countries_fitted", "", slopes.len() as f64);
                if hottest.is_finite() { ctx.report("climate.hottest_country_tas_c", "degC", hottest); }
                ctx.schedule_with_priority(to_next_year(ctx.now()), 3, "tick", json!(null));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut m: ClimateCountries = serde_json::from_value(v).unwrap(); m.load().expect("climate reload"); *self = m; }
}

// ---------------------------------------------------------------------------------------
// people.heat

#[derive(Serialize, Deserialize)]
pub struct PeopleHeat {
    name: String,
    res_deg: f64, frame_res_deg: f64, frame_every_years: i64, niche_c: f64, hot_month_c: f64, crossing_share: f64,
    baseline: String, scale: String,
    pop: BTreeMap<String, f64>,
    crossed_niche: BTreeMap<String, i64>,
    crossed_hot: BTreeMap<String, i64>,
    /// cells mode: the first year each cell's mean passed the niche, by cell index
    cell_year: BTreeMap<u32, i32>,
    /// countries over a marker in the first year evaluated: a state of the climatology, not a change
    baseline_done: bool,
    #[serde(skip)] countries: Vec<Country>,
    #[serde(skip)] clim: Vec<f32>,   // cells mode: annual mean per cell over POWER's climatology, NaN elsewhere
    last_frame_year: i64,
}

impl PeopleHeat {
    pub fn new(name: &str, p: &toml::Value) -> Result<PeopleHeat, String> {
        let res = param_f64(p, "res_deg", 2.0);
        let mut m = PeopleHeat {
            name: name.to_string(), res_deg: res, frame_res_deg: param_f64(p, "frame_res_deg", res), frame_every_years: param_f64(p, "frame_every_years", 5.0) as i64,
            niche_c: param_f64(p, "niche_c", 29.0), hot_month_c: param_f64(p, "hot_month_c", 32.0), crossing_share: param_f64(p, "crossing_share", 0.5),
            baseline: param_str(p, "baseline", "country"), scale: param_str(p, "scale", "110m"),
            pop: BTreeMap::new(), crossed_niche: BTreeMap::new(), crossed_hot: BTreeMap::new(), cell_year: BTreeMap::new(), baseline_done: false, countries: Vec::new(), clim: Vec::new(), last_frame_year: -1_000_000,
        };
        m.load()?;
        Ok(m)
    }
    fn load(&mut self) -> Result<(), String> {
        self.countries = load_countries(&self.scale)?;
        if self.baseline != "cells" { return Ok(()); }
        let root = root();
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let mut f = Fetcher::default_for(&root)?;
        let g = world_grid(self.res_deg);
        let people = cells::worldpop_count(&mut f, &asset_url(&cat, "worldpop-2020-1km")?, self.res_deg)?;
        self.clim = vec![f32::NAN; g.nx * g.ny];
        let boxes = cells::power_boxes(self.res_deg, |c, r| people[r * g.nx + c] > 0.0);
        for &(lat0, lon0) in &boxes {
            let b = cells::power_climatology_box(&mut f, "T2M", lat0, lon0)?;
            let ann = cells::clim_index("ANN").ok_or("no ANN column")?;
            if b.days != cells::CLIM_KEYS.len() { return Err(format!("climatology box {lat0},{lon0}: {} columns, expected {}", b.days, cells::CLIM_KEYS.len())); }
            let members: Vec<usize> = (0..g.nx * g.ny).filter(|&i| people[i] > 0.0 && { let (c, r) = g.col_row(i); box_of(g.lat(r), g.lon(c)) == (lat0, lon0) }).collect();
            for (i, pt) in members.iter().zip(points_for(&b, &g, &members)) {
                if let Some(pt) = pt { if let Some(v) = b.value(pt, ann) { self.clim[*i] = v as f32; } }
            }
        }
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let res = param_f64(p, "res_deg", 2.0);
        let on_cells = param_str(p, "baseline", "country") == "cells";
        let scale = param_str(p, "scale", "110m");
        let mut d = Describe::new("people.heat", name, if on_cells {
            "People living where the annual mean temperature has left the human climate niche, cell by cell: each populated half-degree cell's climatological mean (MERRA-2 through NASA POWER) plus its country's warming since that climatology, against the niche's hot edge; the people in such cells are summed, and a country crosses when a stated share of its people live in them. Counts, not deaths."
        } else {
            "People living in countries whose annual mean temperature has left the human climate niche, and in countries whose hottest month passes a second marker. Counts, not deaths: exposure from a population layer and a temperature layer. A country crosses whole, because the temperature is a country mean; the first year it crosses, a people.crossing event is published for the auditor."
        })
            .priority(4).dt("follows climate.country_tas")
            .param("res_deg", res, "deg", "must match people.countries").param("frame_res_deg", param_f64(p, "frame_res_deg", res), "deg", "frames folded to this grid").param("frame_every_years", param_f64(p, "frame_every_years", 5.0), "yr", "")
            .param("baseline", param_str(p, "baseline", "country"), "", "country: one annual mean per country from its record; cells: each populated cell's own climatological mean from the reanalysis, shifted by its country's warming")
            .param("scale", scale.clone(), "", "must match people.countries")
            .param("niche_c", param_f64(p, "niche_c", 29.0), "degC", "mean annual temperature at the hot edge of the human climate niche, Xu et al. 2020 PNAS (recalled; to be checked)")
            .subscribe("people.population", "{ by_iso3 }").subscribe("climate.country_tas", "{ by_iso3: { tas_c, hot_month_c, tas_anom_cells_c } }")
            .read("country.index", "index", true, "which cell is which country")
            .own("people.outside_niche", if on_cells { "persons" } else { "1" })
            .frame("people_outside_niche", if on_cells { "persons" } else { "1" }).frame("people_niche_crossing_year", "year")
            .view("counter", "people outside the human climate niche", "people.outside_niche", "annual mean above 29 °C where they live; a count, not a toll")
            .view("counter", "countries outside the niche", "people.countries_outside_niche", if on_cells { "more than half their people" } else { "" })
            .view("map", "outside the niche", "people_outside_niche", if on_cells { "people per cell whose annual mean is past 29 °C" } else { "" })
            .view("map", if on_cells { "the year each place left the niche" } else { "the year each country left the niche" }, "people_niche_crossing_year", "fills in as the cursor passes each crossing")
            .view("crossings", "countries crossing", "people.crossing", "the first year a country passes a marker; the source of each marker is in the parameters")
            .view("chart", "people outside the niche", "people.outside_niche", "")
            .report("people.outside_niche", "persons").report("people.outside_niche_pct", "%").report("people.countries_outside_niche", "").report("people.crossings_niche", "")
            .asset(scale_asset(&scale), "names for the crossing events");
        if on_cells {
            d = d.param("crossing_share", param_f64(p, "crossing_share", 0.5), "", "a country crosses when this share of its people live in cells past the niche")
                .read("people.count", "persons", true, "people per cell")
                .publish("people.crossing", "{ iso3, name, year, kind: niche, persons, share } the first year the stated share of a country's people live past the niche; a country already there in the run's first year is not announced")
                .asset("worldpop-2020-1km", "which cells have people").asset("nasa-power-climatology-regional", "each cell's annual mean, 2001 to 2025")
                .note("The warming added to a cell is its country's, from the country record; within a country the pattern of the climatology is held. No mortality here on purpose.");
        } else {
            d = d.param("hot_month_c", param_f64(p, "hot_month_c", 32.0), "degC", "a hottest-month mean marker with no paper behind it: a guess, kept to show the shape until a wet-bulb layer exists")
                .publish("people.crossing", "{ iso3, name, year, kind: niche | hot_month, persons } the first year a country crosses; a country already over a marker in the run's first year is a state of its climate, not a crossing, and is not announced")
                .report("people.hot_month_over", "persons").report("people.crossings_hot_month", "")
                .note("No mortality here on purpose; a mortality layer with published exposure-response curves and their ranges is a separate, later module.");
        }
        d
    }
}

impl Module for PeopleHeat {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) { ctx.subscribe("people.population"); ctx.subscribe("climate.country_tas"); }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "people.population" => {
                if let Some(t) = ev.data.get("by_iso3").and_then(|v| v.as_object()) { self.pop = t.iter().filter_map(|(k, v)| v.as_f64().map(|p| (k.clone(), p))).collect(); }
            }
            "climate.country_tas" if self.baseline == "cells" => self.handle_cells(ev, ctx),
            "climate.country_tas" => {
                let y = ev.data.get("year").and_then(|v| v.as_i64()).unwrap_or(year_of(ctx.now()));
                let Some(by) = ev.data.get("by_iso3").and_then(|v| v.as_object()) else { return };
                let g = world_grid(self.res_deg);
                let index = ctx.world_read_on("country.index", &g).map(|f| f.data.clone());
                let mut per = vec![f32::NAN; self.countries.len()];
                let (mut outside, mut hot_over, mut total, mut n_out) = (0.0f64, 0.0f64, 0.0f64, 0usize);
                let mut crossings = Vec::new();
                for (k, c) in self.countries.iter().enumerate() {
                    let Some(p) = self.pop.get(&c.iso3) else { continue };
                    total += p;
                    let Some(e) = by.get(&c.iso3) else { continue };
                    let tas = e.get("tas_c").and_then(|v| v.as_f64());
                    let hot = e.get("hot_month_c").and_then(|v| v.as_f64());
                    let out = tas.map(|t| t > self.niche_c).unwrap_or(false);
                    let over = hot.map(|h| h > self.hot_month_c).unwrap_or(false);
                    per[k] = if out { 1.0 } else if tas.is_some() { 0.0 } else { f32::NAN };
                    // the first evaluated year sets the baseline: a country already over a marker then is
                    // recorded with year 0 and never announced; a crossing is a change from that state
                    if out { outside += p; n_out += 1; if !self.crossed_niche.contains_key(&c.iso3) { self.crossed_niche.insert(c.iso3.clone(), if self.baseline_done { y } else { 0 }); if self.baseline_done {
                        let sentence = format!("{}: the annual mean temperature of {} passes {} °C, the hot edge of the climate humans have always lived in. {} people live there.", y, c.name, self.niche_c, people_words(*p));
                        crossings.push(json!({ "iso3": c.iso3, "name": c.name, "year": y, "kind": "niche", "persons": p, "tas_c": tas, "sentence": sentence, "source": "Xu et al. 2020 (recalled)" })); } } }
                    if over { hot_over += p; if !self.crossed_hot.contains_key(&c.iso3) { self.crossed_hot.insert(c.iso3.clone(), if self.baseline_done { y } else { 0 }); if self.baseline_done {
                        let sentence = format!("{}: the hottest month in {} now averages above {} °C. {} people. (A marker with no paper behind it.)", y, c.name, self.hot_month_c, people_words(*p));
                        crossings.push(json!({ "iso3": c.iso3, "name": c.name, "year": y, "kind": "hot_month", "persons": p, "hot_month_c": hot, "sentence": sentence, "source": "guess" })); } } }
                }
                if let Some(index) = index {
                    let mut field = vec![f32::NAN; g.nx * g.ny];
                    let mut year_field = vec![f32::NAN; g.nx * g.ny];
                    let year_of_country: Vec<f32> = self.countries.iter().map(|c| self.crossed_niche.get(&c.iso3).map(|yy| if *yy > 0 { *yy as f32 } else { f32::NAN }).unwrap_or(f32::NAN)).collect();
                    for (i, &k) in index.iter().enumerate() { if !k.is_nan() { field[i] = per[k as usize]; year_field[i] = year_of_country[k as usize]; } }
                    ctx.world_write("people.outside_niche", "1", &g, field.clone());
                    if y - self.last_frame_year >= self.frame_every_years {
                        let fac = frame_factor(self.res_deg, self.frame_res_deg);
                        let (cg, a) = coarsen(&g, &field, fac, Fold::Max); let (_, b) = coarsen(&g, &year_field, fac, Fold::Min);
                        ctx.field("people_outside_niche", "1", &cg, a); ctx.field("people_niche_crossing_year", "year", &cg, b); self.last_frame_year = y;
                    }
                }
                self.baseline_done = true;
                for c in crossings { ctx.publish("people.crossing", c); }
                ctx.report("people.outside_niche", "persons", outside);
                if total > 0.0 { ctx.report("people.outside_niche_pct", "%", 100.0 * outside / total); }
                ctx.report("people.hot_month_over", "persons", hot_over);
                ctx.report("people.countries_outside_niche", "", n_out as f64);
                ctx.report("people.crossings_niche", "", self.crossed_niche.values().filter(|y| **y > 0).count() as f64);
                ctx.report("people.crossings_hot_month", "", self.crossed_hot.values().filter(|y| **y > 0).count() as f64);
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut m: PeopleHeat = serde_json::from_value(v).unwrap(); m.load().expect("heat reload"); *self = m; }
}

impl PeopleHeat {
    /// Cells mode: each populated cell's climatological mean plus its country's warming
    /// since the climatology, against the niche; people summed per cell, then by country.
    fn handle_cells(&mut self, ev: &Event, ctx: &mut Ctx) {
        let y = ev.data.get("year").and_then(|v| v.as_i64()).unwrap_or(year_of(ctx.now()));
        let Some(by) = ev.data.get("by_iso3").and_then(|v| v.as_object()) else { return };
        let g = world_grid(self.res_deg);
        let Some(index) = ctx.world_read_on("country.index", &g).map(|f| f.data.clone()) else { return };
        let Some(count) = ctx.world_read_on("people.count", &g).map(|f| f.data.clone()) else { return };
        let warming: Vec<f64> = self.countries.iter().map(|c| by.get(&c.iso3).and_then(|e| e.get("tas_anom_cells_c")).and_then(|v| v.as_f64()).unwrap_or(f64::NAN)).collect();
        let n = self.countries.len();
        let (mut out_c, mut tot_c) = (vec![0.0f64; n], vec![0.0f64; n]);
        let mut field = vec![f32::NAN; g.nx * g.ny];
        let mut year_field = vec![f32::NAN; g.nx * g.ny];
        let mut outside = 0.0f64;
        for i in 0..field.len() {
            let k = index[i]; if k.is_nan() { continue }
            let k = k as usize;
            let p = count[i]; if !(p > 0.0) { field[i] = 0.0; continue }
            tot_c[k] += p as f64;
            let clim = self.clim[i]; if clim.is_nan() || warming[k].is_nan() { continue }
            let t = clim as f64 + warming[k];
            if t > self.niche_c {
                field[i] = p; out_c[k] += p as f64; outside += p as f64;
                let e = self.cell_year.entry(i as u32).or_insert(if self.baseline_done { y as i32 } else { 0 });
                if *e > 0 { year_field[i] = *e as f32; }
            } else { field[i] = 0.0; if let Some(e) = self.cell_year.get(&(i as u32)) { if *e > 0 { year_field[i] = *e as f32; } } }
        }
        let mut crossings = Vec::new();
        let mut n_out = 0usize;
        let total: f64 = tot_c.iter().sum();
        for (k, c) in self.countries.iter().enumerate() {
            if tot_c[k] <= 0.0 { continue }
            let share = out_c[k] / tot_c[k];
            if share >= self.crossing_share {
                n_out += 1;
                if !self.crossed_niche.contains_key(&c.iso3) {
                    self.crossed_niche.insert(c.iso3.clone(), if self.baseline_done { y } else { 0 });
                    if self.baseline_done {
                        let sentence = format!("{}: in {}, {:.0}% of the people ({}) now live where the annual mean temperature is past {} °C, the hot edge of the climate humans have always lived in.", y, c.name, 100.0 * share, people_words(out_c[k]), self.niche_c);
                        crossings.push(json!({ "iso3": c.iso3, "name": c.name, "year": y, "kind": "niche", "persons": out_c[k], "share": share, "sentence": sentence, "source": "Xu et al. 2020 (recalled); MERRA-2 via NASA POWER" }));
                    }
                }
            }
        }
        ctx.world_write("people.outside_niche", "persons", &g, field.clone());
        if y - self.last_frame_year >= self.frame_every_years {
            let fac = frame_factor(self.res_deg, self.frame_res_deg);
            let (cg, a) = coarsen(&g, &field, fac, Fold::Sum); let (_, b) = coarsen(&g, &year_field, fac, Fold::Min);
            ctx.field("people_outside_niche", "persons", &cg, a); ctx.field("people_niche_crossing_year", "year", &cg, b); self.last_frame_year = y;
        }
        self.baseline_done = true;
        for c in crossings { ctx.publish("people.crossing", c); }
        ctx.report("people.outside_niche", "persons", outside);
        if total > 0.0 { ctx.report("people.outside_niche_pct", "%", 100.0 * outside / total); }
        ctx.report("people.countries_outside_niche", "", n_out as f64);
        ctx.report("people.crossings_niche", "", self.crossed_niche.values().filter(|y| **y > 0).count() as f64);
    }
}

// ---------------------------------------------------------------------------------------
// tipping.elements

#[derive(Clone, Serialize, Deserialize)]
struct Element { id: String, title: String, low_c: f64, central_c: f64, high_c: f64, timescale: String, consequence: String, source: String, confidence: String }

#[derive(Serialize, Deserialize)]
pub struct Tipping {
    name: String,
    elements: Vec<Element>,
    passed_low: BTreeMap<String, i64>,
    passed_central: BTreeMap<String, i64>,
    gmst: f64,
}

impl Tipping {
    pub fn new(name: &str, _p: &toml::Value) -> Result<Tipping, String> {
        let path = root().join("data/events/tipping.toml");
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let v: toml::Value = toml::from_str(&text).map_err(|e| format!("tipping.toml: {e}"))?;
        let elements: Vec<Element> = v.get("element").and_then(|a| a.as_array()).into_iter().flatten()
            .map(|e| e.clone().try_into::<Element>().map_err(|err| format!("tipping.toml element: {err}"))).collect::<Result<_, _>>()?;
        if elements.is_empty() { return Err("tipping.toml has no elements".into()); }
        Ok(Tipping { name: name.to_string(), elements, passed_low: BTreeMap::new(), passed_central: BTreeMap::new(), gmst: 0.0 })
    }
    pub fn describe(name: &str, _p: &toml::Value) -> Describe {
        Describe::new("tipping.elements", name, "The climate tipping elements from data/events/tipping.toml, each with the global warming at which an assessment judged it likely to tip (a central estimate and a range) and its timescale, checked every year against the run's global anomaly. A tipping.crossing event the first year the run passes an element's low end and again its central estimate. The table is the claim; the module only compares.")
            .priority(5).dt("follows energy.state, once a year")
            .subscribe("energy.state", "{ gmst_c }")
            .publish("tipping.crossing", "{ id, title, year, level: low | central, threshold_c, gmst_c, timescale, consequence, source, confidence }")
            .report("tipping.elements_past_low", "").report("tipping.elements_past_central", "")
            .view("counter", "tipping elements past their central estimate", "tipping.elements_past_central", "a flag with a citation, not a tip in this run")
            .view("crossings", "tipping thresholds passed", "tipping.crossing", "every row of the table is marked recalled until checked against the paper")
            .asset("events/tipping.toml", "the elements and their thresholds, with provenance and confidence per row")
            .note("Passing a threshold does not tip the element in this run; nothing feeds back yet. It is a flag with a citation, which is what the words-only essays lack.")
    }
}

impl Module for Tipping {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) { ctx.subscribe("energy.state"); ctx.schedule_with_priority(to_next_year(ctx.now()), 5, "tick", json!(null)); }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "energy.state" => { if let Some(g) = ev.data.get("gmst_c").and_then(|v| v.as_f64()) { self.gmst = g; } }
            "tick" => {
                let y = year_of(ctx.now());
                for e in &self.elements {
                    for (level, thr, book) in [("low", e.low_c, &mut self.passed_low), ("central", e.central_c, &mut self.passed_central)] {
                        if self.gmst >= thr && !book.contains_key(&e.id) {
                            book.insert(e.id.clone(), y);
                            let sentence = if level == "central" {
                                format!("{}: the warming passes {} °C, the central estimate at which {} becomes likely: {}, over {}.", y, thr, e.title.to_lowercase(), e.consequence, e.timescale)
                            } else {
                                format!("{}: the warming enters the range ({} °C and up) in which {} could be triggered.", y, thr, e.title.to_lowercase())
                            };
                            ctx.publish("tipping.crossing", json!({ "id": e.id, "title": e.title, "year": y, "level": level, "threshold_c": thr, "gmst_c": self.gmst,
                                "timescale": e.timescale, "consequence": e.consequence, "source": e.source, "confidence": e.confidence, "sentence": sentence }));
                        }
                    }
                }
                ctx.report("tipping.elements_past_low", "", self.passed_low.len() as f64);
                ctx.report("tipping.elements_past_central", "", self.passed_central.len() as f64);
                ctx.schedule_with_priority(to_next_year(ctx.now()), 5, "tick", json!(null));
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { *self = serde_json::from_value(v).unwrap(); }
}

// ---------------------------------------------------------------------------------------
// people.wetbulb

#[derive(Serialize, Deserialize)]
pub struct PeopleWetBulb {
    name: String,
    res_deg: f64, frame_res_deg: f64, frame_every_years: i64, year_from: i64, year_to: i64,
    limit_a_c: f64, limit_b_c: f64, days_to_cross: f64, delta_max_c: f64, delta_step_c: f64, station_max_km: f64, crossing_share: f64,
    source: String, scale: String,
    pop: BTreeMap<String, f64>,
    crossed_a: BTreeMap<String, i64>, crossed_b: BTreeMap<String, i64>,
    /// cells mode: the first year each cell passed limit a on more than days_to_cross days
    cell_year: BTreeMap<u32, i32>,
    baseline_done: bool,
    #[serde(skip)] countries: Vec<Country>,
    /// station mode, per country: the capital's name and the station that stands for it
    #[serde(skip)] capital: Vec<Option<(String, String, f64)>>,
    /// station mode, per country: days per year over each limit, tabulated against the warming delta
    #[serde(skip)] table_a: Vec<Vec<f32>>,
    #[serde(skip)] table_b: Vec<Vec<f32>>,
    #[serde(skip)] max_wb_base: Vec<f32>,
    /// cells mode: the populated cells with a reanalysis point, and their tables
    #[serde(skip)] cell_ids: Vec<u32>,
    #[serde(skip)] cell_a: Vec<f32>,   // cell_ids.len() * steps, row-major
    #[serde(skip)] cell_b: Vec<f32>,
    last_frame_year: i64,
}

impl PeopleWetBulb {
    pub fn new(name: &str, p: &toml::Value) -> Result<PeopleWetBulb, String> {
        let res = param_f64(p, "res_deg", 2.0);
        let source = param_str(p, "source", "station");
        let mut m = PeopleWetBulb {
            name: name.to_string(), res_deg: res, frame_res_deg: param_f64(p, "frame_res_deg", res), frame_every_years: param_f64(p, "frame_every_years", 5.0) as i64,
            year_from: param_f64(p, "year_from", if source == "cells" { 2011.0 } else { 2014.0 }) as i64, year_to: param_f64(p, "year_to", if source == "cells" { 2020.0 } else { 2023.0 }) as i64,
            limit_a_c: param_f64(p, "limit_a_c", 31.0), limit_b_c: param_f64(p, "limit_b_c", 35.0),
            days_to_cross: param_f64(p, "days_to_cross", 3.0), delta_max_c: param_f64(p, "delta_max_c", 8.0), delta_step_c: param_f64(p, "delta_step_c", 0.1),
            station_max_km: param_f64(p, "station_max_km", 80.0), crossing_share: param_f64(p, "crossing_share", 0.5),
            source, scale: param_str(p, "scale", "110m"),
            pop: BTreeMap::new(), crossed_a: BTreeMap::new(), crossed_b: BTreeMap::new(), cell_year: BTreeMap::new(), baseline_done: false,
            countries: Vec::new(), capital: Vec::new(), table_a: Vec::new(), table_b: Vec::new(), max_wb_base: Vec::new(), cell_ids: Vec::new(), cell_a: Vec::new(), cell_b: Vec::new(), last_frame_year: -1_000_000,
        };
        m.load()?;
        Ok(m)
    }
    fn steps(&self) -> usize { (self.delta_max_c / self.delta_step_c).round() as usize + 1 }
    fn load(&mut self) -> Result<(), String> {
        self.countries = load_countries(&self.scale)?;
        if self.source == "cells" { return self.load_cells(); }
        let root = root();
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let mut f = Fetcher::default_for(&root)?;
        let caps = countries::natural_earth_capitals(&mut f, &asset_url(&cat, "natural-earth-110m-populated-places")?)?;
        let stations = countries::isd_history(&mut f, &asset_url(&cat, "noaa-isd-history")?)?;
        let steps = self.steps();
        self.capital = Vec::new(); self.table_a = Vec::new(); self.table_b = Vec::new(); self.max_wb_base = Vec::new();
        let skip = |me: &mut Self| { me.capital.push(None); me.table_a.push(Vec::new()); me.table_b.push(Vec::new()); me.max_wb_base.push(f32::NAN); };
        for ci in 0..self.countries.len() {
            let c = self.countries[ci].clone();
            let Some((cname, lat, lon)) = caps.get(&c.iso3).cloned() else { skip(self); continue };
            let Some((station, sname, dist_km)) = countries::nearest_station(&stations, lat, lon, self.year_from, self.year_to, self.station_max_km) else {
                eprintln!("wetbulb: {} ({cname}): no station reporting {}..{} within {} km; left out", c.iso3, self.year_from, self.year_to, self.station_max_km); skip(self); continue };
            // the daily record: max temperature and mean dew point, giving an afternoon wet-bulb
            let mut days: Vec<(f64, f64)> = Vec::new(); // (tmax, rh at tmax)
            let mut got_years = 0;
            for y in self.year_from..=self.year_to {
                match countries::gsod_year(&mut f, &station, y) {
                    Ok(rows) => { if !rows.is_empty() { got_years += 1; } for (_, tmax, dew) in rows { days.push((tmax, countries::rh_from_dewpoint(tmax, dew))); } }
                    Err(e) => eprintln!("wetbulb: {} {station} {y}: {e}", c.iso3),
                }
            }
            if days.len() < 200 { eprintln!("wetbulb: {} ({cname}, {sname}): only {} days in {got_years} years; left out", c.iso3, days.len()); skip(self); continue }
            // Stull at constant humidity: the humidity-only terms are fixed per day
            let k: Vec<(f64, f64, f64, f64)> = days.iter().map(|&(t, r)| stull_terms(t, r)).collect();
            let per_year = |n: usize| if days.is_empty() { 0.0 } else { n as f64 / (days.len() as f64 / 365.25) };
            let mut ta = vec![0.0f32; steps]; let mut tb = vec![0.0f32; steps];
            let mut max_base = f64::NEG_INFINITY;
            for s_i in 0..steps {
                let d = s_i as f64 * self.delta_step_c;
                let (mut na, mut nb) = (0usize, 0usize);
                for &(t, r, c1, c2) in &k {
                    let wb = stull_wb(t + d, r, c1, c2);
                    if s_i == 0 && wb > max_base { max_base = wb; }
                    if wb >= self.limit_a_c { na += 1; }
                    if wb >= self.limit_b_c { nb += 1; }
                }
                ta[s_i] = per_year(na) as f32; tb[s_i] = per_year(nb) as f32;
            }
            self.capital.push(Some((cname, format!("{sname} ({dist_km:.0} km)"), dist_km))); self.table_a.push(ta); self.table_b.push(tb); self.max_wb_base.push(max_base as f32);
        }
        Ok(())
    }
    /// Cells mode: for every populated cell with a reanalysis point, the days per year over
    /// each limit against the warming, from a baseline decade of daily maximum temperature
    /// and dew point. The tables are kept in the cache beside the answers they came from.
    fn load_cells(&mut self) -> Result<(), String> {
        let root = root();
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let mut f = Fetcher::default_for(&root)?;
        let g = world_grid(self.res_deg);
        let steps = self.steps();
        let derived = f.cache_dir.join(format!("wetbulb_cells_{}_{}_{}_{}_{}_{}_{}.bin", self.res_deg, self.year_from, self.year_to, self.limit_a_c, self.limit_b_c, self.delta_max_c, self.delta_step_c));
        if let Ok(bytes) = std::fs::read(&derived) {
            if bytes.len() >= 8 {
                let n = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
                let st = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
                if st == steps && bytes.len() == 8 + n * (4 + 8 * steps) {
                    let mut p = 8;
                    let rd = |p: &mut usize| { let v = f32::from_le_bytes([bytes[*p], bytes[*p + 1], bytes[*p + 2], bytes[*p + 3]]); *p += 4; v };
                    self.cell_ids = Vec::with_capacity(n); self.cell_a = Vec::with_capacity(n * steps); self.cell_b = Vec::with_capacity(n * steps);
                    for _ in 0..n { self.cell_ids.push(u32::from_le_bytes([bytes[p], bytes[p + 1], bytes[p + 2], bytes[p + 3]])); p += 4; for _ in 0..steps { self.cell_a.push(rd(&mut p)); } for _ in 0..steps { self.cell_b.push(rd(&mut p)); } }
                    return Ok(());
                }
            }
        }
        let people = cells::worldpop_count(&mut f, &asset_url(&cat, "worldpop-2020-1km")?, self.res_deg)?;
        let boxes = cells::power_boxes(self.res_deg, |c, r| people[r * g.nx + c] > 0.0);
        let t0 = std::time::Instant::now();
        // per cell: a histogram of the warming each day needs to reach each limit
        let mut ids: Vec<u32> = Vec::new();
        let mut hist_a: Vec<u32> = Vec::new(); let mut hist_b: Vec<u32> = Vec::new(); let mut ndays: Vec<u32> = Vec::new();
        for (bi, &(lat0, lon0)) in boxes.iter().enumerate() {
            let members: Vec<usize> = (0..g.nx * g.ny).filter(|&i| people[i] > 0.0 && { let (c, r) = g.col_row(i); box_of(g.lat(r), g.lon(c)) == (lat0, lon0) }).collect();
            if members.is_empty() { continue; }
            let base = ids.len();
            for &i in &members { ids.push(i as u32); }
            hist_a.resize(ids.len() * steps, 0); hist_b.resize(ids.len() * steps, 0); ndays.resize(ids.len(), 0);
            for y in self.year_from..=self.year_to {
                let bt = cells::power_daily_box(&mut f, "T2M_MAX", lat0, lon0, y)?;
                let bd = cells::power_daily_box(&mut f, "T2MDEW", lat0, lon0, y)?;
                let pt = points_for(&bt, &g, &members);
                let pd = points_for(&bd, &g, &members);
                let days = bt.days.min(bd.days);
                for (m, (&pa, &pb)) in pt.iter().zip(pd.iter()).enumerate() {
                    let (Some(pa), Some(pb)) = (pa, pb) else { continue };
                    let row = base + m;
                    for d in 0..days {
                        let (Some(t), Some(td)) = (bt.value(pa, d), bd.value(pb, d)) else { continue };
                        ndays[row] += 1;
                        let (t, r, c1, c2) = stull_terms(t, countries::rh_from_dewpoint(t, td));
                        for (limit, hist) in [(self.limit_a_c, &mut hist_a), (self.limit_b_c, &mut hist_b)] {
                            let need = warming_to_reach(limit, t, r, c1, c2);
                            if need <= self.delta_max_c { let bin = (need / self.delta_step_c).ceil().max(0.0) as usize; hist[row * steps + bin.min(steps - 1)] += 1; }
                        }
                    }
                }
            }
            if bi % 20 == 19 || bi + 1 == boxes.len() { eprintln!("wetbulb cells: {}/{} boxes, {} cells, {:.0} s", bi + 1, boxes.len(), ids.len(), t0.elapsed().as_secs_f64()); }
        }
        // cumulative over the warming, as days per year
        let n = ids.len();
        self.cell_ids = ids; self.cell_a = vec![0.0; n * steps]; self.cell_b = vec![0.0; n * steps];
        for row in 0..n {
            let years = (ndays[row] as f64 / 365.25).max(1e-9);
            let (mut ca, mut cb) = (0u32, 0u32);
            for s_i in 0..steps { ca += hist_a[row * steps + s_i]; cb += hist_b[row * steps + s_i]; self.cell_a[row * steps + s_i] = (ca as f64 / years) as f32; self.cell_b[row * steps + s_i] = (cb as f64 / years) as f32; }
        }
        let mut out = Vec::with_capacity(8 + n * (4 + 8 * steps));
        out.extend_from_slice(&(n as u32).to_le_bytes()); out.extend_from_slice(&(steps as u32).to_le_bytes());
        for row in 0..n { out.extend_from_slice(&self.cell_ids[row].to_le_bytes()); for s_i in 0..steps { out.extend_from_slice(&self.cell_a[row * steps + s_i].to_le_bytes()); } for s_i in 0..steps { out.extend_from_slice(&self.cell_b[row * steps + s_i].to_le_bytes()); } }
        std::fs::write(&derived, out).map_err(|e| e.to_string())?;
        Ok(())
    }
    fn days(&self, table: &[f32], delta: f64) -> f64 {
        if table.is_empty() { return f64::NAN; }
        let x = (delta.max(0.0) / self.delta_step_c).min((table.len() - 1) as f64);
        let (i, fr) = (x.floor() as usize, x - x.floor());
        if i + 1 >= table.len() { return table[table.len() - 1] as f64; }
        table[i] as f64 * (1.0 - fr) + table[i + 1] as f64 * fr
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let res = param_f64(p, "res_deg", 2.0);
        let on_cells = param_str(p, "source", "station") == "cells";
        let scale = param_str(p, "scale", "110m");
        let (yf, yt) = if on_cells { (2011.0, 2020.0) } else { (2014.0, 2023.0) };
        let mut d = Describe::new("people.wetbulb", name, if on_cells {
            "Days per year of dangerous afternoon wet-bulb temperature in every populated half-degree cell, and the people living in cells past the limits. A baseline decade of daily maximum temperature and dew point per cell from the MERRA-2 reanalysis (NASA POWER) gives a daily afternoon wet-bulb (Stull 2011); each day is shifted by the cell's country's warming at constant relative humidity and the days over two published limits counted. Exposure cell by cell, summed by country; a country crosses when a stated share of its people live past a limit on more than days_to_cross days a year. Not a death toll."
        } else {
            "Days per year of dangerous afternoon wet-bulb temperature at each country's capital, and the people in those countries. Ten observed years of daily maximum temperature and mean dew point at the weather station nearest the capital (NOAA GSOD) give a daily afternoon wet-bulb (Stull 2011); each is shifted by the country's warming at constant relative humidity and the days over two published limits counted. Exposure at one city per country, not a death toll; a crossing event the first year a capital passes a limit on more than days_to_cross days."
        })
            .priority(4).dt("follows climate.country_tas")
            .param("res_deg", res, "deg", "must match people.countries").param("frame_res_deg", param_f64(p, "frame_res_deg", res), "deg", "frames folded to this grid").param("frame_every_years", param_f64(p, "frame_every_years", 5.0), "yr", "")
            .param("source", param_str(p, "source", "station"), "", "station: one GSOD station per capital; cells: the reanalysis at every populated cell")
            .param("scale", scale.clone(), "", "must match people.countries")
            .param("year_from", param_f64(p, "year_from", yf), "yr", "observed daily window").param("year_to", param_f64(p, "year_to", yt), "yr", if on_cells { "ten years of daily boxes, about 4,600 answers; esim warm cells fetches them ahead" } else { "ten station-years, about 90 KB each" })
            .param("limit_a_c", param_f64(p, "limit_a_c", 31.0), "degC wet-bulb", "the limit for young healthy adults at rest found in chamber studies, Vecellio et al. 2022 J Appl Physiol (recalled; to be checked)")
            .param("limit_b_c", param_f64(p, "limit_b_c", 35.0), "degC wet-bulb", "the theoretical survivability limit, Sherwood and Huber 2010 PNAS (recalled)")
            .param("days_to_cross", param_f64(p, "days_to_cross", 3.0), "d/yr", "a place passes a limit when its afternoon wet-bulb exceeds it on more than this many days in a year")
            .param("delta_max_c", param_f64(p, "delta_max_c", 8.0), "degC", "warming range tabulated at load").param("delta_step_c", param_f64(p, "delta_step_c", 0.1), "degC", "")
            .subscribe("people.population", "{ by_iso3 }").subscribe("climate.country_tas", "{ by_iso3: { tas_anom_c } } the country's warming since the observed window")
            .read("country.index", "index", true, "which cell is which country")
            .own("wetbulb.days_over_a", "d/yr")
            .frame("wetbulb_days_over_31", "d/yr").frame("wetbulb_crossing_year", "year")
            .report("wetbulb.people_over_a", "persons").report("wetbulb.people_over_b", "persons").report("wetbulb.people_days_over_a", "person-days/yr")
            .asset(scale_asset(&scale), "names for the crossing events");
        if on_cells {
            d = d.param("crossing_share", param_f64(p, "crossing_share", 0.5), "", "a country crosses when this share of its people live in cells past limit a")
                .read("people.count", "persons", true, "people per cell")
                .view("counter", "people living where 31 °C wet-bulb is passed on three days a year", "wetbulb.people_over_a", "the limit for healthy adults at rest; every populated cell")
                .view("counter", "people living where 35 °C wet-bulb is passed", "wetbulb.people_over_b", "the theoretical survivability limit")
                .view("counter", "person-days a year over 31 °C wet-bulb", "wetbulb.people_days_over_a", "people times days, which cannot step")
                .view("map", "days a year over 31 °C wet-bulb", "wetbulb_days_over_31", "the hottest cell where frames are folded")
                .view("map", "the year each place crossed 31 °C wet-bulb", "wetbulb_crossing_year", "")
                .view("crossings", "countries crossing", "people.crossing", "")
                .view("chart", "people living where 31 °C wet-bulb is passed", "wetbulb.people_over_a", "")
                .publish("people.crossing", "{ iso3, name, year, kind: wetbulb_a | wetbulb_b, persons, share, days } the first year the stated share of a country's people live where a limit is passed on more than days_to_cross days")
                .report("wetbulb.cells_over_a", "").report("wetbulb.cells_with_data", "").report("wetbulb.countries_over_a", "").report("wetbulb.countries_over_b", "")
                .asset("worldpop-2020-1km", "which cells have people").asset("nasa-power-daily-regional", "daily maximum temperature and dew point per cell for the baseline decade")
                .note("MERRA-2 is a reanalysis at about 50 km: a model's reconstruction constrained by observations, not a measurement. The day's dew point stands for the afternoon's, which runs a little high; constant relative humidity under warming is the standard first-order assumption and is stated as one. The station layer (source = station) is the check on it. The warming applied is the cell's country's.");
        } else {
            d = d.param("station_max_km", param_f64(p, "station_max_km", 80.0), "km", "a capital with no reporting station this close is left out and counted")
                .view("counter", "people whose capital passes 31 °C wet-bulb on three days a year", "wetbulb.people_over_a", "the limit for healthy adults at rest; one station per country")
                .view("counter", "people whose capital passes 35 °C wet-bulb", "wetbulb.people_over_b", "the theoretical survivability limit")
                .view("map", "days a year over 31 °C wet-bulb at the capital", "wetbulb_days_over_31", "")
                .view("map", "the year each capital crossed 31 °C wet-bulb", "wetbulb_crossing_year", "")
                .view("crossings", "capitals crossing", "people.crossing", "")
                .view("chart", "people whose capital passes 31 °C wet-bulb", "wetbulb.people_over_a", "")
                .publish("people.crossing", "{ iso3, name, capital, station, year, kind: wetbulb_a | wetbulb_b, persons, days } the first year a capital passes a limit on days_to_cross days")
                .report("wetbulb.people_capital_over_a", "persons").report("wetbulb.people_capital_over_b", "persons").report("wetbulb.capitals_over_a", "").report("wetbulb.capitals_over_b", "").report("wetbulb.capitals_with_data", "")
                .asset("natural-earth-110m-populated-places", "capital per country").asset("noaa-isd-history", "the station nearest each capital").asset("noaa-gsod", "daily maximum temperature and mean dew point at that station")
                .note("The afternoon wet-bulb is estimated from the day's maximum temperature and the day's mean dew point, which runs a little high; constant relative humidity under warming is the standard first-order assumption and is stated as one. One station stands for a country, so a populous country crossing moves the count by its whole population in one year; source = cells removes that step.");
        }
        d
    }
}

/// Stull (2011) wet-bulb from dry-bulb t and relative humidity r, split into the terms that
/// depend on humidity alone (fixed when humidity is held) and the rest.
fn stull_terms(t: f64, r: f64) -> (f64, f64, f64, f64) {
    let r = r.clamp(5.0, 100.0);
    (t, r, (0.151977 * (r + 8.313659).sqrt()).atan(), -(r - 1.676331).atan() + 0.00391838 * r.powf(1.5) * (0.023101 * r).atan() - 4.686035)
}
fn stull_wb(tp: f64, r: f64, c1: f64, c2: f64) -> f64 { tp * c1 + (tp + r).atan() + c2 }
/// The warming of the dry-bulb, at constant humidity, that brings the day's wet-bulb to
/// `limit`: zero or less when it is already there. Newton on a monotone function.
fn warming_to_reach(limit: f64, t: f64, r: f64, c1: f64, c2: f64) -> f64 {
    let f = |d: f64| stull_wb(t + d, r, c1, c2) - limit;
    let fp = |d: f64| c1 + 1.0 / (1.0 + (t + d + r).powi(2));
    if f(0.0) >= 0.0 { return 0.0; }
    let mut d = -f(0.0) / fp(0.0);
    for _ in 0..5 { d -= f(d) / fp(d); if d < 0.0 { d = 0.0; } }
    d
}

impl Module for PeopleWetBulb {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.subscribe("people.population"); ctx.subscribe("climate.country_tas");
        if self.source == "cells" { ctx.report("wetbulb.cells_with_data", "", self.cell_ids.len() as f64); }
        else { ctx.report("wetbulb.capitals_with_data", "", self.capital.iter().filter(|c| c.is_some()).count() as f64); }
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "people.population" => {
                if let Some(t) = ev.data.get("by_iso3").and_then(|v| v.as_object()) { self.pop = t.iter().filter_map(|(k, v)| v.as_f64().map(|p| (k.clone(), p))).collect(); }
            }
            "climate.country_tas" if self.source == "cells" => self.handle_cells(ev, ctx),
            "climate.country_tas" => {
                let y = ev.data.get("year").and_then(|v| v.as_i64()).unwrap_or(year_of(ctx.now()));
                let Some(by) = ev.data.get("by_iso3").and_then(|v| v.as_object()) else { return };
                let g = world_grid(self.res_deg);
                let index = ctx.world_read_on("country.index", &g).map(|f| f.data.clone());
                let mut per = vec![f32::NAN; self.countries.len()];
                let (mut people_a, mut people_b, mut days_a, mut n_a, mut n_b) = (0.0f64, 0.0f64, 0.0f64, 0usize, 0usize);
                let mut crossings = Vec::new();
                for (k, c) in self.countries.iter().enumerate() {
                    let Some((cap, station, _)) = &self.capital[k] else { continue };
                    let Some(e) = by.get(&c.iso3) else { continue };
                    let Some(d) = e.get("tas_anom_c").and_then(|v| v.as_f64()) else { continue };
                    let (da, db) = (self.days(&self.table_a[k], d), self.days(&self.table_b[k], d));
                    per[k] = da as f32;
                    let p = self.pop.get(&c.iso3).copied().unwrap_or(0.0);
                    days_a += da * p;
                    if da > self.days_to_cross { people_a += p; n_a += 1; if !self.crossed_a.contains_key(&c.iso3) { self.crossed_a.insert(c.iso3.clone(), if self.baseline_done { y } else { 0 }); if self.baseline_done {
                        let sentence = format!("{}: {} ({}) passes {} °C wet-bulb on more than {} afternoons a year, the limit past which a healthy adult at rest can no longer shed heat. {} people live in {}.", y, cap, c.name, self.limit_a_c, self.days_to_cross as i64, people_words(p), c.name);
                        crossings.push(json!({ "iso3": c.iso3, "name": c.name, "capital": cap, "station": station, "year": y, "kind": "wetbulb_a", "persons": p, "days": da, "limit_c": self.limit_a_c, "sentence": sentence, "source": format!("Vecellio et al. 2022 (recalled); station {station}") })); } } }
                    if db > self.days_to_cross { people_b += p; n_b += 1; if !self.crossed_b.contains_key(&c.iso3) { self.crossed_b.insert(c.iso3.clone(), if self.baseline_done { y } else { 0 }); if self.baseline_done {
                        let sentence = format!("{}: {} ({}) passes {} °C wet-bulb, the theoretical limit of human survival outdoors, on more than {} afternoons a year. {} people live in {}.", y, cap, c.name, self.limit_b_c, self.days_to_cross as i64, people_words(p), c.name);
                        crossings.push(json!({ "iso3": c.iso3, "name": c.name, "capital": cap, "station": station, "year": y, "kind": "wetbulb_b", "persons": p, "days": db, "limit_c": self.limit_b_c, "sentence": sentence, "source": format!("Sherwood and Huber 2010 (recalled); station {station}") })); } } }
                }
                self.baseline_done = true;
                if let Some(index) = index {
                    let mut field = vec![f32::NAN; g.nx * g.ny];
                    let mut year_field = vec![f32::NAN; g.nx * g.ny];
                    let year_of_country: Vec<f32> = self.countries.iter().map(|c| self.crossed_a.get(&c.iso3).map(|yy| if *yy > 0 { *yy as f32 } else { f32::NAN }).unwrap_or(f32::NAN)).collect();
                    for (i, &k) in index.iter().enumerate() { if !k.is_nan() { field[i] = per[k as usize]; year_field[i] = year_of_country[k as usize]; } }
                    ctx.world_write("wetbulb.days_over_a", "d/yr", &g, field.clone());
                    if y - self.last_frame_year >= self.frame_every_years {
                        let fac = frame_factor(self.res_deg, self.frame_res_deg);
                        let (cg, a) = coarsen(&g, &field, fac, Fold::Max); let (_, b) = coarsen(&g, &year_field, fac, Fold::Min);
                        ctx.field("wetbulb_days_over_31", "d/yr", &cg, a); ctx.field("wetbulb_crossing_year", "year", &cg, b); self.last_frame_year = y;
                    }
                }
                for c in crossings { ctx.publish("people.crossing", c); }
                ctx.report("wetbulb.people_over_a", "persons", people_a);
                ctx.report("wetbulb.people_over_b", "persons", people_b);
                ctx.report("wetbulb.people_capital_over_a", "persons", people_a);
                ctx.report("wetbulb.people_capital_over_b", "persons", people_b);
                ctx.report("wetbulb.people_days_over_a", "person-days/yr", days_a);
                ctx.report("wetbulb.capitals_over_a", "", n_a as f64);
                ctx.report("wetbulb.capitals_over_b", "", n_b as f64);
            }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut m: PeopleWetBulb = serde_json::from_value(v).unwrap(); m.load().expect("wetbulb reload"); *self = m; }
}

impl PeopleWetBulb {
    /// Cells mode: each populated cell's days over the limits at its country's warming;
    /// people summed per cell, then by country.
    fn handle_cells(&mut self, ev: &Event, ctx: &mut Ctx) {
        let y = ev.data.get("year").and_then(|v| v.as_i64()).unwrap_or(year_of(ctx.now()));
        let Some(by) = ev.data.get("by_iso3").and_then(|v| v.as_object()) else { return };
        let g = world_grid(self.res_deg);
        let Some(index) = ctx.world_read_on("country.index", &g).map(|f| f.data.clone()) else { return };
        let Some(count) = ctx.world_read_on("people.count", &g).map(|f| f.data.clone()) else { return };
        let warming: Vec<f64> = self.countries.iter().map(|c| by.get(&c.iso3).and_then(|e| e.get("tas_anom_c")).and_then(|v| v.as_f64()).unwrap_or(f64::NAN)).collect();
        let n = self.countries.len();
        let steps = self.steps();
        let (mut over_a, mut over_b, mut tot_c, mut days_c) = (vec![0.0f64; n], vec![0.0f64; n], vec![0.0f64; n], vec![0.0f64; n]);
        let mut field = vec![f32::NAN; g.nx * g.ny];
        let mut year_field = vec![f32::NAN; g.nx * g.ny];
        for (i, &k) in index.iter().enumerate() { if !k.is_nan() { field[i] = 0.0; if count[i] > 0.0 { tot_c[k as usize] += count[i] as f64; } } }
        let (mut cells_a, mut people_days) = (0usize, 0.0f64);
        for (row, &id) in self.cell_ids.iter().enumerate() {
            let i = id as usize;
            let k = index[i]; if k.is_nan() { continue }
            let k = k as usize;
            if warming[k].is_nan() { continue }
            let (da, db) = (self.days(&self.cell_a[row * steps..(row + 1) * steps], warming[k]), self.days(&self.cell_b[row * steps..(row + 1) * steps], warming[k]));
            field[i] = da as f32;
            let p = count[i].max(0.0) as f64;
            people_days += da * p; days_c[k] += da * p;
            if da > self.days_to_cross {
                over_a[k] += p; cells_a += 1;
                let e = self.cell_year.entry(id).or_insert(if self.baseline_done { y as i32 } else { 0 });
                if *e > 0 { year_field[i] = *e as f32; }
            } else if let Some(e) = self.cell_year.get(&id) { if *e > 0 { year_field[i] = *e as f32; } }
            if db > self.days_to_cross { over_b[k] += p; }
        }
        let mut crossings = Vec::new();
        let (mut n_a, mut n_b) = (0usize, 0usize);
        for (k, c) in self.countries.iter().enumerate() {
            if tot_c[k] <= 0.0 { continue }
            let (sa, sb) = (over_a[k] / tot_c[k], over_b[k] / tot_c[k]);
            if sa >= self.crossing_share { n_a += 1; if !self.crossed_a.contains_key(&c.iso3) { self.crossed_a.insert(c.iso3.clone(), if self.baseline_done { y } else { 0 }); if self.baseline_done {
                let sentence = format!("{}: in {}, {:.0}% of the people ({}) now live where the afternoon wet-bulb passes {} °C on more than {} days a year, the limit past which a healthy adult at rest can no longer shed heat.", y, c.name, 100.0 * sa, people_words(over_a[k]), self.limit_a_c, self.days_to_cross as i64);
                crossings.push(json!({ "iso3": c.iso3, "name": c.name, "year": y, "kind": "wetbulb_a", "persons": over_a[k], "share": sa, "days": days_c[k] / tot_c[k], "limit_c": self.limit_a_c, "sentence": sentence, "source": "Vecellio et al. 2022 (recalled); MERRA-2 via NASA POWER" })); } } }
            if sb >= self.crossing_share { n_b += 1; if !self.crossed_b.contains_key(&c.iso3) { self.crossed_b.insert(c.iso3.clone(), if self.baseline_done { y } else { 0 }); if self.baseline_done {
                let sentence = format!("{}: in {}, {:.0}% of the people ({}) now live where the afternoon wet-bulb passes {} °C, the theoretical limit of human survival outdoors, on more than {} days a year.", y, c.name, 100.0 * sb, people_words(over_b[k]), self.limit_b_c, self.days_to_cross as i64);
                crossings.push(json!({ "iso3": c.iso3, "name": c.name, "year": y, "kind": "wetbulb_b", "persons": over_b[k], "share": sb, "limit_c": self.limit_b_c, "sentence": sentence, "source": "Sherwood and Huber 2010 (recalled); MERRA-2 via NASA POWER" })); } } }
        }
        self.baseline_done = true;
        ctx.world_write("wetbulb.days_over_a", "d/yr", &g, field.clone());
        if y - self.last_frame_year >= self.frame_every_years {
            let fac = frame_factor(self.res_deg, self.frame_res_deg);
            let (cg, a) = coarsen(&g, &field, fac, Fold::Max); let (_, b) = coarsen(&g, &year_field, fac, Fold::Min);
            ctx.field("wetbulb_days_over_31", "d/yr", &cg, a); ctx.field("wetbulb_crossing_year", "year", &cg, b); self.last_frame_year = y;
        }
        for c in crossings { ctx.publish("people.crossing", c); }
        ctx.report("wetbulb.people_over_a", "persons", over_a.iter().sum());
        ctx.report("wetbulb.people_over_b", "persons", over_b.iter().sum());
        ctx.report("wetbulb.people_days_over_a", "person-days/yr", people_days);
        ctx.report("wetbulb.cells_over_a", "", cells_a as f64);
        ctx.report("wetbulb.countries_over_a", "", n_a as f64);
        ctx.report("wetbulb.countries_over_b", "", n_b as f64);
    }
}

// ---------------------------------------------------------------------------------------
// story.bottomline: the headline of a planetary run, from what the other layers publish.

#[derive(Serialize, Deserialize)]
pub struct Story {
    name: String,
    headlines: u64,
    people: f64, outside: f64, wb: f64,
}

impl Story {
    pub fn new(name: &str, _p: &toml::Value) -> Result<Story, String> { Ok(Story { name: name.to_string(), headlines: 0, people: 0.0, outside: 0.0, wb: 0.0 }) }
    pub fn describe(name: &str, _p: &toml::Value) -> Describe {
        Describe::new("story.bottomline", name, "The bottom line of a planetary run, told as it happens: the headline numbers at the cursor and one plain sentence for every threshold crossed, each sentence written by the module that knows its source. It computes nothing new and it says, in the same place, what this run does not yet measure.")
            .priority(9).dt("follows the crossing events")
            .subscribe("people.crossing", "sentences").subscribe("tipping.crossing", "sentences").subscribe("people.population", "the count")
            .report("story.headlines", "")
            .view("map", "people living outside the human climate niche", "people_outside_niche", "annual mean above 29 °C")
            .view("map", "days a year over 31 °C wet-bulb", "wetbulb_days_over_31", "")
            .view("map", "the year each place crossed 31 °C wet-bulb", "wetbulb_crossing_year", "")
            .view("counter", "people on Earth", "people.world_total", "UN estimates to 2023, the UN medium projection after")
            .view("counter", "warming since 1850", "energy.gmst_anom_c", "")
            .view("counter", "people outside the human climate niche", "people.outside_niche", "annual mean above 29 °C where they live")
            .view("counter", "people living where the wet-bulb limit is passed", "wetbulb.people_over_a", "31 °C on more than three afternoons a year")
            .view("counter", "person-days a year over the wet-bulb limit", "wetbulb.people_days_over_a", "people times days")
            .view("counter", "tipping elements past their central estimate", "tipping.elements_past_central", "")
            .view("counter", "sea level from expansion alone", "sea.thermal_m", "")
            .view("story", "what has happened so far", "crossing", "one sentence per threshold crossed, newest first; the source is beside each")
            .view("text", "what this run does not yet measure", "Not in this run, so not in these numbers: deaths from heat (a published exposure-response layer is next), harvests and food prices, forest cover and the carbon it would release, biodiversity, fresh water, sea level from ice, migration, and any response by people to any of it. The population here is the UN's path, which assumes none of the above changes where people live. Every crossing above is a count of people living where a stated limit is passed, not a toll, and the model runs about 0.15 °C warm in 2024. The warming applied to a place is its country's, from the country record: within a country, the pattern of the recent past is held.", "")
    }
}

impl Module for Story {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) { ctx.subscribe("people.crossing"); ctx.subscribe("tipping.crossing"); ctx.subscribe("people.population"); }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        match ev.topic.as_str() {
            "people.crossing" | "tipping.crossing" => { self.headlines += 1; ctx.report("story.headlines", "", self.headlines as f64); }
            _ => {}
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { *self = serde_json::from_value(v).unwrap(); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warming_to_reach_inverts_the_wet_bulb() {
        for &(t, rh) in &[(30.0, 60.0), (38.0, 40.0), (42.0, 20.0), (25.0, 90.0)] {
            let (t, r, c1, c2) = stull_terms(t, rh);
            for limit in [31.0, 35.0] {
                let d = warming_to_reach(limit, t, r, c1, c2);
                assert!(d >= 0.0);
                if d > 0.0 { assert!((stull_wb(t + d, r, c1, c2) - limit).abs() < 1e-6, "t {t} rh {rh} limit {limit}: d {d}"); }
                else { assert!(stull_wb(t, r, c1, c2) >= limit); }
            }
        }
    }

    #[test]
    fn coarsen_folds_sums_and_extremes_and_keeps_nan_where_empty() {
        let g = LatLonGrid { nx: 4, ny: 2, dlon: 1.0, dlat: 1.0, lon_w: -2.0, lat_n: 1.0 };
        let nan = f32::NAN;
        let data = vec![1.0, 2.0, nan, nan, 3.0, 4.0, nan, 5.0];
        let (cg, s) = coarsen(&g, &data, 2, Fold::Sum);
        assert_eq!((cg.nx, cg.ny, cg.dlon), (2, 1, 2.0));
        assert_eq!(s, vec![10.0, 5.0]);
        let (_, m) = coarsen(&g, &data, 2, Fold::Max); assert_eq!(m, vec![4.0, 5.0]);
        let (_, n) = coarsen(&g, &data, 2, Fold::Min); assert_eq!(n, vec![1.0, 5.0]);
        let (_, a) = coarsen(&g, &vec![nan; 8], 2, Fold::Mean); assert!(a.iter().all(|v| v.is_nan()));
        let (same, d) = coarsen(&g, &data, 1, Fold::Sum); assert_eq!(same, g); assert_eq!(d.len(), 8);
    }

    #[test]
    fn power_boxes_and_points_line_up() {
        let (la, lo) = cells::nearest_power_point(28.75, 77.25);
        assert_eq!((la, lo), (29.0, 77.5));
        assert_eq!(box_of(28.75, 77.25), (20, 70));
        assert_eq!(box_of(-0.25, -0.25), (-10, -10));
        let (_, lo) = cells::nearest_power_point(10.0, 179.9);
        assert!(lo == -180.0 || lo == 180.0);
    }
}
