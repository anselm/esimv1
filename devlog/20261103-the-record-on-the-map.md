# 2026-11-03 — The record on the map: interventions and times in the 3D view

Anselm, late evening: "Do we have locations for interventions? If so can we show a pulse or
animation on the 3d map at those points. Also can we print the times events onto the map 3d
view?" Earlier in the same message: "The rothermel model still feels more accurate than ours but
it is closer now."

## What we hold

Placed and timed, yes: NIFC's event points (825: dozer pushes, drop points, helispots, dip sites,
division and branch breaks, the incident command post at Will Rogers) and event lines (2,334:
completed dozer, hand and road-as-line, planned lines, field-collected fire edge), copied into
`data/events/palisades-2025/nifc/` this afternoon. Their time is `CreateDate`, when the GIS unit
made the feature; the dataset's own `PointDateTime` and `LineDateTime` are empty for this incident,
so every time shown lags the work by some hours. The first point is made at 20:34Z on the 8th, the
first line at 19:40Z on the 9th. 705 of them fall inside the run's window and days. CAL FIRE's
status reports carry resource counts and times but no positions; UL FSRI's 4,785 radio and press
events carry addresses, not coordinates, and would need geocoding.

## What was built

`obs.record` (`crates/modules/src/record.rs`): a module that reads GeoJSON files from the event
folder, keeps the features inside the window, and publishes each as a `record.event` at its time,
at the cell it stands or starts in, with its category, label, note, status, the thinned path of a
line and its length. It is in both Palisades manifests now; every run of both projects was redone
so its log carries the record (the project manifests were re-resolved with `esim project update`).
Nothing in the model reads these events yet. They are the record drawn beside the model, which is
the honest form until the interventions module exists: a dozer line in the log is not a dozer line
in the fire.

The 3D view (`Scene3D.svelte`) now reads the run's log for `record.event`, `fire.ignition` and
`fire.spot`, and draws:

- a ring on the ground where each point event happened, which appears at its time, grows and
  breathes for a while after (six hours of run time for the record, an hour and a half for the
  model's spot fires, the whole run for the ignition), then goes;
- each completed or planned line as a line draped on the terrain from its time on, bright for
  three hours and then dimmed, planned lines fainter;
- a label with the event's time in UTC and its category and note, projected from the point,
  stepped down when labels land on each other, one per distinct text, the latest two dozen;
- a count in the legend of how many points and lines the record has by the cursor.

Colours: the record teal, spot fires orange, the ignition red. The times are the run's own clock,
so the ignition label reads 18:30Z from the event's data rather than the hour the module ticked.

## What it shows

At 59% of the slope-6 run (06:26Z on the 10th) the record has 111 points and 23 lines: helispots
and dip sites in the north, drop points along the western flank, the first completed dozer lines
down the eastern edge above Brentwood, which the model's fire has already crossed. That is the
picture the interventions module has to change: those lines should hold where the record says they
held.

Also fixed on the way: the building boxes had drawn black since the scene was first written (a
`vertexColors` flag on a geometry without colours), so the per-building colouring of the bodies
runs had never actually shown (`devlog/20261103-bodies.md` has the note).

Next, in order: the interventions module (lines as barriers of a declared strength from their
time, with the lag stated), the engine's rate form (`devlog/20261103-the-slope.md`), terrain
wind. The UL events could be geocoded to join the map; the CAL FIRE resource counts could drive
the strength of the lines.

Session: Claude Fable 5.1, the esimv1 room on canada1, 2026-11-03 late evening.
