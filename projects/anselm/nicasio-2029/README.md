# A fire that has not happened

On 1 October 2029, at three in the morning, something lights the ground at the Devil's Gulch
campsite in Samuel P. Taylor State Park, Marin County, California. Six kilometres east, across
the ridge and down in the Nicasio Valley, is a house at 550 Nicasio Valley Road, which Anselm
describes as the home of Jim Fournier's Planetwork and Solstice Grove. He asked for this page on
2026-11-08: "a brand new simulation of 550 Nicasio Valley Rd ... lets pretend there is a fire
starting not far away at Devil's Gulch Campsite ... lets say oct 1st ... lets put this in 2029.
we probably need some wind conditions."

Nothing on this page is a forecast. It is the engine built for the [Palisades
fire](/anselm/palisades-cells) and carried to [Fort McMurray](/anselm/fortmcmurray-2016), set
down on ground it has never been scored on, in a year that has no weather, and asked what it does.
Read it as a statement about the engine as much as about the valley.

## What was borrowed, and from whom

A year in the future has no wind. The run borrows a real day: NOAA's 3 km analysis for 27 to 29
October 2019, played hour for hour so that midnight Pacific time falls on midnight. Those were the
days the National Weather Service's Bay Area office held a Red Flag Warning for the coastal North
Bay, Point Reyes and the North Bay mountains, from the evening of 26 October to the morning of
the 28th (the warning is in the Iowa Environmental Mesonet's archive of such warnings). At the
campsite, on the floor of a wooded canyon, the analysis puts a light wind over the ignition: one
to two metres a second in the night, eight from the north-east through the first afternoon, with
gusts to thirteen. The ridges had more; a 3 km cell does not see a ridge. The air it carries is
dry: the dead fuel moisture the engine derives from the hour's temperature and humidity sits
between three and five percent for most of the first two days.

The ground is today's. The elevation is the Copernicus model, the cover is the European Space
Agency's 2021 map, which calls two thirds of the window tree cover and most of the rest grass,
and the 2,429 buildings and the major roads are OpenStreetMap's. The campsite is where
OpenStreetMap puts it. The house is where the US Census Bureau's geocoder interpolates the street
number along the road, which is to say on the road's centreline; the parcel is not checked, and no
public page we found ties the address to the names Anselm gave. The project's event folder
(`data/events/nicasio-2029/README.md`) lists every source and the day it was read.

Nobody fights the fire. There is no evacuation, no engine at any house, no aircraft, and no
record to score the run against, because there is nothing to score it against.

## What the runs say

The engine's constants are the Palisades guesses, declared on every run page. One number, a
multiplier on every ignition threshold, was fitted to the Palisades' first infrared perimeter at
five and found to need eight at Fort McMurray. Here it is run at three, five and eight, and at five
on the hour's gust instead of its mean wind. Where each run stands twelve hours after midnight,
twenty-four hours after, and at the end of the third day:

```esim
table 2029-10-01T19:00Z fire.burned_acres=burned place.550_nicasio_valley_road.distance_m=metres_from_the_house fire.wind_m_s=wind
```

```esim
table 2029-10-02T07:00Z fire.burned_acres=burned place.550_nicasio_valley_road.distance_m=metres_from_the_house place.550_nicasio_valley_road.reached=house_reached
```

```esim
table 2029-10-04T07:00Z fire.burned_acres=burned_by_the_end place.550_nicasio_valley_road.reached=house_reached fire.built_cells_burned=built_cells_burned
```

The engine does not know. At thresholds times eight, the forest setting, the fire never leaves
its first cell: one cell cannot light the next in this fuel under this wind. At five, the Palisades
setting, it takes the whole window, 46,000 acres, in two days, and the fire is at the house thirty
hours after midnight, at six in the morning on 2 October. At three it is there in nineteen hours.
On the gust it is slower to the house, thirty-four hours, and faster everywhere else; why is not
examined here. Between a fire that goes out and a fire that burns every cell in the
window, the only thing the runs disagree about is the hour. That is a true statement about the
engine. It has a fuel called "tree" that is one thing from a redwood canyon to a spruce stand, no
measure of how much of it there is, and no scoring here to tell three from five.

Two things about the valley stand regardless of the knob. The run's wind is from the north-east
through the first day, which carries fire from the gulch south-west, toward Lagunitas and the
Sir Francis Drake corridor, not east; the house is reached when the fire has filled the ground
between, not when the wind brings it. And the road itself is the only break the engine knows
there. A real October day with the ridges in a Diablo wind, a real suppression effort and the real
state of the fuel in 2029 would each move the hour by more than the runs disagree.

## The view

The run pages open on the three-dimensional view: the ground from the elevation model with the
vertical stretched twice, every OpenStreetMap building a box of its stated or guessed height, the
fire's arrival draped over both at the time cursor, the two places staked and labelled, and the
hour's wind as an arrow. The sun follows the clock, so the night runs are dark; drag the cursor
into the afternoon to see the ground.

## What is wrong with it

- The wind is another day's, from another year, and at 3 km it does not see the ridges that make
  a Marin fire day what it is. A ridge wind and terrain-following flow are not modelled.
- The pace is unscored. The one fitted number was fitted on chaparral under a Santa Ana and refitted
  on boreal spruce; neither is a coast redwood and Douglas fir canyon with an oak and grass
  valley beside it. Live fuel moisture is the Palisades' January guess.
- The house is a point on a road centreline. It counts as reached when a burned cell lies within
  fifty metres of that point; the buildings on the parcel are whatever OpenStreetMap had on the
  day, and nothing says what stands there in 2029.
- Nobody fights the fire, nobody leaves, and the record that would tell us what any of this is
  worth does not exist.
