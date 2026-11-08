//! Modules. Each kind registers two functions in `KINDS`: `build`, which constructs it
//! (and fetches its data), and `describe`, which states its surface from the parameters
//! alone, fetching nothing. The kernel checks wiring from the descriptions; the CLI
//! prints them for people and for the facade.

pub mod water;
pub mod sun;
pub mod heat;
pub mod basin;
pub mod weather;
pub mod veg;
pub mod fauna;
pub mod fire;
pub mod farms;
pub mod rules;
pub mod nitrogen;
pub mod planet;
pub mod countries;
pub mod wildfire;
pub mod record;
pub mod suppression;
pub mod hotspots;
pub mod firecells;
pub mod shock;
pub mod score;
pub mod capacity;
pub mod heatwaves;
pub mod places;

use esim_kernel::describe::Describe;
use esim_kernel::Module;

pub struct Kind {
    pub kind: &'static str,
    pub build: fn(&str, &toml::Value) -> Result<Box<dyn Module>, String>,
    pub describe: fn(&str, &toml::Value) -> Describe,
}

macro_rules! kind {
    ($k:literal, $t:ty) => {
        Kind { kind: $k, build: |n, p| Ok(Box::new(<$t>::new(n, p)?) as Box<dyn Module>), describe: <$t>::describe }
    };
}

pub static KINDS: &[Kind] = &[
    kind!("water.source", water::Source),
    kind!("water.bucket", water::Bucket),
    kind!("sun.irradiance", sun::Sun),
    kind!("heat.slab", heat::Heat),
    kind!("water.basin", basin::Basin),
    kind!("weather.era5", weather::Weather),
    kind!("veg.crop", veg::Veg),
    kind!("fauna.herbivore", fauna::Herbivore),
    kind!("fauna.predator", fauna::Predator),
    kind!("hazard.fire", fire::Fire),
    kind!("agents.farms", farms::Farms),
    kind!("rules.calendar", rules::Rules),
    kind!("nitrogen.basin", nitrogen::Nitrogen),
    kind!("carbon.emissions", planet::EmissionsIn),
    kind!("carbon.pools", planet::Pools),
    kind!("energy.twolayer", planet::TwoLayer),
    kind!("obs.planet", planet::PlanetObs),
    kind!("carbon.gases", planet::Gases),
    kind!("sea.thermal", planet::SeaThermal),
    kind!("forcing.ar6", planet::ForcingAr6),
    kind!("people.countries", countries::PeopleCountries),
    kind!("climate.countries", countries::ClimateCountries),
    kind!("people.heat", countries::PeopleHeat),
    kind!("people.wetbulb", countries::PeopleWetBulb),
    kind!("tipping.elements", countries::Tipping),
    kind!("carbon.tipped", countries::TippedCarbon),
    kind!("story.bottomline", countries::Story),
    kind!("fire.rothermel", wildfire::Rothermel),
    kind!("obs.perimeters", wildfire::Perimeters),
    kind!("obs.dins", wildfire::Structures),
    kind!("obs.record", record::Record),
    kind!("fire.suppression", suppression::Suppression),
    kind!("obs.hotspots", hotspots::Hotspots),
    kind!("fire.cells", firecells::Cells),
    kind!("events.shock", shock::Shock),
    kind!("score.regions", score::Score),
    kind!("policy.capacity", capacity::Capacity),
    kind!("weather.heatwaves", heatwaves::Heatwaves),
    kind!("obs.places", places::Places),
];

fn find(kind: &str) -> Result<&'static Kind, String> {
    KINDS.iter().find(|k| k.kind == kind).ok_or_else(|| format!("unknown module kind {kind}"))
}

pub fn factory(name: &str, params: &toml::Value) -> Result<Box<dyn Module>, String> {
    let kind = params.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    build(kind, name, params)
}

/// Build by kind. The kernel passes params only, so the CLI folds `kind` into them.
pub fn build(kind: &str, name: &str, params: &toml::Value) -> Result<Box<dyn Module>, String> {
    (find(kind)?.build)(name, params)
}

/// Describe a kind under a name and parameters without building it. Nothing is fetched.
pub fn describe(kind: &str, name: &str, params: &toml::Value) -> Result<Describe, String> {
    Ok((find(kind)?.describe)(name, params))
}

pub fn kinds() -> Vec<&'static str> { KINDS.iter().map(|k| k.kind).collect() }
