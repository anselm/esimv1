//! Planetary-scale records: the global scoreboards and the one global forcing series.
//! Annual values keyed by calendar year. Every reader fetches through the same cache and
//! lock as the tiles, so a run pins the bytes it read.
//!
//! Sources (verified from canada1 on 2026-11-01, see data/catalog.toml):
//! - Our World in Data `owid-co2-data.csv` (built from the Global Carbon Project):
//!   the World rows give fossil and land-use-change CO2 in million tonnes per year.
//! - NOAA GML Mauna Loa annual mean CO2, 1959 to the last complete year, ppm.
//! - Law Dome ice core 2000-year spline (Etheridge et al.; MacFarling Meure et al. 2006),
//!   CO2 ppm per year to 2004, for the years before Mauna Loa.
//! - HadCRUT5 analysis global annual anomaly (deg C, relative to 1961-1990).
//! - NASA GISTEMP v4 land-ocean annual mean anomaly (J-D column, relative to 1951-1980).

use crate::fetch::Fetcher;
use std::collections::BTreeMap;

/// Grams of carbon per gram of CO2: 12.011 / 44.009.
pub const GTC_PER_GTCO2: f64 = 12.011 / 44.009;

/// Global fossil and land-use-change emissions in GtC per year, by calendar year.
#[derive(Clone, Debug, Default)]
pub struct Emissions {
    pub fossil_gtc: BTreeMap<i64, f64>,
    pub land_use_gtc: BTreeMap<i64, f64>,
}

pub fn owid_world_emissions(f: &mut Fetcher, url: &str) -> Result<Emissions, String> {
    let path = f.fetch("owid/owid-co2-data.csv", url)?;
    let text = String::from_utf8_lossy(&std::fs::read(&path).map_err(|e| e.to_string())?).into_owned();
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().ok_or("owid: empty")?.split(',').collect();
    let col = |n: &str| header.iter().position(|h| *h == n).ok_or_else(|| format!("owid: no column {n}"));
    let (cy, cco2, cluc) = (col("year")?, col("co2")?, col("land_use_change_co2")?);
    let mut e = Emissions::default();
    for line in lines {
        if !line.starts_with("World,") { continue; }
        let f: Vec<&str> = line.split(',').collect();
        let Some(y) = f.get(cy).and_then(|s| s.parse::<i64>().ok()) else { continue };
        // million tonnes CO2 -> GtC
        if let Some(v) = f.get(cco2).and_then(|s| s.parse::<f64>().ok()) { e.fossil_gtc.insert(y, v / 1000.0 * GTC_PER_GTCO2); }
        if let Some(v) = f.get(cluc).and_then(|s| s.parse::<f64>().ok()) { e.land_use_gtc.insert(y, v / 1000.0 * GTC_PER_GTCO2); }
    }
    if e.fossil_gtc.is_empty() { return Err("owid: no World rows parsed".into()); }
    Ok(e)
}

/// Mauna Loa annual mean CO2 (ppm) by year.
pub fn mauna_loa_annual(f: &mut Fetcher, url: &str) -> Result<BTreeMap<i64, f64>, String> {
    let path = f.fetch("noaa-gml/co2_annmean_mlo.csv", url)?;
    let text = String::from_utf8_lossy(&std::fs::read(&path).map_err(|e| e.to_string())?).into_owned();
    let mut out = BTreeMap::new();
    for line in text.lines() {
        if line.starts_with('#') || line.starts_with("year") { continue; }
        let f: Vec<&str> = line.split(',').collect();
        if let (Some(Ok(y)), Some(Ok(v))) = (f.first().map(|s| s.parse::<i64>()), f.get(1).map(|s| s.parse::<f64>())) { out.insert(y, v); }
    }
    if out.is_empty() { return Err("mauna loa: nothing parsed".into()); }
    Ok(out)
}

/// Law Dome spline CO2 (ppm) by year AD, columns 5 and 6 of the data table.
pub fn law_dome_co2(f: &mut Fetcher, url: &str) -> Result<BTreeMap<i64, f64>, String> { law_dome(f, url, 4, 5) }
/// Law Dome spline CH4 (ppb): columns 1 and 2.
pub fn law_dome_ch4(f: &mut Fetcher, url: &str) -> Result<BTreeMap<i64, f64>, String> { law_dome(f, url, 0, 1) }
/// Law Dome spline N2O (ppb): columns 8 and 9.
pub fn law_dome_n2o(f: &mut Fetcher, url: &str) -> Result<BTreeMap<i64, f64>, String> { law_dome(f, url, 7, 8) }

fn law_dome(f: &mut Fetcher, url: &str, ycol: usize, vcol: usize) -> Result<BTreeMap<i64, f64>, String> {
    let path = f.fetch("noaa-paleo/law2006.txt", url)?;
    let text = String::from_utf8_lossy(&std::fs::read(&path).map_err(|e| e.to_string())?).into_owned();
    let mut out = BTreeMap::new();
    let mut in_table = false;
    for line in text.lines() {
        if line.starts_with("YearAD") { in_table = true; continue; }
        if !in_table { continue; }
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() <= vcol.max(ycol) { continue; }
        let (Ok(y), Ok(v)) = (f[ycol].parse::<f64>(), f[vcol].parse::<f64>()) else { continue };
        out.insert(y.round() as i64, v);
    }
    if out.is_empty() { return Err("law dome: nothing parsed".into()); }
    Ok(out)
}

/// NOAA GML global annual mean of a gas (ppb for CH4 and N2O), same layout as Mauna Loa.
pub fn gml_annual(f: &mut Fetcher, key: &str, url: &str) -> Result<BTreeMap<i64, f64>, String> {
    let path = f.fetch(key, url)?;
    let text = String::from_utf8_lossy(&std::fs::read(&path).map_err(|e| e.to_string())?).into_owned();
    let mut out = BTreeMap::new();
    for line in text.lines() {
        if line.starts_with('#') || line.starts_with("year") { continue; }
        let f: Vec<&str> = line.split(',').collect();
        if let (Some(Ok(y)), Some(Ok(v))) = (f.first().map(|s| s.parse::<i64>()), f.get(1).map(|s| s.parse::<f64>())) { out.insert(y, v); }
    }
    if out.is_empty() { return Err(format!("{key}: nothing parsed")); }
    Ok(out)
}

/// HadCRUT5 global annual anomaly (deg C vs 1961-1990) by year.
pub fn hadcrut5_annual(f: &mut Fetcher, url: &str) -> Result<BTreeMap<i64, f64>, String> {
    let path = f.fetch("hadcrut5/global.annual.csv", url)?;
    let text = String::from_utf8_lossy(&std::fs::read(&path).map_err(|e| e.to_string())?).into_owned();
    let mut out = BTreeMap::new();
    for line in text.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if let (Some(Ok(y)), Some(Ok(v))) = (f.first().map(|s| s.parse::<i64>()), f.get(1).map(|s| s.parse::<f64>())) { out.insert(y, v); }
    }
    if out.is_empty() { return Err("hadcrut5: nothing parsed".into()); }
    Ok(out)
}

/// GISTEMP v4 land-ocean annual mean (J-D, deg C vs 1951-1980) by year; incomplete
/// years ("***") are skipped.
pub fn gistemp_annual(f: &mut Fetcher, url: &str) -> Result<BTreeMap<i64, f64>, String> {
    let path = f.fetch("gistemp/GLB.Ts+dSST.csv", url)?;
    let text = String::from_utf8_lossy(&std::fs::read(&path).map_err(|e| e.to_string())?).into_owned();
    let mut out = BTreeMap::new();
    let mut col = None;
    for line in text.lines() {
        let f: Vec<&str> = line.split(',').collect();
        if f.first() == Some(&"Year") { col = f.iter().position(|h| *h == "J-D"); continue; }
        let (Some(c), Some(Ok(y))) = (col, f.first().map(|s| s.parse::<i64>())) else { continue };
        if let Some(Ok(v)) = f.get(c).map(|s| s.parse::<f64>()) { out.insert(y, v); }
    }
    if out.is_empty() { return Err("gistemp: nothing parsed".into()); }
    Ok(out)
}

/// Mean of a yearly series over an inclusive window of years, if any values fall in it.
pub fn window_mean(s: &BTreeMap<i64, f64>, y0: i64, y1: i64) -> Option<f64> {
    let v: Vec<f64> = s.range(y0..=y1).map(|(_, v)| *v).collect();
    if v.is_empty() { None } else { Some(v.iter().sum::<f64>() / v.len() as f64) }
}

/// The IPCC AR6 WG1 effective radiative forcing time series, 1750 to 2019, one column per
/// component (W/m2), from the Chapter 7 author's repository (Smith et al.; the Zenodo
/// record 10.5281/zenodo.5705391 is the citable copy). Returns column name -> year -> value.
pub fn ar6_erf(f: &mut Fetcher, url: &str) -> Result<BTreeMap<String, BTreeMap<i64, f64>>, String> {
    let path = f.fetch("ipcc-ar6/AR6_ERF_1750-2019.csv", url)?;
    let text = String::from_utf8_lossy(&std::fs::read(&path).map_err(|e| e.to_string())?).into_owned();
    let mut lines = text.lines();
    let header: Vec<String> = lines.next().ok_or("ar6 erf: empty")?.split(',').map(|s| s.trim().to_string()).collect();
    let mut out: BTreeMap<String, BTreeMap<i64, f64>> = BTreeMap::new();
    for line in lines {
        let f: Vec<&str> = line.split(',').collect();
        let Some(Ok(y)) = f.first().map(|s| s.trim().parse::<i64>()) else { continue };
        for (i, h) in header.iter().enumerate().skip(1) {
            if let Some(Ok(v)) = f.get(i).map(|s| s.trim().parse::<f64>()) { out.entry(h.clone()).or_default().insert(y, v); }
        }
    }
    if out.is_empty() { return Err("ar6 erf: nothing parsed".into()); }
    Ok(out)
}
