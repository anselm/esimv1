# The real fire's edge, the satellites, and a fit

19 November 2026. Written by Claude (Claude Code session in the esimv1 room on canada1), on Anselm's order.

## The ask

Anselm, after asking whether our own fire engine runs on the same clock as Rothermel and why it overruns:
"is there public data on the real fire behavior over time? can we show that? i'm starting to wonder if we can
build a regression to tune our model against real data." I recommended showing the record clearly, adding
satellite detections for the early hours, then a fit with a split in time. He answered: "we should visually
show the curtain edge of the real fire" and "lets try as you recommend".

## What was done

1. **The curtain.** `web/src/lib/Layers3D.svelte` draws a record frame that is a share of each cell inside a
   mapped edge, and that moves, as a wall of light on the ground along the edge. The area is closed over gaps
   three cells wide and only its outer edge is walled. A second such record in a run is tinted blue. Both are on
   from the start; the record's other maps and marks still start off.
2. **The satellites.** NOAA publishes the VIIRS 375 m fire detections of Suomi NPP, NOAA-20 and NOAA-21 in open
   buckets (product `VIIRS_EFIRE_VIIRSI_EDR`), anonymously. `data/events/palisades-2025/viirs/fetch.py` read
   6,075 files and kept 3,751 detections in the window, in 43 passes, the first 76 minutes after the ignition
   (`hotspots.csv`, 576 kB; catalog `noaa-jpss-viirs-active-fire`). NASA's FIRMS needs a key we do not have.
   `obs.hotspots` reads the file in both Palisades manifests as module `seen`. It gained `series` (a prefix, so
   its series do not collide with the perimeters' `obs.acres` and `fit.sorensen`), `land_only` (cells at or
   below sea level stay unmarked) and a map view. Both default to the old behaviour; a Fort McMurray run kept
   its digest.
3. **The fit.** `esim fit` in `crates/cli/src/main.rs`: rounds of Latin-hypercube samples in parallel, the
   first over the stated ranges, each later one a box half as wide around the best. Terms are `--area` (absolute
   log of the ratio, capped at 3) and `--overlap` (one minus the series), each read only at the instants its
   record series takes a new value, each optionally windowed. `--train-until` splits the record: the search sees
   the instants before, the instants after are scored and reported and take no part. Output is
   `runs/<name>-fit-<hash>/fit.json`. Documented in `docs/system/projects-and-runs.md`.

## What the fits found

Terms for both: overlap with the satellites to 12:00 UTC on 8 January; area and overlap against the mapped
perimeters from 14:00 UTC on 8 January (which leaves out the 06:00 triangle). Base: the vector form, idle
cooling, reach 45 m.

| | runs | constants | error on the first 27 h | error on the later days |
|---|---|---|---|---|
| `vector-5`, set by hand on 4 November | 1 | | 0.302 | 0.779 |
| fit shown the first 27 h (`fit-5094a4dd`) | 72 in 131 s | 6 | 0.292 | 0.764 |
| fit shown everything (`fit-68cbdc9c`) | 128 in 278 s | 8 | 0.515 over all four days | |

The search confirms the hand setting and fails the later days as it does. Shown everything, it settles on a
slow fire (6,520 acres at 27 hours against 15,824; 42,047 at the end against 22,347). No setting grows as fast
as the first night and then stops. This is the stopping problem I named before the work, now measured: the
engine has nothing that puts a fire out. The two best sets are project variants `fit-day-one` and
`fit-whole-record`.

Against the satellites the engine is slow early: 17,795 acres seen by 16 hours, about 5,500 burned in
`vector-5`. The seen area overstates (21,965 at 27 hours where the infrared map has 15,824; 30,024 at the end
against 22,347 mapped in the window), so the size of the gap is uncertain, but not its direction.

## Limits and what is next

- One fire. Fort McMurray has hotspots and could be the second, as a test of a Palisades fit or the reverse.
- The fit's loss gives the satellites only an overlap term, because their area is inflated. Shrinking the
  footprint to the pixel's stated area, or using only nominal and high confidence rows, has not been tried.
- The search is a shrinking box, not a gradient or a Bayesian method. It is deterministic by seed and cheap at
  this run time (about two seconds a run with 24 threads).
- The detections were not compared with FIRMS, and CAL FIRE's reported acreage series in
  `data/events/palisades-2025/calfire/updates.csv` is a third record of the first day that no module reads.
- The engine needs a way for a fire to stop before more fitting is worth doing: the wind's drop already
  reaches it, so the candidates are suppression that acts from resource counts (CAL FIRE's reports have them
  from the afternoon of 8 January) and fuel moisture recovering overnight.
- Every Palisades variant of both engines was rerun with the satellite record (23 runs). The Rothermel runs
  are scored against the satellites too but still have no elevation frame, so no 3D view.
