# "Recent extreme heatwaves are no match for future events", read for the 2100 scenario

Anselm, 2026-11-07, late: "once this is all validated and done i want you to review this
document for the 2100 scenario", with the PDF of Aglas-Leitner, Perkins-Kirkpatrick, Brunner,
Undorf, Aglas-Leitner, Fischer and Stone, *Nature Communications* 17:11456, published
2026-11-05, doi 10.1038/s41467-026-76809-5, CC BY 4.0. Read in full by Claude (Fable 5.1)
that night; every number below is from the article's text, tables and figure captions, not
the Supplementary Data, which the article cites for the fine values and which was not read.

## What the paper does

Twenty-two observed heatwaves of 2010 to 2023, nine of them the focus (Australia 2019,
China 2022, Europe 2018, India and Pakistan 2019, Northeast Siberia 2020, Northern Africa
and the Middle East 2023, Pacific Northwest 2021, South America 2022, Southern Africa
2015-16), each with a box of coordinates (Table 1) drawn around where it happened. Six
characteristics on daily maximum temperature: median and maximum severity (the day's
exceedance of the climatology as a multiple of the threshold's exceedance), cumulative
heat, duration, median and maximum area. A heatwave is three or more days with the
region's area-weighted Tmax above the calendar-day 90th percentile of 1961 to 1990, the base
period fixed on purpose because human heat tolerance does not move. Projections from CMIP6
(15 models kept after evaluation), CMIP5 (18), and two large initial-condition ensembles
(CanESM5, 50 runs; ACCESS-ESM1.5, 40), under SSP1-2.6 to SSP5-8.5, and by global warming
level from 0 to 4.5 °C in half-degree steps. Reference data Berkeley Earth, with ERA5 as the
second opinion; the two disagree on some events (the 2023 Northern Africa and Middle East
event is 133 days in one and 24 in the other, because one data set splits it).

## What it finds

- The headline is in the title. Even under SSP1-2.6, five of the nine regions project
  heatwaves approaching or reaching the six-month maximum the method allows. Under
  SSP3-7.0 the chance of beating the observed event's median severity goes from under 0.5%
  to about 3% of projected heatwaves in China and from under 3% to about 12% in Australia.
- By warming level: the maximum severities of the Australian 2019, Chinese 2022, European
  2018, Northern African 2023 and Southern African 2015-16 events are "nearly impossible
  without global warming" (under 0.3% at 0 °C) and under 0.8% up to 0.5 °C. At 2.0 °C the
  chance of a worse one is 6 to 8% in China and Southern Africa and nearly one in five in
  Europe and Northern Africa and the Middle East; Australia stays under 2%. Between 2.5 and
  3.0 °C, China and Southern Africa go from about 13% to 22%, Europe and Northeast Siberia
  from 27 to 33% up to 37 to 47%, Northern Africa, the Pacific Northwest and South America
  from about three fifths to 63 to 71%. Past 3 °C, Australia reaches 13 to 29% (at 3.5 to
  4.5 °C), China and Southern Africa a third to 60%, Europe and Northeast Siberia half to
  three quarters, Northern Africa and South America 80 to 98%. The average shift is 5 to 12
  percentage points per half degree, steepest between 2.0 and 3.5 °C. At 4.0 °C and above
  all nine regions see a considerable share of heatwaves with local extreme heat over more
  than two thirds of the region for at least a day.
- Return periods: in 1951 to 2005 the observed events had median return times beyond the
  55-year window in most ensembles, many models unable to produce them at all. In 2015 to
  2054 they fall to low single digits of years in most regions whatever the pathway. In
  2060 to 2099 the majority of ensembles project one or more such heatwaves a year. The one
  exception is the maximum severity of Australia 2019, which stays at 20 to 40 years under
  low emissions, with most of CanESM5's fifty runs unable to reach it.
- The policy line: the NDCs are read as 2.6 to 2.8 °C, that is warming levels of about 2.5
  to 3.0, where "a concerningly high percentage" of projected heatwaves beat the observed
  ones on all six characteristics at once.
- Stated limits, which matter for us: dry-bulb only, no humidity; boxes, not countries;
  impacts and adaptation out of scope by design ("the population and the environment may
  partly adapt", with physiological limits cited against it); the base period fixed; the
  reference data disagree in places; internal variability found to matter little at the
  ensemble scale (the large ensembles' 5th to 95th ranges sit inside the multi-model ranges).

## What the 2100 scenario has, against this

The model has no heatwaves. `people.heat` counts people whose cell's annual mean has left
the 29 °C niche; `people.wetbulb` counts people past humid limits against a reanalysis
baseline; both are exposure by the mean, not events, and both say so. `score.regions`
turns the niche share into a country's heat stress, which is what `policy.capacity` reads
as a pressure. The wild-card deck has no heatwave card, and should not get one with a fixed
chance, because the paper's whole result is that the chance is a function of the warming.
The README's "What each half degree costs" is already in the paper's unit, half degrees of
global warming, which is the hinge.

Where the paper and the model disagree in kind: the paper is about events, duration and
area, which is what kills, breaks grids and makes the news, and which our annual means
cannot see; the model has humidity, which the paper lacks and says it lacks; the paper's
regions are boxes around past events, ours are countries and half-degree cells.

## Ways in, in the order I would build them

1. **The paper's result as a lookup, at the run's warming.** A table
   `data/events/heatwaves.toml`: the nine reference events (Table 2: dates, box, severity,
   area, length, cumulative heat, both data sets) and, per region and per warming level,
   the exceedance share of the observed maximum severity and the return period, transcribed
   from the text and captions above, `checked` against the article, with the Supplementary
   Data number named for each cell we would want finer. A module `heat.waves` reads the
   run's global anomaly each year, interpolates the region's chance and return time, reports
   them as series, and publishes a crossing the first year a region's expected recurrence
   passes once a year. Honest shape: it is the paper read at the run's temperature, not a
   simulation, and the describe says so. The compare page then shows, under one cursor, "a
   heatwave worse than Europe 2018: one in five chance per event at this warming, expected
   yearly from 20xx". An afternoon.
2. **The paper's calculation on our own ground.** The cells cache holds the reanalysis daily
   boxes for the populated world; a `heat.waves` with `source = "cells"` computes the six
   characteristics on daily Tmax over a fixed base (1981 to 2010, since POWER starts in 1981,
   a stated deviation from the paper's 1961 to 1990), then shifts the daily series by the
   run's warming pattern and recounts each year: severity, duration, area per region, per
   country, per cell. A delta method, which keeps today's variability, which is the crude
   part and must be said; but it gives the paper's metrics on our grid every year, and with
   the humidity we have, a wet-bulb version the paper could not do. A day or two.
3. **Heatwaves as the derailment's shocks.** The capacity module reads a stress by the mean.
   The article's own frame, and Dyke's (devlog 2026-11-07, derailment), is that the shocks
   are events: a six-month heatwave over two thirds of a region is a grid, a harvest and an
   election. Once `heat.waves` publishes events with duration and area, `policy.capacity`
   and `score.regions` can read them as knocks, the way they read the wild cards today.
4. **The README.** A paragraph in "What each half degree costs" with the paper's ladder (2.0:
   Europe and Northern Africa one in five; 2.5 to 3.0: the steep part; 3.0 and beyond: most
   regions most of the time) and one in "What the model does with a crossing" saying the
   model counts means and not events until (1) or (2) exists. The source goes in with its
   date read.

Not to do: a heatwave row in the speculative deck; a mortality number, which the paper
refuses to give and we have no curve for.
