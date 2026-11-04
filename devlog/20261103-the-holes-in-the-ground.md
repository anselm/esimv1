# 2026-11-03 — The holes in the ground

Anselm, looking at the three-dimensional view of the engine run, asked why the terrain was
striped, and whether the 3D scene was "simply a 3D surface with the other model draped on
it". The second is true by design: `Scene3D.svelte` draws the terrain frame as a mesh, the
building frame as boxes and the arrival frame as vertex colour. The first was a bug that
both fire engines had been running on all day.

## What it was

`dem_window` in `crates/registry/src/terrain.rs` bins every source pixel centre into an
output cell. Copernicus GLO-30's pixel centres sit on whole arc-seconds; the output grid's
edges also sat on whole arc-seconds (`lon_w` floored to a multiple of the cell). So every
source centre lay exactly on a cell edge, and floating-point rounding chose the side. In the
Palisades window at one arc-second, 222 of 937 columns received no pixel at all and came out
NaN (a repeating 3, 3, 3, 3, 7 pattern), plus the last row. The frames shown on the maps are
reduced to 3-arcsecond blocks (earliest arrival, most common class), which hide a one-column
hole; the full-resolution terrain mesh was the first view that did not.

The slope code (`DemWindow::slope_aspect`, Horn's method) reads a NaN neighbour as 0 m. A
hole between two columns at 450 m is a 450 m trench one cell wide, so the two columns beside
every hole had a slope near 84 degrees facing the hole. Rothermel's slope factor goes with
tan² of that (about 90) and `fire.cells` stretches its reach by tan (about 10) in the uphill
direction. Both engines had been burning across a landscape of east-west cliffs. The Rothermel
overrun that the project page, the limits page and the Substack draft all reported as "four
times the record at 27 hours" was this.

## The fix

The grid is laid half a source pixel off the lattice (`SOURCE_ARCSEC = 1.0`, a constant for
GLO-30 below 50 degrees; the reader warns if a tile's pixel disagrees). At the DEM's own
resolution each cell is now centred on its pixel; at coarser resolutions no pixel centre lies
on an edge. `RECIPE_VERSION` went to `terrain-v3`, so the derived caches are rebuilt. The
window's `lon_w` and `lat_n` move by half an arc-second, and every layer that derives its
grid from the window (cover, buildings, roads, perimeters, inspections) moves with it.

The Boone basin at 15 arc-seconds had no holes (fifteen pixels per cell, one ambiguous at the
edge) but its bins shifted, and `esim regress` drifted: NSE 0.4685 to 0.4751, KGE 0.4352 to
0.4412, PBIAS 31.4 to 28.6 percent. Accepted with `--accept`; this entry is the explanation.
Tests pass.

## What the fire looks like on real ground

Every variant of both projects rerun (`esim project run`, old runs pruned to one per variant).
The first infrared perimeter is 15,824 acres at 27 hours; the last mapped is 23,343.

| run | 27 h before | 27 h after | end before | end after | overlap at end | destroyed reached |
|---|---|---|---|---|---|---|
| rothermel reanalysis | 79,107 | 2,031 | 80,304 | 20,892 | 0.58 | 0.28 |
| rothermel gusts | 75,668 | 2,498 | 80,904 | 34,577 | 0.62 | 0.91 |
| rothermel built-barrier | 67,843 | 1,606 | 68,151 | 19,589 | 0.56 | 0.07 |
| rothermel sh7 | 78,762 | 1,834 | 80,193 | 13,970 | 0.56 | 0.25 |
| cells near-4 | 22,431 | 15,306 | 50,340 | 46,759 | 0.59 | 0.12 |
| cells houses | 23,093 | 15,458 | 50,842 | 51,702 | 0.56 | 0.19 |
| cells houses-jitter | 22,844 | 15,062 | 50,577 | 43,773 | 0.61 | 0.16 |
| cells near-2 | 65,315 | 56,278 | 67,201 | 63,159 | 0.51 | 0.65 |
| cells scale-6 | 69,187 | 0 | 69,504 | 0 | | |

The picture flipped. Rothermel on the 25 km reanalysis wind is now about eight times too
slow on the first day (395 acres at the triangle's hour against 2,920; 2,031 at 27 hours
against 15,824) and reaches the record's size at the end of the fourth day, when the real
fire was being held; the shape it ends with is fair. The gusts variant is the only one that
overruns, and only by the end. Our own engine felt the cliffs less, because its slope term is
linear in the tangent and acts only in the uphill direction; its near-reach runs now sit on
the record's 27-hour acreage almost exactly (15,306 against 15,824 for near-4) and still run
to twice the record by day four because nothing stops them. The scale-6 variant, which burned
69,000 acres on the cliffs, burns nothing at all on real ground: the ignition cell's
neighbours never reach threshold. Destroyed-structure recall in the plausible-pace runs fell
from 0.32 to 0.19 for the houses variant; the fire had been reaching the neighbourhoods partly
along the phantom slopes.

The lesson for the honesty rule: the overrun was reported with the right caveats (no
suppression, no barriers, recalled fuel table) and the wrong cause. Every number on the
pages, in `limits.md`, in `CLAUDE.md` and in the Substack draft was rewritten this
afternoon; the draft's fire paragraph now says the model is slow on the first day, not fast.
The covers of both projects were re-rendered from the new runs.

## Questions he asked along the way, answered on the pages

Both engines are cellular automata on a one-arcsecond grid, neither fitted statistically:
Rothermel is a 1972 physical-empirical rate per fuel type shaped into an ellipse; `fire.cells`
is a heat budget with declared guesses. Elevation enters both as slope and aspect for the
fire's spread and nowhere for the wind: the wind is a flat field (one ERA5 point; HRRR at 3 km
interpolated to each cell), with no terrain channelling and no feedback from the fire (no
convective column, no indraft, no plume-driven spread). Vegetation is a WorldCover class
mapped to a fuel model (Rothermel: six classes to five Scott and Burgan models plus a guessed
urban one) or to one of four kinds with three numbers each (`fire.cells`); no species, fuel
load, time since burn, canopy height or seasonal live moisture. Added to `limits.md` and to
the todo as the next modules: terrain-adjusted wind first, a local indraft second, a fuel map
made for fire third.

Session: Claude Fable 5.1, the esimv1 room on canada1, 2026-11-03 afternoon; this entry was
written at the end of it. Scratch tables in the session scratchpad (`summarise-fire.py`).
