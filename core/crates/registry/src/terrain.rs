//! Terrain layers derived from the elevation base layer: a window of the DEM at a
//! chosen resolution, slope and aspect, and horizon angles for shading. Each derived
//! layer is cached under data/cache/derived keyed by its inputs, so it is computed
//! once per box however many runs ask for it (an overview, in GIS terms).

use crate::catalog::{tile_key, Asset};
use crate::cog;
use crate::fetch::Fetcher;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// A regular lat-lon window of elevations, rows north to south.
#[derive(Clone, Debug)]
pub struct DemWindow {
    pub nx: usize,
    pub ny: usize,
    pub dlon: f64,
    pub dlat: f64,
    pub lon_w: f64,
    pub lat_n: f64,
    pub elev: Vec<f32>,
    /// sha256 of the inputs (tile hashes, bbox, resolution), for cache keys and provenance.
    pub provenance: String,
}

const RECIPE_VERSION: &str = "terrain-v4"; // v2: reader honours PixelIsPoint; v3: grid laid half a pixel off the lattice (no empty columns); v4: cells no pixel centre fell in take the nearest pixel (tiles coarser than the grid, as GLO-30 is in longitude above 50°N)
const SOURCE_ARCSEC: f64 = 1.0; // Copernicus GLO-30 below 50 degrees of latitude

fn derived_path(fetcher: &Fetcher, key: &str) -> PathBuf {
    let dir = fetcher.cache_dir.join("derived");
    let _ = std::fs::create_dir_all(&dir);
    dir.join(key)
}

fn read_f32s(path: &std::path::Path) -> Option<Vec<f32>> {
    let b = std::fs::read(path).ok()?;
    Some(b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect())
}
fn write_f32s(path: &std::path::Path, v: &[f32]) {
    let bytes: Vec<u8> = v.iter().flat_map(|x| x.to_le_bytes()).collect();
    let _ = std::fs::write(path, bytes);
}

/// Build (or load from cache) a window `[lat_s, lat_n] x [lon_w, lon_e]` at
/// `res_arcsec` resolution by block-averaging the one-arc-second tiles that cover it.
/// The window is snapped outward to whole output cells.
pub fn dem_window(fetcher: &mut Fetcher, dem: &Asset, lat_s: f64, lat_n: f64, lon_w: f64, lon_e: f64, res_arcsec: f64) -> Result<DemWindow, String> {
    let d = res_arcsec / 3600.0;
    // GLO-30's pixel centres sit on whole arc-seconds. An output grid whose edges also sit on
    // whole arc-seconds puts every source centre exactly on an edge, and floating-point rounding
    // then decides which cell it falls in: at one arc-second one column in three or four got no
    // pixel at all and came out NaN, which the slope code read as a 400 m trench (found 2026-11-03
    // in the Palisades window). So the grid is laid half a source pixel off the lattice: at the
    // DEM's own resolution each cell is centred on its pixel, and at coarser ones no pixel centre
    // lies on an edge.
    let half = SOURCE_ARCSEC / 3600.0 / 2.0;
    let lon_w = (lon_w / d).floor() * d - half;
    let lat_n = (lat_n / d).ceil() * d + half;
    let nx = ((lon_e - lon_w) / d).ceil() as usize;
    let ny = ((lat_n - lat_s) / d).ceil() as usize;
    // tiles needed
    let mut keys = Vec::new();
    let mut la = (lat_n - 1e-9).floor() as i64;
    let la_s = lat_s.floor() as i64;
    while la >= la_s {
        let mut lo = lon_w.floor() as i64;
        let lo_e = (lon_w + nx as f64 * d - 1e-9).floor() as i64;
        while lo <= lo_e { keys.push(tile_key(la as f64 + 0.5, lo as f64 + 0.5)); lo += 1; }
        la -= 1;
    }
    let mut paths = Vec::new();
    for k in &keys {
        let url = dem.tile_url(k)?;
        paths.push((k.clone(), fetcher.fetch(&format!("{}/{}.tif", dem.id, k), &url)?));
    }
    let mut h = Sha256::new();
    for (k, _) in &paths { h.update(k.as_bytes()); h.update(fetcher.lock.files[&format!("{}/{}.tif", dem.id, k)].sha256.as_bytes()); }
    h.update(format!("{RECIPE_VERSION}|{lon_w:.9}|{lat_n:.9}|{nx}|{ny}|{d:.12}").as_bytes());
    let provenance = format!("{:x}", h.finalize());
    let cache = derived_path(fetcher, &format!("dem-{}.f32", &provenance[..24]));
    if let Some(elev) = read_f32s(&cache) {
        if elev.len() == nx * ny {
            return Ok(DemWindow { nx, ny, dlon: d, dlat: d, lon_w, lat_n, elev, provenance });
        }
    }
    let mut sum = vec![0.0f64; nx * ny];
    let mut cnt = vec![0u32; nx * ny];
    for (_, p) in &paths {
        let r = cog::read(p)?;
        if ((r.dx * 3600.0) - SOURCE_ARCSEC).abs() > 1e-6 { eprintln!("dem: tile pixel is {:.4} arcsec, not {SOURCE_ARCSEC}; the grid may not sit on its centres", r.dx * 3600.0); }
        // map every source pixel centre into an output cell
        for row in 0..r.height {
            let lat = r.lat(row);
            if lat >= lat_n || lat < lat_n - ny as f64 * d { continue; }
            let orow = ((lat_n - lat) / d) as usize;
            for col in 0..r.width {
                let lon = r.lon(col);
                if lon < lon_w || lon >= lon_w + nx as f64 * d { continue; }
                let ocol = ((lon - lon_w) / d) as usize;
                if ocol >= nx || orow >= ny { continue; }
                let v = r.get(col, row);
                if !v.is_finite() || Some(v) == r.nodata { continue; }
                let i = orow * nx + ocol;
                sum[i] += v as f64; cnt[i] += 1;
            }
        }
    }
    let mut elev: Vec<f32> = sum.iter().zip(cnt.iter()).map(|(s, c)| if *c > 0 { (*s / *c as f64) as f32 } else { f32::NAN }).collect();
    // a cell no pixel centre fell in (the tile's pixels are wider than the grid: GLO-30 is 1.5 arcsec
    // in longitude from 50°N and coarser further north, found at Fort McMurray on 2026-11-04 as a
    // missing column in every three) takes the nearest pixel at its own centre
    let empty = cnt.iter().filter(|&&c| c == 0).count();
    if empty > 0 {
        let mut filled = 0usize;
        for (_, p) in &paths {
            let r = cog::read(p)?;
            for i in 0..nx * ny {
                if cnt[i] > 0 || elev[i].is_finite() { continue; }
                let (orow, ocol) = (i / nx, i % nx);
                let lat = lat_n - (orow as f64 + 0.5) * d; let lon = lon_w + (ocol as f64 + 0.5) * d;
                let col = ((lon - r.x0) / r.dx).floor(); let row = ((r.y0 - lat) / r.dy).floor();
                if col < 0.0 || row < 0.0 || col >= r.width as f64 || row >= r.height as f64 { continue; }
                let v = r.get(col as usize, row as usize);
                if !v.is_finite() || Some(v) == r.nodata { continue; }
                elev[i] = v; filled += 1;
            }
        }
        eprintln!("dem: {empty} of {} cells took no pixel centre; {filled} filled from the nearest pixel", nx * ny);
    }
    write_f32s(&cache, &elev);
    Ok(DemWindow { nx, ny, dlon: d, dlat: d, lon_w, lat_n, elev, provenance })
}

impl DemWindow {
    pub fn lat(&self, row: usize) -> f64 { self.lat_n - (row as f64 + 0.5) * self.dlat }
    pub fn lon(&self, col: usize) -> f64 { self.lon_w + (col as f64 + 0.5) * self.dlon }
    fn metres(&self, row: usize) -> (f64, f64) {
        let per_deg = crate::EARTH_RADIUS_M * std::f64::consts::PI / 180.0;
        (per_deg * self.lat(row).to_radians().cos() * self.dlon, per_deg * self.dlat)
    }
    fn at(&self, col: isize, row: isize) -> f32 {
        let c = col.clamp(0, self.nx as isize - 1) as usize;
        let r = row.clamp(0, self.ny as isize - 1) as usize;
        let v = self.elev[r * self.nx + c];
        if v.is_finite() { v } else { 0.0 }
    }

    /// Slope (radians from horizontal) and aspect (radians clockwise from north, the
    /// downhill direction) by Horn's method, as GIS tools compute them.
    pub fn slope_aspect(&self, fetcher: &Fetcher) -> (Vec<f32>, Vec<f32>) {
        let ps = derived_path(fetcher, &format!("slope-{}.f32", &self.provenance[..24]));
        let pa = derived_path(fetcher, &format!("aspect-{}.f32", &self.provenance[..24]));
        if let (Some(s), Some(a)) = (read_f32s(&ps), read_f32s(&pa)) {
            if s.len() == self.elev.len() && a.len() == self.elev.len() { return (s, a); }
        }
        let n = self.nx * self.ny;
        let mut slope = vec![0f32; n];
        let mut aspect = vec![0f32; n];
        for row in 0..self.ny {
            let (dx, dy) = self.metres(row);
            for col in 0..self.nx {
                let (c, r) = (col as isize, row as isize);
                let a = self.at(c - 1, r - 1) as f64; let b = self.at(c, r - 1) as f64; let cc = self.at(c + 1, r - 1) as f64;
                let d = self.at(c - 1, r) as f64; let f = self.at(c + 1, r) as f64;
                let g = self.at(c - 1, r + 1) as f64; let h = self.at(c, r + 1) as f64; let i = self.at(c + 1, r + 1) as f64;
                let dzdx = ((cc + 2.0 * f + i) - (a + 2.0 * d + g)) / (8.0 * dx);
                let dzdy = ((g + 2.0 * h + i) - (a + 2.0 * b + cc)) / (8.0 * dy); // positive southward
                let s = (dzdx * dzdx + dzdy * dzdy).sqrt().atan();
                // downhill direction: -grad; grad = (dzdx east, -dzdy north)
                let east = -dzdx; let north = dzdy;
                let mut az = east.atan2(north); // clockwise from north
                if az < 0.0 { az += std::f64::consts::TAU; }
                slope[row * self.nx + col] = s as f32;
                aspect[row * self.nx + col] = if s < 1e-6 { f32::NAN } else { az as f32 };
            }
        }
        write_f32s(&ps, &slope); write_f32s(&pa, &aspect);
        (slope, aspect)
    }

    /// Horizon elevation angle (radians above horizontal) in `n_az` azimuths clockwise
    /// from north for every cell, by marching rays out to `max_km`. Returned flat:
    /// `[cell * n_az + az]`. Zero means an open horizon.
    pub fn horizons(&self, fetcher: &Fetcher, n_az: usize, max_km: f64) -> Vec<f32> {
        let p = derived_path(fetcher, &format!("horizon{n_az}-{}km-{}.f32", max_km as i64, &self.provenance[..24]));
        if let Some(h) = read_f32s(&p) { if h.len() == self.elev.len() * n_az { return h; } }
        let mut out = vec![0f32; self.elev.len() * n_az];
        for row in 0..self.ny {
            let (dx, dy) = self.metres(row);
            let step = dx.min(dy);
            let steps = ((max_km * 1000.0) / step) as usize;
            for col in 0..self.nx {
                let z0 = self.at(col as isize, row as isize) as f64;
                for k in 0..n_az {
                    let az = (k as f64) * std::f64::consts::TAU / n_az as f64;
                    let (ue, un) = (az.sin(), az.cos());
                    let mut best = 0.0f64;
                    let mut s = 1usize;
                    while s <= steps {
                        let dist = s as f64 * step;
                        let c = col as f64 + ue * dist / dx;
                        let r = row as f64 - un * dist / dy;
                        if c < 0.0 || r < 0.0 || c >= self.nx as f64 || r >= self.ny as f64 { break; }
                        let z = self.at(c as isize, r as isize) as f64;
                        let ang = ((z - z0) / dist).atan();
                        if ang > best { best = ang; }
                        // coarsen the march with distance: terrain far away matters less per metre
                        s += 1 + s / 8;
                    }
                    out[(row * self.nx + col) * n_az + k] = best as f32;
                }
            }
        }
        write_f32s(&p, &out);
        out
    }
}
