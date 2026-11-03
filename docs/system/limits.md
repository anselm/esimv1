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
- A country has one temperature. India at 1.7 billion people is the loudest case of a mean
  hiding a continent. The niche threshold is on the annual mean and knows nothing of
  humidity, which is what kills. The hottest-month marker is a guess and labelled one.
- The wet-bulb layer stands one weather station for a country, estimates an afternoon
  wet-bulb from the day's maximum temperature and mean dew point (a little high), assumes
  constant relative humidity under warming, and has no station within 80 km for
  thirteen capitals. Its two limits are recalled from their papers and not yet checked.
- One station per country also makes steps: the wet-bulb count of people is a whole
  country's population once its capital's station crosses, so a populous country
  crossing moves the count by a billion in a year (the `falling` variant between 2050
  and 2075). The step is the layer's granularity, not a cliff in the world.
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
