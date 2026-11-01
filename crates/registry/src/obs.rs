//! Observation series: the scoreboards. Fetched through the same cache and lock as base
//! layers, parsed into day-stamped values. Days are Unix seconds at 00:00 UTC.

use crate::fetch::Fetcher;
use std::collections::BTreeMap;

pub const CFS_TO_M3S: f64 = 0.028_316_846_592;

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}
pub fn day_secs(y: i64, m: i64, d: i64) -> i64 { days_from_civil(y, m, d) * 86_400 }

/// USGS NWIS daily values for one site and parameter, in SI (discharge 00060 in m3/s).
/// Returns (day, value) with the USGS approval code kept out: provisional data is used
/// and said so in the catalog.
pub fn usgs_daily(fetcher: &mut Fetcher, site: &str, parameter: &str, start: &str, end: &str) -> Result<BTreeMap<i64, f64>, String> {
    let url = format!("https://waterservices.usgs.gov/nwis/dv/?format=rdb&sites={site}&parameterCd={parameter}&statCd=00003&startDT={start}&endDT={end}");
    let key = format!("usgs-dv/{site}-{parameter}-{start}-{end}.rdb");
    let path = fetcher.fetch(&key, &url)?;
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut out = BTreeMap::new();
    let mut col = None;
    for line in text.lines() {
        if line.starts_with('#') { continue; }
        let f: Vec<&str> = line.split('\t').collect();
        if f.first() == Some(&"agency_cd") {
            col = f.iter().position(|h| h.ends_with(&format!("_{parameter}_00003")));
            continue;
        }
        if f.first() == Some(&"5s") { continue; }
        let (Some(c), Some(date)) = (col, f.get(2)) else { continue };
        let Some(v) = f.get(c).and_then(|s| s.parse::<f64>().ok()) else { continue };
        let d: Vec<i64> = date.split('-').filter_map(|s| s.parse().ok()).collect();
        if d.len() != 3 { continue; }
        let si = if parameter == "00060" { v * CFS_TO_M3S } else { v };
        out.insert(day_secs(d[0], d[1], d[2]), si);
    }
    if out.is_empty() { return Err(format!("usgs {site} {parameter}: no values parsed")); }
    Ok(out)
}

/// One GHCN-Daily station, SI units: precipitation mm, snowfall mm, temperatures C.
#[derive(Clone, Debug, Default)]
pub struct GhcnDaily {
    pub id: String,
    pub prcp_mm: BTreeMap<i64, f64>,
    pub snow_mm: BTreeMap<i64, f64>,
    pub tmax_c: BTreeMap<i64, f64>,
    pub tmin_c: BTreeMap<i64, f64>,
}

pub fn ghcn_station(fetcher: &mut Fetcher, id: &str) -> Result<GhcnDaily, String> {
    let url = format!("https://noaa-ghcn-pds.s3.amazonaws.com/csv/by_station/{id}.csv");
    let path = fetcher.fetch(&format!("ghcn-daily/{id}.csv"), &url)?;
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut g = GhcnDaily { id: id.to_string(), ..Default::default() };
    for line in text.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 { continue; }
        if !f[5].is_empty() { continue; } // failed a quality check
        let date = f[1];
        if date.len() != 8 { continue; }
        let (y, m, d) = (date[0..4].parse::<i64>(), date[4..6].parse::<i64>(), date[6..8].parse::<i64>());
        let (Ok(y), Ok(m), Ok(d)) = (y, m, d) else { continue };
        let Ok(v) = f[3].parse::<f64>() else { continue };
        let day = day_secs(y, m, d);
        match f[2] {
            "PRCP" => { g.prcp_mm.insert(day, v / 10.0); }
            "SNOW" => { g.snow_mm.insert(day, v); }
            "TMAX" => { g.tmax_c.insert(day, v / 10.0); }
            "TMIN" => { g.tmin_c.insert(day, v / 10.0); }
            _ => {}
        }
    }
    if g.prcp_mm.is_empty() { return Err(format!("ghcn {id}: no precipitation parsed")); }
    Ok(g)
}

/// Hourly ERA5 reanalysis (via Open-Meteo's archive API, free for non-commercial use)
/// at a regular grid of points. Each point's series are in SI except as named.
#[derive(Clone, Debug, Default)]
pub struct WeatherGrid {
    pub lats: Vec<f64>,
    pub lons: Vec<f64>,
    /// hours since the first hour, as Unix seconds of each hour
    pub times: Vec<i64>,
    /// [point][hour]; point index = lat_i * lons.len() + lon_i
    pub t2m_c: Vec<Vec<f32>>,
    pub precip_mm: Vec<Vec<f32>>,
    pub sw_w_m2: Vec<Vec<f32>>,
    pub rh_pct: Vec<Vec<f32>>,
    pub wind_m_s: Vec<Vec<f32>>,
}

/// Fetch ERA5 hourly for the points of a `step`-degree grid covering the box, for the
/// date range (inclusive, YYYY-MM-DD). One request per year, all points at once.
pub fn era5_hourly(fetcher: &mut Fetcher, lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, step: f64, start: &str, end: &str) -> Result<WeatherGrid, String> {
    let mut lats = Vec::new();
    let mut la = (lat_s / step).floor() * step + step / 2.0;
    while la < lat_n + step / 2.0 { lats.push((la * 1e4).round() / 1e4); la += step; }
    let mut lons = Vec::new();
    let mut lo = (lon_w / step).floor() * step + step / 2.0;
    while lo < lon_e + step / 2.0 { lons.push((lo * 1e4).round() / 1e4); lo += step; }
    let mut pts_lat = Vec::new(); let mut pts_lon = Vec::new();
    for &a in &lats { for &o in &lons { pts_lat.push(format!("{a}")); pts_lon.push(format!("{o}")); } }
    let y0: i64 = start[..4].parse().map_err(|_| "start year")?;
    let y1: i64 = end[..4].parse().map_err(|_| "end year")?;
    let mut g = WeatherGrid { lats: lats.clone(), lons: lons.clone(), ..Default::default() };
    let np = pts_lat.len();
    g.t2m_c = vec![Vec::new(); np]; g.precip_mm = vec![Vec::new(); np]; g.sw_w_m2 = vec![Vec::new(); np]; g.rh_pct = vec![Vec::new(); np]; g.wind_m_s = vec![Vec::new(); np];
    for y in y0..=y1 {
        let s = if y == y0 { start.to_string() } else { format!("{y}-01-01") };
        let e = if y == y1 { end.to_string() } else { format!("{y}-12-31") };
        let url = format!("https://archive-api.open-meteo.com/v1/archive?latitude={}&longitude={}&start_date={s}&end_date={e}&hourly=temperature_2m,precipitation,shortwave_radiation,relative_humidity_2m,wind_speed_10m&models=era5&timezone=UTC&wind_speed_unit=ms",
            pts_lat.join(","), pts_lon.join(","));
        let key = format!("open-meteo-era5/{}_{}_{}_{}_{}deg_{s}_{e}.json", lat_s, lat_n, lon_w, lon_e, step);
        let path = fetcher.fetch(&key, &url)?;
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("open-meteo json: {e}"))?;
        let arr: Vec<serde_json::Value> = match v { serde_json::Value::Array(a) => a, other => vec![other] };
        if arr.len() != np { return Err(format!("open-meteo returned {} points, expected {np}: {}", arr.len(), arr.first().map(|a| a["reason"].to_string()).unwrap_or_default())); }
        for (p, item) in arr.iter().enumerate() {
            let h = &item["hourly"];
            if g.times.len() < h["time"].as_array().map(|a| a.len()).unwrap_or(0) + g.t2m_c[p].len() && p == 0 {
                for t in h["time"].as_array().ok_or("no hourly.time")? {
                    let s = t.as_str().ok_or("time")?;
                    g.times.push(crate::obs::parse_iso_hour(s)?);
                }
            }
            let take = |name: &str, dst: &mut Vec<f32>| {
                for x in h[name].as_array().cloned().unwrap_or_default() { dst.push(x.as_f64().map(|f| f as f32).unwrap_or(f32::NAN)); }
            };
            take("temperature_2m", &mut g.t2m_c[p]);
            take("precipitation", &mut g.precip_mm[p]);
            take("shortwave_radiation", &mut g.sw_w_m2[p]);
            take("relative_humidity_2m", &mut g.rh_pct[p]);
            take("wind_speed_10m", &mut g.wind_m_s[p]);
        }
    }
    Ok(g)
}

/// "2025-06-18T13:00" -> Unix seconds.
pub fn parse_iso_hour(s: &str) -> Result<i64, String> {
    let (d, t) = s.split_once('T').ok_or("iso hour")?;
    let p: Vec<i64> = d.split('-').filter_map(|x| x.parse().ok()).collect();
    let q: Vec<i64> = t.split(':').filter_map(|x| x.parse().ok()).collect();
    if p.len() != 3 || q.is_empty() { return Err(format!("iso hour {s}")); }
    Ok(day_secs(p[0], p[1], p[2]) + q[0] * 3600 + q.get(1).copied().unwrap_or(0) * 60)
}

impl WeatherGrid {
    /// Bilinear value of a variable at (lat, lon) for hour index `h`; clamps to the grid.
    pub fn at(&self, var: &[Vec<f32>], h: usize, lat: f64, lon: f64) -> f32 {
        let (nla, nlo) = (self.lats.len(), self.lons.len());
        if nla == 0 || nlo == 0 { return f32::NAN; }
        let fy = if nla > 1 { ((lat - self.lats[0]) / (self.lats[1] - self.lats[0])).clamp(0.0, (nla - 1) as f64) } else { 0.0 };
        let fx = if nlo > 1 { ((lon - self.lons[0]) / (self.lons[1] - self.lons[0])).clamp(0.0, (nlo - 1) as f64) } else { 0.0 };
        let (y0, x0) = (fy.floor() as usize, fx.floor() as usize);
        let (y1, x1) = ((y0 + 1).min(nla - 1), (x0 + 1).min(nlo - 1));
        let (wy, wx) = ((fy - y0 as f64) as f32, (fx - x0 as f64) as f32);
        let v = |yi: usize, xi: usize| var[yi * nlo + xi].get(h).copied().unwrap_or(f32::NAN);
        let top = v(y0, x0) * (1.0 - wx) + v(y0, x1) * wx;
        let bot = v(y1, x0) * (1.0 - wx) + v(y1, x1) * wx;
        top * (1.0 - wy) + bot * wy
    }
    pub fn hour_index(&self, t: i64) -> Option<usize> {
        let first = *self.times.first()?;
        if t < first { return None; }
        let i = ((t - first) / 3600) as usize;
        if i < self.times.len() { Some(i) } else { None }
    }
}

/// Today's date (UTC) as YYYY-MM-DD, for keys of forecast fetches so each day's forecast
/// is kept as its own file and can be verified later.
pub fn today_utc() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

/// GEFS ensemble members (via Open-Meteo's ensemble API, gfs_seamless) for the same
/// point grid: `past_days` of analysis plus `forecast_days` ahead, one WeatherGrid per
/// member (index 0 is the control). Keyed by today's date.
pub fn gefs_ensemble(fetcher: &mut Fetcher, lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, step: f64, past_days: u32, forecast_days: u32) -> Result<Vec<WeatherGrid>, String> {
    let mut lats = Vec::new();
    let mut la = (lat_s / step).floor() * step + step / 2.0;
    while la < lat_n + step / 2.0 { lats.push((la * 1e4).round() / 1e4); la += step; }
    let mut lons = Vec::new();
    let mut lo = (lon_w / step).floor() * step + step / 2.0;
    while lo < lon_e + step / 2.0 { lons.push((lo * 1e4).round() / 1e4); lo += step; }
    let mut pts_lat = Vec::new(); let mut pts_lon = Vec::new();
    for &a in &lats { for &o in &lons { pts_lat.push(format!("{a}")); pts_lon.push(format!("{o}")); } }
    let np = pts_lat.len();
    let url = format!("https://ensemble-api.open-meteo.com/v1/ensemble?latitude={}&longitude={}&hourly=temperature_2m,precipitation,shortwave_radiation,relative_humidity_2m,wind_speed_10m&models=gfs_seamless&past_days={past_days}&forecast_days={forecast_days}&timezone=UTC&wind_speed_unit=ms",
        pts_lat.join(","), pts_lon.join(","));
    let key = format!("open-meteo-gefs/{}_{}_{}_{}_{}deg_p{past_days}_f{forecast_days}_{}.json", lat_s, lat_n, lon_w, lon_e, step, today_utc());
    let path = fetcher.fetch(&key, &url)?;
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("open-meteo json: {e}"))?;
    let arr: Vec<serde_json::Value> = match v { serde_json::Value::Array(a) => a, other => vec![other] };
    if arr.len() != np { return Err(format!("ensemble returned {} points, expected {np}: {}", arr.len(), arr.first().map(|a| a["reason"].to_string()).unwrap_or_default())); }
    let h0 = &arr[0]["hourly"];
    let n_members = h0.as_object().map(|o| o.keys().filter(|k| k.starts_with("precipitation")).count()).unwrap_or(0);
    if n_members == 0 { return Err("ensemble: no members".into()); }
    let times: Vec<i64> = h0["time"].as_array().ok_or("no hourly.time")?.iter().map(|t| parse_iso_hour(t.as_str().unwrap_or(""))).collect::<Result<_, _>>()?;
    let mut members = Vec::with_capacity(n_members);
    for m in 0..n_members {
        let suffix = if m == 0 { String::new() } else { format!("_member{m:02}") };
        let mut g = WeatherGrid { lats: lats.clone(), lons: lons.clone(), times: times.clone(), ..Default::default() };
        g.t2m_c = vec![Vec::new(); np]; g.precip_mm = vec![Vec::new(); np]; g.sw_w_m2 = vec![Vec::new(); np]; g.rh_pct = vec![Vec::new(); np]; g.wind_m_s = vec![Vec::new(); np];
        for (p, item) in arr.iter().enumerate() {
            let h = &item["hourly"];
            let take = |name: &str, dst: &mut Vec<f32>| {
                for x in h[format!("{name}{suffix}")].as_array().cloned().unwrap_or_default() { dst.push(x.as_f64().map(|f| f as f32).unwrap_or(f32::NAN)); }
            };
            take("temperature_2m", &mut g.t2m_c[p]);
            take("precipitation", &mut g.precip_mm[p]);
            take("shortwave_radiation", &mut g.sw_w_m2[p]);
            take("relative_humidity_2m", &mut g.rh_pct[p]);
            take("wind_speed_10m", &mut g.wind_m_s[p]);
        }
        members.push(g);
    }
    Ok(members)
}
