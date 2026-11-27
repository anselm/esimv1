# A second round on the parts, and the text that tells it

26 November 2026, evening. Claude (Fable 5.2) in the esimv1 room on canada1, session
`1cc66e9f-2882-4c70-b8d3-6d9d3cf1beb1`. Follows `20261126-can-the-parts-be-separated.md`.

Anselm: "go ahead and do another round of improvements. also update any user facing text to
reflect these changes (since part of what we are doing is to tell a story to funders)". Then,
while it ran: "feel free to commit, push, deploy as well."

## What was built

- **Both fire models read the same parts.** `fire.rothermel` has `inputs = "world"`: it fetches
  nothing and reads relief, land cover, weather and fuel moisture from the modules that hold
  them. Palisades 2025 and Devil's Gulch on Rothermel are converted (manifests f2 and f6), so all
  five fire scenarios are assembled from the same parts and differ in the fire model. The gusts
  variants set `weather.wind_use`, not `fire.wind_source`. In this form Rothermel takes the
  window's mean fuel moisture, because it works out its fuel beds once an hour per fuel type.
- **The fire says when each cell caught.** In the separated form both fire models write
  `fire.arrival_s`. This is the answer to the hourly coupling for a neighbour that only listens:
  it replays the hour from that field. A neighbour the fire must react to within the hour still
  needs a shorter step or to be part of the fire; untried.
- **Animals that run ahead of the front** (`fauna.flee`, rewritten). They notice newly burning
  ground within `sense_m` (600 m, a guess), move away at `speed_m_h` (3 km an hour, a guess) in
  five-minute steps replayed from `fire.arrival_s`, shed crowding above four times the starting
  number, and leave the window when driven to its edge. Those overtaken die at 0.3 or reach
  unburned ground. Habitat comes from the land cover module, not from the fire. Still a
  placeholder with no species; the run pages and the limits page say that no number from it is
  an estimate. Two faults found by looking: the first version funnelled animals into single
  cells along diagonals (9,000 per square kilometre, a blank map), fixed by a fairer choice among
  equal directions and the crowding rule; and the window's edge was a wall against which a
  whole population died in Devil's Gulch, fixed by letting them leave.
- **Fields on different cells can meet.** `landscape::resample` and `field_on` lay a field
  written on other cells onto the reader's (nearest value, no averaging; unit test). The weather
  module can write on coarser cells of its own (`cell_arcsec`), and the fires and the animals
  read through `field_on`. No scenario uses a coarser weather grid yet.

## How it was checked

All 36 fire runs were rerun (20 Palisades cells, 4 Palisades 2025, 4 Fort McMurray, 4 Devil's
Gulch, 4 Devil's Gulch on Rothermel). Every fire series of the 28 cell-model runs is identical
to before the round. For the 8 Rothermel runs the burned area is identical at every hour to the
all-in-one module's; the reported fuel moisture differs in the seventh figure (single against
double precision), and the gust series is gone because the weather module writes one wind. The
wildlife ledger closes in every run. The engine's tests pass. Looked at, on the local server:
the front page (light), the Rothermel Palisades run's wiring tab (dark), the Palisades cells
animals tab (dark).

## The text

A survey of reader-facing text (an Explore agent on Opus, read-only) found stale passages in
twenty files. Changed: the front page statement and its count line; the hub README (a new
section, "Whole systems from separate parts"); the engine's overview page (the same argument
with a table of the parts and the limits); goals, modules, layers, limits, terminology,
workspace, projects-and-runs, pages, the protocol's verb count; the READMEs of the engine, the
viewer and the Palisades repository; the five fire project READMEs (a section "What the
scenario is made of" with the wiring figure) and their descriptions; Earth 2100's README (the
globe is a flat map); Portland's README (one line); the kinds page (titles lead, as in the
wiring diagram). Sign-in and accounts are described as dropped wherever they were described as
coming. The three claims the text now makes (a part can be swapped, reused elsewhere, or added
without the others changing) each point at runs that show it, and each page that makes them
states the limits beside them.

## Open

- The account code is still in the tree; only the words are gone.
- The check does not catch ground modules placed after the fire in a manifest.
- Row order between two tables about the same things is still a convention.
- Twenty older modules still learn their grid from the elevation reader.
- The Boone modules are untouched.
- Nobody outside has written a part and joined it. The limits page says so.
