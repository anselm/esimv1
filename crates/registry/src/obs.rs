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
