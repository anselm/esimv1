//! Modules. Each is built by `factory` from its manifest `kind`.

pub mod water;
pub mod sun;
pub mod heat;
pub mod basin;
pub mod weather;
pub mod veg;
pub mod fauna;
pub mod fire;
pub mod farms;

use esim_kernel::Module;

pub fn factory(name: &str, params: &toml::Value) -> Result<Box<dyn Module>, String> {
    let kind = params.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    build(kind, name, params)
}

/// Build by kind. The kernel passes params only, so the CLI folds `kind` into them.
pub fn build(kind: &str, name: &str, params: &toml::Value) -> Result<Box<dyn Module>, String> {
    match kind {
        "water.source" => Ok(Box::new(water::Source::new(name, params)?)),
        "water.bucket" => Ok(Box::new(water::Bucket::new(name, params)?)),
        "sun.irradiance" => Ok(Box::new(sun::Sun::new(name, params)?)),
        "heat.slab" => Ok(Box::new(heat::Heat::new(name, params)?)),
        "water.basin" => Ok(Box::new(basin::Basin::new(name, params)?)),
        "weather.era5" => Ok(Box::new(weather::Weather::new(name, params)?)),
        "veg.crop" => Ok(Box::new(veg::Veg::new(name, params)?)),
        "fauna.herbivore" => Ok(Box::new(fauna::Herbivore::new(name, params)?)),
        "fauna.predator" => Ok(Box::new(fauna::Predator::new(name, params)?)),
        "hazard.fire" => Ok(Box::new(fire::Fire::new(name, params)?)),
        "agents.farms" => Ok(Box::new(farms::Farms::new(name, params)?)),
        other => Err(format!("unknown module kind {other}")),
    }
}

pub fn kinds() -> &'static [&'static str] { &["water.source", "water.bucket", "sun.irradiance", "heat.slab", "water.basin", "weather.era5", "veg.crop", "fauna.herbivore", "fauna.predator", "hazard.fire", "agents.farms"] }
