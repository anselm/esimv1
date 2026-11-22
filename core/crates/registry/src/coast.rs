//! The coast: the hourly wave climate at a point from the ERA5-Ocean reanalysis through
//! Open-Meteo's marine API, and the shoreline and breakwaters of a window from OpenStreetMap.

use crate::fetch::Fetcher;
use crate::obs::{fetch_json_text, parse_iso_hour};
use serde_json::json;

/// Hourly waves at one point: significant height, the direction they come from, mean and peak period.
#[derive(Clone, Debug, Default)]
pub struct WaveSeries {
    pub times: Vec<i64>,
    pub hs_m: Vec<f32>,
    pub dir_deg: Vec<f32>,
    pub tm_s: Vec<f32>,
    pub tp_s: Vec<f32>,
    pub cell_lat: f64,
    pub cell_lon: f64,
}

impl WaveSeries {
    pub fn index_at(&self, t: i64) -> Option<usize> {
        if self.times.is_empty() || t < self.times[0] { return None; }
        Some(match self.times.binary_search(&t) { Ok(i) => i, Err(i) => i.saturating_sub(1) })
    }
}

/// The ERA5-Ocean hours of whole years, fetched one year per request and cached.
pub fn waves_hourly(fetcher: &mut Fetcher, lat: f64, lon: f64, year_from: i32, year_to: i32) -> Result<WaveSeries, String> {
    let lat = (lat * 1e3).round() / 1e3;
    let lon = (lon * 1e3).round() / 1e3;
    let mut s = WaveSeries::default();
    for y in year_from..=year_to {
        let url = format!("https://marine-api.open-meteo.com/v1/marine?latitude={lat}&longitude={lon}&start_date={y}-01-01&end_date={y}-12-31&hourly=wave_height,wave_direction,wave_period,wave_peak_period&models=era5_ocean&timezone=UTC");
        let key = format!("open-meteo-era5-ocean-waves/{lat}_{lon}_{y}.json");
        let text = fetch_json_text(fetcher, &key, &url)?;
        let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("open-meteo marine json: {e}"))?;
        if let Some(r) = v.get("reason") { return Err(format!("open-meteo marine {y}: {r}")); }
        s.cell_lat = v["latitude"].as_f64().unwrap_or(lat); s.cell_lon = v["longitude"].as_f64().unwrap_or(lon);
        let h = &v["hourly"];
        let times: Vec<i64> = h["time"].as_array().ok_or_else(|| format!("open-meteo marine {y}: no hourly.time"))?.iter().map(|t| parse_iso_hour(t.as_str().unwrap_or(""))).collect::<Result<_, _>>()?;
        let take = |name: &str| -> Vec<f32> { h[name].as_array().cloned().unwrap_or_default().iter().map(|x| x.as_f64().map(|f| f as f32).unwrap_or(f32::NAN)).collect() };
        let (hs, dir, tm, tp) = (take("wave_height"), take("wave_direction"), take("wave_period"), take("wave_peak_period"));
        if hs.len() != times.len() || dir.len() != times.len() { return Err(format!("open-meteo marine {y}: ragged arrays")); }
        s.times.extend(times); s.hs_m.extend(hs); s.dir_deg.extend(dir); s.tm_s.extend(tm); s.tp_s.extend(tp);
    }
    Ok(s)
}

/// A mapped line: OpenStreetMap coastline ways and breakwater outlines inside a window, as
/// (lat, lon) vertex lists with the feature's kind and id.
#[derive(Clone, Debug)]
pub struct ShoreWay { pub id: i64, pub kind: String, pub name: String, pub pts: Vec<(f64, f64)> }

fn enc(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() { match b { b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char), _ => out.push_str(&format!("%{b:02X}")) } }
    out
}

/// The coastline and breakwaters of a window through Overpass, reduced to vertex lists and cached.
pub fn osm_shore(fetcher: &mut Fetcher, lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64) -> Result<Vec<ShoreWay>, String> {
    let q = format!("[out:json][timeout:180];(way[\"natural\"=\"coastline\"]({lat_s},{lon_w},{lat_n},{lon_e});way[\"man_made\"=\"breakwater\"]({lat_s},{lon_w},{lat_n},{lon_e}););out geom;");
    let key = format!("osm-overpass/shore_{lat_s}_{lat_n}_{lon_w}_{lon_e}.json");
    let reduce = |raw: &[u8]| -> Result<Vec<u8>, String> {
        let v: serde_json::Value = serde_json::from_slice(raw).map_err(|e| format!("overpass json: {e}"))?;
        if let Some(r) = v.get("remark") { if r.as_str().map(|s| s.contains("error")).unwrap_or(false) { return Err(format!("overpass: {r}")); } }
        let mut out = Vec::new();
        for e in v["elements"].as_array().cloned().unwrap_or_default() {
            if e["type"] != "way" { continue; }
            let pts: Vec<serde_json::Value> = e["geometry"].as_array().cloned().unwrap_or_default().iter().filter_map(|g| Some(json!([g["lat"].as_f64()?, g["lon"].as_f64()?]))).collect();
            if pts.len() < 2 { continue; }
            let tags = &e["tags"];
            let kind = if tags["man_made"] == "breakwater" { "breakwater" } else { "coastline" };
            out.push(json!({ "id": e["id"], "kind": kind, "name": tags["name"].as_str().unwrap_or(""), "pts": pts }));
        }
        serde_json::to_vec(&out).map_err(|e| e.to_string())
    };
    let mut last = String::new();
    let mut path = None;
    for host in ["https://overpass.kumi.systems/api/interpreter", "https://overpass-api.de/api/interpreter"] {
        let url = format!("{host}?data={}", enc(&q));
        match fetcher.fetch_reduced(&key, &url, &reduce) { Ok(p) => { path = Some(p); break; } Err(e) => { last = e; } }
    }
    let path = path.ok_or(last)?;
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("osm shore cache: {e}"))?;
    Ok(v.as_array().cloned().unwrap_or_default().iter().map(|w| ShoreWay {
        id: w["id"].as_i64().unwrap_or(0), kind: w["kind"].as_str().unwrap_or("").into(), name: w["name"].as_str().unwrap_or("").into(),
        pts: w["pts"].as_array().cloned().unwrap_or_default().iter().filter_map(|p| Some((p.get(0)?.as_f64()?, p.get(1)?.as_f64()?))).collect(),
    }).collect())
}
