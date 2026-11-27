# esim

A whole-system simulator built as a bucket of tools: small modules of science on one
clock, each reading real records, exchanging events and shared fields, moving conserved
quantities through ledgers that must balance at every seam. A manifest arranges modules
into a scenario; a run is a folder anyone can replay, score, audit and look at.

The scenarios so far are five wildfires (two that happened, one that has not, on two fire
models), a river basin in Iowa, a harbor's shoreline, Portland after a Cascadia
earthquake, and the planet to 2100. This repository is the hub that presents them. The
engine, the viewer and the larger scenarios are kept in repositories of their own.

## Whole systems from separate parts

The claim under test is that a model of a whole place can be assembled from parts that do
not know each other. A fire scenario here is not one fire program. It is about ten
modules: the relief, the land cover, the major roads, the buildings, the hour's weather,
the dryness of the dead fuel, the fire, a population of animals that runs from it, the
crews that fight it, and the records it is scored against. Each has its own data and its
own author's assumptions. They meet only in shared maps and tables that one module owns
and the others read, and the engine refuses a scenario whose parts do not fit.

Three things follow, and each can be seen in the runs:

- **A part can be swapped.** The Palisades fire and the Devil's Gulch scenario each run on
  two fire models, Rothermel's and our own cell model. Everything else in the scenario is
  the same modules with the same settings.
- **A part can be reused.** Fort McMurray, in boreal Canada, uses the same parts as Los
  Angeles with two of them pointed at other sources: buildings from OpenStreetMap, weather
  from a global reanalysis.
- **A new part can join without the others changing.** The animals were added as one
  module and one line in each scenario. The fire was not edited.

What this does not yet show: the animals are a placeholder with no species; parts see each
other once an hour of simulated time; and two parts that affect each other within the
hour (a fire and the wind it makes) would have to be one part or take a shorter step.
`devlog/20261126-can-the-parts-be-separated.md` has the test and its limits.

**Live, read-only: https://esimv1.exe.xyz/** · version 0.1.1 · code MIT

By Anselm Hook. The writing around this, and the reasons for it, are at **[anselm.substack.com](https://anselm.substack.com)**; the running commentary is **[@anselm](https://x.com/anselm)** on X. Subscribe and follow there if you want to see where it goes.

The planet pages to start with:

- [Earth 2100](https://esimv1.exe.xyz/anselm/planet-2100): one project, six
  assumptions about emissions after 2024, each a variant, read against the tipping
  thresholds in the project's own README.
- [The Palisades fire, January 2025](https://esimv1.exe.xyz/anselm/palisades-2025):
  a known fire run again on its own ground, unsuppressed, and scored against the
  perimeters its mapping teams drew; the first local scenario, and the model's first
  honest overrun.
- [A fire that has not happened](https://esimv1.exe.xyz/anselm/nicasio-2029): a scenario with no
  record, an ignition in a Marin County state park in October 2029 under a borrowed Red Flag
  wind, with a named house as the stake; what the engine can and cannot say without a record.
- [Half Moon Bay: the breakwater and the beach](https://esimv1.exe.xyz/anselm/halfmoonbay-shoreline):
  a one-line shoreline model of the bay south of Pillar Point Harbor since the Corps built the
  breakwater, read against the Corps' and Griggs's erosion rates, with the dredge and sea level to 2075.
- [Portland after Cascadia](https://esimv1.exe.xyz/anselm/portland-cascadia): a
  magnitude 9 earthquake on the Cascadia subduction zone as it affects Portland, set on 1 February
  2027 and followed to the end of that year. Buildings, fuel tanks, bridges, transmission structures and road
  segments are damaged one by one on the state's published ground shaking. The runs then track
  electricity, hospitals, fuel and deaths, and compare holding less fuel at the riverside
  terminals with retrofitting the tanks. Several results disagree with the studies they are
  checked against, and the page says which.
- [Two futures, one clock](https://esimv1.exe.xyz/anselm/planet-2100/compare): emissions
  held at today's rate beside emissions to zero by 2050, under one time cursor.
- [Documentation](https://esimv1.exe.xyz/docs) and [the module kinds](https://esimv1.exe.xyz/kinds).
- [Known future events](https://esimv1.exe.xyz/docs/events/known), the calendar to 2100 and beyond with a source per row, and [Wild cards](https://esimv1.exe.xyz/docs/events/wild-cards), the black swans a run can draw, with their rough chances and effects.
- [RFC 0001](https://esimv1.exe.xyz/docs/rfc/0001-manifest), a proposal: the manifest as a common transport for simulations between competing engines, in JSON, with a schema.
- [Limits today](https://esimv1.exe.xyz/docs/system/limits), to read before any number.

The project page opens with the project's own README, "The road to 2100, and the
thresholds on it": what a tipping point is, the ladder of thresholds with every path
drawn against it, what each half degree costs in people, and what the runs do and do not
do with a crossing.

Nothing on those pages is a forecast. Every run past 2024 rests on a stated assumption,
every threshold carries its source and whether it has been checked, and the page says
what the run does not yet measure.

## What it does

- **Kernel** (`crates/kernel`): a virtual clock in seconds, events ordered by time,
  priority and sequence, shared gridded fields and tables of things (buildings, say) with one
  owner each, cell-addressed events,
  double-entry ledgers closed on a cadence, an append-only log with a digest, snapshots
  and exact replay, TOML manifests with includes and a place stated once.
- **Registry** (`crates/registry`): a catalog of sources with licence and verification
  notes, a fetcher that hashes and caches every file, and readers for elevation tiles,
  gauges, stations, reanalyses, satellite vegetation, emissions, ice cores, temperature
  records, population, country outlines and the station network. Nothing synthetic,
  including in tests.
- **Modules** (`crates/modules`): sixty kinds today, each with a title in plain words. For a basin: solar irradiance
  on terrain, a surface heat slab, reanalysis weather, a water balance routed to the
  gauge, a crop, herbivore and predator cohorts, fire, farm agents, calendar rules, soil
  nitrogen to the river. For a wildfire: the relief, land cover,
  roads, buildings, weather and fuel moisture as separate modules; two fire models that read
  them (Rothermel's spread, and a heat budget on 30 m cells with buildings as bodies);
  firefighting; animals; the incident's mapped perimeters, satellite detections and inspected
  structures as scoreboards. For the planet: observed emissions into a carbon ledger, a carbon
  cycle with sink feedback, methane and nitrous oxide, the IPCC assessed forcings, a
  two-layer energy balance, thermal sea level, the records as scoreboards; then people on
  half-degree cells: WorldPop's pattern at the UN's count, each country's temperature from
  its own record, the human climate niche and the wet-bulb limits checked cell by cell
  against a reanalysis baseline, a weather generator that replays a decade of observed days
  with the warming and counts heatwaves by a published definition, the tipping elements'
  thresholds (every row checked against the assessment's table), the first cascade
  (permafrost and Amazon carbon released into the carbon ledger once an element is past its
  threshold), and a bottom-line module that tells the run as sentences with sources. For an
  earthquake's effect on a city: a published scenario's ground motion and ground failure as fields,
  buildings as fragilities cell by cell fitted to the state's own loss run, every tank, crossing,
  transmission structure and route segment as a component with its own fragility, state and
  repair path on the ground at its cell, the networks computed from what stands, the people as
  super-persons drawn from the occupant rasters, and the deaths after the shaking as claims with
  sources.
- **Auditor** (`crates/audit`): reads a finished run and never the live one: every stock
  spent or built, diversity, shocks and recovery against recurrence, crossings, tears,
  and a verdict of holds, stretching or tearing. It lists what it could not measure.
- **Command line** (`crates/cli`, binary `esim`): run, verify, ensemble, sweep, score,
  audit, check, describe, kinds, projects, sources, regress (the hindcast ladder, which
  every coupling must leave where it was or move on purpose).
- **Viewer and server** (`web/`): Svelte pages composed from each module's own declared
  views in the manifest's order, maps under one time cursor; a run page written in HTML
  with the run's maps, charts and wiring as tags; a project's README rendered with live
  figures from its runs; a Node server that serves the pages and folders. The public copy
  is a showcase: nobody signs in, and scenarios are made on a developer's machine with the
  command line.

## Honesty rules

They are the point of the system and are enforced by code where code can:

- Real records or nothing; a formula check is labelled as one.
- Every parameter is named, defaulted and sourced; a guess says "guess"; a number recalled
  from memory and not yet checked against its paper says "recalled".
- Nothing is tuned by hand; the hindcast harness moves parameters by method, and the
  regression ladder fails any change that moves a hindcast score unless it is accepted
  by hand with its reason in the devlog.
- A run past a record is an assumption, named in the manifest, never a forecast.
- Accounts balance: a ledger leak above tolerance fails the run.
- Runs are reproducible: the same manifest and seed give the same digest; a run folder
  keeps its resolved manifest and the hashes of every file it read.
- Known faults are published. `docs/system/limits.md` is rewritten as they change.

## Try it

Rust stable (rustup) and Node 20 or newer. First runs fetch a few hundred megabytes of
records into `data/cache/` and pin their hashes in `data/catalog.lock.toml`.

```sh
deploy/parts.sh                                                    # clone the engine, the viewer and the scenarios kept elsewhere
(cd core && cargo build --release)
./core/target/release/esim kinds                                        # the module kinds
./core/target/release/esim check core/manifests/p4-planet-people.toml        # wiring from the describes
./core/target/release/esim run core/manifests/p1-planet-co2-1850-2025.toml   # 175 years of planet, about a second
./core/target/release/esim warm cells                                   # once, ~100 min: the 1 km people and the reanalysis boxes the planet-2100 project reads
./core/target/release/esim run core/manifests/m3-water-boone.toml            # a year of basin hydrology, about 40 s
./core/target/release/esim verify core/manifests/m2-heat-boone-week.toml     # determinism and replay
./core/target/release/esim regress                                      # the hindcast ladder against core/regress/baseline.json; --accept records a new one
./core/target/release/esim manifest core/manifests/j3-planet-wild-members.js # a JavaScript manifest as the entity list it resolves to; --id for its hash
(cd core && cargo test --release)                                             # about 40 s with the data cached

# projects: a folder with a manifest, a description and runs; variants are named overrides
./core/target/release/esim project new me/planet --manifest core/manifests/p4-planet-people.toml
./core/target/release/esim project prepare me/planet                    # fetch what its modules need, and stop before the clock
./core/target/release/esim project run me/planet
./core/target/release/esim project run me/planet --set emissions.after_record=zero_by:2050

# the viewer and the server, one process
cd web && npm install && npm run build && node server.mjs          # http://localhost:5181
cd web && npm run dev                                              # development: hot reload, socket proxied to :5181
```

A manifest is a short document in TOML, JSON (the entity form of RFC 0001) or JavaScript
(a hermetic module whose exports are the entities; `esim manifest <file>` prints what any
of them resolves to); `docs/system/manifests.md` has the grammar. On a developer's own
copy the "+ project" form asks a model to draft a manifest from a description against the
kinds' own descriptions, and the kernel checks it before anything runs.

## Layout

```
projects/      <area>/<name>/{project.toml, manifest.toml, README.md, runs/}   (runs are not in git)
docs/          documentation pages of this workspace, docs/<area>/<name>.md
web/           the site: pages, router, the project server
deploy/        parts.sh (clone the parts), deploy.sh (a read-only copy to an exe.dev VM)
parts.list     the parts kept in their own repositories, and the version of each this hub is built against
devlog/        dated entries: the design brief and every decision since
todo.md        open research projects and layers, one line each

cloned here by deploy/parts.sh, each its own repository, none in this one's git:
core/          anselm/esimv1-core: the engine (kernel, registry, modules, auditor, the esim command line),
               its catalog and shared tables, the ladder manifests, the engine's own documentation
viewer/        anselm/esimv1-viewer: the components that draw a run folder
scenarios/     anselm/esimv1-palisades, esimv1-portland, esimv1-earth2100: scenarios with a life of their
               own, each with its record; their project folders are linked into projects/anselm/
```

This repository is the hub: the site that presents the scenarios, and the home of the
smaller ones that have not moved out. Since 22 November 2026 the engine, the viewer and three scenarios are
separate repositories. `ESIM_ROOT` is the engine's folder (`core/` here) and
`ESIM_WORKSPACE` is where `projects/` and loose `runs/` are (this folder); `esim` run from
this folder finds both. A fourth scenario, Sausalito, is `anselm/esimv1-sausalito-model`
and is not shown here yet.

## Where things stand

The basin stack reproduces the shape of the gauge record and runs high on nitrate; the
planet hindcast is within a few percent of Mauna Loa and about 0.15 °C warm in 2024
against HadCRUT5; the cell layers count people living where stated limits are
passed, with the warming still applied per country (the reanalysis's own trend was tried
as a pattern and fails over South Asia); the first cascade moves the assessment's
permafrost and Amazon carbon and comes to a tenth of a degree by 2100. The Portland earthquake
(2026-11-12) is the first scenario in which bridges, roads, electricity and a fuel spill are
computed from individual components instead of being drawn from published ranges. Its first runs
disagree with the studies they are checked against, and the project page lists how. The tanks
release about two thirds of the county study's minimum. Electricity returns in three weeks where
the Oregon Resilience Plan expects one to three months, because transmission structures are the
only part of the grid in the model. The model's two ways of counting the dead from the shaking
disagree in daytime runs. The principle since that night is that a module computes and can be
wrong in a way a record catches; numbers drawn from published ranges are inputs or checks. The
known faults
are on the Limits page and in the devlog, which is the project's memory: start with
`devlog/20261031-design-brief.md`, then the milestone entries, then the latest day.

## Contributing a module

`docs/system/adding-a-module.md` is the recipe: find the record first, name what the
module owns and reads in the shared vocabulary, write the kind with its `describe` beside
its constructor, account for what it moves, register it, run it on a real stage, and add
the manifest to the test that compares declarations against run logs. Competing kinds
for the same thing are welcome; the record and the describes decide.

## Licence

The code is MIT (`LICENSE`). The records the simulator reads are not in this repository;
each is fetched from its publisher under its own terms, listed in `data/catalog.toml` and
on the Sources page, and several are free for non-commercial use only.

The code and the scenarios are at [github.com/anselm/esimv1](https://github.com/anselm/esimv1); the pages at [esimv1.exe.xyz](https://esimv1.exe.xyz/); the writing around it at [anselm.substack.com](https://anselm.substack.com).
