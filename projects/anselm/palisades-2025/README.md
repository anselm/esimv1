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

On the reanalysis wind the model is slow. At the hour of the first infrared map it has
burned about an eighth of the mapped area, and it reaches the record's size only at the
end of the fourth day, when the real fire had been that size for three. The shape it ends
with is fair: a little over half of the burned-and-mapped union agrees. Four variants ask
whether the shortfall is the wind, the houses or the shrub. The reanalysis gusts in place
of the mean wind close part of the first day's gap and overrun by the end; the heavier
chaparral model burns less, not more; built-up land as a barrier changes little. None of
them produces the first night, when the real fire went from the ridge to the sea in hours.

Until the afternoon of 2026-11-03 these runs were made on broken ground. The elevation
window had a column of missing values every third or fourth cell, which the slope code
read as a cliff on either side, and the model then burned four times the record and filled
the window by the second day. The runs on this page are on the corrected ground; the
devlog entry *The holes in the ground* has the account.

## What that means

Two things are missing and they pull in opposite directions. The wind that blew on the
ridges that night was two or three times what a 25 km reanalysis says, and a spread model
on the real wind runs faster; the engine project beside this one, which reads NOAA's 3 km
analysis with its gusts, matches the first day's acreage. And nothing here is fought: by the
second day the real fire was being held on its eastern and northern flanks, so a model that
is slow early and unhindered late can end near the right size without having been right at
any hour. The hourly perimeters, not the final acreage, are the test. Three things are
known and are the next work:

- The wind shaped by the terrain, so the Santa Ana accelerates down the canyons and
  slackens in the lee, instead of one flat value for the window.
- The fuel table, recalled, to be checked line by line against the published models, and a
  fuel map made for fire beside the land-cover classes.
- Suppression as an agent: engines, dozers and aircraft placed by rule, holding lines
  where the record says lines held.

The arrival-time map on each run page shows where the model's fire went first.

## Not in these runs

Spotting (embers carried ahead of the front), crown fire, the fire's own weather, the
structures as fuel in any physical sense, evacuation, and the people. All of them are
on the list; none of them is modelled. The runs beneath carry every number's source, and
[Limits today](?doc=system/limits) says what is known to be wrong.
