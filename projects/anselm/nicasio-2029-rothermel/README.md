# The same fire, on the older model

[A fire at Devil's Gulch, October 2029](/anselm/nicasio-2029) sets a fire that has not happened
in Samuel P. Taylor State Park on our own engine and asks whether it reaches a house six
kilometres east. This project asks the same question of Rothermel's 1972 surface spread model,
the formula the first [Palisades](/anselm/palisades-2025) project ran, on the same ground, the
same ignition, the same borrowed days. Anselm asked for it as a separate simulation on 2026-11-08.
With no record to score either engine against, two engines on one scenario are the nearest thing
to a check: where they agree the agreement is about the ground and the wind, and where they
disagree the disagreement is about the engines.

## What the scenario is made of

The fire on this page is one module among several. The relief and the land cover are each read
once from their own source by their own module. The weather module lays each hour's wind,
temperature and humidity on the ground, and a fuel moisture module turns that air into how dry
the dead fuel is. Rothermel's model reads all of them and fetches nothing itself. Beside it
stand a population of animals that runs from the front, a firefighting module that is switched
off, and the two named places the run watches. None of these parts knows the others: each writes maps that it owns, and
the others read them.

```esim
wiring
```

This is the point of the pair. The [first Devil's Gulch page](/anselm/nicasio-2029) is the
same relief, land cover, fuel moisture and animals modules with our own fire model in this
one's place and a finer weather record. After this fire was separated from its ground and
weather on 26 November 2026, the burned area of all four runs was the same at every hour as
before.

The animals are a placeholder, with no species and guessed senses and speed, and no number of
animals killed or displaced on these pages is an estimate.
[What this is](/docs/system/overview) has the argument and its limits.

## What is different

The fuel is a fuel model per cell from the land cover, through the Scott and Burgan models
(grass GR1 and GR2, shrub SH5 or SH7, the timber-understorey model TU1 for the two thirds of the
window WorldCover calls trees, and a guessed rate for built-up land); every parameter in that
table is recalled from the 2005 paper and marked as unchecked. The wind is the ERA5 reanalysis at
one 25 km point at the window's centre, not the 3 km analysis the other engine reads: the same
days, 27 to 29 October 2019, played from midnight. At that point the reanalysis reads 3 to 8
metres a second from the north through the first day, gusts to 16, and 23 percent humidity at
the driest hour. Fire moves cell to cell down an ellipse headed along the wind and slope. No
spotting, no crown fire, nobody fights it. Spread through the built-up land is a guess the
manifest can set to zero.

## What the runs say

The four variants are the first Palisades project's, unchanged: the reanalysis wind as read, its
gusts as the wind, built-up land as a barrier, and shrubland as the heavier SH7 model. Where each
stands at noon on the first day, at midnight, and at the end of the third day:

```esim
table 2029-10-01T19:00Z fire.burned_acres=burned place.550_nicasio_valley_road.distance_m=metres_from_the_house fire.wind_m_s=wind
```

```esim
table 2029-10-02T07:00Z fire.burned_acres=burned place.550_nicasio_valley_road.distance_m=metres_from_the_house
```

```esim
table 2029-10-04T07:00Z fire.burned_acres=burned_by_the_end place.550_nicasio_valley_road.distance_m=metres_from_the_house place.550_nicasio_valley_road.reached=house_reached
```

Rothermel's fire never reaches the house. On the mean wind it burns about 16,000 acres, almost
all of it in the first two days while the borrowed wind blows, and stops two kilometres short
when the wind drops; on the gusts it burns 23,000 and stops at thirteen hundred metres. The
shrub model and the built-up barrier change nothing here, because the window has almost no
shrub and the fire does not reach the built-up land that would be a barrier. Compare the other
engine on the same ground: at its Palisades setting it takes the whole window, 46,000 acres,
in the same two days and is at the house in thirty hours; at its forest setting it never
leaves the first cell. Rothermel sits between those, which is not evidence that it is right,
only that the formula with its recalled fuel loads is less sensitive to a single knob than an
engine whose thresholds were fitted once.

Three things both engines say. The fire runs west and south first, with the wind, and the house
to the east is behind the fire for the whole first day. The borrowed wind at the valley floor is
light, whichever source reads it, and both fires slow when it drops on the second day. And with
nothing to fight it, the quiet of the second night is the fire's own: no line was cut.

## What is wrong with it

- Every fuel load, depth and moisture of extinction in the fuel table is recalled from Scott and
  Burgan 2005 and not yet checked against the paper. TU1 for a coast redwood and Douglas fir
  canyon is a stand-in.
- The wind is one 25 km point for the whole window, from another year, and the reanalysis reads
  these days lighter than the forecast office's warning; it read the Palisades' Santa Ana the same
  way.
- No spotting, no crown fire, no suppression, and no record, so no score.
