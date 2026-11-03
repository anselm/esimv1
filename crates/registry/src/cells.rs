//! The planet on cells rather than countries: people per cell from WorldPop's 1 km
//! count aggregated to the run's grid, and a daily temperature and dew point per cell
//! for a baseline decade from NASA POWER (MERRA-2 reanalysis at 0.5 by 0.625 degrees,
//! served in 10 by 10 degree boxes, one parameter and one year per request). Both answer
//! anonymously. Verified from canada1 on 2026-11-02.
//!
//! POWER's limits, as its API states them: at most 10 degrees of latitude and longitude
//! and 366 days per regional request; no published rate limit (its errors table has a
//! 429), so the fetcher backs off on one. A baseline decade over the populated boxes is
//! about 4,600 requests, each about 2 MB of JSON reduced here to a 230 KB table of
//! tenths of a degree; the lock keeps the hash of the JSON, the cache keeps the table.

use crate::fetch::Fetcher;
use crate::cog;
use std::io::Read;

/// People per cell on a regular lat-lon grid of `res_deg` (origin 180 W, 90 N, row-major
/// from the north), summed from WorldPop's 1 km 2020 count. The 870 MB GeoTIFF is fetched
/// once and the sum is kept beside it as a small binary, so later loads take a moment.
/// Cells with nothing, and the sea, are zero.
pub fn worldpop_count(f: &mut Fetcher, url: &str, res_deg: f64) -> Result<Vec<f32>, String> {
    let nx = (360.0 / res_deg).round() as usize;
    let ny = (180.0 / res_deg).round() as usize;
    let derived = f.cache_dir.join(format!("worldpop_ppp_2020_count_{}.f32", res_deg));
    if let Ok(bytes) = std::fs::read(&derived) {
        if bytes.len() == nx * ny * 4 { return Ok(bytes.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect()); }
    }
    let file = url.rsplit('/').next().unwrap_or("ppp_2020_1km_Aggregated.tif");
    let path = f.fetch(&format!("worldpop/{file}"), url)?;
    let buf = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut sum = vec![0f64; nx * ny];
    cog::for_each_tile(&buf, |info, col0, row0, w, h, samples| {
        let (x0, y0, dx, dy, tw) = (info.x0, info.y0, info.dx, info.dy, info.tile_w);
        let nodata = info.nodata.unwrap_or(f32::MIN);
        for r in 0..h {
            let lat = y0 - (row0 + r) as f64 * dy - dy / 2.0;
            let row = ((90.0 - lat) / res_deg).floor();
            if row < 0.0 || row >= ny as f64 { continue; }
            let row = row as usize;
            for c in 0..w {
                let v = samples[r * tw + c];
                if !(v > 0.0) || v == nodata || v < -1e30 { continue; }
                let lon = x0 + (col0 + c) as f64 * dx + dx / 2.0;
                let col = (((lon + 180.0) / res_deg).floor() as isize).rem_euclid(nx as isize) as usize;
                sum[row * nx + col] += v as f64;
            }
        }
    })?;
    let out: Vec<f32> = sum.iter().map(|v| *v as f32).collect();
    let bytes: Vec<u8> = out.iter().flat_map(|v| v.to_le_bytes()).collect();
    std::fs::write(&derived, bytes).map_err(|e| e.to_string())?;
    Ok(out)
}

/// One POWER regional answer, reduced: the points (lat, lon) and, per point, one value per
/// day in tenths of a degree (i16::MIN where POWER has none).
pub struct PowerBox { pub points: Vec<(f32, f32)>, pub days: usize, pub data: Vec<i16> }

impl PowerBox {
    pub fn value(&self, point: usize, day: usize) -> Option<f64> {
        let v = self.data[point * self.days + day];
        if v == i16::MIN { None } else { Some(v as f64 / 10.0) }
    }
}

const MAGIC: &[u8; 4] = b"PWR1";

fn encode(points: &[(f32, f32)], days: usize, data: &[i16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + points.len() * 8 + data.len() * 2);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(points.len() as u32).to_le_bytes());
    out.extend_from_slice(&(days as u32).to_le_bytes());
    for (la, lo) in points { out.extend_from_slice(&la.to_le_bytes()); out.extend_from_slice(&lo.to_le_bytes()); }
    for v in data { out.extend_from_slice(&v.to_le_bytes()); }
    out
}

fn decode(bytes: &[u8]) -> Result<PowerBox, String> {
    if bytes.len() < 12 || &bytes[..4] != MAGIC { return Err("not a reduced POWER box".into()); }
    let n = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let days = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
    let mut p = 12;
    let mut points = Vec::with_capacity(n);
    for _ in 0..n { points.push((f32::from_le_bytes([bytes[p], bytes[p + 1], bytes[p + 2], bytes[p + 3]]), f32::from_le_bytes([bytes[p + 4], bytes[p + 5], bytes[p + 6], bytes[p + 7]]))); p += 8; }
    let mut data = Vec::with_capacity(n * days);
    for _ in 0..n * days { data.push(i16::from_le_bytes([bytes[p], bytes[p + 1]])); p += 2; }
    Ok(PowerBox { points, days, data })
}

/// Reduce POWER's regional JSON: features[].geometry.coordinates [lon, lat], and
/// properties.parameter.<PARAM> as { "YYYYMMDD": value } (daily) or month names plus
/// "ANN" (climatology). Keys are sorted as strings, so a daily table runs in date order
/// and a climatology runs ANN, APR, AUG, DEC, FEB, JAN, JUL, JUN, MAR, MAY, NOV, OCT, SEP:
/// read a climatology column by name through `clim_index`, never by position.
fn reduce_json(raw: &[u8], param: &str) -> Result<Vec<u8>, String> {
    let v: serde_json::Value = serde_json::from_slice(raw).map_err(|e| format!("POWER json: {e}"))?;
    if let Some(msgs) = v.get("messages").and_then(|m| m.as_array()) { if !msgs.is_empty() && v.get("features").is_none() { return Err(format!("POWER: {}", msgs.iter().map(|m| m.to_string()).collect::<Vec<_>>().join("; "))); } }
    let feats = v.get("features").and_then(|f| f.as_array()).ok_or("POWER: no features")?;
    let mut points = Vec::new();
    let mut keys: Option<Vec<String>> = None;
    let mut data: Vec<i16> = Vec::new();
    for f in feats {
        let c = f.pointer("/geometry/coordinates").and_then(|c| c.as_array()).ok_or("POWER: no coordinates")?;
        let (lon, lat) = (c[0].as_f64().unwrap_or(f64::NAN) as f32, c[1].as_f64().unwrap_or(f64::NAN) as f32);
        let table = f.pointer(&format!("/properties/parameter/{param}")).and_then(|t| t.as_object()).ok_or_else(|| format!("POWER: no {param}"))?;
        let k = keys.get_or_insert_with(|| { let mut k: Vec<String> = table.keys().cloned().collect(); k.sort(); k });
        for key in k.iter() {
            let x = table.get(key).and_then(|x| x.as_f64()).unwrap_or(-999.0);
            data.push(if x <= -998.0 { i16::MIN } else { (x * 10.0).round().clamp(-32000.0, 32000.0) as i16 });
        }
        points.push((lat, lon));
    }
    let days = keys.map(|k| k.len()).unwrap_or(0);
    Ok(encode(&points, days, &data))
}

/// The box with south-west corner (lat0, lon0) covering 10 degrees each way, inclusive of
/// both edges as POWER answers it (21 latitudes by 17 longitudes).
pub fn power_daily_box(f: &mut Fetcher, param: &str, lat0: i32, lon0: i32, year: i64) -> Result<PowerBox, String> {
    let url = format!("https://power.larc.nasa.gov/api/temporal/daily/regional?parameters={param}&community=RE&longitude-min={lon0}&longitude-max={}&latitude-min={lat0}&latitude-max={}&start={year}0101&end={year}1231&format=JSON", lon0 + 10, lat0 + 10);
    let key = format!("power/daily_{param}_{lat0}_{lon0}_{year}.bin");
    let path = f.fetch_reduced(&key, &url, &|raw| reduce_json(raw, param))?;
    let mut bytes = Vec::new();
    std::fs::File::open(&path).map_err(|e| e.to_string())?.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    decode(&bytes)
}

/// The climatology of a parameter over the same box: twelve monthly means and the annual
/// mean per point, over POWER's climatology period (2001 to 2025 as served on 2026-11-02;
/// the header of the answer states it). Columns are in `CLIM_KEYS` order; `clim_index("ANN")`
/// is the annual mean. (Until 2026-11-03 the reader took the last column as the annual mean;
/// that column is September.)
pub const CLIM_KEYS: [&str; 13] = ["ANN", "APR", "AUG", "DEC", "FEB", "JAN", "JUL", "JUN", "MAR", "MAY", "NOV", "OCT", "SEP"];

/// The column of a climatology table holding `key` ("ANN" or a three-letter month).
pub fn clim_index(key: &str) -> Option<usize> { CLIM_KEYS.iter().position(|k| *k == key) }

pub fn power_climatology_box(f: &mut Fetcher, param: &str, lat0: i32, lon0: i32) -> Result<PowerBox, String> {
    let url = format!("https://power.larc.nasa.gov/api/temporal/climatology/regional?parameters={param}&community=RE&longitude-min={lon0}&longitude-max={}&latitude-min={lat0}&latitude-max={}&format=JSON", lon0 + 10, lat0 + 10);
    let key = format!("power/clim_{param}_{lat0}_{lon0}.bin");
    let path = f.fetch_reduced(&key, &url, &|raw| reduce_json(raw, param))?;
    let mut bytes = Vec::new();
    std::fs::File::open(&path).map_err(|e| e.to_string())?.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    decode(&bytes)
}

/// The 10-degree boxes (south-west corners) that contain any cell for which `wanted`
/// holds; latitudes from 90 S to 80 N, longitudes from 180 W to 170 E.
pub fn power_boxes(res_deg: f64, wanted: impl Fn(usize, usize) -> bool) -> Vec<(i32, i32)> {
    let nx = (360.0 / res_deg).round() as usize;
    let ny = (180.0 / res_deg).round() as usize;
    let mut out = Vec::new();
    for lat0 in (-90..90).step_by(10) {
        for lon0 in (-180..180).step_by(10) {
            let rows = (((90 - lat0 - 10) as f64 / res_deg).floor() as usize)..(((90 - lat0) as f64 / res_deg).ceil() as usize).min(ny);
            let cols = (((lon0 + 180) as f64 / res_deg).floor() as usize)..(((lon0 + 190) as f64 / res_deg).ceil() as usize).min(nx);
            let mut any = false;
            'o: for r in rows.clone() { for c in cols.clone() { if wanted(c, r) { any = true; break 'o; } } }
            if any { out.push((lat0, lon0)); }
        }
    }
    out
}

/// POWER's point nearest a cell centre: latitudes on multiples of 0.5, longitudes on
/// multiples of 0.625 from 180 W.
pub fn nearest_power_point(lat: f64, lon: f64) -> (f32, f32) {
    let la = (lat * 2.0).round() / 2.0;
    let lo = -180.0 + ((lon + 180.0) / 0.625).round() * 0.625;
    let lo = if lo >= 180.0 { lo - 360.0 } else { lo };
    (la as f32, lo as f32)
}
