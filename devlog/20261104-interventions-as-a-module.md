# 2026-11-04 — Interventions as a module, breadcrumbs, and the record that fades

Anselm, early afternoon, four chores. The first three are here; the fourth, Fort McMurray, has its
own entry.

## 1. "please correct modularization of interventions"

His reason, in his words: "a modular approach gives us the ability to repurpose the single fire
simulator for different scenarios around the world." This morning's `lines = "hold"` had the fire
engine subscribe to the record and rasterise the lines itself, one engine's private rule. Now:

- `fire.suppression` (`crates/modules/src/suppression.rs`) sits between the record and the engines.
  It subscribes to `record.event`, and for each completed line (category Completed …, status not
  Archive) publishes a `ground.break`: the cells under the line to `line_width_m`, a `strength`, the
  time it holds from, and the line's category, label and length. It reports lines and cells held.
  In `mode = "off"` it publishes nothing and the record is drawn, not defended.
- `fire.cells` subscribes to `ground.break` and refuses heat and embers on the cells named.
  `fire.rothermel` subscribes too and makes them unburnable. Neither engine knows what a record is.
- Both Palisades manifests carry the module with `mode = "off"`; `vector-5-held` sets
  `suppression.mode = "hold"`. The project manifests were re-resolved and every variant rerun.

The seam is the message, not the module: anything that can say "these cells, this strength, from
now" is an intervention, and the next ones (crews with capacity placed by rule from CAL FIRE's
resource counts, aircraft under a wind limit, retardant as a heat sink) are publishers of the same
message or of a sibling (`ground.sink`). The ledger of resources belongs in that module, so the
auditor sees what was deployed. None of that is built; the note in the describe says so.

## 2. "make fragments clickable so i can go to /anselm"

`/anselm` is now an area page: the project list under one handle. Every path fragment in the
project and run headers is a link (`Crumbs.svelte`: `/ anselm / palisades-cells / 6999ef57`), the
compare page has them too, and "by anselm" under the title goes to the area.

## 3. "have them go away after a bit"

A line in the 3D scene is bright for three hours after its recorded time, fades until twelve, then
goes; points already went after six. At the run's end every line comes back faint, so the last frame
is the whole record on the ground, which is his "the ones at the end of the timeline run that stick
around".

Session: Claude Fable 5.1, the esimv1 room on canada1, 2026-11-04 afternoon.
