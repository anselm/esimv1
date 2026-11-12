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

## Anselm's asks on the fire, 2026-11-03, as a list (his words in quotes; status after the dash)

1. "Volumetric air ... convection, terrain adjusted wind, indrafts, full coupling ... I don't care how
   long it takes to compute ... if we need fast assessments we can have modes." — not started. Order
   decided in `devlog/20261103-beyond-rothermel.md`: terrain-following wind first (one-way), a local
   indraft second, a dense 3D solver last; modes by manifest.
2. "The fire spread in an almost linear fashion ... like a drop of water ... not believable." — the
   phantom cliffs are fixed. Evening: the rings were mostly the formula, slope only in the heat's
   reach and capped (`devlog/20261103-the-slope.md`); with Rothermel's shape on the rate at gain 6
   the fire climbs the canyons in fingers and the perimeter overlap is the best so far (0.70). Morning
   of the 4th: the vector form (`devlog/20261104-the-vector-form.md`) has the first day right in pace
   and shape (0.79 at the infrared perimeter) and the second day everywhere at once; no constant
   reaches the second day, so the next step is the terrain-following wind of 1 and the suppression of 4.
3. "Much finer granularity on different kinds of ignitable/flammable fuel sources ... the cellular
   resolution is far too low ... individual trees." — not started. Plan: 10 m cells with load, canopy
   height, time since burn and seasonal live moisture; trees as individuals only when canopy data
   earns it.
4. "Interventions - there may be calfire logs of actual intervention points." — the record is in
   hand (`data/events/palisades-2025/`: CAL FIRE's resources by report, NIFC's 2,334 lines and 825
   points, UL's radio log). Late evening: `obs.record` publishes NIFC's points and lines into every
   Palisades run at their times, and the 3D view pulses them with their times beside the model's
   ignition and spots (`devlog/20261103-the-record-on-the-map.md`). Afternoon of the 4th: `fire.suppression`, a
   module of its own between the record and the engines, turns the completed lines into `ground.break`
   events both engines honour, and `obs.record` can shift the record's times; neither touches the second
   day's overrun, which precedes the record's first line. Still to do: the
   unplaced suppression (aircraft and crews of the 8th and 9th from CAL FIRE's resource counts, as
   agents with capacity), geocoding UL's addressed events.
5. "More clearly indicate when actual buildings themselves are likely burned - right now they stay
   one color." — done in the first form (2026-11-03 evening, `devlog/20261103-bodies.md`): every
   footprint a body with its own ignition and burn, coloured in the 3D view by the model's state
   against the inspectors' fate, disagreements in orange and blue; scored per footprint. Finding:
   precision stays at the base rate, the model cannot tell a survivor from its neighbour. Next:
   join the county's `AIN` to the assessor's roll for year built and size as vulnerability; CAL
   FIRE's Camp 2018 DINS public view carries pictures, for later.
6. "Our model should diverge from the rothermal model." — agreed; Rothermel stays as the baseline
   module, the work goes into `fire.cells`.
7. "Real predictive assessments of the shelter individual plots of land may have ... for insurance
   purposes, for home buyers." — the per-structure score from 5, then its distribution over an
   ensemble of winds; not started beyond 5.
8. "We can tune over any fire we wish ... the fire that inspired the book Fire Weather." — Fort
   McMurray 2016 started 2026-11-04 (`anselm/fortmcmurray-2016`, `devlog/20261104-fort-mcmurray.md`):
   the engine runs there with OpenStreetMap buildings, ERA5 wind and CWFIS hotspots as the record.
   Still to do: the provincial review's chronology (open.alberta.ca was down), the NBAC final
   perimeter, a boreal fuel kind (spruce crowns; the engine has one "tree"), the hotspot weather
   columns as a wind check. Eaton is the held-out test; nothing fetched for it yet.
9. "Copy anything useful we find into this project for our own reference ... the documentation
   should reflect this as well - our sources - our more aggressive model." — done for today's finds
   (`data/events/palisades-2025/README.md`, `docs/system/data.md`, `docs/system/modules.md`,
   `docs/system/sources.md`); every new source goes the same way.
10. Earlier the same day, still open: the Rothermel fuel table checked against Scott and Burgan; the
    first day scored against something better than a hand-drawn triangle (the UL hourly maps, CAL
    FIRE's reported acreage); the held-low variant on the Earth 2100 compare page.

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
- Suppression from the record: the NIFC Operational Data Archive 2025 holds, for Palisades, 2,334
  event lines (completed dozer and hand lines, roads used as line, dated from 2025-01-09 19:40Z)
  and 825 event points (dozer pushes, drop points, division breaks); checked 2026-11-03. Lines as
  timed barriers of declared strength first, then ICS-209 resource counts as agents with capacity.
- The engine beyond Rothermel, Anselm's four asks and the order to build them (structures as
  bodies; terrain-following wind; fuel as fuel at 10 m; interventions; then the 3D air):
  `devlog/20261103-beyond-rothermel.md`.
- Pre and post imagery: NOAA NGS emergency response tiles (`noaa-ngs-eri-2025`: Maxar 9 to 20
  January, NOAA aerial 28 January, zoom to 21) over the window, before Sentinel-2 (scenes
  S2A_MSIL2A_20250102T183751_R027_T11SLT and S2C_MSIL2A_20250201T183631_R027_T11SLT noted).
- The record copied into `data/events/palisades-2025/` (2026-11-03): CAL FIRE's 344 status reports
  (acres, containment, engines, dozers, helicopters, crews, personnel, structures by time), the UL
  FSRI timestamped dataset (4,785 Palisades rows), the UL timeline report's text with hourly
  fire-activity maps for the first 26 hours, and NIFC's 2,334 event lines and 825 points. From
  them: an `obs.reports` scoreboard of reported acreage by time (CAL FIRE: 10 acres at 11:06, 200
  at 11:31, 1,262 at 15:34, 2,921 at 00:29 on the 8th, 11,802 at 11:45, 15,832 at 13:20 local; the
  press had 2,921 from 18:50 on the 7th, so the triangle's 06:00Z stamp lies between the two); the
  suppression record (resources by day; lines, undated in the archive); the UL hourly maps as the
  first-day perimeters, to be georeferenced from the figures; the Eaton sheet for the held-out
  test; the Lachman fire of 1 January as the origin.
- Satellite hotspot detections (VIIRS, MODIS active fire; OroraTech counted 5,128 over 7 to 12
  January) as a third scoreboard: where heat was seen from orbit each pass.
- Evacuation orders by Genasys zone and time (the Malibu archive names zones MAL-C111-A/B,
  RMB-U030-A/B, MAL-C112-B at 23:30 on 7 January; the state's evacuation-zone service).
- Canopy height (Meta/WRI 1 m, tile 023012311 over the window) as a fuel layer.
- The 3D view. Done by 2026-11-04: structures coloured by their recorded fate, the record's points
  and lines pulsing at their times, the wind arrow, the sun's light and sky at the cursor, the
  vertical stretched twice, the sea, the mapped perimeter draped as dots
  (`devlog/20261104-the-scene-and-the-paths.md`). Still quick: land cover as the ground's colour,
  the cursor playing when scrolled into view, a north mark, spot fires' flight as arcs, hour labels
  on the arrival ramp, lines thickened by length, a camera preset up the canyons from the sea.
  (The terrain banding was the elevation window's missing columns, fixed 2026-11-03.)
- Check `data/events/fuels.toml` line by line against Scott and Burgan 2005 (RMRS-GTR-153); the
  SH5 row decides the Palisades result.
- Barriers: roads and ridgelines from OpenStreetMap, the 2021 Palisades burn scar from the fire
  history, as unburnable cells; then the model's first night against the 2,920-acre figure.
- Suppression as an agent: engines, dozers, aircraft placed by rule, holding lines where the
  record says they held; the daily perimeters are the test.
- A fuel map made for fire (LANDFIRE, needs an email) beside WorldCover's "tree cover".
- Spotting, and the lattice striping (a Huygens front instead of lattice directions).
- Evacuation and the people: WorldPop cells, roads, a clearance time per neighbourhood.
- A second fire with a book behind it: Fort McMurray, May 2016 (John Vaillant, *Fire Weather: A
  True Story from a Hotter World*; title and author confirmed from the cover Anselm sent on
  2026-11-03, when he named it again as the fire to tune on after the Palisades). Canadian doors to
  check: CWFIS hotspots and the National Burned Area Composite for perimeters, ECCC stations for
  wind, Alberta's post-incident reviews for the suppression record; all recalled, none fetched.
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
- The 2100 scenario, effects as claims: a claims table with source, confidence and a switch, one module onto by-name ports, births and deaths on the people (the UN Low variant first), band multipliers, a capacity scale; air and longevity, falling fertility, heat and thinking as the first rows. `devlog/20261109-effects-as-claims.md`
