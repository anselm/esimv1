# esimv1 (folder: earth-simulator-1)

Project-specific orientation only. House policy (devlog, git, UX, writing, deploy) comes
from the harness and is not repeated here.

## What this is

A whole-system simulator in Rust: fields, networks, populations, agents and rules on one
clock, with ledgers that must close at every seam, replayable runs, ensembles, and a
sparse web viewer. Anselm's design brief is `devlog/20261031-design-brief.md`; the
milestone ladder is `devlog/20261031-milestones.md`; `README.md` says what exists and
how to run it. Development runs on canada1; a read-only copy is deployed to esimv1.exe.xyz (since 2026-11-01).

## Where things are

- Code: this folder. Repo: `anselm/esimv1` (renamed from earth-simulator-1 on 2026-11-01; GitHub
  redirects the old name). The local folder is still `earth-simulator-1` until Anselm moves it
  from the manager; the room, the memory index and the harness registry key on the folder.
- Devlog: `devlog/` (dated entries; read the tail first, especially the latest milestone
  entry and `20261031-fitness.md`). System documentation for users: `docs/system/*.md`
  (terminology, honesty, modules, layers, manifests, ...); keep `docs/system/limits.md`
  current when a model's known faults change. `docs/index.json` is written by the server.
- Deploy: a read-only copy at https://esimv1.exe.xyz/ (exe.dev VM `esimv1`, unit `esim.service`),
  by hand with `deploy/deploy.sh esimv1` on Anselm's say; push is not deploy. The VM has Node
  only: no Rust, no data cache; it serves the pages, projects, runs and docs that were rsynced.
- Runs: `runs/` (gitignored, loose runs) and `projects/<area>/<name>/runs/` (project runs).
  Viewer: `web/`. `node server.mjs` (after `npm run build`) is the whole thing on :5181:
  built pages, `/runs/`, `/projects/`, socket.io over the CLI. `npm run dev` is for hot
  reload only and proxies the socket to :5181. On canada1 the server is left running on
  :5181 and Vite on :5180 (PIDs in the session scratchpad; kill by PID, never pkill -f).
- Data: `data/catalog.toml` is the registry's index; `data/cache/` holds fetched bytes
  (gitignored, about 200 MB when warm); `data/catalog.lock.toml` holds their hashes and
  is committed; `data/events/` holds the calendar and hazard tables.

## Project-specific facts

- Toolchain: Rust stable via rustup in `~/.cargo` (installed 2026-10-31); Node 25;
  GDAL 3.8 (apt) as a cross-check tool only, never a dependency. Build with `--release`
  for any run longer than a week of simulated time.
- Every module loads its own data through `esim_registry` using `ESIM_ROOT` (defaults
  to the current directory, so run `esim` from the repo root; tests set it).
- External sources, all anonymous: Copernicus DEM on AWS, GHCN-Daily on AWS, USGS NWIS,
  Open-Meteo (ERA5 archive and GEFS ensemble; free for non-commercial use only; the
  archive lags about five days and sometimes answers with a non-JSON timeout, which the
  fetcher now refuses and retries), MODIS on Microsoft Planetary Computer (one container
  token per run; per-file signing is rate-limited; the catalog has no composites for
  July to November 2025 over tile h11v04).
- The stage is the Boone River above Webster City, Iowa: USGS 05481000, GHCN station
  USC00138806, window 42.3 to 43.2 N and 94.2 to 93.5 W, four one-degree DEM tiles.
- `esim ensemble` warms the cache once before spawning threads; branches must not
  download.
- Every module kind has a `describe(name, params)` beside its `new`, registered in
  `crates/modules/src/lib.rs::KINDS`. When a module gains or loses a topic, field, ledger
  or asset, update its describe; `crates/modules/tests/describe.rs` compares declarations
  against real run logs and fails otherwise. `esim run` refuses a manifest whose wiring
  does not check. The contract is `docs/protocol-v0.md`.
- Country layers (`crates/modules/src/countries.rs`): Natural Earth outlines rasterised at
  2 degrees, UN WPP people (multi-member gzip: use MultiGzDecoder), CRU country temperature
  through the World Bank CCKP API, `data/events/tipping.toml` (every row `recalled`, not
  yet checked against Armstrong McKay et al. 2022). With the AR6 assessed forcings
  (`forcing.ar6`) and the Millar 2017 sink feedback in `carbon.pools`, the planet is
  +1.69 degC in 2024 against +1.54 observed and 451 ppm against 425; both recalled
  parameter sets are to be checked. A project folder's manifest is frozen at creation:
  `esim project update <path> --manifest f` re-resolves it after a bundle changes.
- Cells mode (2026-11-02): `people.countries` with `distribution = "worldpop"` and `scale = "50m"`,
  `people.heat` with `baseline = "cells"`, `people.wetbulb` with `source = "cells"`, all at
  `res_deg = 0.5` with `frame_res_deg = 1.0`. Data: WorldPop 2020 1 km (870 MB GeoTIFF, LZW,
  read tile by tile; the sum per cell is cached beside it) and NASA POWER (MERRA-2) regional
  boxes, 10 degrees and one year per request, about 7,200 answers for 343 populated boxes,
  reduced to tenths-of-a-degree tables (`fetch_reduced`). `esim warm cells` fetches them ahead
  (about 100 minutes at one request a second); the cache is then about 2.5 GB. The country
  modes are the defaults and still reproduce the old runs.
- Manifests may `include = ["other.toml"]`; `Manifest::load` resolves, project folders get
  the resolved text. Projects may declare `[[variant]]` sets; runs record `variant.json`.
- Planetary rungs (`manifests/p1-*`, `crates/modules/src/planet.rs`) are zero-dimensional
  and read five small global records listed in the catalog (OWID emissions, Mauna Loa,
  Law Dome, HadCRUT5, GISTEMP); NOAA's ocean heat content server was down on 2026-11-01,
  so that scoreboard is not wired yet. Model parameters there are published values
  recalled from memory and flagged as such in each describe; nothing is tuned.
- `crates/audit` audits run folders after the fact; `esim run` calls it and writes
  `audit.json`. Populations and watched stocks come from ledger closes, so a module that
  wants the auditor to see a population must keep it as a declared stock in a ledger whose
  unit is `individuals`. Season-long runs confound recovery with seasonality; say so.
- The Anthropic API key in `~/.bashrc` had no credit on 2026-11-01 ("credit balance is too
  low"); the project server then drafts with a local model through ollama and labels it.
- A first ten-day river forecast was issued 2026-11-01 (`devlog/20261101-f1-first-forecast.md`)
  and is to be checked against USGS on or after 2026-11-11.
