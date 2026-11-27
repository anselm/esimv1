# Can the parts be separated? An experiment on the Palisades fire

26 November 2026. Claude (Fable 5.2), esimv1 room.

Anselm: "part of what we are arguing in this pitch is that we are doing 'models of whole systems' - that
means that we must separate modules out where possible - to at least imply that other modules can
participate. For example later we may want to indicate forest animals fleeing before a fire front ... if
the fundamental framework does not easily support separation of concerns then it may mean taking a pause
and refactoring the framework as a whole. So the directive for you isn't necessarily to simply split the
modules - but to also evaluate if the current framework is good enough for this separation of concerns."
And while I worked: "exact same results isn't as critical - our critical goal is to exercise that we can
separate concerns, on some boundary ... we're generally exploring the proposition 'can we build reusable
modular simulation engines'?"

## Verdict

The framework is good enough. No refactor of the kernel is needed for this. The fire module was one piece
because it was written that way, not because the engine forced it. Four gaps showed up, none in the kernel's
design; they are listed below.

## What was done

`fire.cells` read its own relief, weather and fuel dryness. Those are now modules of their own
(`core/crates/modules/src/landscape.rs`), and a fifth reacts to the fire:

- `ground.terrain` owns the grid and writes elevation, slope and aspect as shared fields.
- `weather.surface` writes each hour's wind, temperature and humidity on those cells (HRRR or ERA5).
- `fuel.moisture` reads the air and writes how dry the dead fuel is.
- `fire.cells` with `inputs = "world"` reads all of those and loads no relief or weather. `inputs = "own"`
  is the default and the old behaviour, so every existing run is unchanged.
- `fauna.flee`, a placeholder population, reads the fire's burned cells each hour and moves or dies. The
  fire does not know it exists. Its ledger of individuals closes.

Manifest `scenarios/esimv1-palisades/manifests/f7-palisades-wired.js`, project `anselm/palisades-wired`
(in the Palisades repository), with the settings of the `vector-5` variant.

## What the experiment shows

- **Same fire.** Burned area, wind, fuel moisture, spot fires, overlap with the mapped edge and structure
  recall are identical, point for point, to `palisades-cells/vector-5`. He said this was not the critical
  test; it is still the cleanest evidence that the seam lost nothing.
- **No cost.** 27 s against 30 s for the single module (one run each, so equal within noise). Passing five
  maps of 540,000 cells between modules each hour is not felt.
- **Order works.** Weather at priority 3, moisture at 4, fire at 6, animals at 7, all at the same instant,
  each reading what the one before wrote. This is the kernel's existing rule, unchanged.
- **The wiring is checked.** `esim check` lists who owns and who reads each field and passes; a missing
  writer of a required field is refused before the run.
- **A new participant needed nothing from the fire.** The animals were added by writing one module and one
  line in the manifest.

## What the kernel already had

Topics, events at a cell, messages to a named module, shared fields with one owner and logged writes,
ledgers, a random stream per module, priorities within an instant, and a describe that declares all of it
and is checked. Portland and Earth 2100 already use these with ten to eighteen modules. The fire projects
did not.

## The gaps

1. **The grid has no owner by rule.** Every window module (`fire.cells`, `obs.places`, the records) calls
   the elevation reader to learn the grid, because nothing else says what the cells are. In the experiment
   the weather and animals take the grid from the terrain's field at their first tick, which works. The
   fire still derives it before the clock starts, because it sizes its state when it is created. A stated
   convention (the terrain owns the grid; others read it at their first tick) or a small grid service
   would end the duplication. This is a convention to adopt, not a kernel change.
2. **Shared fields are maps of numbers only.** Buildings as bodies (twelve thousand footprints with
   neighbours), tanks, bridges, agents: lists of things, not grids. Today such a list lives inside one
   module and others see it only as events or as a per-cell summary. Portland works around it with
   `component.state` events. A shared table of bodies, owned by one module and read by others, is the one
   real addition worth designing. It is why land cover, buildings and roads are still inside the fire.
3. **Maps on different grids.** A reader panics if a field is on another grid. Weather at 3 km and fire at
   30 m meet today because the weather module resamples to the fire's cells itself. A shared resampling
   helper (in the registry, not the kernel) would let a module publish on its native grid.
4. **Coupling is by the hour.** The fire takes thirty-second steps inside its hourly tick, and neighbours
   see it once an hour. Animals fleeing "before the front" need either a shorter tick for the fire or the
   fire publishing its front more often. Both are parameters, not framework.

Also: a prepare (`esim prepare`) fetches at creation, so a module that waits for its first tick to learn
the grid cannot fetch grid-dependent data ahead. Not hit here; it will be when land cover is split out.

## Where the boundary should not go

Some concerns are tightly bound and splitting them would be for show: heat transfer between cells, ignition
and burn-out are one computation inside the fire. Spotting could be a module but would need the fire's
thirty-second state. The honest boundary is at things that have their own data, their own clock or their own
reader: relief, air, fuel state, the fire, firefighting, buildings, animals, people, the records.

## Next

- Make the wired form the Palisades cells manifest (results are identical), then Fort McMurray and Nicasio.
- Split land cover and roads out; then buildings, which needs gap 2.
- `fire.rothermel` gets the same `inputs = "world"`.
- A real animals module, if wanted: senses the front at a distance, moves at a speed, has habitat.
- Not deployed. The new project's runs are on canada1 only.

## Later the same day: done for the three cell-fire scenarios

Anselm: "go for it".

- **Gap 1, lists of things: closed in a first form.** The kernel's world has tables beside its fields
  (`WorldTable` in `core/crates/kernel/src/world.rs`; `world_table_write`, `world_table` on the context):
  one row per thing, numeric columns, optionally an outline per row, one owner, written whole, logged with
  a digest like a field. This is an addition to the kernel, kept to one type and two verbs. A module that
  keeps its own state about the same things writes its own table in the same row order; nothing enforces
  that order yet.
- **Gap 2, the grid: closed by a pure function.** `esim_registry::window_grid` gives the cells of a place
  with nothing fetched, and the elevation reader lays its data on exactly those. The new modules and the
  fire use it (`landscape::grid_of`). Twenty older call sites still read the elevation model to learn
  their grid; they agree with it and can be moved over one by one.
- **Three more parts**: `ground.cover`, `roads.major`, `built.footprints` (the table of buildings and two
  maps). `fire.cells` with `inputs = "world"` now fetches nothing at all.
- **The three scenarios run this way**: Palisades cells, Fort McMurray and Devil's Gulch each have ten or
  more modules: relief, land cover, roads, buildings, weather, fuel moisture, fire, animals, firefighting
  where present, and the records. The experiment project `anselm/palisades-wired` is removed; Palisades
  cells is that form now.
- **Checked**: every series of all 28 runs (20 Palisades variants, 4 Fort McMurray, 4 Devil's Gulch)
  compared with what it held before the change: identical, including the variants that burn buildings
  one by one. The engine's tests pass. The wiring tab of Fort McMurray and the animals tab of Devil's
  Gulch looked at on the local server.
- Nicasio's `gust` variant sets `weather.wind_use` now, not `fire.wind_use`.

Still open: gaps 3 and 4 (fields on different grids; hourly coupling), `fire.rothermel` and its two
projects, the Boone modules, a rule for row order between tables, and the animals as more than a
placeholder. Not deployed; the reruns are on canada1 and reach the site with the next deploy.
