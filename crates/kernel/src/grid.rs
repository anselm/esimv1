//! Grids. A field module runs on whatever grid it is handed and never knows which.
//! Area is a property of the grid, never an assumption, because ledgers need it.
//! M0 carries only the local projected tile; regional and global grids arrive with M1.

use serde::{Deserialize, Serialize};

pub trait Grid {
    fn len(&self) -> usize;
    /// Cell area in square metres.
    fn area(&self, cell: usize) -> f64;
    /// Four-neighbourhood, in-bounds only.
    fn neighbors4(&self, cell: usize) -> Vec<usize>;
}

/// A projected tile in metres: `nx` by `ny` cells of `dx` by `dy`, origin at the
/// south-west corner. `crs` names the projection the coordinates are in.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalTile {
    pub nx: usize,
    pub ny: usize,
    pub dx: f64,
    pub dy: f64,
    pub x0: f64,
    pub y0: f64,
    pub crs: String,
}

impl LocalTile {
    pub fn index(&self, i: usize, j: usize) -> usize { j * self.nx + i }
    pub fn ij(&self, cell: usize) -> (usize, usize) { (cell % self.nx, cell / self.nx) }
}

impl Grid for LocalTile {
    fn len(&self) -> usize { self.nx * self.ny }
    fn area(&self, _cell: usize) -> f64 { self.dx * self.dy }
    fn neighbors4(&self, cell: usize) -> Vec<usize> {
        let (i, j) = self.ij(cell);
        let mut v = Vec::with_capacity(4);
        if i > 0 { v.push(self.index(i - 1, j)); }
        if i + 1 < self.nx { v.push(self.index(i + 1, j)); }
        if j > 0 { v.push(self.index(i, j - 1)); }
        if j + 1 < self.ny { v.push(self.index(i, j + 1)); }
        v
    }
}

/// A geographic grid in degrees on WGS 84: `nx` by `ny` cells of `dlon` by `dlat`
/// degrees, north edge at `lat_n`, west edge at `lon_w`, rows from north to south as
/// rasters come. Cell area is the spherical quadrilateral's true area, so a ledger
/// summed over cells is honest at any latitude.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LatLonGrid {
    pub nx: usize,
    pub ny: usize,
    pub dlon: f64,
    pub dlat: f64,
    pub lon_w: f64,
    pub lat_n: f64,
}

pub const EARTH_RADIUS_M: f64 = 6_371_008.8;

impl LatLonGrid {
    pub fn index(&self, col: usize, row: usize) -> usize { row * self.nx + col }
    pub fn col_row(&self, cell: usize) -> (usize, usize) { (cell % self.nx, cell / self.nx) }
    /// Centre of a cell.
    pub fn lon(&self, col: usize) -> f64 { self.lon_w + (col as f64 + 0.5) * self.dlon }
    pub fn lat(&self, row: usize) -> f64 { self.lat_n - (row as f64 + 0.5) * self.dlat }
    pub fn lat_s(&self) -> f64 { self.lat_n - self.ny as f64 * self.dlat }
    pub fn lon_e(&self) -> f64 { self.lon_w + self.nx as f64 * self.dlon }
    /// Area of every cell in a row (constant along a row).
    pub fn row_area(&self, row: usize) -> f64 {
        let top = (self.lat_n - row as f64 * self.dlat).to_radians();
        let bot = (self.lat_n - (row + 1) as f64 * self.dlat).to_radians();
        EARTH_RADIUS_M * EARTH_RADIUS_M * self.dlon.to_radians() * (top.sin() - bot.sin())
    }
    /// Metres per degree of longitude and latitude at a row's centre.
    pub fn metres_per_degree(&self, row: usize) -> (f64, f64) {
        let lat = self.lat(row).to_radians();
        let per_deg = EARTH_RADIUS_M * std::f64::consts::PI / 180.0;
        (per_deg * lat.cos(), per_deg)
    }
    pub fn total_area(&self) -> f64 { (0..self.ny).map(|r| self.row_area(r) * self.nx as f64).sum() }
}

impl Grid for LatLonGrid {
    fn len(&self) -> usize { self.nx * self.ny }
    fn area(&self, cell: usize) -> f64 { self.row_area(cell / self.nx) }
    fn neighbors4(&self, cell: usize) -> Vec<usize> {
        let (i, j) = self.col_row(cell);
        let mut v = Vec::with_capacity(4);
        if i > 0 { v.push(self.index(i - 1, j)); }
        if i + 1 < self.nx { v.push(self.index(i + 1, j)); }
        if j > 0 { v.push(self.index(i, j - 1)); }
        if j + 1 < self.ny { v.push(self.index(i, j + 1)); }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_degree_cell_at_equator_is_about_12_400_km2() {
        let g = LatLonGrid { nx: 1, ny: 1, dlon: 1.0, dlat: 1.0, lon_w: 0.0, lat_n: 0.5 };
        let km2 = g.area(0) / 1e6;
        assert!((km2 - 12_364.0).abs() < 30.0, "{km2}");
    }
    #[test]
    fn whole_sphere() {
        let g = LatLonGrid { nx: 36, ny: 18, dlon: 10.0, dlat: 10.0, lon_w: -180.0, lat_n: 90.0 };
        let sphere = 4.0 * std::f64::consts::PI * EARTH_RADIUS_M * EARTH_RADIUS_M;
        assert!((g.total_area() / sphere - 1.0).abs() < 1e-9);
    }
}
