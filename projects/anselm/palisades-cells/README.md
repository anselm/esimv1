# The same fire, cell by cell

The first Palisades project runs a 1972 spread formula on a 25 km wind and is eight times
too slow on the first day. This one starts from the facts instead and asks what the
simplest engine built on them does. Every cell of the ground knows its height and slope, its cover, how much of it is
under a building and how tall, whether a highway crosses it, and, hour by hour, the wind,
gust, temperature and humidity that NOAA's 3 km analysis put over it. A burning cell
heats the cells around it, further downwind and uphill. A cell lights when the heat it has
taken in outweighs what its fuel and its moisture can absorb, then burns with a strength
and for a time set by what is in it: grass for minutes, chaparral for twenty, a built-up
cell for an hour and more. In strong wind a burning cell throws embers that land downwind
and may start a spot fire. Roads and the sea carry nothing. Nobody fights it.

Every constant in it is a guess and says so on the run pages. One number, a multiplier on
all the ignition thresholds, is varied across the runs below so the model can be read
against the record; the record it is read against is the incident's mapped perimeters and
the 12,081 structures CAL FIRE's inspectors visited, 6,845 of them destroyed. Written
2026-11-03; the numbers below come from the runs as the page is read.

## What the runs say

Where each run stands at the first infrared perimeter, 27 hours in, and at the end:

```esim
table 2025-01-08T21:25Z fire.burned_acres=burned obs.acres=mapped fit.structures_recall=destroyed_reached fit.structures_precision=reached_were_destroyed fit.undamaged_reached=undamaged_overrun fire.wind_m_s=wind_at_ignition
```

```esim
table 2025-01-12T00:00Z fire.burned_acres=burned_by_the_end obs.acres=last_mapped fit.structures_recall=destroyed_reached fit.structures_precision=reached_were_destroyed
```

Two things come out of the sweep, and both are about the engine rather than the fire.

The first is a cliff. With a far reach (heat felt three cells away) the fire either fills the
window in hours or, past a threshold, cannot light the cell next to it at all; there is no
setting in between that walks at the record's pace. With a near reach, heat felt only by
the next cell, the first day comes into range: at four times the threshold the engine has
burned about 4,100 acres at twelve hours where the triangle says 2,920, and about 15,300 at
27 hours where the infrared map says 15,800. Then it keeps going, because nothing in it
stops, and by the fourth day it has twice the record.

The second is the houses. The runs that spread at a plausible pace reach only a small share
of the structures that were in fact destroyed, because a built-up cell is given a high
threshold and the fire walks around the neighbourhoods rather than into them. The real
fire went into Pacific Palisades on the first evening and burned it house to house. Two
further knobs, embers that light houses and a longer reach from house to house, lift the
share reached from about an eighth to about a fifth at the same acreage, and no further.
The far-reach runs reach nearly every destroyed structure, but only because they burn
everything. Between the two is what a structure model has to supply: each house as a body
with its own exposure, not a cell with a share of roof in it.

The evening of the same day added the bodies. In the `bodies` runs every county footprint is a
structure of its own: it feels the burning ground within 35 m and its burning neighbours across
the gap between footprints, takes embers that land on it, ignites past its own threshold, and
burns as a source for the next house. Each is scored against the inspectors' word for that
footprint, and the three-dimensional view colours it accordingly, with a blue for the houses
the record says were destroyed and the model never reached. The result so far is a finding
rather than a fit: at every threshold the model lights destroyed and undamaged houses in the
same proportion, about one destroyed in two, which is the record's own base rate. The
survivors stand among the destroyed on the same streets, and nothing in the footprint, the
height, the gap or the slope tells them apart. What does is the house itself, its roof, vents,
eaves and age, and that is the data the next step has to bring in.

Later that evening Anselm noticed that the older model climbs the canyons and this one did not. It
was the formula: slope entered only the reach of a burning cell's heat, linearly, and the heat form
caps every directional effect at a few times the flat rate, so a 30° slope doubled the spread where
Rothermel's quadratic slope factor multiplies it by about seven. The `near-4-slope` runs put
Rothermel's shape on the rate itself (`slope_rate_gain`, uphill multiplied, downhill divided). At
Rothermel's own gain the fire runs up one canyon and stops on the ridge, since everything beyond is
downhill; at a gain of 6 both ways the fire climbs toward Topanga and along the ridges in fingers,
and its overlap with the mapped perimeter is the best of any run on either engine so far, though it
is half the record's area on the first day. The devlog entry *The slope* has the table.

Until the afternoon of 2026-11-03 these runs, like the first project's, were made on ground
with a column of missing elevation every third or fourth cell, which the slope code read as
cliffs. This engine felt it less than the spread formula does, because it stretches its
reach by the slope rather than its square, but the first day's pace was partly riding those
slopes. The runs on this page are on the corrected ground; the devlog entry *The holes in
the ground* has the account.

## What is honest about it, and what is not

What is real: the ground, the buildings and their heights, the roads, the inspections, the
mapped perimeters, and the wind, which at the ignition point reads 10 m/s with a 21 m/s
gust in the first hour where the 25 km reanalysis read 7 and 13. The reader that decodes
the weather was checked against GDAL at that point. What is not real: every rate and
threshold in the heat budget, the ember model, the idea that a cell's building burns as one
thing. Nothing here predicts anything yet. It is the frame into which the record can now be
fitted, one constant at a time, with the Eaton fire of the same day kept back as the test.

The three-dimensional view on each run page is the ground as the run saw it: the terrain
from the elevation model, every building as a box of its stated height, the fire's arrival
draped over both at the time cursor. Since the late evening of 2026-11-03 it also carries the
record of what people did, from the incident's own GIS: dozer pushes, drop points, helispots and
completed lines appear where and when they were made, pulse for a while, and carry their times,
beside the model's ignition and the spot fires it threw. The model does not yet react to any of
it; the lines are drawn, not defended. [Limits today](?doc=system/limits) has the list.
