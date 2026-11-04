# The same engine, a different country

On the afternoon of 1 May 2016 a fire was found in the forest a few kilometres southwest of
Fort McMurray, Alberta, in a hot, dry spring after a dry winter. Two days later, on 3 May,
it ran into the city: Waterways, Abasand and Beacon Hill burned, the whole town of 88,000
was ordered out by evening, and about 2,400 homes and buildings were lost. The fire grew for
two months and was declared under control in early July at 589,552 hectares, the costliest
disaster in Canada's record. The figures are the English Wikipedia article's, read on
2026-11-04, until the provincial review (MNP for Alberta Agriculture and Forestry, March
2017) is copied; the record we hold is listed in the project's event folder.

This project runs the fire's first five days on the engine built for the Palisades, to find
out what in that engine was the Palisades and what was fire. Anselm asked for it as the
portability test: "the climate, vegetation and circumstances in Fort McMurray are different
and may force us to generalize our model." Written 2026-11-04; the numbers below come from the
runs as the page is read.

## What had to change, and what did not

The ground is the same ground: the Copernicus elevation model and the European Space
Agency's land cover, which calls nearly the whole window tree cover (1.6 million of 2.3
million cells; shrub and grass a quarter of that). Three things were not there and had to be
found elsewhere.

- Buildings. There is no county footprint service with heights; the buildings are
  OpenStreetMap's, 21,380 of them in the window, with heights from their tags where present and
  two storeys otherwise, and no record of which burned.
- Wind. NOAA's 3 km analysis stops at the border. The wind is the ERA5 reanalysis at one
  point, as the first Palisades project used, and it reads this fire mild: 2 to 5 metres a
  second through the days the record has gusts to 72 km/h.
- The record. Nobody mapped this fire hour by hour. The record is the satellite hotspots in
  the Canadian Wildland Fire Information System's archive (2,380 detections in the window over
  the five days, VIIRS, MODIS and AVHRR), each marking the ground within its pixel as seen
  burning by then. That area is a floor on the burned area: cloud, smoke and the hours between
  passes. The engine is scored against it every six hours with the same measures as against the
  Palisades perimeters.

The elevation model also had something to say: above 50° north its pixels are one and a half
arc-seconds wide, and the window sampler, which had put every pixel into a cell, left one cell
in three empty, the same holes as the Palisades had the day before. Cells no pixel reaches now
take the nearest one; the earlier fires are unchanged by it.

## What the runs say

Where each run stands against the ground seen burning from orbit at the start of the third
day (06:00 UTC on 3 May), of the fourth, and at the end:

```esim
table 2016-05-03T06:00Z fire.burned_acres=burned obs.acres=seen_from_orbit fit.sorensen=overlap fire.wind_m_s=wind
```

```esim
table 2016-05-04T06:00Z fire.burned_acres=burned obs.acres=seen_from_orbit fit.sorensen=overlap
```

```esim
table 2016-05-07T00:00Z fire.burned_acres=burned_by_the_end obs.acres=seen_from_orbit fit.sorensen=overlap
```

The Palisades' own setting, thresholds times five, burns two to three times the seen area
from the second day on. Times eight the pace is the record's: within a few hundred acres of the
seen area on the fourth morning and within a tenth by the end, with the first night a third
short. Times three fills the window in three days. So the engine carries over with one constant
moved, and the constant that moved is the one that stands for everything the fuel table does
not know; a boreal spruce stand and a Santa Monica chaparral slope should not share it, and
they do not.

The shape is the failure. The overlap sits between a third and two fifths at every hour, where
the Palisades had four fifths on its first day. The reason is in the wind chart: the reanalysis
point reads a light wind from the south-east on the morning the fire crossed into the city from
the south-west, so the model's fire spreads as a disc from the point of ignition, grows south and
west into the forest, and never reaches Abasand or Beacon Hill at all. The buildings in the model
stand unburned while the record has 2,400 of them gone. This is the Palisades' first lesson
again from the other side: there the 3 km wind gave the first day and the reanalysis did not;
here only the reanalysis exists, and it gives the pace of the week and not the day that
mattered.

## What is honest about it, and what is not

What is real: the ground, the land cover, the buildings and the hotspots. What is a guess: the
ignition hour (about 16:00 local, recalled from reporting and not yet checked against the
review), every fuel constant, the reanalysis as the wind. What is missing: a boreal fuel that
burns as a spruce crown does, the wind that blew, the final perimeter (the National Burned Area
Composite is on the same server and not fetched), the 2,400 lost buildings as a record. The next
step for this fire is the provincial review's chronology and a wind from the day's station
record rather than a reanalysis point. [Limits today](/docs/system/limits) has the list.
