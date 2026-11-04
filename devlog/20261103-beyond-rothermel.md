# 2026-11-03 — Beyond Rothermel: what the fire engine should become

Anselm, late on 2026-11-03, after the holes in the ground were fixed: "I think our model should
diverge from the rothermal model. I don't see any reason to follow ancient reasoning or ancient
models ... I want to be far more aggressive - we can spend the tokens to reason here." His four
asks: (1) volumetric air with convection, terrain-adjusted wind, indrafts, full coupling, cost
no object, with fast modes if needed, because "currently i am seeing the fire spread in an almost
linear fashion ... like a drop of water dropped on a sheet of fabric"; (2) much finer fuel
granularity, "if I was writing this myself I would literally have individual trees"; (3)
interventions from CAL FIRE's logs; (4) buildings shown burning, not one colour. The purpose:
"real predictive assessments of the shelter individual plots of land may have ... for insurance
purposes, for home buyers and for real world use."

This entry is the advice I gave, with the reasoning, so it can be argued with later.

## Where Rothermel belongs

Agree that it should not be the engine. It stays as what it already is in this repository: a
baseline module anyone can run beside ours, because it is what the operational tools (FARSITE,
FlamMap, ELMFIRE; recalled) still run, and a reader who knows fire will want to see it. The
modern physics-based line is a different family: FIRETEC (Los Alamos), WFDS (NIST's fire dynamics
simulator extended to vegetation), and the coupled fire-atmosphere models WRF-SFIRE and CAWFE
(recalled; none fetched today). Those are the reference points for what "aggressive" means, and
WRF-SFIRE is open source and could be run as an external module as a check on anything we build.

## Why the fire looks like a drop on fabric

Four causes, in order of weight, all fixable:

1. The wind is one flat field per hour. An isotropic heat budget under a uniform wind is an
   ellipse by construction. Nothing in the engine can make a finger run down a canyon because
   nothing in the wind knows the canyon is there.
2. Slope enters as a multiplier on the reach in the uphill direction, and until this afternoon
   it was dominated by phantom cliffs; the real slopes are gentle by comparison and the term was
   never tuned on them.
3. Fuel is a cover class. Shrubland is one number across the whole range; the real range is a
   mosaic of chamise, sage, grass, bare rock, 2021 burn scar and ornamental gardens, and the
   fire's fingers follow that mosaic.
4. The maps show 3-arcsecond blocks of the earliest arrival in each block, which smooths every
   front.

## Priorities, in the order I would build them

### 1. Structures as bodies (his 4, and the half of 2 that pays)

The single most valuable thing for his stated customers. For a plot of land the question is not
where the front goes but whether this house ignites, and in wildland-urban fires that is set by
ember exposure, radiant exposure from burning neighbours and vegetation within a few tens of
metres, and the structure's own vulnerability (vents, roof, decks, windows, defensible space).
That is the finding of the post-fire structure studies as I recall them (Cohen's structure
ignition work at the Fire Lab, NIST's case studies, IBHS's testing; recalled, to be cited
properly when the module is written). We hold the test data for exactly this: 13,292 footprints
with heights, 12,081 inspections with 6,845 destroyed, and the record of where the fire was each
hour. Build:

- A structure as an agent: footprint, height, material and year where the inspection form
  states them, distance to each neighbour and to the nearest wildland fuel, a vulnerability
  score with declared guesses.
- Ignition by two channels: radiant exposure from burning cells and burning neighbours (a view
  factor by distance and size, recalled from the flame-radiation literature, declared as a
  guess), and ember flux as a function of upwind burning area and wind speed, with a per-structure
  acceptance probability.
- A burn curve per structure (ignition, flashover, collapse, burnt out) that makes it a heat and
  ember source for its neighbours.
- Fit on the Palisades inspections, test on the Eaton inspections, report the per-structure
  score as a receiver-operating curve, not a single number. Then the insurance product is the
  distribution of outcomes for one parcel across an ensemble of winds.
- In the 3D view every building carries its state and, afterwards, its recorded fate: four
  colours, and a fifth for "the model and the inspectors disagree".

### 2. The wind shaped by the ground (the first half of 1)

A diagnostic terrain-following wind: take the 3 km HRRR field and solve for a mass-consistent
flow over the 30 m terrain, the approach the Fire Lab's WindNinja takes (open source; recalled),
in Rust, on the window. This is days of work, not months, and it is the step that turns the
ellipse into fingers down canyons and slackening in the lee. One-way, so it is still cheap enough
to run an ensemble of winds. It does not need an octree: the window is about 24 km by 18 km; at
30 m by 50 vertical levels that is about 29 million cells, a few gigabytes dense, and canada1
has 128.

### 3. Fuel as fuel (the rest of 2)

Go to 10 m cells, WorldCover's own resolution, and give each one a load, a depth, a canopy
height (the Meta tile for the window is identified), a live moisture by season, and a time since
last burn (the 2021 Palisades scar from the fire history). LANDFIRE's 30 m fuel-model and
canopy layers are the standard here; I noted earlier that the download needs an email, which
is to be checked. Individual trees: with 1 m canopy height a crown can be segmented, but we have
no load per tree and no fuel moisture per tree, so a tree would be a 10 m cell with a taller
number in it. I would stop at 10 m cells with real fuel attributes and keep trees as the step
after, when canopy data earns it.

### 4. Interventions from the record (his 3)

Checked today against the NIFC Operational Data Archive 2025 (the service that gave us the
perimeters): the Palisades incident has 2,334 event lines (the first page of 2,000 holds 1,110
completed dozer line segments, 425 completed hand lines, 283 roads used as line, 95 fire edges
collected in the field, 35 planned dozer lines) and 825 event points (313 dozer pushes, 43 drop
points, 34 division breaks, 30 helispots, 18 dip sites, 4 safety zones, 2 spot fires, 2 mobile
retardant bases). The earliest line carries a creation date of 2025-01-09 19:40Z, about 49
hours after ignition, so the first two days have no line data and the record is of what was
built, not of when it held. Still: the lines as barriers of declared strength, timed by their
dates, is a one-day module, and the daily perimeters are the test of whether the barriers
explain where the fire stopped. Retardant drops are not in this archive; the ICS-209 daily
reports give resource counts, not positions; the after-action reports are prose.

### 5. The air itself (the second half of 1)

Build it, after 1 to 4, as a dense 3D solver in Rust: an anelastic or Boussinesq large-eddy
model on a terrain-following grid over the window, the fire's heat release as a surface source,
the HRRR field as the boundary condition each hour. At 100 m horizontal and 50 levels that is
about 2.2 million cells and a time step of a few seconds; four days of fire is on the order of
a hundred thousand steps, which is minutes to hours on this box. At 30 m it is 29 million cells
and a one-second step, which is hours to a day. Both are feasible without any sparse structure;
the sparse structure belongs to the fuel and structure layers at 1 to 10 m, not to the air.
What the coupled model buys: indrafts, plume-driven spread on the second and third days, the
plume laying embers out ahead, downbursts from a collapsing column. What it costs: it is a
research project with its own validation problem, and until 1 to 4 are in place it would be
accurate air over wrong fuel and invisible houses. The order matters more than the ambition.

### Modes

He is right that speed is a mode. The same manifest with the air solver on is the assessment;
with the diagnostic wind only it is the twenty-minute answer; the structure model runs in both.

## What I would not do

- Replace the cover-class fuel with individual trees before the fuel attributes exist.
- Start with the 3D air, because every later module changes what the air is asked to explain.
- Tune any constant on the Palisades alone; Eaton is held out from the first fit.

Session: Claude Fable 5.1, the esimv1 room on canada1. The archive counts above were fetched
today; everything marked recalled is to be fetched before it is cited on a page.
