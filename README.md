# earth-simulator-1

A whole-system simulator: many representations (fields, networks, populations, agents,
rules), many rates, one shared world state, with conservation checked at every seam by
ledgers that must close. Output is an ensemble of trajectories with the mechanism visible,
so a person can inspect a run, rerun it, and intervene by editing a manifest.

The design brief and every decision since are in `devlog/`, dated. Start with
`devlog/20261031-design-brief.md`, then `devlog/20261031-milestones.md`, then the
milestone entries (`m0` to `m11`, `f1`). `devlog/20261031-fitness.md` scores the
framework against the use cases as rungs land.

## What exists (as of 2026-11-01)

- `crates/kernel`: virtual clock, events ordered by (time, priority, sequence), shared
  world fields, cell-addressed events, double-entry ledgers with declared-stock closing,
  append-only log with digest, snapshots and exact replay, TOML manifests.
- `crates/registry`: catalog of base layers and observation series, hash-locked fetcher
  and cache, readers for Copernicus DEM tiles, MODIS NDVI, USGS daily values,
  GHCN-Daily, ERA5 and GEFS through Open-Meteo; depression filling, D8 flow routing and
  basin delineation; the calendar and hazard tables.
- `crates/modules`: sun (clear-sky irradiance on terrain), heat (slab surface), weather
  (ERA5 and GEFS members), water (snow, soil, evaporation, routing, nitrate tracer),
  crop (phenology, light-use growth, harvest), herbivore and predator cohorts (with a
  cohort-to-individuals zoom), fire hazard, farm agents, calendar rules, nitrogen; and
  the first planetary rung, zero-dimensional: observed emissions into a carbon ledger,
  impulse-response sinks, a two-layer energy balance, and the records (Mauna Loa, Law
  Dome, HadCRUT5, GISTEMP) as scoreboards.
- `crates/audit`: the independent auditor of a finished run: spend-down of every ledger
  stock, diversity of populations, shocks and recovery, tears, a verdict.
- `crates/cli`: `esim run | verify | ensemble | sweep | score | audit | check | describe |
  kinds | project new/run | projects`.
- `projects/`: a folder per project, `<area>/<name>/{project.toml, manifest.toml, runs/}`.
  The folder is the project; the CLI writes it and anyone can run it by hand.
- `web/`: the viewer (Svelte 5, Vite): projects first, a project's runs, a run with its
  maps under one time cursor, the audit, scores, charts, ledgers; and `server.mjs`, one
  Node process that serves the built viewer and the run and project folders and speaks
  socket.io over the CLI: list projects, draft a manifest from a description (Claude, or
  a local model through ollama when the API is not usable), check it, create the folder,
  run it.
- `data/`: `catalog.toml` (assets and places), `catalog.lock.toml` (hashes of fetched
  files), `events/` (calendar and hazards), `cache/` (fetched bytes, not in git).

The stage for the basin milestones is the Boone River above Webster City, Iowa (USGS
05481000), chosen for its 1940-to-present discharge record, its 2012-2017 nitrate record,
and its tile-drained Corn Belt land use. The planetary rungs (`p1` onward) have no stage:
the planet is one box until a module needs it gridded.

## Run it

```sh
# Rust stable (rustup) and Node 20+ are needed. GDAL is optional, for cross-checks.
cargo build --release
./target/release/esim run manifests/m3-water-boone.toml           # a year of hydrology, ~40 s
./target/release/esim score runs/<id> hydro.q_sim_daily_m3_s hydro.q_obs_daily_m3_s
./target/release/esim verify manifests/m2-heat-boone-week.toml    # determinism and replay
./target/release/esim ensemble manifests/f1-boone-forecast.toml --branches 31 --threads 8
./target/release/esim run manifests/p1-planet-co2-1850-2025.toml  # 175 years of planet, 1 s
cargo test --release                                              # ~30 s with the data cached
./target/release/esim project new me/first --manifest manifests/m0-buckets.toml
./target/release/esim project run me/first                         # into projects/me/first/runs
./target/release/esim audit projects/me/first/runs/<id>            # the auditor's verdict
cd web && npm install && npm run build && node server.mjs         # the whole thing on :5181 (ANTHROPIC_API_KEY for drafts)
cd web && npm run dev                                             # development only: hot reload on :5173, socket proxied to :5181
```

First runs fetch about 200 MB (four elevation tiles, weather, MODIS composites, gauge
records) into `data/cache/` and record their hashes in `data/catalog.lock.toml`. Every
module reads data only through the registry; nothing is synthetic, including in tests.

## Describe and check

Every module kind describes its own surface from its parameters alone, fetching nothing:
parameters with defaults and provenance, topics published and subscribed, fields owned and
read with units, ledgers, frames, series and the data assets it needs. The kernel checks a
manifest's wiring from those descriptions before a run, and refuses one that cannot be
right (a required read with no writer, two writers of one field, a ledger in two units, a
send to nobody, an asset missing from the catalog).

```sh
esim check manifests/m10-rules-boone.toml     # the wiring report; exit 1 on errors
esim describe manifests/m10-rules-boone.toml  # descriptions and findings as JSON
esim kinds --json                             # every kind under default parameters
```

The contract between kernel and module, and the wire binding that will let a module run
as a separate process or a wasm component, is `docs/protocol-v0.md`. A test runs real
manifests and compares everything the log records against the declarations, so a
description that drifts from its code fails the build.

## Scoring and tearing

A manifest may name its scoreboards (`[audit] scores = [["sim.series", "obs.series"]]`)
and `esim run` scores them. The auditor then reads the finished run folder, never the
live run: every ledger stock's change over the run (spent down, accumulating, steady),
Shannon evenness over population stocks, shock episodes from the log (`fire.burn` and
`hazard.*` by default; `[audit] shock_topics` to change) with each stock's drop and time
to recover compared against the time to the next shock, extinctions and failed closes as
tears, and a verdict: holds, stretching or tearing. It says what it could not measure
rather than inventing a number. Output: `audit.json` beside `run.json`.

## Rules of the house for this repo

- Real terrain and real records or nothing. A formula check is labelled as one.
- A module accounts the numbers it publishes, in the precision it publishes them, and
  keeps its stocks in f64. The ledgers have caught two precision leaks already.
- Bulk per-cell numbers travel in shared fields; events signal. Cell-addressed events are
  for sparse things (an ignition).
- Nothing is calibrated by hand. Parameters are named, their provenance is stated
  ("guess" when it is one), and the hindcast harness (M12) will move them by method.
- Durable thinking goes in dated `devlog/` entries. Status lives there, not here.
