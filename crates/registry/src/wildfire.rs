//! Data for a wildfire hindcast: land cover as fuel classes (ESA WorldCover 2021, 10 m,
//! read tile by tile into a window), hourly wind with direction and gusts at a point
//! (Open-Meteo's ERA5 archive), and the perimeters an incident's records hold (NIFC's
//! WFIGS final perimeter and the Operational Data Archive's daily perimeters), with a
//! rasteriser so a run's burned area can be scored against them. All anonymous.

use crate::catalog::Asset;
use crate::cog;
use crate::fetch::Fetcher;
use crate::obs::{fetch_json_text, parse_iso_hour};
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

/// WorldCover tile key for a point: tiles are 3 by 3 degrees named by their south-west
/// corner, "N33W120" for 33 to 36 N, 120 to 117 W.
pub fn worldcover_tile_key(lat: f64, lon: f64) -> String {
    let la = ((lat / 3.0).floor() * 3.0) as i32;
    let lo = ((lon / 3.0).floor() * 3.0) as i32;
    format!("{}{:02}{}{:03}", if la < 0 { 'S' } else { 'N' }, la.abs(), if lo < 0 { 'W' } else { 'E' }, lo.abs())
}

/// Land-cover class per cell of a lat-lon window (rows north to south): the most common
/// 10 m class inside each cell, 0 where none. Cached under data/cache/derived.
pub fn landcover_window(fetcher: &mut Fetcher, asset: &Asset, nx: usize, ny: usize, dlon: f64, dlat: f64, lon_w: f64, lat_n: f64) -> Result<Vec<u8>, String> {
    let lat_s = lat_n - ny as f64 * dlat;
    let lon_e = lon_w + nx as f64 * dlon;
    // the tiles under the window's corners (a window never spans more than four)
    let mut keys: Vec<String> = Vec::new();
    for (la, lo) in [(lat_s, lon_w), (lat_s, lon_e - 1e-9), (lat_n - 1e-9, lon_w), (lat_n - 1e-9, lon_e - 1e-9)] {
        let k = worldcover_tile_key(la, lo);
        if !keys.contains(&k) { keys.push(k); }
    }
    let mut paths = Vec::new();
    for k in &keys {
        let url = asset.url_template.replace("{tile}", k);
        paths.push((k.clone(), fetcher.fetch(&format!("{}/{}.tif", asset.id, k), &url)?));
    }
    let mut h = Sha256::new();
    for (k, _) in &paths { h.update(k.as_bytes()); h.update(fetcher.lock.files[&format!("{}/{}.tif", asset.id, k)].sha256.as_bytes()); }
    h.update(format!("landcover-v1|{lon_w:.9}|{lat_n:.9}|{nx}|{ny}|{dlon:.12}|{dlat:.12}").as_bytes());
    let prov = format!("{:x}", h.finalize());
    let dir = fetcher.cache_dir.join("derived");
    let _ = std::fs::create_dir_all(&dir);
    let cache = dir.join(format!("landcover-{}.u8", &prov[..24]));
    if let Ok(b) = std::fs::read(&cache) { if b.len() == nx * ny { return Ok(b); } }
    // count source pixels per cell and class (WorldCover codes are multiples of 10 up to 100, plus 95)
    let class_slot = |c: u8| -> Option<usize> { match c { 10 => Some(0), 20 => Some(1), 30 => Some(2), 40 => Some(3), 50 => Some(4), 60 => Some(5), 70 => Some(6), 80 => Some(7), 90 => Some(8), 95 => Some(9), 100 => Some(10), _ => None } };
    const CODES: [u8; 11] = [10, 20, 30, 40, 50, 60, 70, 80, 90, 95, 100];
    let mut counts = vec![[0u16; 11]; nx * ny];
    for (_, p) in &paths {
        let buf = std::fs::read(p).map_err(|e| e.to_string())?;
        let w = cog::read_window_u8(&buf, lon_w, lon_e, lat_s, lat_n)?;
        for r in 0..w.height {
            let lat = w.y0 - (w.row0 + r) as f64 * w.dy - w.dy / 2.0;
            if lat >= lat_n || lat < lat_s { continue; }
            let row = ((lat_n - lat) / dlat) as usize;
            if row >= ny { continue; }
            for c in 0..w.width {
                let v = w.data[r * w.width + c];
                let Some(slot) = class_slot(v) else { continue };
                let lon = w.x0 + (w.col0 + c) as f64 * w.dx + w.dx / 2.0;
                if lon < lon_w || lon >= lon_e { continue; }
                let col = ((lon - lon_w) / dlon) as usize;
                if col >= nx { continue; }
                counts[row * nx + col][slot] = counts[row * nx + col][slot].saturating_add(1);
            }
        }
    }
    let out: Vec<u8> = counts.iter().map(|c| {
        let (mut best, mut n) = (0u8, 0u16);
        for (i, &k) in c.iter().enumerate() { if k > n { n = k; best = CODES[i]; } }
        best
    }).collect();
    let _ = std::fs::write(&cache, &out);
    Ok(out)
}

/// Hourly weather at one point from the ERA5 archive: what a fire needs that the basin
/// weather grid lacks, the wind's direction and its gusts.
#[derive(Clone, Debug, Default)]
pub struct WindSeries {
    pub times: Vec<i64>,
    pub t2m_c: Vec<f32>,
    pub rh_pct: Vec<f32>,
    pub wind_m_s: Vec<f32>,
    pub dir_deg: Vec<f32>,
    pub gust_m_s: Vec<f32>,
}

impl WindSeries {
    /// Index of the last hour at or before `t` (seconds), if any.
    pub fn index_at(&self, t: i64) -> Option<usize> {
        if self.times.is_empty() || t < self.times[0] { return None; }
        Some(match self.times.binary_search(&t) { Ok(i) => i, Err(i) => i.saturating_sub(1) })
    }
}

pub fn wind_hourly(fetcher: &mut Fetcher, lat: f64, lon: f64, start: &str, end: &str) -> Result<WindSeries, String> {
    let lat = (lat * 1e4).round() / 1e4;
    let lon = (lon * 1e4).round() / 1e4;
    let url = format!("https://archive-api.open-meteo.com/v1/archive?latitude={lat}&longitude={lon}&start_date={start}&end_date={end}&hourly=temperature_2m,relative_humidity_2m,wind_speed_10m,wind_direction_10m,wind_gusts_10m&models=era5&timezone=UTC&wind_speed_unit=ms");
    let key = format!("open-meteo-era5-wind/{lat}_{lon}_{start}_{end}.json");
    let text = fetch_json_text(fetcher, &key, &url)?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("open-meteo json: {e}"))?;
    let h = &v["hourly"];
    let mut s = WindSeries::default();
    for t in h["time"].as_array().ok_or_else(|| format!("open-meteo: no hourly.time ({})", v["reason"]))? {
        s.times.push(parse_iso_hour(t.as_str().ok_or("time")?)?);
    }
    let take = |name: &str| -> Vec<f32> { h[name].as_array().cloned().unwrap_or_default().iter().map(|x| x.as_f64().map(|f| f as f32).unwrap_or(f32::NAN)).collect() };
    s.t2m_c = take("temperature_2m"); s.rh_pct = take("relative_humidity_2m");
    s.wind_m_s = take("wind_speed_10m"); s.dir_deg = take("wind_direction_10m"); s.gust_m_s = take("wind_gusts_10m");
    if s.wind_m_s.len() != s.times.len() { return Err("open-meteo: ragged hourly arrays".into()); }
    Ok(s)
}

/// One mapped perimeter of a fire: when it was current, the acres its record states, how
/// it was mapped, and its rings (outer and inner alike; the rasteriser fills even-odd).
#[derive(Clone, Debug)]
pub struct Perimeter {
    pub time: i64,
    pub acres: f64,
    pub method: String,
    pub rings: Vec<Vec<(f64, f64)>>,
}

const WFIGS: &str = "https://services3.arcgis.com/T4QMspbfLg3qTGWY/arcgis/rest/services";

fn rings_of(geom: &serde_json::Value) -> Vec<Vec<(f64, f64)>> {
    let ring = |r: &serde_json::Value| -> Vec<(f64, f64)> { r.as_array().into_iter().flatten().filter_map(|p| Some((p[0].as_f64()?, p[1].as_f64()?))).collect() };
    match geom["type"].as_str() {
        Some("Polygon") => geom["coordinates"].as_array().into_iter().flatten().map(ring).collect(),
        Some("MultiPolygon") => geom["coordinates"].as_array().into_iter().flatten().flat_map(|poly| poly.as_array().into_iter().flatten().map(ring).collect::<Vec<_>>()).collect(),
        _ => Vec::new(),
    }
}

fn features(fetcher: &mut Fetcher, key: &str, url: &str) -> Result<Vec<serde_json::Value>, String> {
    let text = fetch_json_text(fetcher, key, url)?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("arcgis json: {e}"))?;
    if let Some(e) = v.get("error") { return Err(format!("arcgis: {e}")); }
    Ok(v["features"].as_array().cloned().unwrap_or_default())
}

fn enc(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b { b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char), _ => out.push_str(&format!("%{b:02X}")) }
    }
    out
}

/// The WFIGS interagency perimeter(s) of an incident by name and discovery year: the
/// perimeter of record, usually one feature, current at `poly_DateCurrent`.
pub fn wfigs_perimeters(fetcher: &mut Fetcher, name: &str, year: i32) -> Result<Vec<Perimeter>, String> {
    let wh = format!("UPPER(poly_IncidentName) LIKE '%{}%' AND attr_FireDiscoveryDateTime >= DATE '{year}-01-01' AND attr_FireDiscoveryDateTime < DATE '{}-01-01'", name.to_uppercase(), year + 1);
    let url = format!("{WFIGS}/WFIGS_Interagency_Perimeters/FeatureServer/0/query?where={}&outFields=*&outSR=4326&f=geojson", enc(&wh));
    let key = format!("nifc-wfigs-perimeters/{}_{year}.geojson", name.to_lowercase().replace(' ', "-"));
    let mut out = Vec::new();
    for f in features(fetcher, &key, &url)? {
        let p = &f["properties"];
        let time = p["poly_DateCurrent"].as_i64().or_else(|| p["attr_ModifiedOnDateTime_dt"].as_i64()).unwrap_or(0) / 1000;
        out.push(Perimeter { time, acres: p["poly_GISAcres"].as_f64().unwrap_or(f64::NAN), method: p["poly_MapMethod"].as_str().unwrap_or("").to_string(), rings: rings_of(&f["geometry"]) });
    }
    out.sort_by_key(|p| p.time);
    Ok(out)
}

/// The daily operational perimeters of an incident from NIFC's Operational Data Archive
/// for a year (the Event_Polygon layer), by IRWIN id: each one current at `DateCurrent`
/// with the acres and the mapping method (infrared interpretation, mostly). Sorted by
/// time; rows without a time or acreage are dropped.
pub fn daily_perimeters(fetcher: &mut Fetcher, irwin: &str, year: i32) -> Result<Vec<Perimeter>, String> {
    let wh = format!("IRWINID = '{irwin}' AND FeatureCategory = 'Wildfire Daily Fire Perimeter'");
    let url = format!("{WFIGS}/Operational_Data_Archive_{year}/FeatureServer/6/query?where={}&outFields=*&outSR=4326&f=geojson", enc(&wh));
    let key = format!("nifc-operational-archive/{year}_{}.geojson", irwin.trim_matches(|c| c == '{' || c == '}').to_lowercase());
    let mut out = Vec::new();
    for f in features(fetcher, &key, &url)? {
        let p = &f["properties"];
        let (Some(ms), Some(acres)) = (p["DateCurrent"].as_i64(), p["GISAcres"].as_f64()) else { continue };
        let rings = rings_of(&f["geometry"]);
        if rings.is_empty() { continue; }
        out.push(Perimeter { time: ms / 1000, acres, method: p["MapMethod"].as_str().unwrap_or("").to_string(), rings });
    }
    out.sort_by_key(|p| p.time);
    Ok(out)
}

/// Fill a polygon set on a lat-lon grid (rows north to south): true where a cell's
/// centre is inside by the even-odd rule, so holes stay empty. Scanline per row.
pub fn rasterize(rings: &[Vec<(f64, f64)>], nx: usize, ny: usize, dlon: f64, dlat: f64, lon_w: f64, lat_n: f64) -> Vec<bool> {
    let mut out = vec![false; nx * ny];
    let mut xs: Vec<f64> = Vec::new();
    for row in 0..ny {
        let y = lat_n - (row as f64 + 0.5) * dlat;
        xs.clear();
        for ring in rings {
            for i in 0..ring.len() {
                let (x0, y0) = ring[i];
                let (x1, y1) = ring[(i + 1) % ring.len()];
                if (y0 > y) != (y1 > y) { xs.push(x0 + (y - y0) * (x1 - x0) / (y1 - y0)); }
            }
        }
        if xs.len() < 2 { continue; }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for pair in xs.chunks_exact(2) {
            let (xa, xb) = (pair[0], pair[1]);
            let c0 = (((xa - lon_w) / dlon) - 0.5).ceil().max(0.0) as usize;
            let c1 = (((xb - lon_w) / dlon) - 0.5).floor().min(nx as f64 - 1.0);
            if c1 < 0.0 { continue; }
            for c in c0..=(c1 as usize) { out[row * nx + c] = true; }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn worldcover_keys() {
        assert_eq!(worldcover_tile_key(34.07, -118.55), "N33W120");
        assert_eq!(worldcover_tile_key(42.5, -94.0), "N42W096");
        assert_eq!(worldcover_tile_key(-1.0, 10.0), "S03E009");
    }
    #[test]
    fn rasterize_a_square_with_a_hole() {
        let outer = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let hole = vec![(0.4, 0.4), (0.6, 0.4), (0.6, 0.6), (0.4, 0.6)];
        let m = rasterize(&[outer, hole], 10, 10, 0.1, 0.1, 0.0, 1.0);
        let n = m.iter().filter(|&&b| b).count();
        assert_eq!(n, 100 - 4, "cells inside: {n}");
        assert!(!m[5 * 10 + 5] && m[0] && m[99]);
    }
}

// ---- context for a map: roads, place names and city limits from OpenStreetMap ----------

fn thin(pts: Vec<(f64, f64)>, max: usize) -> Vec<(f64, f64)> {
    if pts.len() <= max { return pts; }
    let step = pts.len() as f64 / max as f64;
    let mut out: Vec<(f64, f64)> = (0..max).map(|i| pts[(i as f64 * step) as usize]).collect();
    if let Some(&last) = pts.last() { out.push(last); }
    out
}
fn r5(x: f64) -> f64 { (x * 1e5).round() / 1e5 }

/// Reduce an Overpass answer (`out geom`) to what a map needs: roads by class, named places
/// as points, and city limits (admin_level 8 relations) as lines.
pub fn reduce_context(raw: &[u8], bbox: (f64, f64, f64, f64)) -> Result<Vec<u8>, String> {
    let (lat_s, lat_n, lon_w, lon_e) = bbox;
    let inside = |p: &(f64, f64)| p.0 >= lon_w - 0.01 && p.0 <= lon_e + 0.01 && p.1 >= lat_s - 0.01 && p.1 <= lat_n + 0.01;
    let v: serde_json::Value = serde_json::from_slice(raw).map_err(|e| format!("overpass json: {e}"))?;
    if let Some(r) = v.get("remark") { if r.as_str().map(|s| s.contains("error")).unwrap_or(false) { return Err(format!("overpass: {r}")); } }
    let (mut roads, mut places, mut bounds) = (Vec::new(), Vec::new(), Vec::new());
    let geom = |g: &serde_json::Value| -> Vec<(f64, f64)> { g.as_array().into_iter().flatten().filter_map(|p| Some((r5(p["lon"].as_f64()?), r5(p["lat"].as_f64()?)))).collect() };
    for e in v["elements"].as_array().into_iter().flatten() {
        let tags = &e["tags"];
        match e["type"].as_str() {
            Some("way") => {
                let pts = thin(geom(&e["geometry"]), 120);
                if pts.len() > 1 { roads.push(serde_json::json!({ "class": tags["highway"].as_str().unwrap_or(""), "name": tags["name"].as_str().unwrap_or(""), "pts": pts })); }
            }
            Some("node") => {
                if let (Some(lat), Some(lon), Some(name)) = (e["lat"].as_f64(), e["lon"].as_f64(), tags["name"].as_str()) {
                    places.push(serde_json::json!({ "name": name, "kind": tags["place"].as_str().unwrap_or(""), "lon": r5(lon), "lat": r5(lat) }));
                }
            }
            Some("relation") => {
                let name = tags["name"].as_str().unwrap_or("").to_string();
                // a city limit comes back whole (Los Angeles is 258 ways); keep the runs inside the window
                for m in e["members"].as_array().into_iter().flatten() {
                    if m["type"].as_str() != Some("way") { continue; }
                    let pts = geom(&m["geometry"]);
                    let mut run: Vec<(f64, f64)> = Vec::new();
                    for p in pts.iter().chain(std::iter::once(&(f64::NAN, f64::NAN))) {
                        if p.0.is_finite() && inside(p) { run.push(*p); }
                        else { if run.len() > 1 { bounds.push(serde_json::json!({ "name": name, "pts": thin(std::mem::take(&mut run), 200) })); } run.clear(); }
                    }
                }
            }
            _ => {}
        }
    }
    serde_json::to_vec(&serde_json::json!({ "roads": roads, "places": places, "boundaries": bounds, "source": "OpenStreetMap contributors, ODbL, through the Overpass API" })).map_err(|e| e.to_string())
}

fn context_key(lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64) -> String { format!("osm-overpass/context_{lat_s}_{lat_n}_{lon_w}_{lon_e}.json") }

/// Fetch (once; the lock keeps the raw answer's hash, the cache the reduced map) the
/// roads, places and city limits inside a window.
pub fn context(fetcher: &mut Fetcher, lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64) -> Result<serde_json::Value, String> {
    let q = format!("[out:json][timeout:180];(way[\"highway\"~\"^(motorway|trunk|primary|secondary|tertiary)$\"]({lat_s},{lon_w},{lat_n},{lon_e});node[\"place\"~\"^(city|town|suburb|neighbourhood|village|quarter)$\"]({lat_s},{lon_w},{lat_n},{lon_e});relation[\"boundary\"=\"administrative\"][\"admin_level\"=\"8\"]({lat_s},{lon_w},{lat_n},{lon_e}););out geom;");
    // overpass-api.de answered 406 and then 502 (at capacity) on 2026-11-03; the kumi mirror answered in four minutes
    let bbox = (lat_s, lat_n, lon_w, lon_e);
    let key = context_key(lat_s, lat_n, lon_w, lon_e);
    let mut last = String::new();
    let mut path = None;
    for host in ["https://overpass.kumi.systems/api/interpreter", "https://overpass-api.de/api/interpreter"] {
        let url = format!("{host}?data={}", enc(&q));
        match fetcher.fetch_reduced(&key, &url, &|raw| reduce_context(raw, bbox)) { Ok(p) => { path = Some(p); break; } Err(e) => { last = e; } }
    }
    let path = path.ok_or(last)?;
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| format!("context cache: {e}"))
}

/// The cached context for a window, never fetching (for a describe).
pub fn cached_context(fetcher: &Fetcher, lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64) -> Option<serde_json::Value> {
    let name = context_key(lat_s, lat_n, lon_w, lon_e).replace(['/', ' '], "_");
    let text = std::fs::read_to_string(fetcher.cache_dir.join(name)).ok()?;
    serde_json::from_str(&text).ok()
}

// ---- structures and their fate: CAL FIRE's damage inspections -------------------------

/// One inspected structure: where it stands, what it is, and the damage category the
/// inspection recorded ("Destroyed (>50%)", "Major (26-50%)", "Minor (10-25%)",
/// "Affected (1-9%)", "No Damage").
#[derive(Clone, Debug)]
pub struct Structure { pub lon: f64, pub lat: f64, pub damage: String, pub kind: String }

/// Every point of a DINS feature service, paged 2,000 at a time, anonymously; each page is
/// cached and locked by its bytes.
pub fn dins_points(fetcher: &mut Fetcher, service: &str, key_prefix: &str) -> Result<Vec<Structure>, String> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for page in 0..50 {
        let url = format!("{service}/query?where=1%3D1&outFields=DAMAGE,STRUCTURETYPE&outSR=4326&f=geojson&resultOffset={offset}&resultRecordCount=2000");
        let text = fetch_json_text(fetcher, &format!("{key_prefix}/page{page:02}.geojson"), &url)?;
        let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("dins json: {e}"))?;
        if let Some(e) = v.get("error") { return Err(format!("dins: {e}")); }
        let feats = v["features"].as_array().cloned().unwrap_or_default();
        let n = feats.len();
        for f in feats {
            let c = &f["geometry"]["coordinates"];
            let (Some(lon), Some(lat)) = (c[0].as_f64(), c[1].as_f64()) else { continue };
            out.push(Structure { lon, lat, damage: f["properties"]["DAMAGE"].as_str().unwrap_or("").to_string(), kind: f["properties"]["STRUCTURETYPE"].as_str().unwrap_or("").to_string() });
        }
        let more = v["properties"]["exceededTransferLimit"].as_bool().unwrap_or(n == 2000);
        if n == 0 || !more { break; }
        offset += n;
    }
    Ok(out)
}

// ---- HRRR: the 3 km analysis wind, hour by hour, from NOAA's open bucket -----------------

use crate::grib::{self, Lcc};

/// One analysis hour of HRRR over a window: a small block of the 3 km grid (`ni` by `nj`
/// from `i0`, `j0`), with the 10 m wind components, the gust, the 2 m temperature and
/// humidity, and the projection to place a point in it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HrrrHour {
    pub grid: Lcc,
    pub i0: usize, pub j0: usize, pub ni: usize, pub nj: usize,
    pub u: Vec<f32>, pub v: Vec<f32>, pub gust: Vec<f32>, pub t2m_c: Vec<f32>, pub rh_pct: Vec<f32>,
}

impl HrrrHour {
    /// Bilinear sample of one field at a point, or NaN outside the block.
    pub fn at(&self, field: &[f32], lat: f64, lon: f64) -> f32 {
        let (i, j) = self.grid.ij(lat, lon);
        let (fi, fj) = (i - self.i0 as f64, j - self.j0 as f64);
        if fi < 0.0 || fj < 0.0 || fi >= (self.ni - 1) as f64 || fj >= (self.nj - 1) as f64 { return f32::NAN; }
        let (ia, ja) = (fi.floor() as usize, fj.floor() as usize);
        let (tx, ty) = ((fi - ia as f64) as f32, (fj - ja as f64) as f32);
        let g = |ii: usize, jj: usize| field[jj * self.ni + ii];
        (g(ia, ja) * (1.0 - tx) + g(ia + 1, ja) * tx) * (1.0 - ty) + (g(ia, ja + 1) * (1.0 - tx) + g(ia + 1, ja + 1) * tx) * ty
    }
}

const HRRR: &str = "https://noaa-hrrr-bdp-pds.s3.amazonaws.com";

/// Fetch the analysis (`f00`) records of one hour (`date` as YYYYMMDD, `hh` 0 to 23) and keep
/// the block over the window. Each record is fetched by byte range from the file's index and
/// reduced before it is cached; the lock holds the raw record's hash.
pub fn hrrr_hour(fetcher: &mut Fetcher, date: &str, hh: u8, lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64) -> Result<HrrrHour, String> {
    let file = format!("{HRRR}/hrrr.{date}/conus/hrrr.t{hh:02}z.wrfsfcf00.grib2");
    let idx_path = fetcher.fetch(&format!("hrrr/{date}_t{hh:02}z.idx"), &format!("{file}.idx"))?;
    let entries = grib::parse_idx(&std::fs::read_to_string(&idx_path).map_err(|e| e.to_string())?);
    let bbox = (lat_s, lat_n, lon_w, lon_e);
    let mut block: Option<(Lcc, usize, usize, usize, usize)> = None;
    let mut fields: Vec<Vec<f32>> = Vec::new();
    for (var, level) in [("UGRD", "10 m above ground"), ("VGRD", "10 m above ground"), ("GUST", "surface"), ("TMP", "2 m above ground"), ("RH", "2 m above ground")] {
        let e = entries.iter().find(|e| e.var == var && e.level == level && e.forecast == "anl").ok_or_else(|| format!("hrrr {date} t{hh:02}z: no {var} at {level}"))?;
        let (start, end) = grib::range_of(&entries, e.n, None).ok_or("range")?;
        let key = format!("hrrr/{date}_t{hh:02}z_{var}_{lat_s}_{lat_n}_{lon_w}_{lon_e}.json");
        let reduce = |raw: &[u8]| -> Result<Vec<u8>, String> {
            let rec = grib::read(raw)?;
            let g = &rec.grid;
            let corners = [(bbox.0, bbox.2), (bbox.0, bbox.3), (bbox.1, bbox.2), (bbox.1, bbox.3)].map(|(la, lo)| g.ij(la, lo));
            let i0 = corners.iter().map(|c| c.0).fold(f64::INFINITY, f64::min).floor().max(0.0) as usize;
            let j0 = corners.iter().map(|c| c.1).fold(f64::INFINITY, f64::min).floor().max(0.0) as usize;
            let i1 = (corners.iter().map(|c| c.0).fold(f64::NEG_INFINITY, f64::max).ceil() as usize + 1).min(g.nx - 1);
            let j1 = (corners.iter().map(|c| c.1).fold(f64::NEG_INFINITY, f64::max).ceil() as usize + 1).min(g.ny - 1);
            let (ni, nj) = (i1 - i0 + 1, j1 - j0 + 1);
            let mut vals = Vec::with_capacity(ni * nj);
            for j in j0..=j1 { for i in i0..=i1 { vals.push(rec.values[j * g.nx + i]); } }
            serde_json::to_vec(&serde_json::json!({ "grid": g, "i0": i0, "j0": j0, "ni": ni, "nj": nj, "values": vals })).map_err(|e| e.to_string())
        };
        let path = fetcher.fetch_range_reduced(&key, &file, start, end, &reduce)?;
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let vals: Vec<f32> = v["values"].as_array().into_iter().flatten().map(|x| x.as_f64().unwrap_or(f64::NAN) as f32).collect();
        if block.is_none() {
            let g: Lcc = serde_json::from_value(v["grid"].clone()).map_err(|e| e.to_string())?;
            block = Some((g, v["i0"].as_u64().unwrap() as usize, v["j0"].as_u64().unwrap() as usize, v["ni"].as_u64().unwrap() as usize, v["nj"].as_u64().unwrap() as usize));
        }
        fields.push(vals);
    }
    let (grid, i0, j0, ni, nj) = block.unwrap();
    let mut it = fields.into_iter();
    let (u, v, gust, t2m, rh) = (it.next().unwrap(), it.next().unwrap(), it.next().unwrap(), it.next().unwrap(), it.next().unwrap());
    Ok(HrrrHour { grid, i0, j0, ni, nj, u, v, gust, t2m_c: t2m.iter().map(|k| k - 273.15).collect(), rh_pct: rh })
}

#[cfg(test)]
mod hrrr_tests {
    use super::*;
    /// The Santa Ana hour: HRRR's 3 km analysis at the Palisades, 18Z on 7 January 2025, against
    /// the 25 km reanalysis (6.9 m/s mean, 13 m/s gust). Needs the network; skipped without a cache dir.
    #[test]
    fn palisades_first_hour_wind() {
        let root: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", ".."].iter().collect();
        if !root.join("data/cache").exists() { eprintln!("skipped"); return; }
        let mut f = Fetcher::default_for(&root).unwrap();
        let h = hrrr_hour(&mut f, "20250107", 18, 34.0, 34.16, -118.72, -118.46).expect("hrrr hour");
        let (lat, lon) = (34.0723, -118.5451);
        let (u, v, g) = (h.at(&h.u, lat, lon), h.at(&h.v, lat, lon), h.at(&h.gust, lat, lon));
        let spd = (u * u + v * v).sqrt();
        let from = ((-u).atan2(-v).to_degrees() + 360.0) % 360.0;
        eprintln!("HRRR 18Z at the ignition: {spd:.1} m/s from {from:.0}, gust {g:.1}, t {:.1} C, rh {:.0}%  (block {}x{} at {},{})", h.at(&h.t2m_c, lat, lon), h.at(&h.rh_pct, lat, lon), h.ni, h.nj, h.i0, h.j0);
        assert!(spd.is_finite() && spd > 0.5 && spd < 60.0, "{spd}");
    }
}

// ---- buildings: LA County's footprints with the damage inspection joined ----------------

/// A building footprint: its outer ring, the stated height (the service does not say its
/// unit; the values read as feet), and the maximum damage the inspection recorded.
#[derive(Clone, Debug)]
pub struct Building { pub ring: Vec<(f64, f64)>, pub height: f64, pub damage: String }

/// Every footprint of a feature service matching `where`, paged, anonymously.
pub fn building_footprints(fetcher: &mut Fetcher, service: &str, key_prefix: &str, wh: &str) -> Result<Vec<Building>, String> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for page in 0..60 {
        let url = format!("{service}/query?where={}&outFields=HEIGHT,DINS_MaximumDamage&outSR=4326&f=geojson&resultOffset={offset}&resultRecordCount=2000", enc(wh));
        let text = fetch_json_text(fetcher, &format!("{key_prefix}/page{page:02}.geojson"), &url)?;
        let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("buildings json: {e}"))?;
        if let Some(e) = v.get("error") { return Err(format!("buildings: {e}")); }
        let feats = v["features"].as_array().cloned().unwrap_or_default();
        let n = feats.len();
        for f in feats {
            let rings = rings_of(&f["geometry"]);
            let Some(ring) = rings.into_iter().next() else { continue };
            out.push(Building { ring, height: f["properties"]["HEIGHT"].as_f64().unwrap_or(f64::NAN), damage: f["properties"]["DINS_MaximumDamage"].as_str().unwrap_or("").to_string() });
        }
        let more = v["properties"]["exceededTransferLimit"].as_bool().unwrap_or(n == 2000);
        if n == 0 || !more { break; }
        offset += n;
    }
    Ok(out)
}
