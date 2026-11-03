# A fire run again on its own ground

On 7 January 2025 at 10:30 in the morning a fire started in the hills above Pacific
Palisades, Los Angeles, in a Santa Ana wind. It burned 23,448 acres, destroyed 6,845
buildings and killed twelve people before it was contained on 31 January (CAL FIRE's
incident record). This project runs that fire again on the real terrain, with the real land
cover and the hour's wind from a weather reanalysis, lit where and when the record says,
with nobody fighting it. Then it holds the result against the perimeters the incident's
own mapping teams drew as the fire grew. Every number on this page is read from the runs
below it; the prose was written on 2026-11-03 and the runs may have changed since.

## What is in the runs

The ground is the Copernicus 30 m elevation model, from which the slope and aspect of
every cell come. The fuel is the European Space Agency's 10 m land-cover map, each 30 m
cell taking its most common class: shrubland becomes a chaparral fuel model, grassland a
grass model, tree cover a light timber-and-shrub model, built-up land a guessed rate the
model moves fire through houses at, and the sea does not burn. The wind, humidity and
temperature come hour by hour from the ERA5 reanalysis at one point in the middle of the
window, a 25 km average that reads a Santa Ana event mild: the reanalysis gives 6 to 8
metres a second with gusts to 16 on the first night, where news reports put the gusts on
the ridges at two and three times that (recalled from the reporting, not fetched).

Fire moves cell to cell by Rothermel's spread model, the one the United States has used in
its fire behaviour tools since 1972: a rate of spread from the fuel bed, the fuel's
moisture, the wind and the slope, shaped into an ellipse that points down the sum of the
wind and the slope. Nothing is tuned. The fuel parameters were written from memory of the
standard fuel models and have not yet been checked against the published tables; the page
on limits says so.

## What the record says, and what the model says

The incident's mapped perimeters are the measure. The first one, at 06:00 on 8 January
UTC, is a triangle drawn by hand with the acreage typed in (2,920 acres); the first real
perimeter is an infrared map from the following evening, 27 hours after ignition. Where the
paths stand at that hour, and at the end of the runs:

```esim
table 2025-01-08T21:25Z fire.burned_acres=burned obs.acres=mapped fit.sorensen=overlap fit.over_acres=burned_outside_the_map fit.missed_acres=mapped_but_unburned fire.wind_m_s=wind fire.gust_m_s=gust
```

```esim
table 2025-01-12T00:00Z fire.burned_acres=burned_by_the_end obs.acres=last_mapped fit.sorensen=overlap
```

The model overruns the record by a wide margin. On the reanalysis wind it burns about four
times the area the mapping teams found at 27 hours, and by the second day it has burned
every cell in the window that can burn, from Malibu to Brentwood, which the real fire did
not. Four variants ask whether the overrun is a matter of the wind, the
houses or the shrub: the reanalysis gusts in place of the mean wind, built-up land as a
barrier instead of a carrier, and the heavier chaparral model. None of them closes the
gap; the barrier variant removes the houses from the count and nothing more.

## What that means

A spread model with no suppression, no roads, no fuel breaks and no burn scars, on a
landscape where every cell carries fuel, is an upper bound: the fire the land would
allow. At the hour of the first mapped figure, eleven and a half hours in, the model has
more than twenty times the acreage the triangle states, and the difference is the things
this model does not have. Three of them are known and are the next work:

- The fuel table, recalled, to be checked line by line against the published models.
- Roads, ridgelines and the 2021 Palisades burn scar as barriers, from OpenStreetMap and
  the fire history.
- Suppression as an agent: engines, dozers and aircraft placed by rule, holding lines
  where the record says lines held.

The overlap scores above are honest about the shape as well as the size: about a third of
the burned-and-mapped union agrees at 27 hours, because the model's fire does cover the
real one and then goes far beyond it. The arrival-time map on each run page shows where
it went first.

## Not in these runs

Spotting (embers carried ahead of the front), crown fire, the fire's own weather, the
structures as fuel in any physical sense, evacuation, and the people. All of them are
on the list; none of them is modelled. The runs beneath carry every number's source, and
[Limits today](?doc=system/limits) says what is known to be wrong.
