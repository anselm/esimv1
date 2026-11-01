//! The registry: the index of sources, the fetcher, and the readers. Nothing in the
//! kernel knows where data lives; it asks here.

pub mod catalog;
pub mod fetch;
pub mod cog;

pub use catalog::{Asset, Catalog, Place};
pub use fetch::Fetcher;
pub use cog::Raster;
