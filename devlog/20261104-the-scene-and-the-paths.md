# 2026-11-04 — The scene, the paths, and the story under the runs

Three asks from Anselm in the late morning, after the vector form was pushed.

"Improve the 3d rendering with a night/day backdrop and also stretch the vertical elevation by 2x
for visual clarity - what else are quick wins for visualization?" Then: "revise the docs on the
main page to add more detail about the scenario - based on all of the accumulated text we now have.
also, i think we need to move the variants and runs above the story", and "revise svelte router to
use urls such as /anselm/pallisades-2025 instead of ?project= ? what other things compete for the
/anselm namespace?"

## The scene

`Scene3D.svelte` now:

- stretches the vertical twice, ground and buildings alike (`ZS`), and says so in the legend;
- lights the ground from where the sun stands over the window at the cursor, with the backdrop
  following it: a January afternoon is pale blue, the hour after sunset warm at the horizon, the
  night near black with a cold glow so the ground still reads. The sun's place comes from the
  low-precision solar formula (the Astronomical Almanac's, recalled; good to a fraction of a degree,
  which is more than a backdrop needs). The caption carries the sun's elevation and the local clock
  time, the local zone taken from the longitude (whole hours; the solar zone, not the legal one, and
  it says so);
- colours cells at or below sea level as sea, so the bay is a bay;
- drapes the mapped perimeter current at the cursor on the terrain as dots along its edge, so the
  model and the record are on one picture for the first time in three dimensions. This did not draw
  on the first two tries and the reason is worth a line: the page plays, the cursor moves every tick,
  and an effect keyed on the cursor cancelled its own fetch before the frame arrived. The fire frames
  key on the frame index and never had the problem; the perimeter now does the same;
- draws the wind at the ignition point as an arrow at the north-west corner, 250 m per m/s, pointing
  the way it blows, and names it in the caption.

Checked light and dark at a January afternoon, dusk and night.

Quick wins still on the table, each an hour or less: land cover as the ground's colour where nothing
has burned (the frame is in every run); the cursor playing when the figure scrolls into view; a
north mark; the structures' fate drawn when the bodies variants are chosen (already there) and the
spot fires' flight as a short arc from their source cell; hour labels on the arrival ramp; the
record's lines thickened by their length; a second camera preset looking up the canyons from the
sea. The one that is not quick is land cover at 10 m with trees, which is item 3 of the asks.

## The paths

The viewer routes by path: `/anselm/palisades-2025` is the project, `/anselm/palisades-2025/<run id>`
a run, `/anselm/planet-2100/compare` the compare page, `/docs/system/limits` a documentation page,
`/runs` and `/run/<id>` the loose runs, `/new`, `/kinds`, `/all` as before. The query forms are still
read, so old links work, and the server falls back to the page for an extensionless path under
`/docs/`, `/runs/` and `/projects/`, which it serves as files otherwise. Markdown links in the docs
and the READMEs were changed from `?doc=` to `/docs/`, since a relative query link on a path URL
would have resolved under the project.

What competes for `/anselm`: nothing that is a page, and the list that is not is `RESERVED_AREAS` in
`server.mjs`, which the area claim already refused before today and the router now treats as top-level
words: `docs`, `doc`, `system`, `runs`, `run`, `new`, `kinds`, `api`, `compare`, `all`, `projects`,
`project`, `index`, `assets`, `socket.io`, `guest`, `esim`, `admin`. The first three segments
after the handle are a project name, then a run id or `compare`. A page of the area's own
(`/anselm/notes`, the crumpled namespace in the todo) would be a project name without a manifest,
which the router does not yet distinguish; that is the next decision, not today's.

## The story under the runs

The project page now shows the variants and the runs first and the README after them. The first
Palisades project's README gained a section, *The fire*, written from the record the event folder
holds: the Lachman rekindle, the first hour as UL's reconstruction gives it, CAL FIRE's reported
acreages, the incident's sparse maps, the GIS archive's lines and their dates, the resource counts,
and what the reporting after the fire says about the first day (as summarised to Anselm by his own
tool and passed on; the Los Angeles Times refuses fetches from this box, so the page says it was not
read here). The second project's README points to it. Heckler flags nothing beyond its usual
metronome note on the opening paragraph.

Session: Claude Fable 5.1, the esimv1 room on canada1, 2026-11-04 late morning.
