//! World Bank indicators through the public API v2 (World Development Indicators and, under
//! `source=3`, the Worldwide Governance Indicators), anonymous JSON. One request per
//! indicator returns every country's latest value (`mrv=1`) or every value in a year range;
//! the raw answer is hashed into the lock and reduced to the latest value per ISO3 code.
use crate::Fetcher;
use std::collections::BTreeMap;

/// Latest value per ISO3 country code for one indicator: `code` is the API's indicator id
/// (`SP.DYN.LE00.IN`, or `GOV_WGI_VA.EST` with `source = Some(3)`); `date` is an API date
/// expression (`"2012:2025"`) for sparse series, else the most recent value is asked for.
/// Aggregates (regions, income groups) come back with an empty ISO3 and are dropped.
pub fn latest_by_country(f: &mut Fetcher, url_template: &str, code: &str, source: Option<u32>, date: Option<&str>) -> Result<BTreeMap<String, (i64, f64)>, String> {
    let mut url = url_template.replace("{code}", code);
    url.push_str(match date { Some(d) => &*format!("&date={d}").leak(), None => "&mrv=1" });
    if let Some(s) = source { url.push_str(&format!("&source={s}")); }
    let key = format!("worldbank-wdi/{}-{}.json", code, date.map(|d| d.replace(':', "-")).unwrap_or_else(|| "latest".into()));
    let path = f.fetch_reduced(&key, &url, &|raw| {
        let v: serde_json::Value = serde_json::from_slice(raw).map_err(|e| format!("worldbank {code}: {e}"))?;
        let rows = v.get(1).and_then(|r| r.as_array()).ok_or_else(|| format!("worldbank {code}: no rows ({})", String::from_utf8_lossy(&raw[..raw.len().min(200)])))?;
        let mut latest: BTreeMap<String, (i64, f64)> = BTreeMap::new();
        for r in rows {
            let iso = r["countryiso3code"].as_str().unwrap_or("");
            let (Some(val), Some(y)) = (r["value"].as_f64(), r["date"].as_str().and_then(|d| d.parse::<i64>().ok())) else { continue };
            if iso.len() != 3 { continue; }
            match latest.get(iso) { Some((ly, _)) if *ly >= y => {}, _ => { latest.insert(iso.to_string(), (y, val)); } }
        }
        if latest.is_empty() { return Err(format!("worldbank {code}: no country values")); }
        serde_json::to_vec(&latest).map_err(|e| e.to_string())
    })?;
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| format!("worldbank {code}: cached: {e}"))
}
