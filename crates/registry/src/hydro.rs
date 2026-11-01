//! Flow routing layers from a DEM window: depression filling (priority flood, Barnes
//! et al. 2014), D8 flow direction, flow accumulation in cells and square metres, and
//! the basin above a pour point. Cached like the other derived layers.

use crate::fetch::Fetcher;
use crate::terrain::DemWindow;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// D8 neighbour offsets, clockwise from east: (dcol, drow).
pub const D8: [(isize, isize); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];

#[derive(Clone, Debug)]
pub struct FlowNet {
    pub nx: usize,
    pub ny: usize,
    /// filled elevation
    pub filled: Vec<f32>,
    /// index of the downstream cell, or usize::MAX at an outlet or edge
    pub down: Vec<usize>,
    /// upstream cell count including self
    pub accum: Vec<u32>,
    /// upstream area in m2 including self
    pub accum_m2: Vec<f32>,
    /// cells in upstream-to-downstream order (a topological order)
    pub order: Vec<usize>,
}

#[derive(PartialEq)]
struct Pq(f32, usize);
impl Eq for Pq {}
impl PartialOrd for Pq { fn partial_cmp(&self, o: &Self) -> Option<Ordering> { Some(self.cmp(o)) } }
impl Ord for Pq { fn cmp(&self, o: &Self) -> Ordering { o.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal).then(o.1.cmp(&self.1)) } }

fn read_cache(f: &Fetcher, key: &str) -> Option<Vec<u8>> { std::fs::read(f.cache_dir.join("derived").join(key)).ok() }
fn write_cache(f: &Fetcher, key: &str, b: &[u8]) { let _ = std::fs::create_dir_all(f.cache_dir.join("derived")); let _ = std::fs::write(f.cache_dir.join("derived").join(key), b); }

impl FlowNet {
    /// Build or load the flow network for a DEM window. Cells with no data drain nowhere.
    pub fn build(fetcher: &Fetcher, dem: &DemWindow, row_area_m2: &[f64]) -> FlowNet {
        let (nx, ny) = (dem.nx, dem.ny);
        let n = nx * ny;
        let key = format!("flownet-v1-{}.bin", &dem.provenance[..24]);
        if let Some(b) = read_cache(fetcher, &key) {
            if b.len() == n * (4 + 8 + 4 + 4) {
                let mut off = 0;
                let filled: Vec<f32> = (0..n).map(|i| f32::from_le_bytes(b[off + i * 4..off + i * 4 + 4].try_into().unwrap())).collect(); off += n * 4;
                let down: Vec<usize> = (0..n).map(|i| u64::from_le_bytes(b[off + i * 8..off + i * 8 + 8].try_into().unwrap()) as usize).collect(); off += n * 8;
                let accum: Vec<u32> = (0..n).map(|i| u32::from_le_bytes(b[off + i * 4..off + i * 4 + 4].try_into().unwrap())).collect(); off += n * 4;
                let accum_m2: Vec<f32> = (0..n).map(|i| f32::from_le_bytes(b[off + i * 4..off + i * 4 + 4].try_into().unwrap())).collect();
                let order = topo_order(&down, n);
                return FlowNet { nx, ny, filled, down, accum, accum_m2, order };
            }
        }
        // Priority flood with a tiny epsilon so flats drain.
        let mut filled = vec![f32::NAN; n];
        let mut closed = vec![false; n];
        let mut heap = BinaryHeap::new();
        let valid = |i: usize| dem.elev[i].is_finite();
        for i in 0..n {
            let (c, r) = (i % nx, i / nx);
            let edge = c == 0 || r == 0 || c == nx - 1 || r == ny - 1;
            let next_to_nodata = D8.iter().any(|(dc, dr)| {
                let (cc, rr) = (c as isize + dc, r as isize + dr);
                cc >= 0 && rr >= 0 && (cc as usize) < nx && (rr as usize) < ny && !valid(rr as usize * nx + cc as usize)
            });
            if valid(i) && (edge || next_to_nodata) { heap.push(Pq(dem.elev[i], i)); closed[i] = true; filled[i] = dem.elev[i]; }
            if !valid(i) { closed[i] = true; }
        }
        let eps = 1e-3f32;
        while let Some(Pq(z, i)) = heap.pop() {
            let (c, r) = (i % nx, i / nx);
            for (dc, dr) in D8 {
                let (cc, rr) = (c as isize + dc, r as isize + dr);
                if cc < 0 || rr < 0 || cc as usize >= nx || rr as usize >= ny { continue; }
                let j = rr as usize * nx + cc as usize;
                if closed[j] { continue; }
                closed[j] = true;
                filled[j] = dem.elev[j].max(z + eps);
                heap.push(Pq(filled[j], j));
            }
        }
        // D8 on the filled surface, steepest descent with distance weighting.
        let mut down = vec![usize::MAX; n];
        for i in 0..n {
            if !filled[i].is_finite() { continue; }
            let (c, r) = (i % nx, i / nx);
            let (dxm, dym) = metres(dem, r);
            let mut best = 0.0f64; let mut bj = usize::MAX;
            for (dc, dr) in D8 {
                let (cc, rr) = (c as isize + dc, r as isize + dr);
                if cc < 0 || rr < 0 || cc as usize >= nx || rr as usize >= ny { continue; }
                let j = rr as usize * nx + cc as usize;
                if !filled[j].is_finite() { continue; }
                let dist = ((dc as f64 * dxm).powi(2) + (dr as f64 * dym).powi(2)).sqrt();
                let s = (filled[i] - filled[j]) as f64 / dist;
                if s > best { best = s; bj = j; }
            }
            down[i] = bj;
        }
        let order = topo_order(&down, n);
        let mut accum = vec![0u32; n];
        let mut accum_m2 = vec![0f32; n];
        for &i in &order {
            if !filled[i].is_finite() { continue; }
            accum[i] += 1;
            accum_m2[i] += row_area_m2[i / nx] as f32;
            let j = down[i];
            if j != usize::MAX { accum[j] += accum[i]; accum_m2[j] += accum_m2[i]; }
        }
        let mut b = Vec::with_capacity(n * 20);
        for v in &filled { b.extend_from_slice(&v.to_le_bytes()); }
        for v in &down { b.extend_from_slice(&(*v as u64).to_le_bytes()); }
        for v in &accum { b.extend_from_slice(&v.to_le_bytes()); }
        for v in &accum_m2 { b.extend_from_slice(&v.to_le_bytes()); }
        write_cache(fetcher, &key, &b);
        FlowNet { nx, ny, filled, down, accum, accum_m2, order }
    }

    /// Snap a point to the cell of greatest accumulation within `radius` cells.
    pub fn snap(&self, col: usize, row: usize, radius: usize) -> usize {
        let mut best = row * self.nx + col; let mut ba = 0u32;
        for r in row.saturating_sub(radius)..=(row + radius).min(self.ny - 1) {
            for c in col.saturating_sub(radius)..=(col + radius).min(self.nx - 1) {
                let i = r * self.nx + c;
                if self.accum[i] > ba { ba = self.accum[i]; best = i; }
            }
        }
        best
    }

    /// Mask of cells draining to `pour` (inclusive).
    pub fn basin(&self, pour: usize) -> Vec<bool> {
        let n = self.nx * self.ny;
        let mut m = vec![false; n];
        m[pour] = true;
        // walk in downstream-to-upstream order: a cell is in if its downstream is in
        for &i in self.order.iter().rev() {
            if i == pour { continue; }
            let j = self.down[i];
            if j != usize::MAX && m[j] { m[i] = true; }
        }
        m
    }
}

fn metres(dem: &DemWindow, row: usize) -> (f64, f64) {
    let per_deg = crate::EARTH_RADIUS_M * std::f64::consts::PI / 180.0;
    (per_deg * dem.lat(row).to_radians().cos() * dem.dlon, per_deg * dem.dlat)
}

/// Kahn's algorithm over the D8 tree: every cell appears after all its upstream cells.
fn topo_order(down: &[usize], n: usize) -> Vec<usize> {
    let mut indeg = vec![0u32; n];
    for &j in down { if j != usize::MAX { indeg[j] += 1; } }
    let mut stack: Vec<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
    stack.reverse();
    let mut order = Vec::with_capacity(n);
    while let Some(i) = stack.pop() {
        order.push(i);
        let j = down[i];
        if j != usize::MAX { indeg[j] -= 1; if indeg[j] == 0 { stack.push(j); } }
    }
    order
}
