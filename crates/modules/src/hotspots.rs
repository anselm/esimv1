//! Satellite hotspots as the record of a fire's progress, for fires nobody mapped hour by hour.
//! Reads a CSV of detections (CWFIS's Fire M3 archive: rep_date, sensor, lat, lon, …; the NASA
//! FIRMS archive has the same shape), keeps those inside the window and the run's days, and at a
//! fixed cadence marks every cell within each detection's footprint as seen burning by then, then
//! scores the fire module's burned cells against that: acres seen, overlap (Sørensen), missed and
//! overrun. A hotspot is a pixel that was hot when a satellite passed, not a perimeter: cloud,
//! smoke and the gaps between passes make the seen area a floor on the burned area, and a pixel
//! marks ground it only partly covers. The frame `obs_hotspots` is the seen-by-then share per block.
use esim_kernel::manifest::{param_duration, param_f64, param_str};
use esim_kernel::{Ctx, Duration, Event, LatLonGrid, Module, Time};
use esim_registry::{dem_window, Catalog, Fetcher};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

const ACRE_M2: f64 = 4046.856;

#[derive(Clone, Serialize, Deserialize)]
struct Spot { t: i64, lat: f64, lon: f64, sensor: String, radius_m: f64 }

#[derive(Serialize, Deserialize)]
pub struct Hotspots {
    name: String,
    lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64,
    file: String,
    start: String, end: String,
    every: Duration,
    frame_res_arcsec: f64,
    time_field: String, lat_field: String, lon_field: String, sensor_field: String,
    default_radius_m: f64,
    done: usize,
    scheduled: bool,
    #[serde(skip)] grid: Option<LatLonGrid>,
    #[serde(skip)] spots: Vec<Spot>,
    #[serde(skip)] seen: Vec<bool>,
    #[serde(skip)] area_m2: Vec<f64>,
    #[serde(skip)] metres: Vec<(f64, f64)>,
}

/// The ground a detection stands for, by sensor: half the nominal pixel. VIIRS I-band 375 m,
/// VIIRS M-band 750 m, MODIS and AVHRR about 1 km at nadir, Landsat OLI 30 m. Recalled; nadir values,
/// so the swath edge is under-drawn.
fn radius_for(sensor: &str, default: f64) -> f64 {
    match sensor { "VIIRS-I" => 187.0, "VIIRS-M" => 375.0, "MODIS" | "AVHRR" => 500.0, "OLI" => 15.0, _ => default }
}

impl Hotspots {
    pub fn new(name: &str, p: &toml::Value) -> Result<Hotspots, String> {
        let mut o = Hotspots {
            name: name.to_string(),
            lat_s: param_f64(p, "lat_s", 56.55), lat_n: param_f64(p, "lat_n", 56.85), lon_w: param_f64(p, "lon_w", -111.75), lon_e: param_f64(p, "lon_e", -111.15),
            res_arcsec: param_f64(p, "res_arcsec", 1.0),
            file: param_str(p, "file", "fortmcmurray-2016/cwfis/hotspots.csv").to_string(),
            start: param_str(p, "start", "2016-05-01T00:00:00Z").to_string(), end: param_str(p, "end", "2016-05-07T00:00:00Z").to_string(),
            every: param_duration(p, "every", "6h")?,
            frame_res_arcsec: param_f64(p, "frame_res_arcsec", 4.0),
            time_field: param_str(p, "time_field", "rep_date").to_string(), lat_field: param_str(p, "lat_field", "lat").to_string(), lon_field: param_str(p, "lon_field", "lon").to_string(), sensor_field: param_str(p, "sensor_field", "sensor").to_string(),
            default_radius_m: param_f64(p, "default_radius_m", 500.0),
            done: 0, scheduled: false,
            grid: None, spots: Vec::new(), seen: Vec::new(), area_m2: Vec::new(), metres: Vec::new(),
        };
        o.load()?;
        Ok(o)
    }
    fn load(&mut self) -> Result<(), String> {
        let root = std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let cat = Catalog::load(&root.join("data/catalog.toml"))?;
        let dem = cat.asset("copernicus-dem-glo30").ok_or("catalog has no copernicus-dem-glo30")?;
        let mut fe = Fetcher::default_for(&root)?;
        let w = dem_window(&mut fe, dem, self.lat_s, self.lat_n, self.lon_w, self.lon_e, self.res_arcsec)?;
        let grid = LatLonGrid { nx: w.nx, ny: w.ny, dlon: w.dlon, dlat: w.dlat, lon_w: w.lon_w, lat_n: w.lat_n };
        self.area_m2 = (0..grid.ny).flat_map(|r| std::iter::repeat_n(grid.row_area(r), grid.nx)).collect();
        self.metres = (0..grid.ny).map(|r| { let (mx, my) = grid.metres_per_degree(r); (mx * grid.dlon, my * grid.dlat) }).collect();
        self.seen = vec![false; grid.nx * grid.ny];
        let t0 = Time::parse(&self.start)?.0; let t1 = Time::parse(&self.end)?.0;
        let path = root.join("data/events").join(&self.file);
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut lines = text.lines();
        let header: Vec<&str> = lines.next().ok_or("hotspots: empty file")?.split(',').map(|s| s.trim()).collect();
        let col = |n: &str| header.iter().position(|h| *h == n);
        let (it, ila, ilo) = (col(&self.time_field).ok_or("hotspots: no time column")?, col(&self.lat_field).ok_or("hotspots: no lat column")?, col(&self.lon_field).ok_or("hotspots: no lon column")?);
        let isen = col(&self.sensor_field);
        let mut spots = Vec::new();
        for line in lines {
            let f: Vec<&str> = line.split(',').collect();
            let (Some(ts), Some(la), Some(lo)) = (f.get(it), f.get(ila), f.get(ilo)) else { continue };
            let (Ok(lat), Ok(lon)) = (la.parse::<f64>(), lo.parse::<f64>()) else { continue };
            if lat < self.lat_s || lat > self.lat_n || lon < self.lon_w || lon > self.lon_e { continue; }
            // "2016-05-02 00:23:00", taken as UTC (the archive's metadata does not state a zone)
            let iso = if ts.len() >= 19 && ts.as_bytes()[10] == b' ' { format!("{}T{}Z", &ts[..10], &ts[11..19]) } else { ts.to_string() };
            let Ok(t) = Time::parse(&iso) else { continue };
            if t.0 < t0 || t.0 > t1 { continue; }
            let sensor = isen.and_then(|i| f.get(i)).map(|s| s.trim().to_string()).unwrap_or_default();
            spots.push(Spot { t: t.0, lat, lon, radius_m: radius_for(&sensor, self.default_radius_m), sensor });
        }
        spots.sort_by_key(|s| s.t);
        self.spots = spots;
        self.grid = Some(grid);
        let done = self.done; self.done = 0;
        self.mark_until_count(done);
        Ok(())
    }
    /// Mark the cells within each of the next spots' radii as seen, up to `n` spots in all.
    fn mark_until_count(&mut self, n: usize) {
        let Some(g) = self.grid.clone() else { return };
        while self.done < n.min(self.spots.len()) {
            let s = self.spots[self.done].clone();
            let c0 = ((s.lon - g.lon_w) / g.dlon).floor() as isize; let r0 = ((g.lat_n - s.lat) / g.dlat).floor() as isize;
            let row = (r0.max(0) as usize).min(g.ny - 1);
            let (mx, my) = self.metres[row];
            let (rc, rr) = ((s.radius_m / mx).ceil() as isize, (s.radius_m / my).ceil() as isize);
            for dr in -rr..=rr { for dc in -rc..=rc {
                let (c, r) = (c0 + dc, r0 + dr);
                if c < 0 || r < 0 || c >= g.nx as isize || r >= g.ny as isize { continue; }
                if ((dc as f64 * mx).powi(2) + (dr as f64 * my).powi(2)).sqrt() <= s.radius_m.max(mx * 0.5) { self.seen[r as usize * g.nx + c as usize] = true; }
            } }
            self.done += 1;
        }
    }
}

impl Module for Hotspots {
    fn name(&self) -> &str { &self.name }
    fn init(&mut self, ctx: &mut Ctx) {
        let mut by: std::collections::BTreeMap<String, usize> = Default::default();
        for s in &self.spots { *by.entry(s.sensor.clone()).or_default() += 1; }
        ctx.publish("hotspots.loaded", json!({ "file": self.file, "count": self.spots.len(), "first": self.spots.first().map(|s| Time(s.t).to_iso()), "last": self.spots.last().map(|s| Time(s.t).to_iso()), "by_sensor": by, "time_zone": "rep_date taken as UTC" }));
        if !self.scheduled {
            // a tick every `every` from the first detection to the end
            let now = ctx.now();
            let t_end = Time::parse(&self.end).map(|t| t.0).unwrap_or(now.0);
            let mut t = self.spots.first().map(|s| s.t).unwrap_or(now.0).max(now.0);
            t = t - t.rem_euclid(self.every.0.max(1)) + self.every.0;
            let mut n = 0;
            while t <= t_end && n < 10_000 { ctx.schedule_with_priority(Duration(t - now.0), 9, "hotspots", json!(null)); t += self.every.0; n += 1; }
            self.scheduled = true;
        }
    }
    fn handle(&mut self, ev: &Event, ctx: &mut Ctx) {
        if ev.topic != "hotspots" { return; }
        let now = ctx.now();
        let g = self.grid.clone().unwrap();
        let n = self.spots.iter().take_while(|s| s.t <= now.0).count();
        self.mark_until_count(n);
        let obs_m2: f64 = self.seen.iter().zip(&self.area_m2).filter(|(m, _)| **m).map(|(_, a)| a).sum();
        let k = (self.frame_res_arcsec / self.res_arcsec).round().max(1.0) as usize;
        let fg = LatLonGrid { nx: g.nx.div_ceil(k), ny: g.ny.div_ceil(k), dlon: g.dlon * k as f64, dlat: g.dlat * k as f64, lon_w: g.lon_w, lat_n: g.lat_n };
        let mut frame = vec![0f32; fg.nx * fg.ny];
        for fr in 0..fg.ny { for fc in 0..fg.nx {
            let (mut cnt, mut inside) = (0usize, 0usize);
            for r in fr * k..((fr + 1) * k).min(g.ny) { for c in fc * k..((fc + 1) * k).min(g.nx) { cnt += 1; if self.seen[r * g.nx + c] { inside += 1; } } }
            frame[fr * fg.nx + fc] = if cnt > 0 { inside as f32 / cnt as f32 } else { 0.0 };
        } }
        ctx.field("obs_hotspots", "1", &fg, frame);
        ctx.report("obs.acres", "acres", obs_m2 / ACRE_M2);
        ctx.report("obs.hotspots_so_far", "detections", n as f64);
        if let Some(f) = ctx.world_read("fire.burned_frac") {
            if f.data.len() == self.seen.len() {
                let (mut both, mut sim, mut missed, mut over) = (0.0, 0.0, 0.0, 0.0);
                for i in 0..self.seen.len() {
                    let s = f.data[i] >= 0.5;
                    let a = self.area_m2[i];
                    if s { sim += a; }
                    match (s, self.seen[i]) { (true, true) => both += a, (false, true) => missed += a, (true, false) => over += a, _ => {} }
                }
                let sorensen = if sim + obs_m2 > 0.0 { 2.0 * both / (sim + obs_m2) } else { 0.0 };
                ctx.report("sim.acres", "acres", sim / ACRE_M2);
                ctx.report("fit.sorensen", "1", sorensen);
                ctx.report("fit.missed_acres", "acres", missed / ACRE_M2);
                ctx.report("fit.over_acres", "acres", over / ACRE_M2);
                ctx.publish("hotspots.scored", json!({ "time": now.to_iso(), "obs_acres": obs_m2 / ACRE_M2, "sim_acres": sim / ACRE_M2, "sorensen": sorensen, "detections": n }));
            }
        }
    }
    fn snapshot(&self) -> serde_json::Value { serde_json::to_value(self).unwrap() }
    fn restore(&mut self, v: serde_json::Value) { let mut o: Hotspots = serde_json::from_value(v).unwrap(); o.load().expect("hotspots reload"); *self = o; }
}

impl Hotspots {
    pub fn describe(name: &str, p: &toml::Value) -> esim_kernel::describe::Describe {
        esim_kernel::describe::Describe::new("obs.hotspots", name, "Satellite hotspots as the record of a fire's progress, for fires nobody mapped hour by hour: detections from a CSV (CWFIS's Fire M3 archive; NASA FIRMS has the same shape) inside the window and the run's days, each marking the ground within its sensor's half-pixel as seen burning by then; at a fixed cadence the fire module's burned cells are scored against the seen area (acres, Sørensen overlap, missed and overrun). A hotspot is a pixel that was hot when a satellite passed, not a perimeter: cloud, smoke and the gaps between passes make the seen area a floor, and a pixel marks ground it only partly covers.")
            .priority(9).dt(&format!("every {}", param_str(p, "every", "6h")))
            .param("lat_s", 56.55, "deg", "").param("lat_n", 56.85, "deg", "").param("lon_w", -111.75, "deg", "").param("lon_e", -111.15, "deg", "").param("res_arcsec", 1.0, "arcsec", "")
            .param("file", "fortmcmurray-2016/cwfis/hotspots.csv", "", "CSV under data/events/").param("start", "2016-05-01T00:00:00Z", "", "").param("end", "2016-05-07T00:00:00Z", "", "")
            .param("every", "6h", "", "the scoring cadence").param("frame_res_arcsec", 4.0, "arcsec", "the seen-by-then frames are the share of each block seen, on this grid")
            .param("time_field", "rep_date", "", "'YYYY-MM-DD HH:MM:SS', taken as UTC (the archive's metadata states no zone), or ISO").param("lat_field", "lat", "", "").param("lon_field", "lon", "", "").param("sensor_field", "sensor", "", "")
            .param("default_radius_m", 500.0, "m", "half-pixel for a sensor not in the table (VIIRS-I 187, VIIRS-M 375, MODIS and AVHRR 500, OLI 15; recalled nadir values)")
            .publish("hotspots.loaded", "{ file, count, first, last, by_sensor, time_zone }").publish("hotspots.scored", "{ time, obs_acres, sim_acres, sorensen, detections }")
            .read("fire.burned_frac", "1", false, "the fire module's burned cells")
            .frame("obs_hotspots", "1")
            .report("obs.acres", "acres").report("obs.hotspots_so_far", "detections").report("sim.acres", "acres").report("fit.sorensen", "1").report("fit.missed_acres", "acres").report("fit.over_acres", "acres")
            .asset("copernicus-dem-glo30", "the grid")
            .view("chart", "Seen burning from orbit", "obs.acres", "the ground within a hotspot's footprint by this time; a floor on the burned area").view("chart", "Overlap with the hotspots", "fit.sorensen", "Sørensen of the model's burned cells against the seen area")
            .note("Fire M3 hotspots: Natural Resources Canada, Canadian Forest Service; Open Government Licence - Canada, with an end-user agreement on the download page. Copied to data/events/<event>/cwfis/ for the window and the days.")
    }
}
