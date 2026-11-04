<!-- order: 5 -->
# Layers

Modules sort themselves into layers by what they read and what they own, from the slow
physical bottom to the fast human top. The layers are a way of thinking about the stack,
not a structure in the code; the kernel sees only modules, topics and fields.

| layer | what it integrates | kinds today | records |
|---|---|---|---|
| 0 solar | irradiance on terrain | `sun.irradiance` | elevation |
| 1 composition | what is in the air | `carbon.emissions`, `carbon.pools`, `carbon.gases` | emissions, ice cores, Mauna Loa, NOAA gases |
| 2 energy | where the heat goes | `energy.twolayer`, `heat.slab` | HadCRUT5, GISTEMP |
| 3 abiotic systems | water, weather, ocean, ice, sea | `water.basin`, `weather.era5`, `sea.thermal` | gauges, stations, ERA5, GEFS |
| 4 biogeochemistry | the cycles and their feedbacks | `nitrogen.basin` | nitrate at the gauge |
| 5 producers | plants | `veg.crop` | MODIS NDVI |
| 6 consumers | animals and their food webs | `fauna.herbivore`, `fauna.predator`, `hazard.fire` | none yet |
| 7 people | population, land use, rules, exposure | `agents.farms`, `rules.calendar`, `people.countries`, `climate.countries`, `people.heat`, `people.wetbulb` | UN population, WorldPop 1 km, CRU country temperatures, MERRA-2 daily heat and humidity (NASA POWER), NOAA GSOD stations |
| across | thresholds and their sources, and what crossing them moves | `tipping.elements`, `carbon.tipped` | the assessment table, checked row by row against the paper |

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
