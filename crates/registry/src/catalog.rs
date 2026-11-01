use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Catalog {
    #[serde(default, rename = "asset")]
    pub assets: Vec<Asset>,
    #[serde(default, rename = "place")]
    pub places: Vec<Place>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub title: String,
    pub kind: String,
    #[serde(default)] pub crs: String,
    #[serde(default)] pub resolution: String,
    #[serde(default)] pub units: String,
    #[serde(default)] pub format: String,
    #[serde(default)] pub url_template: String,
    #[serde(default)] pub tile_list: String,
    #[serde(default)] pub source: String,
    #[serde(default)] pub licence: String,
    #[serde(default)] pub verified: String,
    #[serde(default)] pub notes: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Place {
    pub id: String,
    pub title: String,
    #[serde(default)] pub why: String,
    #[serde(default)] pub gauge: String,
    #[serde(default)] pub verified: String,
    #[serde(default)] pub dem_tiles: Vec<String>,
    #[serde(default)] pub notes: String,
}

impl Catalog {
    pub fn load(path: &Path) -> Result<Catalog, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        toml::from_str(&text).map_err(|e| format!("catalog: {e}"))
    }
    pub fn asset(&self, id: &str) -> Option<&Asset> { self.assets.iter().find(|a| a.id == id) }
    pub fn place(&self, id: &str) -> Option<&Place> { self.places.iter().find(|p| p.id == id) }
}

impl Asset {
    /// URL of a one-degree tile keyed like "N42_W094" (latitude floor, longitude floor).
    pub fn tile_url(&self, key: &str) -> Result<String, String> {
        let (lat, lon) = key.split_once('_').ok_or_else(|| format!("tile key {key}"))?;
        Ok(self.url_template.replace("{lat}", lat).replace("{lon}", lon))
    }
}

/// Tile key for the one-degree cell containing a point: floor of lat and lon.
pub fn tile_key(lat: f64, lon: f64) -> String {
    let la = lat.floor() as i32;
    let lo = lon.floor() as i32;
    format!("{}{:02}_{}{:03}", if la < 0 { "S" } else { "N" }, la.abs(), if lo < 0 { "W" } else { "E" }, lo.abs())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keys() {
        assert_eq!(tile_key(42.43, -93.81), "N42_W094");
        assert_eq!(tile_key(-0.5, 6.2), "S01_E006");
    }
}
