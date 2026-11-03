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
- Nothing feeds back: a tipping element flagged is not tipped; the Amazon does not release
  its carbon; permafrost does not thaw in the run.
- Sea level is thermal expansion only, a floor.
- The warming applied to a place is its country's. People now sit on half-degree cells
  (WorldPop's 2020 pattern, the UN's count), each populated cell has its own climatological
  mean and its own baseline decade of daily heat and humidity from the MERRA-2 reanalysis
  (NASA POWER), and exposure is computed cell by cell before it is summed; but the warming
  added to every cell of a country is one number from the country's record regressed on
  the global anomaly. Deserts and mountains keep their climatology and share their
  country's trend. A gridded warming pattern is the next step. The niche threshold is on
  the annual mean and knows nothing of humidity or season; the 2020 settlement pattern is
  held for every year.
- The wet-bulb layer's baseline is a reanalysis, not a measurement: MERRA-2 at about 50 km,
  daily maximum temperature and the day's dew point (standing for the afternoon's, which
  runs a little high), ten years, constant relative humidity under warming (the standard
  first-order assumption, stated as one). Its two limits are recalled from their papers
  and not yet checked. The one-station-per-capital form is kept as `source = station`
  and is the check on it.
- A country "crosses" a limit when half its people live where the limit is passed on more
  than three days a year. The counts of people and person-days are continuous; the
  country event is a stated rule on them, chosen so that a sentence can be written, and
  the half is a choice, not a finding.
- Population after 2023 is the UN's medium projection, which assumes a world where none of
  the crossings change where people live. Before 1950 it is held flat and called a
  placeholder.
- The tipping table's numbers are recalled from the assessment and not yet checked against
  it. Every row says so.

## The basin

- The crop grows too slowly and takes up too little nitrogen, so nitrate at the gauge
  runs about twice the record.
- Snow, soil and routing are a lumped water balance per cell; peaks are early and sharp.
- Herbivores and predators are cohorts with guessed rates and no temperature response; the
  predator does not know it is hot.
- Fire spread rates are not a fire model's; a season's drying confounds the auditor's
  recovery measure.

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
