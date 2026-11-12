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
  boxes, 10 degrees and one year per request, about 7,550 answers for 343 populated boxes (daily boxes, climatology and the 1981 to 2025 monthly means),
  reduced to tenths-of-a-degree tables (`fetch_reduced`). `esim warm cells` fetches them ahead
  (105 minutes on 2026-11-03, about one request a second); the cache is then 3.3 GB. POWER's
  reduced tables keep keys in string order, so a climatology's columns run ANN, APR, AUG, ...;
  read them by name (`cells::clim_index`), never by position (the annual mean was read as
  September until 2026-11-03). The per-country warming slope fitted to CRU runs high for small
  islands and Gulf states (Puerto Rico 1.78, Qatar 1.75 °C per °C global); a gridded pattern
  was tried as a replacement by a per-cell pattern from MERRA-2's own 1981 to 2025 trend
  (`pattern = "cells"`, `cells::warming_pattern`, 5-degree blocks): it fails over South Asia,
  where MERRA-2 cools while the globe warms, so `climate.countries` draws the map for the record
  and the cell layers keep `pattern = "country"`; a CMIP6 pattern (WorldClim's downscaled
  tiles) is the noted next route. The country modes are the defaults and still reproduce the old runs.
- The regression ladder (2026-11-03): `esim regress` runs `regress/ladder.toml` (planet to 2025
  with and without people, the Boone year) against `regress/baseline.json` within stated
  tolerances and fails on drift; `--accept` records a new baseline, which is a decision to
  explain in the devlog. Run it before committing anything that touches a planet or basin module.
- The first cascade (2026-11-03): `carbon.tipped` releases the carbon a `[element.carbon]` table
  in `data/events/tipping.toml` states (permafrost abrupt thaw per degree, Amazon 30 GtC,
  permafrost collapse 125 GtC; Armstrong McKay et al. 2022, checked) into the carbon ledger
  once the element is past its central estimate (`trigger = "low"` for the range's bottom, the
  project's `held-low` variant). It comes to about +5 ppm and +0.02 °C by 2099 in the held
  variant, +0.1 °C at the low end. Every tipping row is now `checked` against the paper's Table 1.
- Manifests may `include = ["other.toml"]`; `Manifest::load` resolves, project folders get
  the resolved text. Projects may declare `[[variant]]` sets; runs record `variant.json`.
- A project's `README.md` (2026-11-03) is rendered on its page by `web/src/lib/Readme.svelte`:
  Markdown, with fenced `esim` blocks as live figures (`ladder`; `table <year> series=label …`),
  drawn from the runs through `web/src/lib/runs.js`. `Describe.tables` carries small tables a
  module read (the tipping rows; the countries in outline-file order, which is what the
  `country_index` frame indexes; the compare page's country picker read `docs/countries.json`,
  the 110 m order, against 50 m runs until 2026-11-03). `docs/system/projects-and-runs.md`
  has the embed vocabulary; `projects/anselm/planet-2100/README.md` is the first README and
  the lay reader's account of the 2100 runs.
- Accounts (2026-11-03): `ESIM_PUBLIC=1` makes the server ask attest (attest.monster; code in
  `~/projects/2027/attest`) who a socket is: `web/lib/attest.mjs` verifies the handshake
  (adapted from matchmaker), the handle is the only area a person may write, claimed into
  `projects/<handle>/area.toml` on first use; the word filter lives on attest (its
  `packages/handle-guard`), not here. `web/scripts/attest-e2e.mjs` runs the whole flow against a
  local attest with a scratch database (18 checks). `web/lib/attest-verify.mjs` is a copy of
  attest's verifier until that package is on npm. The live esimv1 copy is read-only and unaffected.
- The Palisades fire (2026-11-03): `fire.rothermel` and `obs.perimeters` in
  `crates/modules/src/wildfire.rs`, data through `crates/registry/src/wildfire.rs` (WorldCover
  2021 by 3-degree tile, read windowed by `cog::read_window_u8`; ERA5 hourly wind with direction
  and gusts at one point; NIFC's daily perimeters by IRWIN id and the WFIGS perimeter of record,
  all anonymous). Fuel models in `data/events/fuels.toml`, every row recalled from Scott and
  Burgan 2005 and unchecked. Project `anselm/palisades-2025` (manifest `f2-palisades-2025`,
  variants reanalysis, gusts, built-barrier, sh7) runs in about 10 s and, on the corrected ground,
  is about eight times too slow at 27 hours on the reanalysis wind (2,031 acres against 15,824)
  and ends the fourth day near the record's size (20,892 against 23,343, overlap 0.58); the gusts
  variant overruns by the end. The first mapped perimeter is a hand-drawn triangle. Fire reaches
  three cells along 32 directions; progress is per approach direction (summing over neighbours
  ran the front several times too fast).
- The holes in the ground (2026-11-03 afternoon, `devlog/20261103-the-holes-in-the-ground.md`):
  until then `dem_window` put Copernicus pixel centres exactly on output cell edges, one column
  in three or four came out NaN at one arc-second, Horn's slope read each as a cliff, and both fire
  engines ran on east-west cliffs (Rothermel burned 79,000 acres at 27 hours). The grid is now
  laid half a source pixel off the lattice (`terrain-v3`); the Boone regress baseline moved
  slightly and was accepted. Any run made before that afternoon on a one-arcsecond window is
  suspect. Neither fire module adjusts the wind for terrain or feeds the fire back into it.
- Our own fire engine (2026-11-03, `crates/modules/src/firecells.rs`, kind `fire.cells`): a heat
  budget on 1-arcsec cells with buildings (LA County LARIAC footprints with heights, paged
  anonymously), roads as breaks (OSM context), HRRR 3 km wind each hour (`crates/registry/src/grib.rs`
  reads templates 3.30 and 5.0/5.3 by byte range from the open bucket; `Fetcher::fetch_range_reduced`;
  cross-checked with GDAL), spotting, a declared `ignite_scale` knob. Project `anselm/palisades-cells`
  (manifest `f3-palisades-cells`; variants scale-2..6 and near-2..4 with reach 45 m, houses,
  houses-jitter). On the corrected ground near-4 has 15,306 acres at 27 hours against 15,824 mapped
  and twice the record by day four; destroyed-structure recall 0.12 to 0.19. `obs.dins`
  (CAL FIRE's 12,081 inspections) scores structures by cell. `structures = "on"` (evening of
  2026-11-03, `devlog/20261103-bodies.md`) makes every footprint a body scored per footprint
  (`fit.buildings_*`), with frames `building_fate` and `building_arrival_h` that the 3D view
  colours; precision sits at the base rate (0.48) whatever the threshold, because nothing we hold
  tells a survivor from its neighbour; the county footprints carry `AIN`, the assessor's parcel
  number, and the assessor's roll is the way to year built and size. The public DINS views carry
  only DAMAGE and STRUCTURETYPE; the full form is in a non-public data-share view. Slope
  (`devlog/20261103-the-slope.md`): until the evening the engine put slope only in the heat's reach,
  linearly and capped, so it spread in rings while Rothermel's quadratic slope factor climbed the
  canyons; `slope_rate_gain` (default 0, unchanged runs) and `downslope_gain` put Rothermel's shape
  on the rate; variants near-4-slope (gain 20, stalls on ridges), near-4-slope-6 (best overlap yet,
  0.70, half the first day's area), near-4-slope-soft (20 up, 2 down, overruns). Next for the engine:
  heat as I·f(d)·(1+φ_w+φ_s) with wind and slope as vectors, then terrain-following wind. A `scene` view renders in 3D (`Scene3D.svelte`; its building boxes drew black until the late evening of 2026-11-03 because the box material had `vertexColors` set with no colour attribute, so instance colours never showed;
  three.js 0.170, needs `terrain_elev_m` and `building_height` frames); headless screenshots need the
  swiftshader flags, which the scratchpad shot script now sets. Never `pgrep -f`/`kill` a pattern
  that appears in the running command line (killed my own shell this way).
- The record on the map (2026-11-03 late, `devlog/20261103-the-record-on-the-map.md`): `obs.record`
  (`crates/modules/src/record.rs`) reads GeoJSON from `data/events/<event>/` and publishes
  `record.event` at each feature's time and cell; both Palisades manifests carry it (NIFC points and
  lines, `CreateDate`, which lags the work; the dataset's own time fields are empty). `Scene3D`
  reads the run's `log.jsonl` for `record.event`, `fire.ignition`, `fire.spot` and draws pulsing
  rings, draped lines and time labels (UTC) at the cursor. The model does not react to the record.
  A project's manifest is frozen: after a manifest change run `esim project update <area/name>
  --manifest manifests/x.toml` (area/name, not the folder path) and rerun the variants.
- The vector form (2026-11-04 morning, `devlog/20261104-the-vector-form.md`): `fire.cells` has
  `form = "vector"` (heat as I·f(d)·(1+|φ|)·ellipse, wind and slope as vectors with Rothermel's shape,
  Anderson's ellipse from `wildfire::length_to_breadth`; `wind_phi`, `wind_phi_exp`, `phi_cap`, `waf`)
  and `cooling = "idle"` (heat decays only while no dose arrives; always-on decay has a floor that is
  every cliff the engine met). Variants vector-5 (first day right: 16,525 acres at 27 h against 15,824,
  overlap 0.79), vector-8, vector-5-held. The second day overruns (54,000 acres at 49 h against 19,629)
  and no constant reaches it; candidates are the day-2 wind over the mountains and unplaced suppression.
  Interventions are a module (`fire.suppression`, `crates/modules/src/suppression.rs`, afternoon of
  2026-11-04, Anselm: a modular approach lets one engine run other fires): in `mode = "hold"` it turns the
  record's completed lines into `ground.break { cells, strength, from }` events; `fire.cells` and
  `fire.rothermel` subscribe to `ground.break` and never read the record. Both Palisades manifests carry it
  with `mode = "off"`; the `vector-5-held` variant sets `suppression.mode = "hold"` (does nothing to the
  numbers: the record's first line is after the overrun). `obs.record` has `time_shift_h`. `esim project prune
  <area/name> --keep 0` deletes every run of the project (did it by accident; all runs are deterministic
  and were rerun).
- Path URLs (2026-11-04, Anselm's ask): the viewer routes by path, `/<area>/<name>`, `/<area>/<name>/<run id>`,
  `/<area>/<name>/compare`, `/docs/<area>/<name>`, `/runs`, `/run/<id>`, `/new`, `/kinds`, `/all`
  (`web/src/lib/router.svelte.js`; the old `?project=` forms are still read). The server serves
  `/projects/`, `/runs/` and `/docs/` as files and falls back to the page for an extensionless path
  under them. `RESERVED_AREAS` in `server.mjs` is the list no handle may take. Markdown links in docs
  and READMEs use `/docs/system/limits`, not `?doc=`. The project page shows variants and runs before
  the README (Anselm, 2026-11-04). The 3D scene stretches the vertical twice, lights by the sun's
  position at the cursor (low-precision solar formula, recalled), colours the sea, drapes the mapped
  perimeter as dots and draws the wind as an arrow at the north-west corner.
- Fort McMurray 2016 (2026-11-04, Anselm: the second fire, to test how portable the engine is;
  `manifests/f4-fortmcmurray-cells.toml`, project `anselm/fortmcmurray-2016`,
  `data/events/fortmcmurray-2016/`): the same engine on a boreal fire outside the United States.
  What had to generalise: buildings from OpenStreetMap through Overpass (`buildings = "osm"`,
  `registry::wildfire::osm_buildings`, heights to feet so the pipeline is unchanged), wind from
  ERA5 at one point because HRRR stops at the border (`wind = "era5"`, which existed), and the
  record as satellite hotspots (`obs.hotspots`, CWFIS Fire M3 archive copied for the window) because
  nobody mapped the fire hour by hour. Copernicus pixels are 1.5 arcsec wide at 56°N: `dem_window`'s scatter left one cell in
  three NaN (the holes again); `terrain-v4` fills such cells from the nearest pixel, regress shows no
  drift. Overpass's admin-boundary relation is skipped for windows over 0.1 square degrees. Result:
  thresholds ×8 match the hotspot-seen pace from day 3, overlap a third to two fifths because the
  ERA5 point wind reads the 3 May run into the city as a light south-easterly. The detection hour (about 16:00 MDT on 1 May) is recalled and unchecked; the provincial
  review on open.alberta.ca answered 520 that day.
- Breadcrumbs and areas (2026-11-04): `/anselm` is an area page (`ProjectList` with `area`), every path
  fragment in the headers is a link (`web/src/lib/Crumbs.svelte`), the byline links to the area. In the
  3D scene a line fades out twelve hours after its time and the whole record stays at the run's end.
- The deep history and the actors figure (2026-11-04 evening, `devlog/20261104-deep-history-and-the-actors.md`):
  the first Palisades README carries *The ground before the fire* (Radtke 1982, Keeley 2005, Mensing 1999,
  the Park Service, Lake Elsinore; the big fires come at the same rate under every regime, people changed
  the ignitions, the fuel's age and the stakes; a time-since-last-fire layer is the ask) and *The actors*, a
  README figure (` ```esim actors `, `web/src/lib/Actors.svelte`): `name: caption` and `a -> b: words`
  lines, words ending in a parenthesis beginning "not" drawn dashed as what the world has and the model lacks.
- The world scored (2026-11-05, Anselm: a total bottom line per region, a glyph per region on a globe, run forward
  so knock-backs show): `score.regions` (`crates/modules/src/score.rs`) ranks nine World Bank indicators per country
  (`crates/registry/src/worldbank.rs`, catalog `worldbank-wdi-api`, anonymous; governance only as `GOV_WGI_VA.EST`
  with `source=3`) to 0..1 bands, total = mean of bands present, publishes `region.scored` tables (centroids from
  the Natural Earth rings), moves bands by `people.crossing` and `shock.crossing` under declared placeholder rules.
  `web/src/lib/Globe.svelte` (view kind `globe`, hooked in `ModuleViews`) draws the table at the cursor as stacked
  instanced glyphs on a sphere with the outlines; three.js OrbitControls; hover for the bands. In the 2100 manifest.
- Derailment and cascades (2026-11-07, Anselm: "we are still failing to really consider knock on effects", with
  Dyke's Conversation piece of 2026-11-05 on derailment risk; `devlog/20261107-derailment.md`): `policy.capacity`
  (`crates/modules/src/capacity.rs`) computes a world capacity to cut emissions from the run's own pressures
  (`region.pressure`, published yearly by `score.regions`: the share of people under heat stress and the
  people-weighted voice, fairness and means bands; wild-card knocks by kind; conflicts drawn where the stress is,
  which take a fraction of one country through `people.shock { iso3 }`) with a fast fall and a slow recovery, and
  in `mode = "on"` publishes `emissions.capacity`, which `carbon.emissions` applies to each year's planned fall
  (`rate()` walks the plan from the record's last year; with no capacity ever published it returns the plan bit
  for bit, so every older run is unchanged). It also reports the options: budget and cut rate now needed to hold
  `level_c` and `level2_c` under a recalled TCRE. The manifest carries it in `mode = "watch"` (reports only).
  `tipping.elements` has `coupling` (0 by default) and reads `[[coupling]]` rows from `data/events/tipping.toml`
  (Wunderling et al. 2021, read on the publisher's page that day; the fractions are the paper's ensemble net at
  full strength, the attribution per link is ours). Project variants: zero-by-2050-derailed, -derailed-wild,
  falling-derailed, held-cascade, zero-by-2050-everything. After a manifest change: `esim project update
  anselm/planet-2100 --manifest manifests/p4-planet-people.toml`, then rerun every variant (5 s each).
- RFC 0001 (2026-11-07, Anselm: a public schema for a manifest as an RFC, a theory of an interoperability
  standard between implementations of this engine, in JSON, written to persuade): `docs/rfc/0001-manifest.md`
  with `docs/rfc/manifest.schema.json`; docs area `rfc`, section "Standards". Draft 4 names the source form.
  `esim events --write` renders `data/events/future-known.toml` and `future-speculative.toml` as `docs/events/known.md` and
  `wild-cards.md` (section "The future"); the deploy runs it, so those pages are regenerated and must be committed like the
  indexes. The overview page and the root README link the RFC as a proposal and the two event pages (Anselm, 2026-11-07).
- Reading lists (2026-11-08, Anselm: "we probably should track these articles in the readme"): a project README ends with
  *What this page has read*, each article with the day it was read, what was taken and what is only recalled; the practice is
  in `docs/system/telling.md`. Earth 2100's lists Dyke, Laybourn and Abrams 2026 (ERL 21 204029, the derailment paper, read in
  full 2026-11-08; `policy.capacity` lacks its stabilising loop, a fiscal state and the social lowering of thresholds, noted as
  the next knobs), the Conversation piece, Aglas-Leitner 2026, Wunderling 2021, Armstrong McKay 2022, and the recalled sets.
- Three notations and the bus (2026-11-07 evening, Anselm: "support a .js notation (we can presume quickjs) ... also support
  json ... continue to support existing notation"; "refactor the core engine to be an event bus and to handle dynamic
  registration of listeners"; `devlog/20261107-three-notations.md`): `Manifest::load` reads `.toml` (as before), `.json`
  (the entity form of RFC 0001, `crates/kernel/src/entities.rs`: presence is type, replacement by id, ranges unroll to
  variants, unknown components carried and reported by the check) and `.js` (`crates/kernel/src/js.rs`, rquickjs 0.14:
  exports are entities, export name is id, arrays spliced, `run`/`place`/`audit` short form, no Date, Math.random throws,
  imports only of neighbours by relative path, `.toml` imports give an entity list, 10 s and 256 MB budget). All three
  resolve to one `Manifest`; `esim manifest <file> [--toml|--js|--id]`; the id is the sha256 of the canonical entity text
  (JCS style, `entities::es_number`) and is the same for the TOML planet and its `.js` twin. Run and project folders keep
  `manifest.json` (with `from`) beside `manifest.toml`; a project from a program keeps `source/`. The kernel plays the
  manifest onto the bus: `manifest.entity` events at the start (no sequence numbers, so no digest moved), births mid-run
  from any module's publish, `manifest.refused` for a refused one, snapshots carry `born`. `events.shock` modes are
  off, draw, named (the check does not catch a wrong mode; the build does). Project `anselm/planet-wild-members` is the
  worked example from `manifests/j3-planet-wild-members.js` (eight seeds of the deck, emissions held).
- Heatwaves (2026-11-07 night, Anselm: "we don't have heatwaves? ... there should be some kind of periodic heatwave ...
  lets do some kind of weather generator - can be imperfect"; `devlog/20261107-heatwaves-no-match.md` has the paper and
  the build): `weather.heatwaves` (`crates/modules/src/heatwaves.rs`, in the 2100 manifest after `wetbulb`) holds a decade
  of daily T2M_MAX per populated one-degree cell (the wet-bulb layer's POWER boxes, `work_res_deg = 1.0`, tables cached as
  `data/cache/heatwaves_*.bin`, about 125 MB), draws one observed year per simulated year by the kernel RNG, shifts it by
  the cell's warming since the decade (`slope` from `climate.country_tas` times the global anomaly less the decade's
  HadCRUT5 mean; `pattern = "cells"` uses the cell slope) and counts heatwaves as Aglas-Leitner et al. 2026 do (Tx90p of
  the decade itself, 15-day window, runs of 3 or more; severity, cumulative heat, duration, people share at the peak).
  Publishes `heat.wave` (the longest notable one per country per year), `heat.crossing` (first month-long and season-long
  over half a country's people; the story hears it), `heat.pressure` (per country, for a capacity hook; nothing reads it
  yet). `spread_per_degree` (0, a guess) is the only knob for the variability the delta method lacks. Adds about 7 s to a
  2100 run. The country helpers in `countries.rs` are `pub(crate)` for it.
- Devil's Gulch 2029 (2026-11-08, Anselm: "a brand new simulation of 550 Nicasio Valley Rd ... a fire starting not far
  away at Devil's Gulch Campsite ... oct 1st ... in 2029 ... we probably need some wind conditions"; use the 3D view with the
  vertical at least 2x; `devlog/20261108-nicasio-2029.md`): the first scenario with no record, `manifests/f5-nicasio-2029.toml`,
  project `anselm/nicasio-2029` (variants scale-3, scale-5, scale-8, gust), sources in `data/events/nicasio-2029/README.md`.
  `fire.cells` has `wind_at`, the simulated instant `wind_start` stands for, so a run can borrow another day's HRRR hours
  (here 27 to 29 October 2019, the Bay Area office's Red Flag Warning 6 of 2019 in the IEM VTEC archive; empty leaves old runs
  unchanged). `obs.places` (`crates/modules/src/places.rs`) stakes named points with a provenance note, publishes
  `place.marked` at the start, reads `fire.burned_frac` hourly, reports the distance to the nearest burned cell per place and
  publishes `place.reached` and a `place.crossing` sentence once (own cell burned or within `reach_m`, 50 m, because a
  geocoded address sits on the road centreline). `Scene3D` draws both topics as marks with labels that do not fade. The
  campsite is OpenStreetMap relation 17685328; the house is the US Census geocoder's interpolation (on the road); the
  description of the place is Anselm's. At thresholds times 3 or 5 the fire takes the whole window in two days and reaches
  the house at 19 to 34 hours; at 8 it never leaves its cell: the engine does not know, and the page says so.
  The same scenario on `fire.rothermel` (which gained the same `wind_at`, ERA5 point wind) is `manifests/f6-nicasio-2029-rothermel.toml`,
  project `anselm/nicasio-2029-rothermel` with the Palisades' four variants: 16,000 to 23,000 acres, stops 1.3 to 2.2 km short of
  the house (Anselm: "add the rosenthal model also - a separate sim", read as Rothermel).
- Half Moon Bay (2026-11-08 evening, Anselm: "a separate sim for the half moon bay dredging issue ... the army core of
  engineers built sea walls back in the day and they are eroding the coastline and filling the harbor with silt. We may need
  a new fluid dynamics model"; `devlog/20261108-halfmoonbay-shoreline.md`): `coast.oneline` (`crates/modules/src/coast.rs`,
  readers `crates/registry/src/coast.rs`), a one-line shoreline model on today's OpenStreetMap coast from the East
  Breakwater's root, ERA5-Ocean hourly waves through Open-Meteo's marine API (catalog `open-meteo-era5-ocean-waves`, the
  half-degree cell 37.5/-122.5), the headland and breakwater as diffracting shadows, CERC transport at the faces between
  transects (transport at the transects checkerboards), a bluff where the beach runs out, a fitted leak through the
  breakwater into the harbor (`leak_frac` 0.2), a fitted direction offset (`dir_bias = "fit"`, -22.6 deg, zero net drift
  beyond 3 km) and K fitted at 0.005 (80 times below SPM; the sweep is in the devlog); a `sand` ledger that must close.
  Project `anselm/halfmoonbay-shoreline` (manifest `c1-halfmoonbay-oneline`; variants as-built, before-1959, sealed,
  to-2075-held, to-2075-slr-high, to-2075-dredged), record `data/events/halfmoonbay/record.toml` (USACE 2015 draft EA
  citing Lin et al. 2015; Griggs 2026 abstract; the Harbor District). `esim run --set` is not a thing: overrides go through
  `esim project run --set`. The viewer has a diverging ramp for fields named change/anomaly/delta.
- Portland and Cascadia (2026-11-12, Anselm: "spin up a new scenario for this" with the r/Portland thread on the
  Oregonian's story that a Big One nightmare scenario drives Portland to cut fuel storage along the Willamette;
  "What's the best way to model this? What do we want to show to people?"; `devlog/20261112-portland-cascadia.md`):
  three kinds. `quake.ground` (`crates/modules/src/quake.rs`) holds DOGAMI O-18-02's synthetic site PGA, PGV,
  liquefaction and landslide layers cut to the window (`data/events/portland-cascadia/`, 16 MB, made with GDAL as the
  folder's README says; the registry reads the cut GeoTIFFs) as world fields and publishes `quake.rupture` at the
  instant; `quake.buildings` turns them into damage states, deaths, injuries and displaced on DOGAMI's own inventory
  (footprints counted per cell, each unit's people spread by floor area) with recalled Hazus forms and six numbers
  fitted to DOGAMI's Hazus run (`loss_unit_*.csv`, the record), reported as `fit.*` (Portland's extensive, complete
  and displaced within a few percent, deaths 0.69 by day and 1.7 by night, pattern r 0.9); `lifelines.recovery`
  (`lifelines.rs`) is the year after: twelve Willamette crossings closed for days drawn log-uniformly in the county's
  "weeks to years", thirteen hospitals on 72 h of generator fuel, power and water as ramps to days drawn from the
  Oregon Resilience Plan's valley ranges, and a `fuel` ledger in gallons: the hub's 233.5 million gallons (the 2022 study's two-thirds fill; 298.7 million was a misread of its flammable capacity until the evening of 2026-11-12), the 2022
  county study's release range drawn by the seed (43 percent to the river), the response's draw, resupply from day
  14, closing to zero. Project `anselm/portland-cascadia` (manifest `q1-portland-cascadia`; variants wet-day,
  wet-night, dry-day, cut-20, retrofit, burnside-ready, seed-2, seed-3; six seconds a run). The story view on the
  lifelines module carries every `*.crossing` sentence. Not yet: a 3D scene (no heights in the inventory), the
  USGS grid as an alternative ground, a hospital-level inventory, the water network.
- Wild cards (2026-11-05, Anselm: "i just want a placeholder ... like a video game with wild card factors
  optionally"): `events.shock` (`crates/modules/src/shock.rs`) draws `data/events/future-speculative.toml` by the
  run's seed each year from `from` (default 2025), or plays one named card (`mode = "named"`, `event`, `at`);
  `scale` multiplies the chances; cards may repeat. Effects go out as `forcing.extra` (name shock), `emissions.scale`
  (new port on `carbon.emissions`, product of factors by name) and `people.shock` (new port on `people.countries`:
  a fraction of every country, cumulative, reported as `people.shock_deaths`); each card is a `shock.crossing`
  with a sentence, so the story and the compare page show it. `manifests/p4-planet-people.toml` carries it with
  `mode = "off"`; project variants wild-1/2/3 (seeds), wild-4x (`scale = 4`), eruption-2040 (named VEI 7).
  With emissions held the deck moves 2100 by tenths of a degree and 200 to 560 million people; regress unchanged.
- The log ramp and the future-event lists (2026-11-05 morning, Anselm): `FieldMap` has a `log` colour stretch, the
  default when a field's positive values span three decades (people per cell, days a year), because the linear ramp
  to the run's maximum left the 2100 maps blank while the cover, drawn by hand on a log scale, showed the Sahel and
  India. `data/events/future-known.toml` (24 dated events) and `future-speculative.toml` (17 with rough annual
  chances and model-term effects) are reusable lists written from memory, every row `recalled` and unchecked, meant
  for digitaltwins.wiki too; no module reads them yet (`events.timeline`, `events.shock` and `people.impact` are
  designed in `devlog/20261105-impact-and-events.md`).
- Docs sections and the telling page (2026-11-04 evening, Anselm): `<!-- section: name -->` on a docs page's second
  line groups it on `/docs` (Start here, The machine, Projects and the record, Telling it, Changing it; the order is
  `SECTIONS` in `DocsList.svelte`, the server writes `section` into `docs/index.json`). `docs/system/telling.md` is the
  house practice for a project page: priming, event, aftermath; the actors figure on named frameworks with the gaps
  dashed; the story carries the reader. The first Palisades README has all three parts, *After the fire* as of 2026-11-04.
- Front page and covers (2026-11-03): `featured = true` in project.toml puts a project on the
  front page as a card; `cover` and `cover_caption` name a picture beside the manifest.
  Earth 2100 (renamed from "The planet, with people on it") and the Palisades fire are featured.
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
