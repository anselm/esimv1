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
