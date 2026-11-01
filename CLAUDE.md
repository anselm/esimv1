# earth-simulator-1

Project-specific orientation only. House policy (devlog, git, UX, writing, deploy) comes
from the harness and is not repeated here.

## What this is

A whole-system simulator in Rust: fields, networks, populations, agents and rules on one
clock, with ledgers that must close at every seam, replayable runs, ensembles, and a
sparse web viewer. Anselm's design brief is `devlog/20261031-design-brief.md`; the
milestone ladder is `devlog/20261031-milestones.md`; `README.md` says what exists and
how to run it. Not deployed anywhere; it runs on canada1.

## Where things are

- Code: this folder. Repo: `anselm/earth-simulator-1` (private, nothing pushed by agents).
- Devlog: `devlog/` (dated entries; read the tail first, especially the latest milestone
  entry and `20261031-fitness.md`).
- Deploy: none. See `project.toml`.
- Runs: `runs/` (gitignored). Viewer: `web/` (`npm run dev` serves `../runs`).
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
- A first ten-day river forecast was issued 2026-11-01 (`devlog/20261101-f1-first-forecast.md`)
  and is to be checked against USGS on or after 2026-11-11.
