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
pub mod coast;
pub mod quake;
pub mod lifelines;
pub mod aftermath;
pub mod fragility;
pub mod tanks;
pub mod bridges;
pub mod power;
pub mod routes;
pub mod depot;
pub mod surfacewater;
pub mod hospitals;
pub mod agents;
pub mod losses;

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
    kind!("coast.oneline", coast::OneLine),
    kind!("quake.ground", quake::Ground),
    kind!("quake.buildings", quake::Buildings),
    kind!("portland.lifelines", lifelines::Lifelines),
    kind!("people.aftermath", aftermath::Aftermath),
    kind!("fuel.tanks", tanks::Tanks),
    kind!("roads.bridges", bridges::Bridges),
    kind!("power.structures", power::Power),
    kind!("roads.routes", routes::Routes),
    kind!("fuel.depot", depot::Depot),
    kind!("water.surface", surfacewater::SurfaceWater),
    kind!("health.hospitals", hospitals::Hospitals),
    kind!("people.agents", agents::Agents),
    kind!("obs.losses", losses::Losses),
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
    let mut d = (find(kind)?.describe)(name, params);
    if d.layer.is_empty() { d.layer = layer_of(kind).to_string(); }
    Ok(d)
}

/// The layers of the world as a person would list them, in reading order: what is not alive
/// (natural things, then physical artifacts, then symbolic ones), then what is alive, people
/// last because they read everything and write back to all of it. `across` is a disturbance
/// that cuts through the layers; `record` is a module that scores a run against one. The order
/// is who reads whom, not slow to fast (Anselm, 2026-11-13: built before life so the abiotic
/// realm is clumped; law and money are artifacts). A tag for pages; the kernel is flat.
/// Each layer's number orders it and leaves room: a layer between water (3000) and air (4000)
/// would be 3500. `across` and `record` are 0, because they are not places in the order.
pub const LAYERS: [(u32, &str, &str); 12] = [
    (1000, "sun", "Sun"), (2000, "ground", "Ground"), (3000, "water", "Water"), (4000, "air", "Air and weather"),
    (5000, "built", "Built structures"), (6000, "flows", "Flows through the built"), (7000, "rules", "Law and money"),
    (8000, "plants", "Plants"), (9000, "animals", "Animals"), (10000, "people", "People"),
    (0, "across", "Disturbances"), (0, "record", "The record"),
];

/// The layer a kind belongs to when its describe does not say: by the kind's family, with the
/// exceptions named first.
pub fn layer_of(kind: &str) -> &'static str {
    match kind {
        "quake.ground" | "hazard.fire" | "carbon.tipped" => "across",
        "quake.buildings" => "built",
        "portland.lifelines" | "fuel.depot" => "flows",
        "health.hospitals" => "built",
        "agents.farms" => "people",
        "nitrogen.basin" => "water",
        _ => match kind.split('.').next().unwrap_or("") {
            "sun" => "sun",
            "water" | "sea" | "coast" => "water",
            "weather" | "carbon" | "energy" | "forcing" | "heat" | "climate" => "air",
            "fuel" | "roads" => "built",
            "power" => "flows",
            "rules" | "policy" => "rules",
            "veg" => "plants",
            "fauna" => "animals",
            "people" => "people",
            "fire" | "events" | "tipping" => "across",
            "obs" | "score" | "story" => "record",
            _ => "",
        },
    }
}

pub fn kinds() -> Vec<&'static str> { KINDS.iter().map(|k| k.kind).collect() }
