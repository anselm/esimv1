//! M7 Tearing: fire as the first hazard. The hazard row comes from
//! data/events/hazards.toml; the manifest sets it off, on (stochastic, from the module's
//! own seeded stream, so ensemble branches sample it) or forced at a date and place.
//! Ignition is a cell-addressed event, the kernel's first sparse spatial event: modules
//! subscribed at that cell hear it (the herbivore flees). Spread is a daily cellular
//! automaton over basin cells: a burning cell ignites each neighbour with a probability
//! that rises with fuel, dryness, temperature and wind, and falls to zero without fuel.
//! Burning publishes `fire.burn` with a field of burned fractions; the crop module sends
//! biomass and litter to the atmosphere, the cohorts lose a share of their animals.
//! Rothermel's spread model is the reference to port when a fire case needs rates of
//! spread in metres per minute; this is the mechanism, not the physics.

use esim_kernel::manifest::{param_duration, param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module, Time};
use esim_registry::events::EventTable;
use esim_registry::hydro::delineate;
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Fire {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64, gauge_lat: f64, gauge_lon: f64, snap_radius: usize,
    dt: Duration,
    hazard_id: String,
    setting: String,
    forced_date: String,
    forced_lat: f64, forced_lon: f64,
    ignition_per_cell_day: f64,
    spread_base: f64,
    fuel_min_gc_m2: f64,
    soil_capacity_mm: f64,
    burn_days: u32,
    refractory_days: u32,
    // state per basin cell: 0 unburnt, >0 burning days left, <0 refractory days left (negative count)
    state: Vec<i32>,
    forced_done: bool,
    ignitions_total: f64,
    burned_km2_total: f64,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] cells: Vec<usize>,
    #[serde(skip)] area_km2: Vec<f64>,
    #[serde(skip)] nbrs: Vec<Vec<usize>>,
    #[serde(skip)] row_title: String,
    #[serde(skip)] row_source: String,
    #[serde(skip)] row_confidence: String,
}

impl Fire {
    pub fn new(name: &str, p: &toml::Value) -> Result<Fire, String> {
        let mut f = Fire {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 42.3), lat_n: param_f64(p, "lat_n", 43.2), lon_w: param_f64(p, "lon_w", -94.2), lon_e: param_f64(p, "lon_e", -93.5),
            res_arcsec: param_f64(p, "res_arcsec", 30.0), gauge_lat: param_f64(p, "gauge_lat", 42.4319722), gauge_lon: param_f64(p, "gauge_lon", -93.8059444), snap_radius: param_f64(p, "snap_radius", 3.0) as usize,
            dt: param_duration(p, "dt", "1d")?,
            hazard_id: param_str(p, "hazard_id", "cropland-fire-corn-belt").to_string(),
            setting: param_str(p, "setting", "").to_string(),
            forced_date: param_str(p, "forced_date", "").to_string(),
            forced_lat: param_f64(p, "forced_lat", 42.7), forced_lon: param_f64(p, "forced_lon", -93.85),
            ignition_per_cell_day: param_f64(p, "ignition_per_cell_day", 2e-6),
            spread_base: param_f64(p, "spread_base", 0.35),
            fuel_min_gc_m2: param_f64(p, "fuel_min_gc_m2", 60.0),
            soil_capacity_mm: param_f64(p, "soil_capacity_mm", 150.0),
            burn_days: param_f64(p, "burn_days", 1.0) as u32,
            refractory_days: param_f64(p, "refractory_days", 60.0) as u32,
            state: Vec::new(), forced_done: false, ignitions_total: 0.0, burned_km2_total: 0.0,
            grid: None, cells: Vec::new(), area_km2: Vec::new(), nbrs: Vec::new(), row_title: String::new(), row_source: String::new(), row_confidence: String::new(),
        };
        f.load()?;
        Ok(f)
    }
    fn load(&mut self) -> Result<(), String> {
        let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let table = EventTable::load(&root.join("data/events/hazards.toml"))?;
        let row = table.get(&self.hazard_id).ok_or_else(|| format!("hazards.toml has no row {}", self.hazard_id))?;
        if self.setting.is_empty() { self.setting = row.setting.clone().unwrap_or_else(|| "on".into()); }
        self.row_title = row.title.clone(); self.row_source = row.source.clone(); self.row_confidence = row.confidence.clone();
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let dem = cat.asset("copernicus-dem-glo30").ok_or("catalog has no copernicus-dem-glo30")?;
        let mut f = Fetcher::default_for(&root)?;
        let w = dem_window(&mut f, dem, self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let grid = LatLonGrid { nx: w.nx, ny: w.ny, dlon: w.dlon, dlat: w.dlat, lon_w: w.lon_w, lat_n: w.lat_n };
        let row_area: Vec<f64> = (0..grid.ny).map(|r| grid.row_area(r)).collect();
        let (_, _, cells, mask) = delineate(&f, &w, &row_area, self.gauge_lat, self.gauge_lon, self.snap_radius);
        let mut pos = vec![usize::MAX; grid.nx * grid.ny];
        for (k, &i) in cells.iter().enumerate() { pos[i] = k; }
        self.nbrs = cells.iter().map(|&i| {
            let (c, r) = (i % grid.nx, i / grid.nx);
            let mut v = Vec::new();
            if c > 0 && mask[i - 1] { v.push(pos[i - 1]); }
            if c + 1 < grid.nx && mask[i + 1] { v.push(pos[i + 1]); }
            if r > 0 && mask[i - grid.nx] { v.push(pos[i - grid.nx]); }
            if r + 1 < grid.ny && mask[i + grid.nx] { v.push(pos[i + grid.nx]); }
            v
        }).collect();
        self.area_km2 = cells.iter().map(|&i| row_area[i / grid.nx] / 1e6).collect();
        if self.state.len() != cells.len() { self.state = vec![0; cells.len()]; }
        self.grid = Some(grid); self.cells = cells;
        Ok(())
    }
    fn cell_of(&self, lat: f64, lon: f64) -> Option<usize> {
        let g = self.grid.as_ref().unwrap();
        let col = ((lon - g.lon_w) / g.dlon) as usize; let row = ((g.lat_n - lat) / g.dlat) as usize;
        let i = row * g.nx + col;
        self.cells.iter().position(|&c| c == i)
    }
}

impl Module for Fire {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        // the hazard row is logged, so the run's log says where this hazard came from
        ctx.publish("hazard.loaded", json!({ "id": self.hazard_id, "title": self.row_title, "setting": self.setting, "confidence": self.row_confidence, "source": self.row_source }));
        ctx.report("fire.setting", "0=off 1=on 2=forced", match self.setting.as_str() { "off" => 0.0, "forced" => 2.0, _ => 1.0 });
        ctx.schedule_with_priority(self.dt, 6, "tick", json!(null));
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tick" { return; }
        let g = self.grid.clone().unwrap();
        let now = ctx.now();
        let n = self.cells.len();
        if self.setting == "off" { ctx.schedule_with_priority(self.dt, 6, "tick", json!(null)); return; }
        // state the hazard depends on
        let soil: Option<Vec<f32>> = ctx.world_read("water.soil_mm").map(|f| f.data.clone());
        let biomass: Option<Vec<f32>> = ctx.world_read("veg.biomass_gc_m2").map(|f| f.data.clone());
        let t2m: Option<Vec<f32>> = ctx.world_read("wx.t2m_c").map(|f| f.data.clone());
        let wind: Option<Vec<f32>> = ctx.world_read("wx.wind_m_s").map(|f| f.data.clone());
        let litter_frac = 1.0; // litter is fuel too; the crop module decides how much burns
        let fuel = |k: usize| -> f64 { biomass.as_ref().map(|b| b[self.cells[k]] as f64).unwrap_or(0.0) * litter_frac };
        let dryness = |k: usize| -> f64 { soil.as_ref().map(|s| (1.0 - (s[self.cells[k]] as f64 / self.soil_capacity_mm)).clamp(0.0, 1.0)).unwrap_or(0.5) };
        let heat = |k: usize| -> f64 { t2m.as_ref().map(|t| ((t[self.cells[k]] as f64 - 5.0) / 25.0).clamp(0.0, 1.0)).unwrap_or(0.5) };
        let windf = |k: usize| -> f64 { wind.as_ref().map(|w| (1.0 + (w[self.cells[k]] as f64 / 5.0).min(3.0)) / 2.0).unwrap_or(1.0) };
        // 1. ignitions
        let mut ignited: Vec<usize> = Vec::new();
        if self.setting == "forced" && !self.forced_done {
            if let Ok(t) = Time::parse(&self.forced_date) {
                if now.0 >= t.0 {
                    if let Some(k) = self.cell_of(self.forced_lat, self.forced_lon) { ignited.push(k); }
                    self.forced_done = true;
                }
            }
        }
        if self.setting == "on" {
            for k in 0..n {
                if self.state[k] != 0 || fuel(k) < self.fuel_min_gc_m2 { continue; }
                let p = self.ignition_per_cell_day * dryness(k).powi(2) * heat(k) * windf(k);
                if ctx.rng().f64() < p { ignited.push(k); }
            }
        }
        for &k in &ignited {
            if self.state[k] == 0 { self.state[k] = self.burn_days as i32; }
            self.ignitions_total += 1.0;
            ctx.publish_at(self.cells[k], "fire.ignition", json!({ "hazard": self.hazard_id, "setting": self.setting, "lat": g.lat(self.cells[k] / g.nx), "lon": g.lon(self.cells[k] % g.nx) }));
        }
        // 2. everything burning today burns (ignitions included): publish the field
        let burning_now: Vec<usize> = (0..n).filter(|&k| self.state[k] > 0).collect();
        let mut burned = vec![f32::NAN; g.nx * g.ny];
        for k in 0..n { burned[self.cells[k]] = 0.0; }
        let mut burned_km2 = 0.0;
        for &k in &burning_now { burned[self.cells[k]] = 1.0; burned_km2 += self.area_km2[k]; }
        if !burning_now.is_empty() {
            ctx.world_write("fire.burned_frac", "1", &g, burned.clone());
            ctx.publish("fire.burn", json!({ "cells": burning_now.len(), "km2": burned_km2 }));
            ctx.field("fire_burning", "1", &g, burned);
        }
        self.burned_km2_total += burned_km2;
        // 3. spread: today's burning cells light neighbours, which burn from tomorrow
        let mut newly: Vec<usize> = Vec::new();
        for &k in &burning_now {
            for &j in &self.nbrs[k] {
                if self.state[j] != 0 || fuel(j) < self.fuel_min_gc_m2 { continue; }
                let p = self.spread_base * (fuel(j) / 400.0).min(1.0) * dryness(j) * (0.5 + 0.5 * heat(j)) * windf(j);
                if ctx.rng().f64() < p { newly.push(j); }
            }
        }
        newly.sort_unstable(); newly.dedup();
        // 4. advance: today's burners finish and rest; tomorrow's start
        for &k in &burning_now {
            self.state[k] -= 1;
            if self.state[k] == 0 { self.state[k] = -(self.refractory_days as i32); }
        }
        for k in 0..n { if self.state[k] < 0 && !burning_now.contains(&k) { self.state[k] += 1; } }
        for &j in &newly { if self.state[j] == 0 { self.state[j] = self.burn_days as i32; } }
        ctx.report("fire.ignitions_day", "1/day", ignited.len() as f64);
        ctx.report("fire.burning_km2", "km2", burned_km2);
        ctx.report("fire.burned_km2_total", "km2", self.burned_km2_total);
        ctx.schedule_with_priority(self.dt, 6, "tick", json!(null));
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut f: Fire = serde_json::from_value(v).unwrap(); f.load().expect("fire reload"); *self = f; }
}
