//! weather.heatwaves: a weather generator for the planet runs, which otherwise have no
//! days. The planet is an energy balance and the cell layers shift a climatology by the
//! warming; a heatwave is a run of days, and nothing in the 2100 scenario could be hot on a
//! Tuesday until this module (Anselm, 2026-11-07: "there should be some kind of periodic
//! heatwave ... lets do some kind of weather generator - can be imperfect").
//!
//! What it does: a baseline decade of daily maximum temperature per populated one-degree
//! cell (MERRA-2 through NASA POWER, the same boxes the wet-bulb layer reads). Each
//! simulated year it draws one of those observed years by the run's seed and replays it,
//! shifted by the cell's warming since that decade (its country's share of the global
//! anomaly, or its own slope with pattern = cells), and counts heatwaves the way
//! Aglas-Leitner et al. 2026 count them (Nature Communications 17:11456, read 2026-11-07):
//! three or more days over the calendar-day 90th percentile of a fixed base, severity as
//! the day's excess over the climatology in units of the threshold's excess, cumulative
//! heat, duration, and the share of a country's people under extreme heat on the peak day.
//!
//! What is imperfect, on purpose and said in the describe: the base is the observed decade
//! itself (the paper uses 1961 to 1990; POWER begins in 1981), so "exceptional" here means
//! exceptional for 2011 to 2020; the delta method keeps today's variability while the mean
//! moves, and the paper finds heatwaves lengthen and widen faster than the mean alone
//! predicts, so this understates them unless `spread_per_degree` is set, and that number is
//! a guess; the weather of one year is drawn whole, so a cell's days are coherent but years
//! are independent, which is not the climate's memory.

use crate::countries::{self, Fold};
use esim_kernel::{Ctx, Event, Module};
use esim_kernel::describe::Describe;
use esim_kernel::manifest::{param_f64, param_str};
use esim_registry::catalog::Catalog;
use esim_registry::cells;
use esim_registry::countries::Country;
use esim_registry::fetch::Fetcher;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

const SLOTS: usize = 366;
const MISSING: i16 = i16::MIN;

#[derive(Serialize, Deserialize)]
pub struct Heatwaves {
    name: String,
    res_deg: f64,
    work_res_deg: f64,
    frame_res_deg: f64,
    frame_every_years: f64,
    scale: String,
    year_from: i64,
    year_to: i64,
    pattern: String,
    pattern_from: i64,
    pattern_to: i64,
    threshold_pct: f64,
    window_days: usize,
    min_days: usize,
    spread_per_degree: f64,
    notable_days: usize,
    month_days: usize,
    season_days: usize,
    area_share: f64,
    // state
    g_obs: f64,
    crossed_month: BTreeMap<String, i64>,
    crossed_season: BTreeMap<String, i64>,
    last_frame_year: i64,
    baseline_done: bool,
    // tables, reloaded on restore
    #[serde(skip)] cell_ids: Vec<u32>,
    #[serde(skip)] series: Vec<i16>,   // n * years * SLOTS, tenths of a degree, MISSING where none
    #[serde(skip)] clim: Vec<i16>,     // n * SLOTS
    #[serde(skip)] thr: Vec<i16>,      // n * SLOTS
    #[serde(skip)] years: usize,
    #[serde(skip)] countries: Vec<Country>,
    #[serde(skip)] slope: Vec<f32>,
    #[serde(skip)] country_clim: Vec<f32>, // k * SLOTS, people-weighted mean Tmax climatology per country
    #[serde(skip)] country_thr: Vec<f32>,
    #[serde(skip)] country_tables_done: bool,
}

#[derive(Clone, Copy, Default)]
struct CellYear { days: u16, longest: u16, max_sev: f32, cum: f32 }

impl Heatwaves {
    pub fn new(name: &str, p: &toml::Value) -> Result<Heatwaves, String> {
        let mut m = Heatwaves {
            name: name.into(),
            res_deg: param_f64(p, "res_deg", 0.5), work_res_deg: param_f64(p, "work_res_deg", 1.0),
            frame_res_deg: param_f64(p, "frame_res_deg", 1.0), frame_every_years: param_f64(p, "frame_every_years", 5.0),
            scale: param_str(p, "scale", "110m").into(),
            year_from: param_f64(p, "year_from", 2011.0) as i64, year_to: param_f64(p, "year_to", 2020.0) as i64,
            pattern: param_str(p, "pattern", "country").into(), pattern_from: param_f64(p, "pattern_from", 1981.0) as i64, pattern_to: param_f64(p, "pattern_to", 2025.0) as i64,
            threshold_pct: param_f64(p, "threshold_pct", 90.0), window_days: param_f64(p, "window_days", 15.0) as usize, min_days: param_f64(p, "min_days", 3.0) as usize,
            spread_per_degree: param_f64(p, "spread_per_degree", 0.0),
            notable_days: param_f64(p, "notable_days", 14.0) as usize, month_days: param_f64(p, "month_days", 30.0) as usize, season_days: param_f64(p, "season_days", 60.0) as usize,
            area_share: param_f64(p, "area_share", 0.5),
            g_obs: 0.0, crossed_month: BTreeMap::new(), crossed_season: BTreeMap::new(), last_frame_year: -1_000_000, baseline_done: false,
            cell_ids: Vec::new(), series: Vec::new(), clim: Vec::new(), thr: Vec::new(), years: 0, countries: Vec::new(), slope: Vec::new(),
            country_clim: Vec::new(), country_thr: Vec::new(), country_tables_done: false,
        };
        if m.work_res_deg < m.res_deg { return Err(format!("{name}: work_res_deg must be at least res_deg")); }
        m.load()?;
        Ok(m)
    }

    fn load(&mut self) -> Result<(), String> {
        let root = countries::root();
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let mut f = Fetcher::default_for(&root)?;
        self.countries = countries::load_countries(&self.scale)?;
        let g_all = countries::global_anomaly(&mut f, &cat)?;
        let obs: Vec<f64> = (self.year_from..=self.year_to).filter_map(|y| g_all.get(&y).copied()).collect();
        if obs.is_empty() { return Err(format!("{}: no global anomaly for {} to {}", self.name, self.year_from, self.year_to)); }
        self.g_obs = obs.iter().sum::<f64>() / obs.len() as f64;
        if self.pattern == "cells" {
            let fine = countries::load_pattern(&mut f, &cat, self.res_deg, self.pattern_from, self.pattern_to)?;
            let (_, coarse) = countries::coarsen(&countries::world_grid(self.res_deg), &fine, self.fac(), Fold::Mean);
            self.slope = coarse;
        }
        self.years = (self.year_to - self.year_from + 1) as usize;
        let derived = f.cache_dir.join(format!("heatwaves_{}_{}_{}_{}_{}.bin", self.work_res_deg, self.year_from, self.year_to, self.threshold_pct, self.window_days));
        if let Ok(bytes) = std::fs::read(&derived) {
            if self.decode(&bytes) { return Ok(()); }
        }
        let people = cells::worldpop_count(&mut f, &countries::asset_url(&cat, "worldpop-2020-1km")?, self.res_deg)?;
        let g = countries::world_grid(self.res_deg);
        let gw = countries::world_grid(self.work_res_deg);
        let (_, people_w) = countries::coarsen(&g, &people, self.fac(), Fold::Sum);
        let boxes = cells::power_boxes(self.work_res_deg, |c, r| people_w[r * gw.nx + c] > 0.0);
        let t0 = std::time::Instant::now();
        let mut ids: Vec<u32> = Vec::new();
        let mut series: Vec<i16> = Vec::new();
        for (bi, &(lat0, lon0)) in boxes.iter().enumerate() {
            let members: Vec<usize> = (0..gw.nx * gw.ny).filter(|&i| people_w[i] > 0.0 && { let (c, r) = gw.col_row(i); countries::box_of(gw.lat(r), gw.lon(c)) == (lat0, lon0) }).collect();
            if members.is_empty() { continue; }
            let base = ids.len();
            for &i in &members { ids.push(i as u32); }
            series.resize(ids.len() * self.years * SLOTS, MISSING);
            for (yi, y) in (self.year_from..=self.year_to).enumerate() {
                let bt = cells::power_daily_box(&mut f, "T2M_MAX", lat0, lon0, y)?;
                let pt = countries::points_for(&bt, &gw, &members);
                for (m, &pa) in pt.iter().enumerate() {
                    let Some(pa) = pa else { continue };
                    let row = base + m;
                    for d in 0..bt.days.min(SLOTS) {
                        series[(row * self.years + yi) * SLOTS + d] = bt.data[pa * bt.days + d];
                    }
                }
            }
            if bi % 40 == 39 || bi + 1 == boxes.len() { eprintln!("heatwaves: {}/{} boxes, {} cells, {:.0} s", bi + 1, boxes.len(), ids.len(), t0.elapsed().as_secs_f64()); }
        }
        self.cell_ids = ids;
        self.series = series;
        self.tables();
        std::fs::write(&derived, self.encode()).map_err(|e| e.to_string())?;
        eprintln!("heatwaves: {} cells, {} years, tables in {:.0} s", self.cell_ids.len(), self.years, t0.elapsed().as_secs_f64());
        Ok(())
    }

    fn fac(&self) -> usize { countries::frame_factor(self.res_deg, self.work_res_deg) }

    /// Per cell and calendar day, the climatology and the threshold over the decade within
    /// a window of days either side (wrapping the year; slot 365 takes slot 364's).
    fn tables(&mut self) {
        let n = self.cell_ids.len();
        self.clim = vec![MISSING; n * SLOTS];
        self.thr = vec![MISSING; n * SLOTS];
        let w = self.window_days as i64;
        let mut samples: Vec<i16> = Vec::with_capacity(self.years * (2 * self.window_days + 1));
        for row in 0..n {
            for d in 0..365usize {
                samples.clear();
                for yi in 0..self.years {
                    for k in -w..=w {
                        let dd = ((d as i64 + k).rem_euclid(365)) as usize;
                        let v = self.series[(row * self.years + yi) * SLOTS + dd];
                        if v != MISSING { samples.push(v); }
                    }
                }
                if samples.len() < 10 { continue; }
                let mean = samples.iter().map(|&v| v as f64).sum::<f64>() / samples.len() as f64;
                samples.sort_unstable();
                let rank = ((self.threshold_pct / 100.0) * (samples.len() as f64 - 1.0)).round() as usize;
                self.clim[row * SLOTS + d] = mean.round() as i16;
                self.thr[row * SLOTS + d] = samples[rank.min(samples.len() - 1)];
            }
            self.clim[row * SLOTS + 365] = self.clim[row * SLOTS + 364];
            self.thr[row * SLOTS + 365] = self.thr[row * SLOTS + 364];
        }
    }

    fn encode(&self) -> Vec<u8> {
        let n = self.cell_ids.len();
        let mut out = Vec::with_capacity(16 + n * 4 + (self.series.len() + self.clim.len() + self.thr.len()) * 2);
        out.extend_from_slice(b"HWV1");
        out.extend_from_slice(&(n as u32).to_le_bytes());
        out.extend_from_slice(&(self.years as u32).to_le_bytes());
        for id in &self.cell_ids { out.extend_from_slice(&id.to_le_bytes()); }
        for v in self.series.iter().chain(self.clim.iter()).chain(self.thr.iter()) { out.extend_from_slice(&v.to_le_bytes()); }
        out
    }
    fn decode(&mut self, b: &[u8]) -> bool {
        if b.len() < 12 || &b[..4] != b"HWV1" { return false; }
        let n = u32::from_le_bytes([b[4], b[5], b[6], b[7]]) as usize;
        let years = u32::from_le_bytes([b[8], b[9], b[10], b[11]]) as usize;
        if years != self.years || b.len() != 12 + n * 4 + n * (years + 2) * SLOTS * 2 { return false; }
        let mut p = 12;
        self.cell_ids = (0..n).map(|_| { let v = u32::from_le_bytes([b[p], b[p + 1], b[p + 2], b[p + 3]]); p += 4; v }).collect();
        let mut take = |count: usize| -> Vec<i16> { let v: Vec<i16> = b[p..p + count * 2].chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect(); p += count * 2; v };
        self.series = take(n * years * SLOTS);
        self.clim = take(n * SLOTS);
        self.thr = take(n * SLOTS);
        true
    }

    /// The work grid's country per cell (the most populous sub-cell's) and people per cell.
    fn coarse_index(&self, index: &[f32], count: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let g = countries::world_grid(self.res_deg);
        let gw = countries::world_grid(self.work_res_deg);
        let fac = self.fac();
        let mut idx = vec![f32::NAN; gw.nx * gw.ny];
        let mut people = vec![0.0f32; gw.nx * gw.ny];
        for j in 0..gw.nx * gw.ny {
            let (cw, rw) = gw.col_row(j);
            let mut best = (-1.0f32, f32::NAN);
            for dr in 0..fac { for dc in 0..fac {
                let (c, r) = (cw * fac + dc, rw * fac + dr);
                if c >= g.nx || r >= g.ny { continue; }
                let i = g.index(c, r);
                let p = count.get(i).copied().unwrap_or(0.0).max(0.0);
                people[j] += p;
                if !index[i].is_nan() && p > best.0 { best = (p, index[i]); }
            } }
            idx[j] = best.1;
        }
        (idx, people)
    }

    /// Each country's people-weighted mean Tmax climatology and threshold per calendar day,
    /// from the decade unshifted, so a country-level heatwave is judged against the country's
    /// own distribution (the paper's regional definition). Needs the index, so computed on
    /// the first event and again after a restore; deterministic either way.
    fn country_tables(&mut self, idx: &[f32], people: &[f32]) {
        let nk = self.countries.len();
        let mut acc = vec![0.0f64; nk * self.years * SLOTS];
        let mut wsum = vec![0.0f64; nk * self.years * SLOTS];
        for (row, &id) in self.cell_ids.iter().enumerate() {
            let k = idx[id as usize]; if k.is_nan() { continue }
            let (k, p) = (k as usize, people[id as usize] as f64);
            if p <= 0.0 { continue }
            for yi in 0..self.years { for d in 0..SLOTS {
                let v = self.series[(row * self.years + yi) * SLOTS + d];
                if v == MISSING { continue }
                acc[(k * self.years + yi) * SLOTS + d] += v as f64 / 10.0 * p;
                wsum[(k * self.years + yi) * SLOTS + d] += p;
            } }
        }
        self.country_clim = vec![f32::NAN; nk * SLOTS];
        self.country_thr = vec![f32::NAN; nk * SLOTS];
        let w = self.window_days as i64;
        let mut samples: Vec<f64> = Vec::new();
        for k in 0..nk {
            for d in 0..365usize {
                samples.clear();
                for yi in 0..self.years { for s in -w..=w {
                    let dd = ((d as i64 + s).rem_euclid(365)) as usize;
                    let ws = wsum[(k * self.years + yi) * SLOTS + dd];
                    if ws > 0.0 { samples.push(acc[(k * self.years + yi) * SLOTS + dd] / ws); }
                } }
                if samples.len() < 10 { continue; }
                let mean = samples.iter().sum::<f64>() / samples.len() as f64;
                samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let rank = ((self.threshold_pct / 100.0) * (samples.len() as f64 - 1.0)).round() as usize;
                self.country_clim[k * SLOTS + d] = mean as f32;
                self.country_thr[k * SLOTS + d] = samples[rank.min(samples.len() - 1)] as f32;
            }
            self.country_clim[k * SLOTS + 365] = self.country_clim[k * SLOTS + 364];
            self.country_thr[k * SLOTS + 365] = self.country_thr[k * SLOTS + 364];
        }
        self.country_tables_done = true;
    }

    pub fn describe(name: &str, p: &toml::Value) -> Describe {
        let scale = param_str(p, "scale", "110m").to_string();
        let (yf, yt) = (param_f64(p, "year_from", 2011.0) as i64, param_f64(p, "year_to", 2020.0) as i64);
        let mut d = Describe::new("weather.heatwaves", name,
            "A weather generator for the planet: each year one observed year of daily maximum temperature per populated one-degree cell (MERRA-2 through NASA POWER, a baseline decade) is drawn by the run's seed and replayed, shifted by the cell's warming since that decade, and heatwaves are counted as Aglas-Leitner et al. 2026 count them: three or more days over the calendar-day 90th percentile, severity in units of the threshold's excess over the climatology, cumulative heat, duration, and the share of a country's people under extreme heat on the peak day. A country's heatwave is judged on its people-weighted mean. Counts and events, not deaths.")
            .priority(5).dt("follows climate.country_tas")
            .param("res_deg", param_f64(p, "res_deg", 0.5), "deg", "the shared grid; must match people.countries")
            .param("work_res_deg", param_f64(p, "work_res_deg", 1.0), "deg", "the daily series' grid, coarser than the shared one to keep a decade of days in memory (about 125 MB at one degree)")
            .param("frame_res_deg", param_f64(p, "frame_res_deg", 1.0), "deg", "frames folded to this grid").param("frame_every_years", param_f64(p, "frame_every_years", 5.0), "yr", "")
            .param("scale", scale.clone(), "", "must match people.countries")
            .param("year_from", yf, "yr", "the observed decade replayed and used as the base; POWER begins in 1981, the paper's base is 1961 to 1990, so exceptional here means exceptional for this decade").param("year_to", yt, "yr", "")
            .param("pattern", param_str(p, "pattern", "country"), "", "country: every cell warms by its country's fitted share of the global anomaly; cells: by its own reanalysis slope")
            .param("pattern_from", param_f64(p, "pattern_from", 1981.0), "yr", "cells only").param("pattern_to", param_f64(p, "pattern_to", 2025.0), "yr", "")
            .param("threshold_pct", param_f64(p, "threshold_pct", 90.0), "%", "the calendar-day percentile a heatwave day exceeds (Tx90p in the paper; it also reports 95, 97.5 and 99)")
            .param("window_days", param_f64(p, "window_days", 15.0), "d", "days either side of the calendar day pooled for the climatology and the threshold (the paper's first window; its second 31-day smoothing is not applied)")
            .param("min_days", param_f64(p, "min_days", 3.0), "d", "consecutive days over the threshold that make a heatwave (Perkins and Alexander 2013, as the paper applies it)")
            .param("spread_per_degree", param_f64(p, "spread_per_degree", 0.0), "1/degC", "a guess with no paper behind it: how much each day's anomaly from the climatology widens per degree of local warming; 0 is the pure delta method, which the paper's results say understates future length and area")
            .param("notable_days", param_f64(p, "notable_days", 14.0), "d", "a country's heatwave at least this long is published as an event (the longest per country per year)")
            .param("month_days", param_f64(p, "month_days", 30.0), "d", "the first year a country has a heatwave this long is a crossing").param("season_days", param_f64(p, "season_days", 60.0), "d", "and this long a second one")
            .param("area_share", param_f64(p, "area_share", 0.5), "", "the share of a country's people under extreme heat on a day for the country to count as covered")
            .subscribe("climate.country_tas", "{ year, gmst_c, by_iso3: { slope } }")
            .read("country.index", "index", true, "which cell is which country").read("people.count", "persons", true, "people per cell")
            .publish("heat.wave", "{ iso3, name, year, start_doy, days, max_severity, cumulative_c_days, area, persons, replayed_year, sentence } the longest notable heatwave per country each year")
            .publish("heat.crossing", "{ iso3, name, year, kind: heatwave_month | heatwave_season, days, area, persons, sentence, source } the first year a country's heatwave passes a month, and a season")
            .publish("heat.pressure", "{ year, by_iso3: { days, longest, covered } } each year: people-weighted heatwave days, the longest run, and whether a month-long heatwave covered the stated share")
            .report("heat.people_days", "person-days/yr").report("heat.people_month_long", "persons").report("heat.longest_days", "d").report("heat.countries_month_long", "").report("heat.events", "").report("heat.max_severity", "1").report("heat.replayed_year", "yr")
            .frame("heatwave_days", "d/yr").frame("heatwave_longest_days", "d").frame("heatwave_max_severity", "1")
            .view("counter", "person-days of heatwave a year", "heat.people_days", "people times days over the 90th percentile in runs of three or more")
            .view("counter", "people in a month-long heatwave", "heat.people_month_long", "living in a cell with a run of 30 days or more this year")
            .view("counter", "the longest heatwave anywhere", "heat.longest_days", "days, in the worst cell")
            .view("map", "heatwave days this year", "heatwave_days", "days over the calendar-day 90th percentile of 2011 to 2020, in runs of three or more")
            .view("map", "the longest heatwave this year", "heatwave_longest_days", "days")
            .view("map", "the worst day's severity", "heatwave_max_severity", "2 means the day's excess over the climatology was twice the threshold's")
            .view("crossings", "heatwaves", "heat.wave", "the longest notable heatwave per country each year, with its severity and the share of people under it")
            .view("chart", "person-days of heatwave", "heat.people_days", "")
            .view("chart", "the longest heatwave anywhere", "heat.longest_days", "")
            .asset("worldpop-2020-1km", "which cells have people").asset("nasa-power-daily-regional", "daily maximum temperature per cell for the baseline decade").asset("hadcrut5-global-annual", "the global anomaly of the baseline decade, which the run's anomaly is shifted from").asset(countries::scale_asset_name(&scale), "names for the events")
            .note("Imperfect on purpose. The weather of one observed year is replayed whole, shifted by the warming: days within a year are coherent, years are independent, and the spread of the days stays as it was unless spread_per_degree says otherwise. Aglas-Leitner et al. 2026 (Nature Communications 17:11456, read 2026-11-07) find that projected heatwaves grow longer and wider faster than the mean warming alone would make them, so with the default this layer is a floor. The base is the replayed decade, which was already about a degree warmer than 1850 to 1900, so a heatwave here is exceptional against 2011 to 2020, not against the paper's 1961 to 1990.")
            .note("MERRA-2 is a reanalysis at about 50 km and reads extremes conservatively; a country's heatwave is judged on the people-weighted mean of its cells, so a large country with its people in one corner is that corner.");
        if param_str(p, "pattern", "country") == "cells" { d = d.asset("nasa-power-monthly-regional", "the annual means the cells' slopes are fitted on"); }
        d
    }

    fn year(&mut self, ev: &Event, ctx: &mut Ctx) {
        let y = ev.data.get("year").and_then(|v| v.as_i64()).unwrap_or(countries::year_of(ctx.now()));
        let Some(by) = ev.data.get("by_iso3").and_then(|v| v.as_object()) else { return };
        let Some(gmst) = ev.data.get("gmst_c").and_then(|v| v.as_f64()) else { return };
        let g = countries::world_grid(self.res_deg);
        let Some(index) = ctx.world_read_on("country.index", &g).map(|f| f.data.clone()) else { return };
        let Some(count) = ctx.world_read_on("people.count", &g).map(|f| f.data.clone()) else { return };
        let (idx, people) = self.coarse_index(&index, &count);
        if !self.country_tables_done { self.country_tables(&idx, &people); }
        let gw = countries::world_grid(self.work_res_deg);
        let nk = self.countries.len();
        let slope_k: Vec<f64> = self.countries.iter().map(|c| by.get(&c.iso3).and_then(|e| e.get("slope")).and_then(|v| v.as_f64()).unwrap_or(f64::NAN)).collect();
        let ga = gmst - self.g_obs;
        let yi = ((ctx.rng().f64() * self.years as f64) as usize).min(self.years - 1);
        let on_pattern = self.pattern == "cells";
        // per cell, this year's weather
        let n = self.cell_ids.len();
        let mut per: Vec<CellYear> = vec![CellYear::default(); n];
        let mut mean_acc = vec![0.0f64; nk * SLOTS];
        let mut mean_w = vec![0.0f64; nk * SLOTS];
        let mut over_p = vec![0.0f64; nk * SLOTS];
        let mut tot_c = vec![0.0f64; nk];
        for j in 0..gw.nx * gw.ny { let k = idx[j]; if !k.is_nan() { tot_c[k as usize] += people[j] as f64; } }
        for (row, &id) in self.cell_ids.iter().enumerate() {
            let i = id as usize;
            let k = idx[i]; if k.is_nan() { continue }
            let k = k as usize;
            let shift = countries::cell_warming(on_pattern, &self.slope, i, Some(ga), slope_k[k] * ga);
            if shift.is_nan() { continue }
            let p = people[i] as f64;
            let widen = 1.0 + self.spread_per_degree * shift;
            let (mut run, mut run_sev, mut run_cum) = (0usize, 0.0f32, 0.0f32);
            let mut cy = CellYear::default();
            let commit = |cy: &mut CellYear, run: usize, sev: f32, cum: f32| { if run >= self.min_days { cy.days += run as u16; cy.longest = cy.longest.max(run as u16); cy.max_sev = cy.max_sev.max(sev); cy.cum += cum; } };
            for d in 0..SLOTS {
                let v = self.series[(row * self.years + yi) * SLOTS + d];
                let (c, t) = (self.clim[row * SLOTS + d], self.thr[row * SLOTS + d]);
                if v == MISSING || c == MISSING || t == MISSING { commit(&mut cy, run, run_sev, run_cum); run = 0; run_sev = 0.0; run_cum = 0.0; continue; }
                let (c, t) = (c as f64 / 10.0, t as f64 / 10.0);
                let x = c + (v as f64 / 10.0 - c) * widen + shift;
                if p > 0.0 { mean_acc[k * SLOTS + d] += x * p; mean_w[k * SLOTS + d] += p; }
                if x > t {
                    if p > 0.0 { over_p[k * SLOTS + d] += p; }
                    run += 1;
                    let sev = if t > c { ((x - c) / (t - c)) as f32 } else { 1.0 };
                    run_sev = run_sev.max(sev); run_cum += (x - t) as f32;
                } else { commit(&mut cy, run, run_sev, run_cum); run = 0; run_sev = 0.0; run_cum = 0.0; }
            }
            commit(&mut cy, run, run_sev, run_cum);
            per[row] = cy;
        }
        // per country: the people-weighted mean's own heatwaves
        let mut events = Vec::new();
        let mut crossings = Vec::new();
        let mut pressure = serde_json::Map::new();
        let (mut n_month, mut max_sev) = (0usize, 0.0f64);
        let mut days_c = vec![0.0f64; nk];
        let mut longest_c = vec![0u16; nk];
        for (row, &id) in self.cell_ids.iter().enumerate() {
            let i = id as usize; let k = idx[i]; if k.is_nan() { continue }
            let k = k as usize; let p = people[i] as f64;
            days_c[k] += per[row].days as f64 * p; longest_c[k] = longest_c[k].max(per[row].longest);
        }
        for (k, c) in self.countries.iter().enumerate() {
            if tot_c[k] <= 0.0 { continue }
            let mut best: Option<(usize, usize, f64, f64, f64)> = None; // start, len, max sev, cum, area at peak
            let (mut run, mut start, mut sev, mut cum, mut area) = (0usize, 0usize, 0.0f64, 0.0f64, 0.0f64);
            let close = |run: usize, start: usize, sev: f64, cum: f64, area: f64, best: &mut Option<(usize, usize, f64, f64, f64)>| {
                if run >= self.min_days && best.map(|b| run > b.1).unwrap_or(true) { *best = Some((start, run, sev, cum, area)); }
            };
            for d in 0..SLOTS {
                let (w, ct, cc) = (mean_w[k * SLOTS + d], self.country_thr[k * SLOTS + d], self.country_clim[k * SLOTS + d]);
                if w <= 0.0 || ct.is_nan() || cc.is_nan() { close(run, start, sev, cum, area, &mut best); run = 0; continue; }
                let x = mean_acc[k * SLOTS + d] / w;
                if x > ct as f64 {
                    if run == 0 { start = d; sev = 0.0; cum = 0.0; area = 0.0; }
                    run += 1;
                    let s = if ct > cc { (x - cc as f64) / (ct - cc) as f64 } else { 1.0 };
                    sev = sev.max(s); cum += x - ct as f64; area = area.max(over_p[k * SLOTS + d] / tot_c[k]);
                } else { close(run, start, sev, cum, area, &mut best); run = 0; }
            }
            close(run, start, sev, cum, area, &mut best);
            let covered = best.map(|b| b.1 >= self.month_days && b.4 >= self.area_share).unwrap_or(false);
            if let Some((start, len, s, cm, ar)) = best {
                max_sev = max_sev.max(s);
                if len >= self.month_days { n_month += 1; }
                let persons = ar * tot_c[k];
                if len >= self.notable_days && self.baseline_done {
                    let sentence = format!("{}: {}, a heatwave of {} days from day {} of the year, the country's mean daily maximum over its 90th percentile of {} to {}; at its peak {:.0}% of the people ({}) were under extreme heat; severity {:.1}, cumulative {:.0} degree-days.", y, c.name, len, start + 1, self.year_from, self.year_to, 100.0 * ar, countries::people_words(persons), s, cm);
                    events.push(json!({ "iso3": c.iso3, "name": c.name, "year": y, "start_doy": start + 1, "days": len, "max_severity": s, "cumulative_c_days": cm, "area": ar, "persons": persons, "replayed_year": self.year_from + yi as i64, "sentence": sentence }));
                }
                for (kind, need, label, crossed) in [("heatwave_month", self.month_days, "a month", &mut self.crossed_month), ("heatwave_season", self.season_days, "a season", &mut self.crossed_season)] {
                    if len >= need && ar >= self.area_share && !crossed.contains_key(&c.iso3) {
                        crossed.insert(c.iso3.clone(), if self.baseline_done { y } else { 0 });
                        if self.baseline_done {
                            let sentence = format!("{}: in {}, the first heatwave of {} ({} days) over more than half the people ({}), the country's mean daily maximum past its 90th percentile of {} to {} for that long; it was once a {}-day summer's exception.", y, c.name, label, len, countries::people_words(persons), self.year_from, self.year_to, self.min_days);
                            crossings.push(json!({ "iso3": c.iso3, "name": c.name, "year": y, "kind": kind, "days": len, "area": ar, "persons": persons, "sentence": sentence, "source": "MERRA-2 via NASA POWER, one observed year replayed with the warming; the heatwave definition of Aglas-Leitner et al. 2026 (read 2026-11-07)" }));
                        }
                    }
                }
            }
            pressure.insert(c.iso3.clone(), json!({ "days": days_c[k] / tot_c[k], "longest": longest_c[k], "covered": covered }));
        }
        self.baseline_done = true;
        // frames and reports
        let (mut people_days, mut people_month, mut longest) = (0.0f64, 0.0f64, 0u16);
        let mut f_days = vec![f32::NAN; gw.nx * gw.ny]; let mut f_long = vec![f32::NAN; gw.nx * gw.ny]; let mut f_sev = vec![f32::NAN; gw.nx * gw.ny];
        for (row, &id) in self.cell_ids.iter().enumerate() {
            let i = id as usize; let cy = per[row]; let p = people[i] as f64;
            if idx[i].is_nan() { continue }
            people_days += cy.days as f64 * p; longest = longest.max(cy.longest);
            if cy.longest as usize >= self.month_days { people_month += p; }
            f_days[i] = cy.days as f32; f_long[i] = cy.longest as f32; f_sev[i] = cy.max_sev;
        }
        if y - self.last_frame_year >= self.frame_every_years as i64 {
            let fac = countries::frame_factor(self.work_res_deg, self.frame_res_deg);
            let (cg, a) = countries::coarsen(&gw, &f_days, fac, Fold::Max); let (_, b) = countries::coarsen(&gw, &f_long, fac, Fold::Max); let (_, s) = countries::coarsen(&gw, &f_sev, fac, Fold::Max);
            ctx.field("heatwave_days", "d/yr", &cg, a); ctx.field("heatwave_longest_days", "d", &cg, b); ctx.field("heatwave_max_severity", "1", &cg, s);
            self.last_frame_year = y;
        }
        let n_events = events.len();
        for e in events { ctx.publish("heat.wave", e); }
        for c in crossings { ctx.publish("heat.crossing", c); }
        ctx.publish("heat.pressure", json!({ "year": y, "by_iso3": pressure }));
        ctx.report("heat.people_days", "person-days/yr", people_days);
        ctx.report("heat.people_month_long", "persons", people_month);
        ctx.report("heat.longest_days", "d", longest as f64);
        ctx.report("heat.countries_month_long", "", n_month as f64);
        ctx.report("heat.events", "", n_events as f64);
        ctx.report("heat.max_severity", "1", max_sev);
        ctx.report("heat.replayed_year", "yr", (self.year_from + yi as i64) as f64);
    }
}

impl Module for Heatwaves {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) { ctx.subscribe("climate.country_tas"); }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) { if ev.topic == "climate.country_tas" { self.year(ev, ctx); } }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut m: Heatwaves = serde_json::from_value(v).unwrap(); m.load().expect("heatwaves reload"); *self = m; }
}
