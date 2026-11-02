//! Countries as the coarse unit of the planetary stack: their outlines (Natural Earth),
//! their people (UN World Population Prospects), their temperature records (CRU TS through
//! the World Bank Climate Change Knowledge Portal). All three answer anonymously; all three
//! are keyed by ISO 3166-1 alpha-3 here. Verified from canada1 on 2026-11-01.

use crate::fetch::Fetcher;
use std::collections::BTreeMap;
use std::io::Read;

/// One country outline: rings in (lon, lat) degrees; holes are rings too, resolved by
/// even-odd filling, which is what `contains` does.
#[derive(Clone, Debug)]
pub struct Country {
    pub iso3: String,
    pub name: String,
    pub continent: String,
    pub rings: Vec<Vec<(f64, f64)>>,
    pub bbox: (f64, f64, f64, f64), // lon_w, lat_s, lon_e, lat_n
}

impl Country {
    pub fn contains(&self, lon: f64, lat: f64) -> bool {
        let (w, s, e, n) = self.bbox;
        if lon < w || lon > e || lat < s || lat > n { return false; }
        let mut inside = false;
        for ring in &self.rings {
            let m = ring.len();
            if m < 3 { continue; }
            let mut j = m - 1;
            for i in 0..m {
                let (xi, yi) = ring[i];
                let (xj, yj) = ring[j];
                if (yi > lat) != (yj > lat) && lon < (xj - xi) * (lat - yi) / (yj - yi) + xi { inside = !inside; }
                j = i;
            }
        }
        inside
    }
}

/// Natural Earth admin-0 countries from the project's GeoJSON. `ISO_A3` is -99 for a few
/// entries (France, Norway, Kosovo among them); `ADM0_A3` is then used.
pub fn natural_earth_countries(f: &mut Fetcher, url: &str) -> Result<Vec<Country>, String> {
    let path = f.fetch("natural-earth/ne_110m_admin_0_countries.geojson", url)?;
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("geojson: {e}"))?;
    let mut out = Vec::new();
    for feat in v["features"].as_array().into_iter().flatten() {
        let p = &feat["properties"];
        let mut iso3 = p["ISO_A3"].as_str().unwrap_or("-99").to_string();
        if iso3 == "-99" { iso3 = p["ADM0_A3"].as_str().unwrap_or("-99").to_string(); }
        let name = p["NAME"].as_str().unwrap_or("").to_string();
        let continent = p["CONTINENT"].as_str().unwrap_or("").to_string();
        let g = &feat["geometry"];
        let mut rings: Vec<Vec<(f64, f64)>> = Vec::new();
        let ring_of = |r: &serde_json::Value| -> Vec<(f64, f64)> {
            r.as_array().into_iter().flatten().filter_map(|pt| Some((pt[0].as_f64()?, pt[1].as_f64()?))).collect()
        };
        match g["type"].as_str() {
            Some("Polygon") => for r in g["coordinates"].as_array().into_iter().flatten() { rings.push(ring_of(r)); },
            Some("MultiPolygon") => for poly in g["coordinates"].as_array().into_iter().flatten() { for r in poly.as_array().into_iter().flatten() { rings.push(ring_of(r)); } },
            _ => continue,
        }
        let (mut w, mut s, mut e, mut n) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        for r in &rings { for &(x, y) in r { w = w.min(x); e = e.max(x); s = s.min(y); n = n.max(y); } }
        out.push(Country { iso3, name, continent, rings, bbox: (w, s, e, n) });
    }
    if out.is_empty() { return Err("natural earth: no features".into()); }
    Ok(out)
}

/// UN World Population Prospects 2024, total population by country (ISO3) and year, in
/// persons: the Estimates variant to 2023, the Medium projection variant after. The file
/// is a gzipped CSV of about 17 MB.
pub fn un_wpp_population(f: &mut Fetcher, url: &str) -> Result<BTreeMap<String, BTreeMap<i64, f64>>, String> {
    let path = f.fetch("un-wpp/WPP2024_TotalPopulationBySex.csv.gz", url)?;
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let mut text = String::new();
    flate2::read::MultiGzDecoder::new(&bytes[..]).read_to_string(&mut text).map_err(|e| format!("wpp gunzip: {e}"))?;
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().ok_or("wpp: empty")?.trim_start_matches('\u{feff}').split(',').collect();
    let col = |n: &str| header.iter().position(|h| *h == n).ok_or_else(|| format!("wpp: no column {n}"));
    let (ciso, ctype, cvar, ctime, cpop) = (col("ISO3_code")?, col("LocTypeName")?, col("Variant")?, col("Time")?, col("PopTotal")?);
    let mut out: BTreeMap<String, BTreeMap<i64, f64>> = BTreeMap::new();
    for line in lines {
        // the file quotes location names that contain commas; split carefully
        let fields = split_csv(line);
        if fields.get(ctype).map(|s| s.as_str()) != Some("Country/Area") { continue; }
        let Some(iso) = fields.get(ciso).filter(|s| !s.is_empty()) else { continue };
        let variant = fields.get(cvar).map(|s| s.as_str()).unwrap_or("");
        if variant != "Estimates" && variant != "Medium" { continue; }
        let (Some(Ok(y)), Some(Ok(pop))) = (fields.get(ctime).map(|s| s.parse::<i64>()), fields.get(cpop).map(|s| s.parse::<f64>())) else { continue };
        let e = out.entry(iso.clone()).or_default();
        // Estimates win where both exist (they overlap in 2024)
        if variant == "Medium" && e.contains_key(&y) { continue; }
        e.insert(y, pop * 1000.0);
    }
    if out.is_empty() { return Err("wpp: no country rows".into()); }
    Ok(out)
}

fn split_csv(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut q = false;
    for c in line.chars() {
        match c {
            '"' => q = !q,
            ',' if !q => { out.push(std::mem::take(&mut cur)); }
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

/// CRU TS 4.07 country mean temperature from the World Bank Climate Change Knowledge
/// Portal API: annual means 1901 to 2022 (deg C) for every country, one JSON.
/// Keys in the file are "YYYY-07"; the map is by year.
pub fn cckp_country_annual_tas(f: &mut Fetcher, url: &str) -> Result<BTreeMap<String, BTreeMap<i64, f64>>, String> {
    let path = f.fetch("worldbank-cckp/cru_tas_annual_all_countries.json", url)?;
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("cckp: {e}"))?;
    let mut out = BTreeMap::new();
    for (iso, series) in v["data"].as_object().into_iter().flatten() {
        let mut m = BTreeMap::new();
        for (k, val) in series.as_object().into_iter().flatten() {
            if let (Ok(y), Some(t)) = (k[0..4].parse::<i64>(), val.as_f64()) { m.insert(y, t); }
        }
        if !m.is_empty() { out.insert(iso.clone(), m); }
    }
    if out.is_empty() { return Err("cckp: no countries".into()); }
    Ok(out)
}

/// The same portal's monthly series 1901 to 2022, by country, keyed (year, month).
pub fn cckp_country_monthly_tas(f: &mut Fetcher, url: &str) -> Result<BTreeMap<String, BTreeMap<(i64, i64), f64>>, String> {
    let path = f.fetch("worldbank-cckp/cru_tas_monthly_all_countries.json", url)?;
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("cckp monthly: {e}"))?;
    let mut out = BTreeMap::new();
    for (iso, series) in v["data"].as_object().into_iter().flatten() {
        let mut m = BTreeMap::new();
        for (k, val) in series.as_object().into_iter().flatten() {
            if let (Ok(y), Ok(mo), Some(t)) = (k[0..4].parse::<i64>(), k[5..7].parse::<i64>(), val.as_f64()) { m.insert((y, mo), t); }
        }
        if !m.is_empty() { out.insert(iso.clone(), m); }
    }
    if out.is_empty() { return Err("cckp monthly: no countries".into()); }
    Ok(out)
}

/// Centred running mean over `window` years (odd), keeping keys with a full window.
pub fn smooth(s: &BTreeMap<i64, f64>, window: i64) -> BTreeMap<i64, f64> {
    let h = window / 2;
    let mut out = BTreeMap::new();
    for (&y, _) in s {
        let vals: Vec<f64> = (y - h..=y + h).filter_map(|k| s.get(&k).copied()).collect();
        if vals.len() as i64 == window { out.insert(y, vals.iter().sum::<f64>() / vals.len() as f64); }
    }
    out
}

/// Ordinary least squares slope and intercept of y on x over paired keys.
pub fn regress(x: &BTreeMap<i64, f64>, y: &BTreeMap<i64, f64>) -> Option<(f64, f64, usize)> {
    let pairs: Vec<(f64, f64)> = x.iter().filter_map(|(k, a)| y.get(k).map(|b| (*a, *b))).collect();
    let n = pairs.len();
    if n < 10 { return None; }
    let (mx, my) = (pairs.iter().map(|p| p.0).sum::<f64>() / n as f64, pairs.iter().map(|p| p.1).sum::<f64>() / n as f64);
    let sxx: f64 = pairs.iter().map(|p| (p.0 - mx).powi(2)).sum();
    if sxx <= 0.0 { return None; }
    let sxy: f64 = pairs.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    let slope = sxy / sxx;
    Some((slope, my - slope * mx, n))
}
