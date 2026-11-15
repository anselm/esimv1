<!-- order: 5 -->
<!-- section: The machine -->
<!-- author: Claude -->
<!-- ordered-by: Anselm -->
<!-- written: 2026-11-01 -->
<!-- revised: 2026-11-14 -->
<!-- revision: 1 -->
# Layers

Modules sort into layers the way a person would list what a place is made of. The order is who
reads whom, not slow to fast. What is not alive comes first: natural things, then physical
artifacts, then symbolic ones. What is alive follows, with people last, because people read
everything below them and write back to all of it, the rules included. A building reads the
ground and the weather and almost nothing living, while living things read the built world all
the time (a road splits a habitat, a fence stops a deer), so the built comes before life. Law and
money are artifacts too: a zoning line, a storage cap and a price are designations laid on the
map, and an animal never meets the law, only the fence the law put there.

The layers are a label on each module kind, used to group and order modules on a page and to
show which layers a run leaves empty. They are not a structure in the code; the kernel sees only
modules, topics and fields. Each kind's describe carries its layer (`layer_of` in
`crates/modules/src/lib.rs`), and a run page with an elevation frame shows the modules by layer
in three dimensions, each layer switched on and off.

| number | layer | what it holds | kinds today |
|---|---|---|---|
| 1000 | sun | irradiance on terrain | `sun.irradiance` |
| 2000 | ground | elevation, soil, geology | none; the relief is read from a frame |
| 3000 | water | water on and in the ground, the sea, the cycles it carries | `water.basin`, `water.bucket`, `water.source`, `water.surface`, `nitrogen.basin`, `sea.thermal`, `coast.oneline` |
| 4000 | air and weather | composition, energy, weather | `carbon.emissions`, `carbon.pools`, `carbon.gases`, `forcing.ar6`, `energy.twolayer`, `heat.slab`, `weather.era5`, `weather.heatwaves`, `climate.countries` |
| 5000 | built structures | buildings, tanks, bridges, roads, hospitals | `quake.buildings`, `fuel.tanks`, `roads.bridges`, `roads.routes`, `health.hospitals` |
| 6000 | flows through the built | electricity, fuel, piped water, vehicles | `power.structures`, `fuel.depot`, `portland.lifelines` |
| 7000 | law and money | designations and ledgers | `rules.calendar`, `policy.capacity` |
| 8000 | plants | producers | `veg.crop` |
| 9000 | animals | consumers and their food webs | `fauna.herbivore`, `fauna.predator` |
| 10000 | people | population, exposure, the aftermath | `people.countries`, `people.heat`, `people.wetbulb`, `people.agents`, `people.aftermath`, `agents.farms` |
| | disturbances (across) | events that cut through the layers and write to several | `quake.ground`, `fire.cells`, `fire.rothermel`, `fire.suppression`, `hazard.fire`, `events.shock`, `tipping.elements`, `carbon.tipped` |
| | the record | modules that score a run against what was observed or published | `obs.*`, `score.regions`, `story.bottomline` |

Each layer has a number, a thousand apart, so that a layer can be put between two others without
renumbering: a layer between water and air would be 3500. Everything up to 7000 is not alive and
everything from 8000 is. The numbers and the grouping are expected to keep changing; they exist
for readers, and no code depends on their values.

A disturbance is not a layer. An earthquake or a fire publishes what happened, and every subject
module that cares subscribes; it never passes through the layers in order.

## How the layers couple

Downward dependencies are the norm: a crop reads the weather's temperature and the
water's soil moisture; a herd reads the crop's biomass; a predator reads the herd. Those
reads are declared, and the kernel orders modules at an instant by priority so that a
reader sees this tick's write, not last tick's. Upward feedbacks exist and are declared
the same way: the crop's leaf area changes the water's evaporation; the fire's burn
changes the crop's carbon and the herd's numbers; a country's warming comes from the
energy balance, and one day a forest's dieback will put carbon back into layer 1.

The lag between layers is a property of the physics, not of the code. The ocean holds
heat for decades; a population responds in years; a weather field in hours. A run that
shows layers 1 and 2 already shows the commitment that layer 7 will feel later, which is
why the planet runs report the deep ocean beside the surface.

## Patches

The layers picture is too flat for a planet. A patch is anything bound to one place: one
module for the Amazon basin, or a bundle for a river basin with its own weather, water,
plants, animals, people and rules, resolved on its own grid. A manifest that includes
many patches is a world-spanning layer; a module never has to span the planet itself. A
planetary simulation is many patches on one clock, coupled through shared global state
(the carbon and energy ledgers, the global temperature anomaly) and through boundary
fluxes (a river into a sea, rainfall from a circulation into a basin). The country layer
is the coarsest possible patchwork: 177 patches with one temperature each, chosen
because borders are where people's options change and because records exist at that
unit. Finer patches (a river basin, a city, a reef) use the same kinds with their own
place, and nested manifests let a patch be written once and included where it belongs.

## What belongs in which layer, and what does not

A kind belongs in the layer whose records it reads and whose quantities it owns. A kind
that reads a layer-3 field and owns a layer-5 field is a layer-5 kind with a declared
coupling, which is normal. A kind that produces layer-7 consequences (a riot, a
migration) from nothing but words is not a kind; it is an essay, and the project does not
run essays. The upper layers are where honesty gets hardest and where the rules in
[Honesty](/docs/system/honesty) do the most work: counts before models, published
response curves with their ranges, and the word "guess" wherever it is true.
