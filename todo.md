# todo

Research projects and open work, one line each. The thinking behind each line is in the
dated `devlog/` entry it names; status lives there too, not here. Asked for by Anselm on
2026-11-01.

## Research projects (hindcasts against events with known endings)

- The end of the African Humid Period: an AMOC-linked monsoon step and the Sahara's
  drying, abrupt or gradual. `devlog/20261101-research-agenda.md`
- The Four Corners megadrought and the Ancestral Puebloan departure, from the tree-ring
  drought atlas. `devlog/20261101-research-agenda.md`
- The AMOC as a record (RAPID array) and as a conceptual box model with a real
  bifurcation, labelled conceptual. `devlog/20261101-research-agenda.md`
- The 2003, 2010, 2015 and 2021 heat events as precedents for country exposure.
  `devlog/20261101-the-2100-scenario.md`

## Making the planet page read as it should (Anselm, 2026-11-02; `devlog/20261102-why-it-reads-mild.md`)

- Records to date on the first screen: the hindcast's record years as a sequence with
  sources, before any assumption; the present tense is the most convincing thing we have.
- The deep-time axis: EPICA Dome C and Vostok CO2 from NOAA paleoclimatology as a context
  module, so the present is drawn as the spike it is.
- Paleo analogues: for a year's CO2, the last time the air was there and what the sea
  stood at then; numbers to be checked (Dutton et al. 2015, Pliocene literature) first.
- Extremes, not means: per country per year, how often the previous record month falls,
  and when the 2024 record becomes an ordinary year.
- A few named places in the bottom line, chosen by their crossings, each with its own
  sentence, instead of planet-sized totals alone.
- Coincidence: the count of stated limits a country has crossed this year, mapped zero to
  five; the stress board on the platform list is this.
- Cascade: consequence modules that subscribe to crossings and move a stock someone reads.
  Done 2026-11-03: permafrost and Amazon carbon into the carbon ledger (`carbon.tipped`;
  a tenth of a degree by 2100 at the low end). Next, in this order: ice-sheet
  crossings into sea level; AMOC into regional temperature (its sign differs; keep it for
  that reason); country temperature into harvest; harvest and heat into migration and
  migration into who lives under which heat. Every parameter recalled until checked, and
  the feedbacks with the other sign (CO2 fertilisation, aerosols) included so the
  selection is not the frightening ones.

## Layers to build

- Humidity, then wet-bulb exposure, replacing the guessed hottest-month marker; the Gulf
  first. `devlog/20261101-research-agenda.md`
- Aerosol forcing from a published assessment series, cited. `devlog/20261101-the-2100-scenario.md`
- A box carbon cycle with real stocks in place of the impulse response, then the hindcast
  harness against Mauna Loa. `devlog/20261101-p1-planet.md`
- Ice and sea level beyond thermal expansion; a satellite altimetry scoreboard.
- A rainfall-reading layer, so that forest patches (the Amazon) can hang off it.
- Soil life and soil carbon as a layer (Anselm, 2026-11-01, after the New Scientist piece
  on the decline of Earth's microbiome). Honest form: soil organic carbon stocks per
  country or cell from SoilGrids (one snapshot, anonymous) spent down under cultivation
  and heat by published rates, with the harvest and the carbon ledger coupled; there is
  no global time series of microbial biomass to score against, so it would be a stock
  with a stated response, labelled so, until a record exists.
- Heat mortality with published exposure-response curves and their ranges, last.
- Check every `confidence = "recalled"` row in `data/events/tipping.toml` against the paper.

## Accounts and the namespace

- Real accounts through attest.monster (passkey identity; the handle is the area; a user
  may create only under their handle). About a day on the server; do it together with
  the public deployment, after aerosols and the carbon sinks. `devlog/20261101-docs-and-accounts.md`
- Crumple the namespace: handles at the top level (`/anselm/planet`, `/anselm/notes`), not
  under `/projects/`; `docs`, `system`, `runs`, `new`, `kinds`, `api` become reserved
  top-level names. An area then holds projects and pages alike. Path-based URLs in the
  viewer first, folders second. Anselm, 2026-11-01: flattening is cognitively simpler
  even with conceptual collisions.
- The swear-word filter from his other project, on area and project names only, when
  names become public.

## Communicating a scenario (Anselm, 2026-11-03; `devlog/20261103-readme-and-the-ladder.md`)

- More embeds for a README: a counter and a chart of one series, a map at a year, the
  crossings list, a sentence from the story; each one a module's declared view, so the
  README's vocabulary is the describe's.
- A time cursor shared by every figure on a README, as the compare page has.
- The drafting facade writes a first README beside the manifest it drafts, labelled as its draft.
- Materials in the folder (images, a bibliography, source PDFs where licensed) listed on the page.
- `docs/<area>/` pages fold into the area's own folder once the namespace is crumpled.
- The China wet-bulb step (3.0 to 3.5 °C on the growing path) checked against station
  records (GSOD) for the Yangtze and North China plains, and the Indus plain's absence
  explained by the pattern or the baseline, one or the other.

## The fire (Anselm, 2026-11-03; `devlog/20261103-the-front-door-and-the-fire.md`, `devlog/20261103-our-own-engine.md`)

- Terrain-adjusted wind, one-way: the 3 km HRRR field shaped by the DEM so the Santa Ana
  accelerates down the canyons and slackens in the lee (Anselm, 2026-11-03: "is the wind
  volumetric?"; it is flat). The biggest gap for the first night now that the ground is real.
- The fire's own wind, cheapest step: a local indraft toward intense cells added to the ambient
  field, declared as a guess (Anselm: "in a real forest fire the wind is influenced by the fire
  itself"). Full coupling (WRF-Fire) is noted and out of reach for this window.
- Vegetation beyond a cover class: fuel load and species (LANDFIRE, needs an email), time since
  last burn (the 2021 scar), canopy height (Meta tile below), seasonal live moisture.
- The engine's structure physics, continued: on the corrected ground the first pass (embers
  into houses, house to house, per-cell jitter) takes destroyed-structure recall from 0.12 to
  0.19 at the record's 27-hour acreage; next, structures as individual bodies from the
  footprints, ignition by radiant exposure and ember load, the building's material where the
  inspection form states it (the data-share view has it).
- Fit the engine's constants to the perimeters and the inspections with the Eaton fire (same
  day, same wind, 14,021 acres, DINS exists) held back as the test; a parameter sweep through
  `esim ensemble --set`.
- Suppression from the record: ICS-209 resource counts and where the infrared perimeters
  stopped advancing, as agents with capacity; then the interventions Anselm asked for.
- Pre and post imagery: Sentinel-2 L2A scenes S2A_MSIL2A_20250102T183751_R027_T11SLT (before)
  and S2C_MSIL2A_20250201T183631_R027_T11SLT (after) on Planetary Computer, anonymous STAC;
  needs a 16-bit windowed COG read.
- Canopy height (Meta/WRI 1 m, tile 023012311 over the window) as a fuel layer.
- The 3D view: land cover on the ground, structures coloured by their recorded fate, a wind
  arrow, the time cursor playing. (The terrain banding was the elevation window's missing
  columns, fixed 2026-11-03; `devlog/20261103-the-holes-in-the-ground.md`.)
- Check `data/events/fuels.toml` line by line against Scott and Burgan 2005 (RMRS-GTR-153); the
  SH5 row decides the Palisades result.
- Barriers: roads and ridgelines from OpenStreetMap, the 2021 Palisades burn scar from the fire
  history, as unburnable cells; then the model's first night against the 2,920-acre figure.
- Suppression as an agent: engines, dozers, aircraft placed by rule, holding lines where the
  record says they held; the daily perimeters are the test.
- A fuel map made for fire (LANDFIRE, needs an email) beside WorldCover's "tree cover".
- Spotting, and the lattice striping (a Huygens front instead of lattice directions).
- Evacuation and the people: WorldPop cells, roads, a clearance time per neighbourhood.
- A second fire with a book behind it: Fort McMurray, May 2016 (John Vaillant's *Fire Weather*,
  which Anselm pointed at on 2026-11-03; author recalled, title from the link). Canadian doors to
  check: CWFIS and the National Burned Area Composite for perimeters, ECCC stations for wind.
- The speed test, answered: one fire day at 30 m runs in about three seconds; operator-in-the-loop
  is a surface question (live wind, a clock, decisions injected).

## Before release (Anselm, 2026-11-03; `devlog/20261103-before-release.md`)

- The front page as a gallery: each project leads with one picture made from its own runs
  (the ladder, a map at a date) and one sentence, not a text list.
- Maps that look like places: coastlines, a graticule, a quantile stretch, a basemap under
  the basin fields; the README page given more air than the run pages.
- A README figure that plays its cursor when scrolled into view.
- The fire scenario: built 2026-11-03 (see The fire above); what remains is making it right.
- Whether the held-low variant belongs on the public compare page: Anselm's call.
- Accounts and the crumpled namespace: see below.

## Neighbours (Anselm, 2026-11-03; `devlog/20261103-ageospatial.md`)

- ageospatial.com: "one record per property, each fact with its source and date", four
  questions (what is here, what defines it, what is exposed, what has changed). Build the
  structure record: one card per building in a fire window with sourced facts and the hour
  the model's fire arrived per variant; the facade answers "what should I know about this
  building?" from it.
- Exposure as a run output per building: the arrival hour's distribution over an ensemble.
- Before and after imagery beside each structure's inspection outcome.

## Platform

- A model-ensemble warming pattern in place of one fitted slope per country: WorldClim's
  CMIP6 downscaled annual means (2081 to 2100, anonymous GeoTIFFs) minus its 1970 to 2000
  baseline, per model, divided by that model's global warming (which must come from
  somewhere other than the land-only tiles). The reanalysis's own trend was tried and fails
  over South Asia; small islands and Gulf states still cross the niche on the fitted slope,
  not the heat. `devlog/20261103-ladder-pattern-cascade.md`
- Lenton et al. 2023's per-country table (Supplementary Data, not fetched) against ours, once
  the pattern is a model's; the India gap is the pattern.
- A log or quantile colour stretch for sparse people maps; under the run-wide stretch one
  Dhaka cell sets the top and the rest reads faint.
- A per-source lock shard: the POWER entries are 7,200 of the lock file's 3 MB.
- A country stress board at the cursor (count of crossed markers per country, ranked), after
  the hormuz room's board; the breaking-points row shape for every new threshold layer.
  `devlog/20261101-people-layers.md`
- An overview world map per project; coastlines over world fields; a pin for "your place".
- A second conversational turn on a drafted manifest.
- Auditing ensembles.
- The wire binding and a stdio module host. `docs/protocol-v0.md`
- `scenarios/` as a curated ladder. `devlog/20261101-roots.md`
