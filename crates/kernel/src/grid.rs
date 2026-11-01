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
