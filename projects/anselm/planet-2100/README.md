# The road to 2100, and the thresholds on it

This is the project's own account of its runs, written for a reader who is not a
scientist. Every number on this page is drawn from the runs listed below it as the page
is read; if the runs change, the page changes with them. Nothing here is a forecast. Each
line is one assumption about emissions after 2024, named, run through the same model.

## The actors

What decides where the planet is in 2100, drawn as the assessments draw it. The IPCC's 2021
summary for policymakers opens with the sentence the whole field now stands on: "It is
unequivocal that human influence has warmed the atmosphere, ocean and land." Its chain runs
from emissions to the air to the heat: each 1,000 billion tonnes of CO₂ emitted warms the
surface by a best estimate of 0.45 °C, likely between 0.27 and 0.63, and the land and the ocean
have taken up "a near-constant proportion (globally about 56% per year)" of what people emit
over the past six decades. The Global Carbon Budget counts that every year: in 2023 about
11 billion tonnes of carbon went up, 5.9 stayed in the air, 2.9 went into the ocean and 2.3
into the land (Friedlingstein and others, 2024). Behind the emissions stands Kaya's identity,
the frame the IPCC's scenarios are built on: emissions are how many people there are, how
rich, how much energy each unit of wealth takes, and how much carbon each unit of energy
releases. Ahead of the warming stand the thresholds, the sixteen elements of Armstrong McKay
and colleagues' 2022 assessment, each with the warming at which it likely tips; the figure
groups them by what they do when they go: thaw and dieback give up carbon, reefs and currents
change the ocean, ice sheets raise the sea. So the human model has nine actors around the
warming, and each knock-on has its own arrow.

This is the map people agree on, and it is not the simulation. A solid arrow is a relationship
these runs model. A dashed one is a relationship the world has and the runs lack, and the
largest is the first: here the emissions after 2024 are an assumption, named and held, not
something the people in the model decide.

```esim
actors
people: how many, how rich, how much energy, from what
emissions: fossil fuel and cleared land
sinks: the ocean and the land
air: CO₂ for centuries, methane for a decade
thaw and dieback: permafrost, the Amazon
reefs and currents: coral, the overturning circulation
ice sheets: Greenland, West Antarctica
sea: the water, the ice
heat: days past the niche, air too humid to cool in
warming
people -> emissions: set them (not derived here)
emissions -> air: most stays for centuries
emissions -> sinks: about half is taken up
sinks -> air: give back more as they warm and fill
air -> warming: traps the heat
warming -> thaw and dieback: passes their doors
thaw and dieback -> air: give up their carbon
warming -> reefs and currents: bleaches, slows
reefs and currents -> warming: a slower ocean shifts the heat (not modelled)
warming -> ice sheets: past their doors already
ice sheets -> warming: darker ground where ice was (not modelled)
ice sheets -> sea: slide into it over millennia (not modelled)
warming -> sea: swells the water
warming -> heat: more days past the limits
heat -> people: counted (not moved, not killed)
sea -> people: takes the coast (not modelled)
```

## What a tipping point is

Most of what warming does is gradual: a little hotter each decade, the sea a little
higher. A tipping point is different. It is a threshold past which a part of the planet
starts to change by itself and does not stop when the warming stops. An ice sheet that
has begun to slide keeps sliding. A forest that has dried past a certain point keeps
turning into grassland. The 2022 assessment this project uses (Armstrong McKay and
colleagues, in *Science*) lists sixteen such elements and gives each one the global warming
at which it is likely to tip, a range around that figure, and how long the change takes
once it has begun. The threshold is a door. The timescale is how long the room takes to
empty once the door has closed: for an ice sheet, thousands of years; for a coral reef,
about ten.

Two things to hold on to. The thresholds are estimates with ranges, and the assessment
calls a crossing *possible* from the bottom of the range and *likely* from the central
figure; this page uses the words the same way. And passing a threshold before 2100 does
not mean seeing the consequence before 2100. Greenland's door is behind us in every run
below, and the sea level it commits arrives over about ten thousand years.

## Where the paths go

```esim
ladder
```

The model runs about 0.2 °C warm today (1.68 °C above 1850 in its 2024, against 1.46 °C
in the HadCRUT5 record), so it passes 1.5 °C in 2018 where the record first does in 2024.
Read every year on this page as five or six years early.

What the ladder shows:

- Five elements are past their central estimate in the model's 2024: the Greenland and
  West Antarctic ice sheets, the low-latitude coral reefs, the abrupt thaw of permafrost
  (all at 1.5 °C) and the Barents Sea's winter ice (1.6 °C). The record reached 1.5 °C for
  the first time in 2024. In the assessment's words these are now likely, not certain,
  and the ice sheets' consequences play out over millennia.
- To 2040 the futures have not separated. Every path, including emissions to zero by
  2050, passes the Labrador Sea (1.8 °C) by 2029 and the mountain glaciers (2.0 °C) by
  2040. The first decades are already in the system.
- After 2040 they part. Emissions to zero by 2050 levels off at 2.1 °C and all but stops
  climbing (0.01 °C a decade by the 2090s). Emissions held at today's rate are still
  climbing 0.2 °C a decade in the 2090s; emissions growing 1% a year, 0.35 °C a decade.
- With emissions held, the West African monsoon's threshold (2.8 °C) is passed in 2073
  and the East Antarctic basins' (3.0 °C) in 2083, and the century ends 0.14 °C short of
  the Amazon's (3.5 °C). The trend path passes the Amazon's in 2093 and the growing path in
  2087. No path reaches the 4.0 °C rung, where the Atlantic overturning circulation, the
  boreal forest and the deep permafrost sit together; the growing path ends 0.04 °C below
  it, still climbing.
- *Possible* is a wider story. By 2040 every path is inside the possible range of
  thirteen of the sixteen elements, and by 2100 the held path is inside fourteen. Only the
  Arctic's winter ice and the East Antarctic ice sheet, whose ranges begin at 4.5 and
  5.0 °C, stay out of reach on every path.

Where the paths stand when the runs end:

```esim
table 2100 energy.gmst_anom_c=warming carbon.co2_ppm=CO₂ tipping.elements_past_central=past_the_central_estimate tipping.elements_past_low=inside_the_possible_range people.outside_niche=people_outside_the_niche wetbulb.people_over_a=people_where_31_°C_wet-bulb_is_passed carbon.tipped_cumulative_gtc=carbon_released_by_tipped_elements sea.thermal_m=sea_level_from_expansion
```

## What each half degree costs

The tipping elements are the planet's thresholds. People have two of their own in these
runs: the human climate niche, the band of annual mean temperature that nearly everyone
has always lived inside, whose upper edge is 29 °C; and a wet-bulb temperature of 31 °C,
the measured limit of what a healthy body can shed as heat on a humid afternoon. The
runs count the people living where each is passed. Along the fastest-warming path, with
the UN's population growing from 8.2 to 10.2 billion beneath it:

| warming | year reached | people outside the niche | people where 31 °C wet-bulb is passed on three afternoons a year |
|---|---|---|---|
| 1.7 °C | 2024 | 147 million (1.8%) | 6.5 million |
| 2.0 °C | 2036 | 259 million (2.9%) | 11 million |
| 2.5 °C | 2054 | 552 million (5.6%) | 20 million |
| 3.0 °C | 2071 | 952 million (9.3%) | 35 million |
| 3.5 °C | 2086 | 1.38 billion (13.4%) | 204 million |
| 3.9 °C | 2098 | 1.76 billion (17.3%) | 470 million |

The niche count climbs smoothly and ever faster: the first half degree after today costs
about 110 million people, the last about 400 million, part of it population growth and
most of it the heat. The wet-bulb count has a step in it. Between 3.0 and 3.5 °C it goes
from 35 million to 204 million, and the step has a place: in the 2070s and 2080s the
humid plains of eastern China cross 31 °C on three afternoons a year, cell after cell,
with Japan's coast and Taiwan following. A region moves from almost never to every summer
within about fifteen years. That is a transition in the human layer of the same shape as
the planet's, and it is the one thing on this page that the runs found rather than read
from a table.

One absence to name. The Indus and Ganges plains, which station records put first in
line for humid heat, barely cross in these runs. The baseline here is a reanalysis that
reads humid extremes two to four degrees mild against the stations, and India's warming
is scaled from its own record, which has run cool. The China step is probably real and
probably not the first; see the limits.

## The world, scored

Anselm, 2026-11-05: the era of cheap externalities is ending, regions differ in what they have
left, in their ecosystems, air, water, wealth, fairness and institutions, and all of that
together is a total bottom line a person should be able to read at a glance, for every region,
on a globe, and watch move. The run pages now carry that globe. Every country is a glyph at its
centroid, nine bands stacked, the total its height. The bands are nine World Bank
indicators, each the latest value the Bank holds for the country: life expectancy, income per
person, the Gini index, the governance estimate of voice and accountability, fine-particle air
pollution, forest cover, protected land and sea, renewable fresh water per person, and CO₂ per
person. Each is turned into a score from 0 to 1 by its rank among the countries that report it.
So a band says where a country sits among its peers and never what a good number is, and a
country missing a band is not penalised. Norway stands near the top at about three quarters,
Canada and Brazil near seven tenths, the United States at six tenths with a low fairness band
and the lowest footprint band of all, Bangladesh and Nigeria near four tenths. Those are the
record's numbers, ranked, with nothing of ours in them.

What moves the glyphs after the run's first year is ours, and a placeholder: a country whose
people pass the climate niche loses a quarter of its living and water bands, one that passes
the wet-bulb limits a quarter more of living, and a wild card knocks one band for the world for
some years by its kind, a pandemic the living, a crash the means, a war the voice, an eruption
the air. The rules are written in the module's parameters so they can be argued with. Drag the
globe to turn it and hover a glyph for its bands; the mean score over countries is charted
beside it. The knock-backs are small on this scale because the rules are mild and global; a
reactor leak that takes one region's air and water for a decade is the kind of card that would
show, and the deck does not hold it yet.

## Wild cards

Nothing above happens by surprise. Emissions follow their assumption, the sinks their
equations, the thresholds their table, and the population the UN's path. The world will not
be like that: a pandemic came in 2020, a financial crisis in 2008, an eruption that cooled
the planet for two years in 1991, and something of each kind is likely before 2100. So the
project carries a deck of wild cards, drawn by the run's seed. The deck is
`data/events/future-speculative.toml`: twenty-two things that could happen, each with the
rough annual chance commonly quoted for it and an effect in the model's terms, a pulse on the
energy balance for some years, a scaling of emissions for some years, a fraction of everyone.
Every chance and every effect in it was written from memory and is marked so; the cards are
placeholders, not forecasts, and a card may come up twice. The runs below draw the deck each
year from 2025 with three different seeds, draw it four times as often, or play one named
card, a Tambora-class eruption in 2040, on top of emissions held at the 2024 rate.

```esim
table 2100 people.world_total=people energy.gmst_anom_c=warming shock.count=wild_cards people.shock_deaths=taken_by_wild_cards
```

What the deck does to this model is small in the total and visible in the story. A seed
draws about a dozen to twenty cards in seventy-five years, most of them the frequent cheap
ones (a financial crisis every decade or so, a cholera decade, an eruption of 1991's size),
and the century ends a hundredth of a degree cooler and a couple of hundred million people
short of the UN's path. Four times the draw takes half a billion and a tenth of a degree. The eruption shows as a
two-year dip on the warming chart and nothing at 2100. That smallness is the finding: with
emissions held, the slow things decide the century and the shocks are weather on top of it,
unless a shock is of the kind that changes the emissions path for good, which no card in this
deck yet does. The run page lists every card drawn, with the chance it was drawn at and what
the run did with it, in the story beside the thresholds crossed.

## What the model does with a crossing

The ladder is a measuring stick held against the runs. For thirteen of the sixteen
elements the runs do not feel the rungs: passing one writes a sentence with a citation
and changes nothing else. Three elements carry carbon in the assessment, and those feed
back: the abrupt thaw of permafrost (per degree of warming), the Amazon's partial dieback
(30 billion tonnes of carbon) and the deep permafrost's collapse (125 billion tonnes)
release their carbon into the same ledger as fossil fuel, over the assessment's
timescales, once their threshold is passed. With emissions held that comes to about
5 ppm and 0.02 °C by 2100; with every element tipping from the low end of its range (the
held-low variant) it is 69 billion tonnes and 0.07 °C. Small, because the big stores
release over centuries. What is not in the runs: ice into the sea, the overturning
circulation slowing, the darker ground where forest or sea ice used to be. Each would add
to the warming or the sea, and none is modelled yet.

## What this page cannot tell you

- The years are the model's, not the world's, and the model runs a few years ahead of the
  record. The thresholds are an assessment's estimates with wide ranges; the ladder draws
  the central figure and the possible zone beneath it, and leaves the top of each range
  to the table.
- The people counts are exposure, not deaths: people living where a stated limit is
  passed, on a settlement pattern held at 2020 and the UN's population path, in a world
  where nobody moves. Deaths, harvests, water, migration and any response by anyone are
  not in these runs. This is the largest thing the page gets wrong, and it is wrong in a
  known direction: the model's population climbs smoothly to 2100 everywhere, where a
  country like Bangladesh, low, crowded and in the path of both the sea and the humid
  heat, would in any granular reading see people leave in numbers and people die. Impact,
  and the non-linear turns that make it, is the layer this project does not yet have, and
  it is the next one (Anselm, 2026-11-04).
- The warming at any place is its country's, from the country's own record scaled by the
  run's global warming; the reanalysis beneath the wet-bulb count reads humid heat mild.
  Both shortfalls pull the people counts down, not up.

The run pages beneath carry every number's source, and
[Limits today](/docs/system/limits) says what is known to be wrong.
