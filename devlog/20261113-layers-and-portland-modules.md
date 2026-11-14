# Layers as people see them, and the Portland modules sorted

13 November 2026. Claude Code session in the esimv1 room on canada1
(session_018TGCFiQHbUYn12HkNti2xk, Fable 5.2), in conversation with Anselm from his phone.

## The conversation

Anselm asked for a cold read: "From a human perspective, what are all the 'layers' of this
simulation". Then his own separation: "abiotic phenomena - solar irradiance, geophysical
phenomena such as land soil composition elevation, hydrology which seems to intersect with land a
lot, volumetric weather, and built landscapes such as roads and homes and buildings and bridges,
and vehicles and fuel terminals and hospitals and electricity, and life and trees and animals,
fire is practically a layer and maybe oil spills, and then money and laws, and people - maybe the
order is not perfect but one can structure these in a linear way largely".

Where it settled, in his moves:

- "Why not swap built structures and life? Then the abiotic realm is clumped?" Yes. At run time a
  building reads the ground and the weather and almost nothing living, while living things read
  the built world constantly.
- "law and money primarily affect built structure - yes people obey laws but animals only obey
  fences that law designates. In a sense law is part of the abiotic realm?" Yes. A law is an
  artifact: a designation laid on the map. Money is a ledger.
- "I am thinking about having explicit layers even if empty."
- "Layers are more syntactic sugar for us I think?" Yes: a declared tag on a kind, for grouping
  and order on the page, and for showing the empty ones. The kernel stays flat.
- A quake or a fire is not a layer: "it is simpler to directly affect all modules it impacts".
  That is how the bus already works. A disturbance publishes; subject modules subscribe.

The order as it stood at the end of the conversation (not yet written into `docs/system/layers.md`):

Not alive: 1 sun; 2 ground (elevation, soil, geology); 3 water on and in the ground; 4 atmosphere
and weather; 5 built structures; 6 flows through the built (vehicles, fuel, power, piped water);
7 law and money. Alive: 8 plants; 9 animals; 10 people. Across all: fire, spills, earthquakes,
floods. The order is who reads whom, not slow to fast; people come last because they read
everything and write back to all of it.

## The mismatch he named

"Our machine designed modules and relationships are not similar to the human model." The Portland
modules were cut by method and data source: `hazard.components` is everything damaged by a
fragility draw (tanks, bridges, power structures, roads), `lifelines.recovery` everything that
steps daily (bridges again, hospitals, electricity, water, the fuel account), people are in three
modules. A count of reuse: of 46 kinds, three have run in more than one place, all in the fire
family; all six Portland kinds were single-use.

His rule: "Portland modules should be Portland prefixed Or generalized."

## The sorting, and the first batch (done)

| Kind | Call | State |
|---|---|---|
| `lifelines.recovery` | prefix | now `portland.lifelines`; shrinks as its subjects are pulled out |
| `quake.ground` | generalize | done: the event folder's `ground.toml` names the maps, the fault, the agency and the source; the code names none of them |
| `people.aftermath` | generalize | done: the device share sums any `power_dependent_dme_*` and `population_*` keys; each claim has a `short` the sentence uses; the month rule's text and the "leaves out" text come from `aftermath.toml` |
| `people.agents` | generalize | done: same device-share rule; no Portland or DOGAMI in its text beyond one dated note on the unexplained daytime gap |
| `quake.buildings` | generalize after one cut | done (second batch, below): the comparison is `obs.losses` |
| `hazard.components` | generalize by splitting | after that: tanks, bridges, power structures, road segments as subject modules on one fragility library |

None of the three generalized kinds has a default folder now; a manifest must name one. They still
default to Portland's window when a manifest gives no place, which no manifest relies on.
`people.aftermath` is general across earthquakes, not across hazards: it listens to `quake.toll`.

All ten Portland variants were rerun. Every series is identical to the runs deployed on the
night of 12 November; the run ids changed because the manifest names a new kind. `cargo test`
(describe) and `esim regress` pass. Committed, not pushed, not deployed: the live copy still
serves the previous runs.

## The second batch: the comparison out of the buildings (done)

Anselm: "Carry on."

- `obs.losses` (`crates/modules/src/losses.rs`) is new. On `quake.toll` it reads five per-cell
  world fields the buildings module now writes (`quake.cell_extensive`, `quake.cell_complete`,
  `quake.cell_deaths`, `quake.cell_displaced`, `quake.cell_injuries_hospital`), sums them over
  the study's small areas and over one named group of larger areas, and reports the same series
  as before (`fit.portland_*`, `fit.units_*`, `quake.portland_*`; the middle word is the group's
  `key` in the folder's `losses.toml`). The comparison is its own sentence on the run page
  (`fit.crossing`) and the study's table is `published_losses` (was `dogami_losses`).
- `data/events/portland-cascadia/losses.toml` names the study, the rasters, the tables, the
  columns, the group and the rows to show. The code names none of them.
- `quake.buildings` no longer loads the unit raster, the jurisdiction raster or the loss tables,
  has no `record` parameter and no default folder, and says nothing of Portland or DOGAMI. Its
  toll sentence ends at the toll. Variants that set `buildings.record` now set `losses.record`,
  and `wet-night` also sets `losses.time_of_day`.

Against the runs of the first batch, every series is the same except the eleven the new module
reports, which differ by at most 8 parts in a billion: the per-cell fields are stored in single
precision before they are summed. A first version moved the unit correlation in the fourth
decimal (0.8946 to 0.8944) because it admitted units with no buildings; a unit now enters only
through a cell that has buildings, as before.

`cargo test` (describe) and `esim regress` pass. The base run's page was looked at: the new
module's panel, its two counters and its place in the wiring figure. Committed, not pushed, not
deployed.

Left for the third batch: `hazard.components` split by subject; `portland.lifelines` shrinking as
hospitals, the fuel account and water are pulled out. The manifest's comment on the fitted
parameters and the README's account of the fit still stand as written.

## The third batch: `hazard.components` split by subject (14 November, done)

Anselm: "Go ahead and split more pieces."

`hazard.components` is deleted. In its place, each on the event folder's tables and
`fragility.toml`, with no place or agency named in the code:

| Kind | File | What it owns | What it publishes |
|---|---|---|---|
| `fuel.tanks` | `tanks.rs` | the tanks, their damage and their releases | `component.state` (tank), `tank.release`, `tanks.released` |
| `roads.bridges` | `bridges.rs` | the bridges, their damage states and repair clocks | `component.state` (bridge) |
| `power.structures` | `power.rs` | the transmission structures, crews, the service map `power.up` | `power.state` |
| `roads.routes` | `routes.rs` | the road graph, segment closures, reachability from an origin | `component.state` (route), `routes.state` |

`fragility.rs` is the shared library: the curve, the draw of a damage state from the ground at a
cell, the folder readers. It is not a module.

What had to be said in the folder instead of the code: the tanks' design year and the study's
release range (`[tank]` in `fragility.toml`), the structures' file and column names (`[pole]`),
and which named place the roads are measured from (`origin` in `routes.toml`).

`portland.lifelines` now listens to `tanks.released`, `power.state` and `routes.state` in place of
`hub.release` and `network.state`, and learns whether the grid reaches a hospital by reading the
`power.up` field at the hospital, as the agents already did. `roads.routes` hears the bridges'
states from `roads.bridges` and publishes its day's state on a later tick than the bridges', so
the order within a day is bridges, then routes, then lifelines.

### The numbers moved, and why

Each module draws from its own random stream, seeded by the run's seed and the module's name. One
module became four, so the same seed now gives different tanks, bridges, structures and segments.
This is the same kind of change as a new seed; no rule changed. One rule did change slightly:
the nearest-structure search scales longitude by the cosine of the latitude where it used a
constant for Portland's latitude (78.0 km per degree against 77.8).

Base run, before and after: tanks failed 233 to 270; release 60.6 to 67.8 million gallons (river
26.2 to 24.3); structures failed 586 to 614; bridges, ten closed in both, but all ten now at
complete damage where three had lighter damage; hospitals without power on day 3, three to none;
first hospital reachable from the hub, day 111 to 119; hub after the spill 172.9 to 165.7 million
gallons. The buildings, the agents and the `ranges` variant's own draws are untouched, so the
death counts from the shaking are as they were.

The project README and the earthquake section of the limits page were reread against the ten new
runs and every changed figure rewritten; the README says the numbers are of 14 November and why
they differ. The cover is a new still of the base run and the card is regenerated.

`cargo test` (describe) and `esim regress` pass. The run page was looked at: the tanks' map and
counters, the bridges' panel. Committed, not pushed, not deployed.

### What is still Portland's

`portland.lifelines` holds the hospitals and their generators, the fuel account, water on the
Plan's schedule, the hub's own toll, the story sentences and the `ranges` mode. The next subjects
to pull out are hospitals and the fuel account. Two known disagreements stand: `retrofit` means
built after 2004 in the lifelines' ranges mode and after the folder's design year (1993) in
`fuel.tanks`; and the story gives a bridge's reopening one day early.

## An article he sent

Anselm, 14 November: "I saw an article this am that is interesting", KOIN, Michaela Bourgeois,
"Portland City Council approves reduced fuel storage along Willamette River amid Cascadia
concerns". The fetch tool returned a summary, not the text. Points in it that bear on the model
and are not yet on the page: the ordinance requires a public health and natural hazards risk
reduction plan for the hub by 30 September 2027, covering evacuation from Linnton and North
Portland west of the railroad cut and firefighting access that does not rely on bridges likely to
be damaged; the planning bureau reports to the council in 2028, 2031 and 2036; a 2012 DOGAMI study
is quoted in the ordinance; a September report by McCullough Research prompted the mayor's
ordinance; the hub has 11 terminals. Added to the README's reading list as read through a summary
with nothing taken; to be checked against the ordinance's text before use.

Later the same day he pasted the article's text. The README now uses it: a paragraph in *The fuel
terminals* on the plan due by 30 September 2027 (evacuation of Linnton and North Portland west of
the railroad cut, firefighting capacity that does not depend on the Willamette bridges), the
reports of 2028, 2031 and 2036, the 2012 DOGAMI study as the ordinance quotes it, and the
McCullough Research report; and a sentence beside the bridges result that the city plans on the
same assumption the model's result resembles. The reading list entry says what was taken and
notes that the article's "holds 90 percent" differs from the county's "passes through".

## The place in three dimensions, by layer (14 November)

Anselm: "I would like to have a map in 3d using real elevation data for the region with an
ability to turn modules grouped by layer off and on. I had generated a visualization style
reference here that may be useful - we worked hard at making this communicate visually:
https://github.com/anselm/bay-area-twin".

The reference is Bay Layers: a dark canvas, luminous wire-like relief, four layers on the same
coordinates that separate vertically to be seen apart and come together to show they are one
place, teal relief, amber built world, small tracked labels, constant slow motion.

What was built:

- **The layer tag.** `Describe` has a `layer`; `esim_modules::describe` fills it from `layer_of`
  by the kind's family, so all 46 kinds have one without touching their describes. `LAYERS` in
  `crates/modules/src/lib.rs` holds the order agreed on the 13th. `docs/system/layers.md` is
  rewritten to that order.
- **`web/src/lib/Layers3D.svelte`**, on the run page above the modules, for any run with an
  elevation frame and layered modules. The relief is the run's own `terrain_elev_m` as contour
  lines every 20 m over a dark surface, vertical times four. Each module's frames are points of
  light in its layer's colour at the cursor's time; a frame of harm runs from the layer's colour
  to red. The things modules place (tanks, bridges, road segments, hospitals, the hub) are marks
  from the log, red when down, with pins and names for bridges, hospitals and the hub. A panel
  lists every layer in order with its modules; a layer with no module is listed, greyed, as
  empty. A slider lifts the layers apart. It turns slowly unless the reader prefers less motion.
- Nothing is a map tile or a picture: every point is from the run folder.

Looked at in a headless browser on the base run, together and apart. Three faults found by
looking and fixed: the canvas was drawing at its default 300 by 150 pixels because the resize
ran before the scene existed; the electricity frame lit the whole city as a dot grid, and is now
drawn where service is missing; names piled up downtown and now give way to each other, failures
first.

Not done: the bridges are pins, not spans; roads are points at segment centres because the run
does not carry their geometry; there is no water surface; the earthquake itself is not animated;
frames belong to their module's layer, so the people in buildings sit in the built layer. The
other projects' runs need rerunning before they show the view.

Anselm, the same day: "This additional 3d view can be near the top of the display." On the run
page it already sits directly under the timeline. The project page now opens with it too
(`web/src/lib/ProjectScene.svelte`): the newest run of the project's first variant that the view
can draw, opened a week after the event, with a timeline of its own and a link to the run. When a
project has no such run the cover picture is shown as before; the cover file still makes the
share card. Looked at on the Portland page at desktop width and at 420 pixels, where the layer
list starts folded.

## Roads and bridges drawn, the last pieces split out, water, effects, one 3D view for all (14 November)

Anselm, in three messages: "Ok split the modules. Can you try render roads and bridges."; "Add
hydrology. Animate earthquake slightly for 3d view. Can we show some kind of visual special effect
when a bridge, oil tank or hospital fails ... a light weight effects engine that allows some
customization ... I will leave efficiency issues up to you. I think we should also generalize this
3d view because many of the scenarios such as the pallisades scenarios also will use 3d."

### The split

- `health.hospitals` (`hospitals.rs`) and `fuel.depot` (`depot.rs`) are out of
  `portland.lifelines`. A hospital reads the grid at its place from `power.up`, runs its
  generator down, asks the depot for a day's fuel (`fuel.request`) when `routes.state` says a
  truck can reach it, and is settled on a later tick once the depot has answered
  (`fuel.granted`). The depot owns the `fuel` ledger: the spill on `tanks.released`, the
  response's draw, resupply, and the published deaths at the depot (`depot.toll`).
- `portland.lifelines` in computed mode now restores water on its schedule, tells the story of
  the bridges, roads and services, and gathers `lifeline.state`. Its `ranges` mode is the older
  all-in-one version, kept whole.
- **A module can be switched off in a manifest**: `enabled = false` in its params drops it
  (`Manifest::parse`). The `ranges` variant sets `depot.enabled` and `hospitals.enabled` to false.
- Order within a day: bridges and power 3, routes 4, depot 5, hospitals 6 and their settling 7,
  lifelines 8.

Against the runs of the third batch, every figure the page quotes is unchanged except the
depot's own toll in the base run (1.5 to 1.4 deaths), because the new modules' draws decide
little: no hospital is damaged in any run, and which hospitals lose power follows from power and
roads, whose streams did not change. All ten ledgers close.

### Water

`water.surface` (`surfacewater.rs`): the cells that are permanent open water in ESA WorldCover
2021 (51 km² of the window, the Willamette and the Columbia), as a world field and a frame, and
the tanks' releases toward water set down on the water cell nearest each tank (24.3 million
gallons in the base run, none stranded). It is not hydrology: no discharge, stage, tide or
movement. He asked for hydrology; this gives the water layer an occupant and the spill a place to
arrive, and says what is missing. A flow model on the Willamette (the USGS gauge at Portland, the
tide at the Columbia) is the real next step.

### Roads and bridges on the map

- `roads.routes` carries the segments' outlines (from the folder's `routes.geojson`, each ring
  thinned to a point every 30 m) and the area's main roads from OpenStreetMap (the existing
  `context` fetch, slimmed to motorway through secondary, 34,000 points).
- `roads.bridges` carries each bridge's lines: OpenStreetMap ways tagged as bridges
  (`registry::wildfire::osm_bridges`, new), each way given to the bridge point it passes within
  180 m of. All twelve bridges got lines.
- Overpass was slow: one mirror held a connection open for twelve minutes without answering, and
  the other answered 502. `fetch_reduced` now gives up on a request after 300 s. The first run took
  about ten minutes to fetch both; they are cached and in the lock.
- A run's `describe.json` is now 1.5 MB with the geometry.

### The effects engine

`web/src/lib/effects.js`: one pooled point cloud (6,000 particles, one draw call, a small shader),
recipes by name (`fail`, `failSmall`, `collapse`, `dark`, `restore`, `mark`, `embers`, `dust`), each
a list of specs (count, shape, speed, life, size, colours, gravity, drag, swirl, delay). Nothing
is allocated per frame and the update returns at once when nothing is alive. The 3D view emits
at a thing's place when the cursor steps forward across one of its changes: a tank fails, a
bridge closes, a hospital loses power, something comes back. When hundreds change at one instant
the bursts are thinned. At the event the scene trembles for about two and a half seconds, the
relief flares and dust lifts. An "effects" switch turns it off.

### One 3D view

`Layers3D.svelte` no longer knows Portland. It draws any run with an elevation frame and layered
modules: frames as light, frames of heights as standing lines, and placed things from
`component.state`, `lifeline.*`, `fire.ignition`, `fire.spot`, `place.*` and `record.event`,
located by their coordinates or by their cell. The record is a layer, off at first. A
disturbance's standing map starts off and a moving one (a fire's front) starts on. Checked on
`anselm/palisades-cells` (vector-5, rerun so its describe carries layers): the Santa Monica
relief, the roads, the ignition and the fire's spread draw with no fire-specific code. The older
`Scene3D` (buildings as boxes, the sun, the wind arrow) is still used inside the fire modules'
own panels and was not removed.

### Seen and not seen

Looked at in a headless browser: Portland before, at the instant of the earthquake (effects in
flight, routes turning red, relief flaring) and a week after; Palisades at thirty hours. Not
seen: the effects in motion over time, a real device, a phone. The other Palisades variants and
the other projects still need rerunning to show the view. The cover is now a still of the 3D view.
`cargo test` (describe) and `esim regress` pass. Committed, not pushed, not deployed.
