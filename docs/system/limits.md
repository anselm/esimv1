<!-- order: 16 -->
<!-- section: Changing it -->
# Limits today

What the system and its models get wrong or do not do, as of 2026-11-01. This page is
meant to be read before any result and to be rewritten as things change; the devlog
holds the history.

## The planet

- The carbon cycle is an impulse response with the Millar 2017 sink feedback (recalled
  parameters); it runs 26 ppm high in 2024 (451 against 425 measured, bias +2.8%). A box
  carbon cycle with real stocks and the hindcast harness are the next steps.
- Forcing is CO2, methane and N2O computed here, plus the IPCC AR6 assessed series for
  everything else (aerosols, ozone, the minor gases, land use, volcanoes, the sun), held
  at their 2019 values after 2019. With aerosols in, the 2024 anomaly is +1.69 °C against
  +1.54 measured and the hindcast's temperature skill is 0.86; crossing years are a few
  years early rather than twenty. Holding aerosols flat after 2019 is a warm-side
  assumption, since they are falling.
- One thing feeds back: once the run passes a tipping element's central estimate, the
  carbon its row of the assessment states (permafrost abrupt thaw per degree of warming,
  the Amazon's 30 GtC, permafrost collapse's 125 GtC; Armstrong McKay et al. 2022, checked
  against the paper's Table 1) is released into the carbon ledger over the row's timescale
  as a first-order approach, which is a shape the assessment does not give. It comes to
  about 16 GtC, 5 ppm and 0.02 °C by 2099 with emissions held, and 69 GtC and 0.1 °C with
  every element tipping at the low end of its range. Everything else flagged is not
  tipped: no ice into the sea, no albedo change from sea ice or forest (the energy balance
  has no term for it, which is also why the boreal forest elements, whose net sign is the
  albedo's, are left out), no methane as itself.
- Sea level is thermal expansion only, a floor.
- The warming applied to a place is its country's. People now sit on half-degree cells
  (WorldPop's 2020 pattern, the UN's count), each populated cell has its own climatological
  mean and its own baseline decade of daily heat and humidity from the MERRA-2 reanalysis
  (NASA POWER), and exposure is computed cell by cell before it is summed; but the warming
  added to every cell of a country is one number from the country's record regressed on
  the global anomaly. Deserts and mountains keep their climatology and share their
  country's trend, and for small countries the fitted slope is not physical: Puerto Rico
  warms 1.78 °C and Qatar 1.75 °C per degree of global warming in the fit, against about
  1.0 for the United States and 0.82 for India, so the Caribbean islands and the small Gulf
  states that leave the niche after 2040 do so on the slope, not the heat. A pattern from
  the reanalysis's own 1981 to 2025 trend per cell was built and is drawn on the run page
  (`climate_slope_cells`) but not applied: over South Asia MERRA-2 cools by 0.4 °C between
  the 2000s and the 2010s while the globe warms, which no thermometer record does, so its
  trend is the reanalysis's own fault there. A model-ensemble pattern is the next step. The niche threshold is on the annual mean and knows
  nothing of humidity or season; the 2020 settlement pattern is held for every year.
  Against the published count (Lenton et al. 2023, read on 2026-11-03: 62 ± 26 million
  people at 29 °C and up at +1.0 °C, 2.1 ± 0.8 billion at +2.7 °C, 3.3 billion at +3.6 °C),
  this model is high now (147 million at +1.7 °C in 2024, on a warmer model year and a 2001
  to 2025 climatology against their 1960 to 1990 one) and low later (1.32 billion at +3.4
  °C in 2099 with emissions held). The later gap is the pattern: their local warming is
  CMIP6's, in which land warms 1.5 times the global mean and India more; ours is the
  observed country slope, India's 0.82, which carries the record's aerosol haze forward.
- The wet-bulb layer's baseline is a reanalysis, not a measurement: MERRA-2 at about 50 km,
  daily maximum temperature and the day's dew point (standing for the afternoon's, which
  runs a little high), ten years, constant relative humidity under warming (the standard
  first-order assumption, stated as one). Its two limits are recalled from their papers
  and checked against their papers on 2026-11-03 (Vecellio et al. 2022 measured 30.55 ±
  0.98 °C; Sherwood and Huber 2010 argued 35 °C). A 50 km reanalysis does not reach the
  station extremes of the Gulf coast (Raymond, Matthews and Horton 2020 found ERA-Interim's
  extremes 2 to 4 °C below the Gulf stations'), so this layer reads the humid heat mild: 6.5 million people live where 31 °C
  wet-bulb is passed on three days a year in 2024, and 38 people past 35 °C in 2099 with
  emissions held. The one-station-per-capital form is kept as `source = station` and is
  the check on it; it read the same limit in steps instead (1.58 billion in 2099, India
  and Pakistan entering whole at their capitals' first three days). Neither is the
  measurement.
- The one transition the runs find rather than read from a table is a step in the
  wet-bulb count between 3.0 and 3.5 °C of global warming (35 to 204 million on the
  growing path): the humid plains of eastern China crossing 31 °C in the 2070s and 2080s,
  cell after cell, with Japan's coast and Taiwan after. The Indus and Ganges plains barely
  cross in any run, which station records contradict; the reanalysis baseline reads humid
  extremes mild and India's warming is scaled from a record that has run cool. The China
  step is probably real and probably not first.
- The heatwaves (since 2026-11-07) are replayed weather, not weather: one observed year of
  daily maximum temperature per one-degree cell (2011 to 2020, MERRA-2) drawn by the seed each
  year and shifted by the warming. The base is that decade itself, so "exceptional" means
  exceptional for the 2010s, a decade already a degree warmer than the 1961 to 1990 base the
  heatwave literature uses. The spread of the days does not widen as the mean climbs unless a
  knob with no paper behind it is set, and Aglas-Leitner et al. 2026 find projected
  heatwaves lengthen and widen faster than the mean alone would make them, so the layer is a
  floor. Years are independent, so a run has no persistent drought or blocking. By the 2050s
  on the held path some tropical cells are over their threshold every day, which is the
  method's answer and should be read as "the whole year is now a 2010s summer", not as a
  365-day heatwave. A country's heatwave is judged on the people-weighted mean of its cells,
  so a one-cell island crosses first and often. Nothing in the model reads the heatwaves
  yet: the derailment loop still runs on annual-mean stress.
- A country "crosses" a limit when half its people live where the limit is passed on more
  than three days a year. The counts of people and person-days are continuous; the
  country event is a stated rule on them, chosen so that a sentence can be written, and
  the half is a choice, not a finding.
- Population after 2023 is the UN's medium projection, which assumes a world where none of
  the crossings change where people live. Before 1950 it is held flat and called a
  placeholder.
- The tipping table's rows were checked against the assessment's Table 1 on 2026-11-03;
  the model passes each threshold about six years before the record would, since it runs
  0.2 °C warm in 2024 (1.5 °C in 2018 against the record's 2024).

## The basin

- The crop grows too slowly and takes up too little nitrogen, so nitrate at the gauge
  runs about twice the record.
- Snow, soil and routing are a lumped water balance per cell; peaks are early and sharp.
- Herbivores and predators are cohorts with guessed rates and no temperature response; the
  predator does not know it is hot.
- Fire spread rates are not a fire model's; a season's drying confounds the auditor's
  recovery measure.

## The fire

- Rothermel's spread model on one 25 km reanalysis wind is slow on a Santa Ana day. On the
  Palisades fire (2026-11-03, corrected ground) it has about an eighth of the mapped area at
  27 hours (2,000 acres against 15,800) and reaches the record's size at the end of the fourth
  day, when the real fire was being held; the gusts variant has 2,500 at 27 hours and overruns
  by the end. Ending near the right size is not being right at any hour. Until that afternoon
  the elevation window had a NaN column every third or fourth cell (pixel centres on cell
  edges, `terrain.rs`), the slope code read each as a cliff, and the same model burned four
  times the record; the old figures are in the devlog, not here.
- The wind is flat. Neither fire module adjusts it for terrain (no channelling down canyons,
  no lee), and neither feeds the fire back into it (no convective column, no indraft, no
  plume-driven spread); both are one-way models that read a wind and never write one.
- The region scores (`score.regions`, 2026-11-05) are ranks, not measures: a band is where a country
  stands among the countries that report that indicator, so the world's mean cannot fall by every
  country getting worse together, only by the rules that move bands after the first year, and those
  rules are placeholders (a quarter off living and water past the niche; a global knock per wild-card
  kind). The indicators' years differ by band; the governance band is one of six the Bank publishes;
  nothing regional happens to a region, since the wild cards carry no place yet.
- Derailment (`policy.capacity`, 2026-11-07) is the shape of Dyke's argument with guessed numbers: the capacity
  to cut falls with the share of people under heat stress by a gain of four, amplified by a people-weighted
  inequality term, falls at half the gap a year and recovers at a tenth times the voice band; the wild-card
  knocks and the conflict draw (two percent a year times the stress share, one percent of a country's people,
  a knock by its share of the world) are placeholders declared in the parameters and nothing else. The
  pressures it reads are the run's heat exposure, which this model under-reads over South Asia, so the capacity
  falls less than a fuller impact layer would make it; the debt loop, the backlash and the loss of cooperation
  the article names are all folded into one number with no finance, no politics and no countries' positions
  in it. The options (budget and cut needed) use a recalled TCRE and the run's own warming, which is about
  0.2 °C warm, so the 2 °C budget closes in the 2030s on every path; read them as the model's, not the world's.
  The coupled tipping elements (`tipping.coupling`) carry the paper's ensemble-mean net lowering as a fraction
  of a different assessment's thresholds, applied once per element when any of its pushers has passed, with
  no timescale: the paper's cascades are committed over centuries, and here a shifted threshold is passed the
  year the warming reaches it.
- Wild cards (`events.shock`, 2026-11-05) are placeholders: the deck's annual chances are recalled
  orders of magnitude, its effects are three blunt levers (a global forcing pulse, a global emissions
  factor, a global death fraction with no place or age), a card may repeat, and no card changes the
  emissions path for good, so with emissions held the deck moves the century by tenths of a degree and
  a few hundred million people. It is not a risk model; it is a way to see a run end somewhere else.
- Vegetation is a cover class, not a fuel: WorldCover's shrubland is one thing whether it is
  coastal sage or forty-year chamise; no fuel load, no time since last burn, no canopy
  height, no seasonal live moisture, no crown fire.
- Every fuel parameter (`data/events/fuels.toml`) is recalled from Scott and Burgan 2005
  and not yet checked; the moisture, wind-adjustment and ellipse equations are recalled
  too. The ellipse's wind is in m/s with the FARSITE coefficients, which is consistent with
  the mile-per-hour form by the ratio of the coefficients and otherwise unverified.
- The wind is one ERA5 point at 25 km for the whole window, which reads a Santa Ana event
  at a third of the gusts news reports put on the ridges (recalled, not fetched); the gust variant is the upper bound the
  reanalysis allows, not the wind that blew.
- WorldCover calls most of the Santa Monica Mountains "tree cover" (196 of 427 km² in the
  window; shrubland 45), so chaparral is partly modelled as a light timber-and-shrub
  understorey. A fuel map made for fire (LANDFIRE) needs an email to fetch and is not used.
- Fire moves along 32 lattice directions within three cells; a narrow wind-driven ellipse
  still loses rate between them, and the arrival map shows striping along lattice
  directions. The first mapped perimeter (06:00Z on 8 January) is a hand-drawn triangle
  with a typed acreage, not a map; the first infrared perimeter is 27 hours in.
- Our own engine (`fire.cells`, 2026-11-03) is a heat budget with guessed constants. One
  multiplier on the ignition thresholds is swept across the runs; with heat felt three cells
  away there is a cliff (the window burns in hours, or the next cell never lights), with heat
  felt only by the next cell the first day matches the record's acreage (15,300 against 15,800
  at 27 hours on the corrected ground) and the fire then keeps growing because nothing stops
  it, to twice the record by the fourth day. The plausible-pace runs reach an eighth to a fifth
  of the destroyed structures: a built-up cell is given a high threshold and the engine walks
  around the neighbourhoods; embers into houses and a house-to-house reach are knobs, not
  physics. With `structures = "on"` (the `bodies` variants) every footprint burns as its own
  body and is scored per footprint: recall can be set anywhere by the threshold, but precision
  stays at the record's base rate of about 0.48 to 0.54, because the model has nothing that
  tells a survivor from its destroyed neighbour. The house's own vulnerability (roof, vents,
  eaves, year built) is not in any public field we hold; the assessor's roll by parcel number
  is the noted route. Until the evening of 2026-11-03 the engine was nearly blind to slope: it
  entered only the reach of the heat, linearly, under a form that caps any directional effect at
  a few times the flat rate, so the fire spread in rings while Rothermel's ran up the canyons. A
  rate factor with Rothermel's shape (`slope_rate_gain`, off by default) is now a declared option;
  at the literature's gain it stalls on ridge tops because the downhill half is a division, and at
  a gain of 6 it gives the best perimeter overlap so far (0.70) at half the first day's area. On the morning
  of 2026-11-04 the engine gained its second form (`form = "vector"`, `devlog/20261104-the-vector-form.md`):
  wind and slope as vectors with Rothermel's shape, Anderson's ellipse for the flanks and back, and a
  heat budget that decays only when idle, since one that decays always has a floor and the floor is a
  cliff. With it the first day is right for the first time on either engine (16,525 acres at 27 hours
  against 15,824, overlap 0.79) and the second day is wrong in a way no constant reaches: the model
  fills the mountain block north to Calabasas by 49 hours where the record grew by 3,800 acres. The
  record's completed lines, made to hold (`fire.suppression`, a module between the record and the
  engines, from the afternoon of 2026-11-04), change nothing, because the record's first line is
  dated after the overrun; and nothing else about suppression is modelled: no crews, no aircraft,
  no engines at houses, no retardant. The candidates are the wind over the mountains on the second day and the
  suppression the record does not place. The HRRR 3 km wind it uses reads 10 m/s with 21 m/s gusts at the ignition point in
  the first hour (checked against GDAL); the reanalysis read 7 and 13.
- Fort McMurray 2016 (the second fire, `anselm/fortmcmurray-2016`, 2026-11-04): the engine ports,
  with OpenStreetMap buildings, the ERA5 point wind and satellite hotspots as the record, and at
  thresholds times eight its pace matches the ground seen from orbit from the third day; the Palisades'
  setting burns two to three times that. The shape is wrong at every hour (overlap a third to two
  fifths): the reanalysis reads 2 to 5 m/s where the record has gusts to 72 km/h, the model spreads as
  a disc and never enters the city the fire burned on 3 May. The ignition hour is recalled; the fuel
  is one "tree" kind for a spruce forest; the hotspot area is a floor, not a perimeter. Above 50°N the
  elevation model's pixels are 1.5 arcsec wide; until this afternoon the sampler left one cell in three
  empty there (`terrain-v4` fills them from the nearest pixel).
- Devil's Gulch 2029 (`anselm/nicasio-2029`, 2026-11-08): a scenario with no record, so nothing scores it.
  The wind is another year's (27 to 29 October 2019, HRRR, through `wind_at`), and at 3 km it reads 1 to
  8 m/s at the canyon floor on a day the forecast office warned of ridge gusts; the ridges are not modelled.
  Two thirds of the window is one fuel, "tree", and the engine's pace there is between everything (the
  whole window in two days at thresholds times 3 or 5) and nothing (times 8): the hour the fire reaches
  the house runs from 19 to 34 hours or never. The house is a geocoded point on the road centreline,
  reached within 50 m. Nobody fights the fire.
- The same scenario on Rothermel's model (`anselm/nicasio-2029-rothermel`): 16,000 to 23,000 acres and
  one to two kilometres short of the house, on a 25 km reanalysis point from the same borrowed days and
  recalled fuel loads, with TU1 standing in for a redwood canyon. Two unscored engines agreeing on the
  direction and the slowing is the only check either has.
- The 3D view takes building heights as feet because the county's service does not state a
  unit.
- Built-up land carries fire at a guessed rate; nothing about structures is physical.
  No spotting, no crown fire, no fire weather of the fire's own, no evacuation, no people.

## The coast

- Half Moon Bay (`anselm/halfmoonbay-shoreline`, 2026-11-08): a one-line shoreline model, not a fluid
  dynamics model: no currents, no cross-shore profile, no setup, no reflection, no tide. The waves are a
  50 km reanalysis cell's, their direction fitted so the bay starts in balance, and the transport
  coefficient is fitted eighty times below the open-coast value to the Corps' bluff rate; the leak
  through the breakwater is fitted to the harbor's accumulation. With those, the pattern stands
  (erosion to 1.4 km, Miramar still, sealing the breakwater changes nothing on the beach) but the
  headland-only line still loses half a metre of bluff a year where the record has eight centimetres, so
  the breakwater's effect comes out a doubling where the record has seven times. Today's shoreline stands
  in for 1961's; the revetments are not placed; the southern end imports sand.

## The earthquake

Portland after a Cascadia earthquake (`anselm/portland-cascadia`). Status as of the runs
of 12 November 2026; the [project page](/anselm/portland-cascadia) gives the figures in full.

- **No record to score against.** The ground shaking is the output of DOGAMI's scenario model, and
  the building damage is compared with DOGAMI's own loss estimate, to which six of the model's
  numbers were fitted. Agreement with that estimate is agreement between two models.
- **Recalled values.** The fragility curves for buildings, tanks and the two modern bridges, and
  every casualty rate, are recalled from memory of the Hazus manual and have not been checked
  against it. The curve for the older bridges and the closure times for road segments are
  judgements made for this project. Each is declared in `fragility.toml`.
- **The death toll.** The model counts the dead from the shaking twice, cell by cell and by agents,
  and the two disagree in daytime runs: 1,000, 564 and 602 by cell for seeds 1 to 3 against 670,
  460 and 550 by agents. At night they agree. The cause has not been found, and the run's total
  uses the agents' count.
- **Night and dry soil.** Only shaking makes a building collapse in the model, and nearly all of
  its deaths happen in collapses. So the toll on dry soil is almost the same as on saturated soil,
  where DOGAMI's falls by a third, and the night-time toll is nearly two thirds of the daytime
  toll, where DOGAMI's is about a quarter.
- **The spill.** In the base run 233 of the 587 tanks fail and release 60.6 million gallons, about
  two thirds of the 2022 county study's minimum of 95 million. No run models fire in the spilled
  fuel or the fuel's movement down the river. Deaths at the terminals are a number drawn in the
  study's range of 0 to 7 and scaled by the release.
- **Electricity.** Transmission structures are the only part of the power system in the model.
  Ninety percent of people keep their electricity at the moment of the earthquake, and everyone
  has it by day 22, where the Oregon Resilience Plan expects one to three months. The number of
  hospitals that lose power and the deaths after the shaking both depend on this result.
- **Bridges.** Ten of the twelve Willamette bridges close in the base run. The seven that take the
  worst level of damage are given a 365-day repair, a judgement, and are still closed when the
  run ends, eleven months after the earthquake. The `burnside-ready` variant has no effect on the computed runs, because the setting it
  changes is read only by the `ranges` version.
- **Fuel use.** The response's daily use, the hospitals' generator fuel and the date and rate of
  resupply are guesses. The response's use is counted only once a fuel truck can reach a hospital
  from the terminals, which is day 111 in the base run, so it uses no fuel for 110 days.
- **Water.** Service follows a schedule drawn from the Resilience Plan's range. The pipes are not
  in the model.
- **Deaths after the shaking.** These rest on one rate, deaths per person-day without electricity
  in Puerto Rico after Hurricane Maria, and on a guessed daily rate for the badly injured with no
  open hospital. Cold and unsafe water are in the table and switched off for want of a source.
  The base run adds 2 deaths by the end of 2027 and the `ranges` variant 66. The difference is the
  length of the power cut.
- **The agents** (`people.agents`, 93,520 agents of ten people each) are placed at a home and a
  workplace and decide nothing. Nobody leaves, shelters, travels or waits at a hospital.
- **Stale sentences in the runs.** In the runs of 12 November the story says "The grid is down
  everywhere" and "Day 1: power is back for half the people" in computed runs where 90 percent
  kept their electricity, and it gives the Broadway Bridge's reopening as day 20 for a 21-day
  repair. `service.routes_open` reads 1.000 in computed runs. The first two sentences are
  corrected in the code and will change when the runs are next made.
- **The building count.** A run reports 297,803 buildings. The data folder's note on how the
  source was prepared gives 297,120 outlines. The difference has not been explained.
- **Not in the model:** aftershocks, the tsunami (which does not reach Portland), the Columbia
  River bridges and Interstate 5 to the north, debris on the roads, sewage, telephones and data,
  food, the airport, the port, shelters, people leaving the city, and hospital beds, staff and
  supplies.

## The system

- Modules are compiled in; the wire binding is specified and not built, so a kind from
  another hand cannot yet be run without rebuilding.
- The vocabulary is governed by this documentation and a test, not by a published
  ontology.
- The facade is one turn and has only run through a local model, because the Anthropic
  key had no credit on the day.
- Ensembles are not audited; the auditor reads single trajectories.
- Everything runs on one machine; the shape for a public service is decided and not
  built.
- Several licence texts are still to be fetched; nothing here is fit to publish outward
  until they are.

## How to read any number here

Find the module that produced it, read its describe for the parameters' provenance, read
the scores for how its layer did against the record, read the audit for what the
auditor could and could not measure, and read this page for what is known to be wrong.
If the number still looks important, it is a question for the next module, not a
finding.
