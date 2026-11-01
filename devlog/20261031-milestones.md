# 2026-10-31 — Milestones: a speedrun from abiotic to biotic to human

Anselm, same day: "before we code let's list milestones - we should try speedrun basics from
abiotic to simple biotic ecosystems to finally human implicated systems." This supersedes the
staging in `20261031-deliverables.md`; the deliverables there (D, S, V, H, F, P, X) are the
parts, this is the order they are built in. Proposed by the agent, not yet agreed. Durations
are guesses and are meant to be short: the point of a speedrun is to reach the seams fast
with the smallest thing that closes a ledger at each rung.

## One stage, many actors

Everything runs on one real small basin from M1 onward, so each rung inherits the last
rung's data and the ledgers accumulate. Pick a basin with a stream gauge, MODIS coverage, a
disturbance history and farms, so it rolls straight into the hypoxia chain later. The
tile-drained Corn Belt is the natural place to look; the specific basin is chosen when the
registry (M4) can fetch its data. Each milestone is done when its ledger closes, a picture
exists, a score exists where a scoreboard does, and the devlog entry is written.

## M0 Kernel seed (days)

Clock, event queue in (time, priority, sequence) order, ledger type, append-only log with
replay, minimal manifest, module trait, one trivial module. Determinism property test from
the first commit. Nothing visible yet.

## Abiotic

- M1 Sun (days). Irradiance as a field over the basin's elevation tile: slope, aspect,
  horizon shading, daily and annual cycle, a clear-sky atmosphere factor. Picture: the basin
  lit through a day. Energy ledger: every joule arriving is accounted for. Score: against a
  clear-sky model or a nearby radiation station if one exists.
- M2 Heat (days). Surface temperature as a second field at a different rate, driven by M1.
  Seam test 1. Energy ledger closes across two rates.
- M3 Water (one to two weeks). Rain on cells, infiltration and soil moisture, runoff, flow
  along the DEM-derived network, discharge at the outlet. Seam test 2. Water ledger closes
  from sky to gauge. Score: the basin's stream gauge.
- M4 Weather from outside (one week). Reanalysis for the past and a forecast ensemble for the
  future as boundary conditions, fetched by hash through the registry. First real use of the
  catalog and fetcher. Branches become the forecast members. The hindcast harness starts
  here in stub form, because weather is the first thing with a dated record.

## Biotic

- M5 Green (two weeks). Plant functional types as fractional cover per cell, grown by soil
  moisture, temperature and light, dying by drought and shade, feeding back to
  evapotranspiration and albedo. Carbon and water ledgers close. Score: MODIS greenness for
  the basin, the first biotic scoreboard. The second representation (populations) enters
  the store.
- M6 Eaters (two weeks). Cohorts in cells, a herbivore and a predator, feeding on cover and
  each other, moving by spatial events to neighbouring cells. Population ledger (counts and
  biomass) closes. Picture: predator and prey cycling across the basin. This is the spatial
  event mechanism's first real test.
- M7 Tearing (one to two weeks). A disturbance that crosses a threshold: fire (Rothermel) or
  a flood from M3, as a hazard with a rate read from the state of M5 and M2. Vegetation and
  cohorts respond; a regime shift is possible. First use of the hazard table, with off, on,
  and forced settings.
- M8 Zoom (two weeks, the research rung). The cohorts of one sub-basin become individuals
  with positions and back again, count conserved. Coarse results drive fine boundaries.
  Seam test 6. If this rung slips, everything after it still proceeds at one scale.

## Human-implicated

- M9 Takers (two weeks). A land-use agent, a farm or a logging firm, reads cover and writes
  a field: harvest, clearing, planting. Money ledger joins the others. Seam test 4. The
  third representation (agents) enters the store.
- M10 Rules (two weeks). A zoning or water-right change on a calendar date alters impervious
  surface or withdrawals, which alters runoff, which alters what the taker can do. Seam test
  5, the hard one. First use of the calendar table. The fourth representation (rules).
- M11 Nitrogen and markets (three to four weeks). Fertilizer applied by the farms, nitrogen
  through soil and tile drains into the reach network, load at the outlet, a crop
  disposition ledger (feed, ethanol, export, food) and a price. Nitrogen and money ledgers
  close. Score: USGS nitrate loads. The hypoxia chain is now running from field to river
  mouth, and the fifth representation (networks carrying something other than water) is in.
- M12 Hindcast and ensembles (three to four weeks). Start in 1990 with 1990 knowledge, run to
  2020, a thousand branches, score against the gauge, MODIS and the loads. Add the Gulf end
  of the chain (hypoxic area against the July cruise) if the basin's loads can be routed
  onward with a published model. Job runner and result store land here.
- M13 Facade (three to four weeks). Narrator over a branch log with every sentence linked to
  its ledger entry; a manifest diff from a natural-language what-if; a sober viewer of
  distributions, ledgers, one map, one timeline. Streaming of a live branch.

## After the ladder

The fire-to-water chain and the parcel case reuse M1 to M10 on a new basin and a new
parcel; by then those are configuration, not code. Product steps (public repo, hosted
service, a named user per chain) are as in the deliverables entry.

## Why this order

Each rung adds exactly one representation or one seam, and the previous rung's scoreboard
stays in force, so a regression anywhere shows up as a ledger that stops closing or a score
that drops. The abiotic rungs are cheap and well-understood, which is what a speedrun wants.
The biotic rungs introduce populations, spatial events and thresholds in the order that
each depends on the last. The human rungs introduce agents, rules and markets the same way,
and arrive at a chain with a forty-year scoreboard without a single abstract exercise.
