//! `water.surface`: where the water is. The run's cells that are permanent open water, read from
//! ESA WorldCover 2021 (a satellite land-cover map at 10 m), as a world field other modules can
//! read and a frame for the map. It also receives what the tanks release toward water and sets
//! each release down on the water cell nearest the tank. It does not move water or anything on it:
//! no flow, no stage, no tide, no plume. That is what a hydrology module would add; this one gives
//! the water layer its first occupant and the spill a place to arrive.
use crate::fragility::coarse_of;
use crate::quake::{cell_of, root, window_grid};
use esim_kernel::manifest::{param_f64, param_str};
use esim_kernel::{Ctx, Describe, Event, LatLonGrid, Module};
use esim_registry::wildfire::landcover_window;
use esim_registry::{Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;

const WATER_CLASS: u8 = 80;   // WorldCover: permanent water bodies

#[derive(Serialize, Deserialize)]
pub struct SurfaceWater {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    coarse_arcsec: f64, reach_cells: usize,
    arrived_gal: f64, stranded_gal: f64,
    #[serde(skip)] grid: Option<LatLonGrid>, #[serde(skip)] water: Vec<f32>, #[serde(skip)] on_water: Vec<f32>,
    coarse: Option<LatLonGrid>,
}

impl SurfaceWater {
    pub fn new(name: &str, p: &toml::Value) -> Result<SurfaceWater, String> {
        let mut o = SurfaceWater {
            name: name.to_string(),
            lat_s: crate::fragility::need_f64(p, "lat_s", name)?, lat_n: crate::fragility::need_f64(p, "lat_n", name)?, lon_w: crate::fragility::need_f64(p, "lon_w", name)?, lon_e: crate::fragility::need_f64(p, "lon_e", name)?, res_arcsec: param_f64(p, "res_arcsec", 1.0),
            coarse_arcsec: param_f64(p, "coarse_arcsec", 10.0), reach_cells: param_f64(p, "reach_cells", 60.0) as usize,
            arrived_gal: 0.0, stranded_gal: 0.0, grid: None, water: Vec::new(), on_water: Vec::new(), coarse: None,
        };
        o.load()?;
        Ok(o)
    }
    fn load(&mut self) -> Result<(), String> {
        let (g, _) = window_grid(self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let root = root();
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let lc = cat.asset("esa-worldcover-2021").ok_or("catalog has no esa-worldcover-2021")?;
        let mut fe = Fetcher::default_for(&root)?;
        let cover = landcover_window(&mut fe, lc, g.nx, g.ny, g.dlon, g.dlat, g.lon_w, g.lat_n)?;
        self.water = cover.iter().map(|&c| if c == WATER_CLASS { 1.0 } else { 0.0 }).collect();
        let (cg, _) = coarse_of(&g, self.coarse_arcsec, self.res_arcsec);
        if self.on_water.len() != cg.nx * cg.ny { self.on_water = vec![0.0; cg.nx * cg.ny]; }
        self.coarse = Some(cg); self.grid = Some(g);
        Ok(())
    }
    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let _ = param_str(p, "folder", "");
        Describe::new("water.surface", name, "This module says where the water is. It reads ESA WorldCover 2021, a land-cover map made from satellite images at 10 metres, and marks each cell of the run's grid that is mostly permanent open water: rivers, lakes and harbours. Other modules can read that map. When the tanks module reports a release that goes toward water, this module sets the gallons down on the water cell nearest the tank, so the map shows where the spill meets the river. It does not model water: there is no flow, no river level, no tide and no movement of anything on the surface, so the fuel stays where it arrives. A release with no water within reach is counted as stranded.")
            .priority(1).dt("on a tank's release")
            .param("coarse_arcsec", param_f64(p, "coarse_arcsec", 10.0), "arcsec", "the cell size of the map of fuel on the water")
            .param("reach_cells", param_f64(p, "reach_cells", 60.0), "cells", "how far from a tank, in grid cells, the nearest water is looked for")
            .subscribe("tank.release", "{ gal, river } at a tank's location: fuel leaving a failed tank")
            .own("water.open", "1")
            .frame("water_open", "1").frame("fuel_on_water_gal", "gal")
            .report("water.area_km2", "km2").report("water.fuel_arrived_gal", "gal").report("water.fuel_stranded_gal", "gal")
            .asset("esa-worldcover-2021", "the land-cover tile the water cells are read from")
            .view("map", "open water", "water_open", "1 where a cell is mostly permanent water in ESA WorldCover 2021")
            .view("map", "fuel arriving on the water", "fuel_on_water_gal", "gallons set down on the water cell nearest each failed tank that releases toward water, on a coarser grid; the fuel is not moved after that")
            .view("counter", "open water", "water.area_km2", "the area of the water cells, in square kilometres")
            .view("counter", "fuel arriving on the water", "water.fuel_arrived_gal", "the releases toward water that found a water cell within reach")
            .view("text", "what this module leaves out", "Everything that makes water hydrology: discharge, river level, tides, currents, groundwater and rain. The water cells are a 2021 snapshot from satellite images and miss narrow streams. Fuel is set down at the nearest water and never moves, so the map shows where a spill would enter the river and nothing about where it would go.", "")
    }
}

impl Module for SurfaceWater {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        ctx.subscribe("tank.release");
        let g = self.grid.clone().expect("grid");
        ctx.world_write("water.open", "1", &g, self.water.clone());
        ctx.field("water_open", "1", &g, self.water.clone());
        let cg = self.coarse.clone().unwrap();
        ctx.field("fuel_on_water_gal", "gal", &cg, self.on_water.clone());
        // a cell's area from its latitude
        let mut km2 = 0.0;
        for r in 0..g.ny { let lat = g.lat_n - (r as f64 + 0.5) * g.dlat; let a = (g.dlat * 110.57) * (g.dlon * 111.32 * lat.to_radians().cos()); km2 += a * self.water[r * g.nx..(r + 1) * g.nx].iter().map(|&v| v as f64).sum::<f64>(); }
        ctx.report("water.area_km2", "km2", km2);
        ctx.report("water.fuel_arrived_gal", "gal", self.arrived_gal); ctx.report("water.fuel_stranded_gal", "gal", self.stranded_gal);
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "tank.release" { return; }
        if ev.data.get("river").and_then(|v| v.as_bool()) != Some(true) { return; }
        let gal = ev.data.get("gal").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let Some(cell) = ev.at.map(|c| c as usize) else { return; };
        let g = self.grid.clone().expect("grid");
        let (r0, c0) = ((cell / g.nx) as i64, (cell % g.nx) as i64);
        // the nearest water cell, searched ring by ring out to the reach
        let mut best: Option<(i64, usize)> = None;
        'rings: for k in 0..=self.reach_cells as i64 {
            for dr in -k..=k { for dc in -k..=k {
                if dr.abs().max(dc.abs()) != k { continue; }
                let (r, c) = (r0 + dr, c0 + dc);
                if r < 0 || c < 0 || r >= g.ny as i64 || c >= g.nx as i64 { continue; }
                let i = r as usize * g.nx + c as usize;
                if self.water[i] > 0.5 { let d2 = dr * dr + dc * dc; if best.map_or(true, |b| d2 < b.0) { best = Some((d2, i)); } }
            } }
            if best.is_some() { break 'rings; }
        }
        match best {
            Some((_, i)) => {
                let cg = self.coarse.clone().unwrap();
                let (lat, lon) = (g.lat_n - ((i / g.nx) as f64 + 0.5) * g.dlat, g.lon_w + ((i % g.nx) as f64 + 0.5) * g.dlon);
                self.on_water[cell_of(&cg, lat, lon)] += gal as f32; self.arrived_gal += gal;
                ctx.field("fuel_on_water_gal", "gal", &cg, self.on_water.clone());
            }
            None => self.stranded_gal += gal,
        }
        ctx.report("water.fuel_arrived_gal", "gal", self.arrived_gal); ctx.report("water.fuel_stranded_gal", "gal", self.stranded_gal);
        let _ = json!(null);
    }
    fn snapshot(&self) -> serde_json::Value { let mut v = serde_json::to_value(self).unwrap(); v["on_water"] = json!(self.on_water); v }
    fn restore(&mut self, v: serde_json::Value) { let on: Vec<f32> = v.get("on_water").and_then(|x| serde_json::from_value(x.clone()).ok()).unwrap_or_default(); let mut o: SurfaceWater = serde_json::from_value(v).unwrap(); o.on_water = on; o.load().expect("surface water reload"); *self = o; }
}
