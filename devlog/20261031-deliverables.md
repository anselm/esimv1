# 2026-10-31 — Proposed deliverables

Anselm asked "what would a good list of deliverables be?" Proposed by the agent, building on
the brief, the runtime recommendation (Rust kernel, wasm modules under the deterministic
profile) and the three validation use cases (hypoxia, fire to water, one parcel). Not yet
agreed. Durations are the agent's guesses. Every deliverable has a done test that a machine
or an observation decides, and each ends with a dated devlog entry. Nothing here is a product
until stage 6.

## Stage 0: kernel (about four weeks)

- D0 Workspace. Cargo workspace, tests running on canada1, this devlog. Done: `cargo test`
  green with one trivial module stepping.
- D1 Clock and scheduler. One virtual clock, event queue ordered by (time, priority,
  sequence), conservative multi-rate advance: a module never passes the slowest neighbour
  that writes what it reads. Done: property tests prove the same seed and inputs give a
  byte-identical event log on one thread and on eight.
- D2 Ledgers. Conserved quantities with units (mass, water, energy, carbon, nitrogen, money,
  people), closed every period to a stated tolerance. Done: a module that leaks one gram
  fails the test suite.
- D3 Event log. Append-only, periodic snapshots, replay from any snapshot, fork at an
  instant, diff two branches on one topic. Done: replay from a mid-run snapshot reproduces
  the original log bitwise.
- D4 Manifest. A declarative document (TOML or JSON) naming modules by version, sockets,
  parameters, data sources, clock mode, calendar and hazard settings. Done: a run is fully
  specified by one manifest and one seed, and a one-line manifest diff is an intervention.
- D5 Module boundary. One Rust trait for native modules, and a Wasmtime host running the
  same trait for wasm modules under the deterministic profile, with a private memory per
  module and a shared world-state memory. Done: one module compiled both ways yields the
  same event log.
- D6 World state and reference data. Parallel typed arrays with units for entities and
  fields; a query interface over large read-mostly reference data (elevation first). Done:
  a GEBCO or SRTM tile served to a module through the query interface.

## Stage 1: seams (one to two weeks each, each a closing ledger)

- S1 Two fields, two rates. Daily irradiance driving an hourly surface temperature field.
  Energy ledger closes.
- S2 Field to network. Rain on cells to runoff to river reaches. Water ledger closes.
- S3 Network to population. Reach stage drives a floodplain forest cohort. Carbon and water
  ledgers close.
- S4 Population to agents. The forest feeds a logging firm whose decision changes the
  forest. Carbon and money ledgers close.
- S5 Agents to rules to fields. A zoning change alters impervious surface, which alters
  runoff. The hard one. Water ledger closes across the rule boundary.
- S6 Nested scales. The same modules at parcel, basin and continent; coarse results set
  fine boundaries; a population switches between density and individuals with the count
  conserved. Population ledger closes across the switch.

## Stage 2: validation core (about three weeks, can overlap stage 1)

- V1 Observation store and scoring. Observed series with provenance, skill scores for point
  forecasts, rank histograms and CRPS for ensembles. Done: a known-bad forecast scores worse
  than a known-good one on a fixture.
- V2 Hindcast harness. Start date, knowledge cutoff, calendar as of the start, run to an
  end date, score. Done: a hindcast of S1 against a real irradiance series.
- V3 Ensemble runner. N seeded branches across threads or machines, output as a
  distribution with each branch's log attached. Done: a thousand branches of S2 on canada1
  with timing recorded.
- V4 Calendar and hazards. The two event tables from the brief, seeding the queue before a
  run; hazards as modules emitting rates, each settable off, on, or forced. Done: a forced
  hazard appears in exactly the branches and dates the manifest says.

## Stage 3: the spine, Gulf hypoxia (about two months)

- H1 Data. County fertilizer and manure, crop calendar, tile drainage, USGS outlet nitrate
  loads, the July cruise series, shrimp landings and prices. All with provenance in the
  observation store.
- H2 Reach routing module. A SPARROW-style nutrient delivery module, proven by matching
  published outputs for the basin.
- H3 Hypoxia module. A NOAA-style load-to-area relation driven by summer weather branches.
- H4 Hindcast. Forty seasons, scored against the cruise. Done: a stated fraction of years
  land inside the ensemble band, and the misses are explained in the devlog.
- H5 Intervention. A cover-crop subsidy in a chosen year as a manifest diff, rerun, and a
  diff of the two branches on hypoxic area and on the crop disposition ledger.

## Stage 4: fire to water (about two months)

- F1 Fire spread module (Rothermel), proven against a documented fire's perimeter.
- F2 Post-fire hydrology: burn severity to infiltration to runoff and sediment, with a
  debris-flow hazard whose rate reads burn severity and storm intensity.
- F3 Hindcast one real fire and watershed pair against gauge turbidity and sediment records.
- F4 Consequence chain: reservoir, treatment cost, a fish or ungulate cohort. Ledgers close.

## Stage 5: the parcel and the facade (about six weeks)

- P1 A parcel manifest nested in its basin, running all five representations for twenty
  years, with the county's rules and the climate in the pipeline.
- P2 Narrator. An LLM turns one branch's log into prose, every sentence linked to the
  ledger entry or event that produced it. Done: a reader can click any sentence to its
  evidence.
- P3 Conversational manifest editor. Natural language in, manifest diff out, rerun, narrated
  comparison. Done: "what if the county upzones in 2030" produces a valid diff and a rerun.
- P4 Minimal viewer. Branch distributions, ledgers, one map, one timeline. Sober, per house
  style. No globe.

## Stage 6: product

- X1 Public repository with ODD-shaped documentation for every module and the kernel.
- X2 Hosted service that accepts a manifest and returns an ensemble, billable by branch-year
  (a guess at the unit).
- X3 One named real user per chain who has run it on a working-day question. This is the
  only deliverable that proves the product claim.
- X4 A written account of what the model gets wrong, per chain, kept current.

## Cross-cutting rules

- A deliverable is done when its test passes and its devlog entry exists, not when the code
  compiles.
- Every ported module is proven against the original's published outputs before it is used.
- No deliverable adds a parameter without a source for its value or a marked guess.
- Stage 3 starts before stage 1 ends; the seams are built inside the spine, not in the
  abstract.
