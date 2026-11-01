//! MODIS vegetation indices (MOD13A1, 16-day, 500 m) from Microsoft's Planetary
//! Computer: STAC search, short-lived SAS signing (anonymous), COG fetch into the cache
//! keyed by item id, and sampling from the sinusoidal grid onto a lat-lon grid.
//! Quality flags are not yet applied; cloud-contaminated composites remain.

use crate::cog;
use crate::fetch::Fetcher;

pub const MODIS_SPHERE_R: f64 = 6_371_007.181;

#[derive(Clone, Debug)]
pub struct NdviComposite {
    pub id: String,
    /// composite start day, Unix seconds at 00:00 UTC
    pub day: i64,
    pub href: String,
}

fn stac_search(collection: &str, bbox: [f64; 4], start: &str, end: &str) -> Result<Vec<NdviComposite>, String> {
    let body = serde_json::json!({ "collections": [collection], "bbox": bbox, "datetime": format!("{start}T00:00:00Z/{end}T23:59:59Z"), "limit": 200 });
    let resp = ureq::post("https://planetarycomputer.microsoft.com/api/stac/v1/search").set("Content-Type", "application/json")
        .send_string(&body.to_string()).map_err(|e| format!("stac search: {e}"))?;
    let v: serde_json::Value = serde_json::from_str(&resp.into_string().map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for f in v["features"].as_array().cloned().unwrap_or_default() {
        let id = f["id"].as_str().unwrap_or("").to_string();
        let sd = f["properties"]["start_datetime"].as_str().unwrap_or("");
        if sd.len() < 10 { continue; }
        let p: Vec<i64> = sd[..10].split('-').filter_map(|x| x.parse().ok()).collect();
        if p.len() != 3 { continue; }
        let href = f["assets"]["500m_16_days_NDVI"]["href"].as_str().unwrap_or("").to_string();
        if href.is_empty() { continue; }
        out.push(NdviComposite { id, day: crate::obs::day_secs(p[0], p[1], p[2]), href });
    }
    out.sort_by_key(|c| c.day);
    Ok(out)
}

/// One read token for the whole MODIS container (valid about forty-five minutes), so a
/// run signs once however many files it needs. Retries on 429 with a pause.
fn container_token() -> Result<String, String> {
    let url = "https://planetarycomputer.microsoft.com/api/sas/v1/token/modiseuwest/modis-061-cogs";
    let mut last = String::new();
    for attempt in 0..5 {
        match ureq::get(url).call() {
            Ok(r) => {
                let text = r.into_string().map_err(|e| e.to_string())?;
                let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                return v["token"].as_str().map(|s| s.to_string()).ok_or("sas token: no token".into());
            }
            Err(e) => { last = e.to_string(); std::thread::sleep(std::time::Duration::from_secs(3 * (attempt + 1))); }
        }
    }
    Err(format!("sas token: {last}"))
}

/// Basin-grid NDVI frames for every MOD13A1 composite whose start lies in [start, end].
/// Returns (day, frame of NDVI in [-0.2, 1], NaN where nodata) for a lat-lon grid given
/// as (nx, ny, dlon, dlat, lon_w, lat_n). The STAC search itself is cached by its
/// arguments so a rerun is offline.
pub fn ndvi_frames(fetcher: &mut Fetcher, grid: (usize, usize, f64, f64, f64, f64), bbox: [f64; 4], start: &str, end: &str) -> Result<Vec<(i64, Vec<f32>)>, String> {
    let (nx, ny, dlon, dlat, lon_w, lat_n) = grid;
    let list_key = format!("modis-13a1/search_{}_{}_{}_{}_{start}_{end}.json", bbox[0], bbox[1], bbox[2], bbox[3]);
    let list_path = fetcher.cache_dir.join(list_key.replace(['/', ' '], "_"));
    let items: Vec<NdviComposite> = if let Ok(t) = std::fs::read_to_string(&list_path) {
        serde_json::from_str::<Vec<(String, i64, String)>>(&t).map_err(|e| e.to_string())?.into_iter().map(|(id, day, href)| NdviComposite { id, day, href }).collect()
    } else {
        let items = stac_search("modis-13A1-061", bbox, start, end)?;
        let raw: Vec<(String, i64, String)> = items.iter().map(|c| (c.id.clone(), c.day, c.href.clone())).collect();
        std::fs::write(&list_path, serde_json::to_string(&raw).unwrap()).map_err(|e| e.to_string())?;
        items
    };
    // one tile per date is assumed for this basin (h11v04); if two tiles cover a bbox,
    // frames would need mosaicking, which is not done yet
    let mut out = Vec::new();
    let mut token: Option<String> = None;
    for it in items {
        let key = format!("modis-13a1/{}_NDVI.tif", it.id);
        let path = match fetcher.cache_dir.join(key.replace(['/', ' '], "_")).exists() {
            true => fetcher.fetch(&key, &it.href)?,          // present: no signing needed
            false => {
                if token.is_none() { token = Some(container_token()?); }
                let signed = format!("{}?{}", it.href, token.as_ref().unwrap());
                fetcher.fetch(&key, &signed)?
            }
        };
        let r = cog::read(&path)?;
        let mut frame = vec![f32::NAN; nx * ny];
        for row in 0..ny {
            let lat = (lat_n - (row as f64 + 0.5) * dlat).to_radians();
            let y = MODIS_SPHERE_R * lat;
            let rr = ((r.y0 - y) / r.dy).floor();
            if rr < 0.0 || rr >= r.height as f64 { continue; }
            for col in 0..nx {
                let lon = (lon_w + (col as f64 + 0.5) * dlon).to_radians();
                let x = MODIS_SPHERE_R * lon * lat.cos();
                let cc = ((x - r.x0) / r.dx).floor();
                if cc < 0.0 || cc >= r.width as f64 { continue; }
                let v = r.get(cc as usize, rr as usize);
                if Some(v) == r.nodata || v < -2000.0 { continue; }
                frame[row * nx + col] = v / 10_000.0;
            }
        }
        out.push((it.day, frame));
    }
    Ok(out)
}
