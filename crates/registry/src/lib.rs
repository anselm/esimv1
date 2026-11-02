//! The registry: the index of sources, the fetcher, and the readers. Nothing in the
//! kernel knows where data lives; it asks here.

pub mod catalog;
pub mod fetch;
pub mod cog;
pub mod terrain;
pub mod hydro;
pub mod obs;
pub mod modis;
pub mod events;
pub mod planet;
pub mod countries;

pub const EARTH_RADIUS_M: f64 = 6_371_008.8;

pub use catalog::{Asset, Catalog, Place};
pub use fetch::Fetcher;
pub use cog::Raster;
pub use terrain::{dem_window, DemWindow};
pub use hydro::FlowNet;
