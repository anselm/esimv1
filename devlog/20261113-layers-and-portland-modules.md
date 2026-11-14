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
