//! Shared world state: named fields on grids, written only by their owning module and
//! only through the context, so every write is an ordered, logged event. Readers see
//! the latest write at or before their own time, which with in-order delivery is the
//! conservative rule of the brief in its single-threaded form.

use crate::grid::LatLonGrid;
use crate::time::Time;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldField {
    pub unit: String,
    pub grid: LatLonGrid,
    pub data: Vec<f32>,
    pub owner: String,
    pub written_at: Time,
    pub version: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct World {
    pub fields: BTreeMap<String, WorldField>,
}

impl World {
    pub fn get(&self, name: &str) -> Option<&WorldField> { self.fields.get(name) }
    /// Returns the new version and a digest of the data, for the log.
    pub(crate) fn write(&mut self, owner: &str, name: &str, unit: &str, grid: &LatLonGrid, data: Vec<f32>, now: Time) -> Result<(u64, u64), String> {
        if data.len() != grid.nx * grid.ny { return Err(format!("world field {name}: {} values for a {}x{} grid", data.len(), grid.nx, grid.ny)); }
        let bytes: Vec<u8> = data.iter().flat_map(|v| v.to_le_bytes()).collect();
        let digest = crate::hash::fnv1a(&bytes);
        match self.fields.get_mut(name) {
            Some(f) => {
                if f.owner != owner { return Err(format!("world field {name} is owned by {}, not {owner}", f.owner)); }
                if &f.grid != grid { return Err(format!("world field {name}: grid changed")); }
                f.data = data; f.written_at = now; f.version += 1;
                Ok((f.version, digest))
            }
            None => {
                self.fields.insert(name.to_string(), WorldField { unit: unit.to_string(), grid: grid.clone(), data, owner: owner.to_string(), written_at: now, version: 1 });
                Ok((1, digest))
            }
        }
    }
}
