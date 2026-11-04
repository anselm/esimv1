<!-- order: 16 -->
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
- The 3D view takes building heights as feet because the county's service does not state a
  unit.
- Built-up land carries fire at a guessed rate; nothing about structures is physical.
  No spotting, no crown fire, no fire weather of the fire's own, no evacuation, no people.

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
