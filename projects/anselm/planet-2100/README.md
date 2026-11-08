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
these runs model. A dashed one is a relationship the world has and the runs lack. Until
2026-11-07 the largest gap was the first arrow: the emissions after 2024 were an assumption,
named and held, not something the people in the model decided. Now there is a tenth actor,
the capacity to act, and the derailed variants below close that loop with a placeholder rule;
the clean variants still hold the plan whatever the heat does, and say so.

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
heat: days past the niche, air too humid to cool in, heatwaves replayed from a decade
capacity: the room to act, debt, conflict, consent
warming
people -> emissions: set them, as a plan
heat -> capacity: eats the room to act (derailed variants)
capacity -> emissions: slows the cut (derailed variants)
emissions -> air: most stays for centuries
emissions -> sinks: about half is taken up
sinks -> air: give back more as they warm and fill
air -> warming: traps the heat
warming -> thaw and dieback: passes their doors
thaw and dieback -> air: give up their carbon
warming -> reefs and currents: bleaches, slows
reefs and currents -> warming: a slower ocean shifts the heat (not modelled)
reefs and currents -> thaw and dieback: a slower Atlantic dries the Amazon (cascade variants)
ice sheets -> reefs and currents: meltwater weakens the overturning (cascade variants)
warming -> ice sheets: past their doors already
ice sheets -> warming: darker ground where ice was (not modelled)
ice sheets -> sea: slide into it over millennia (not modelled)
warming -> sea: swells the water
warming -> heat: more days past the limits, longer heatwaves
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

## Heatwaves, replayed

Until the evening of 2026-11-07 nothing on this page could be hot on a Tuesday. The planet
is an energy balance, one number for the whole Earth, and the people layers shift a
climatology by it; a heatwave is a run of days, and there were no days. Now there are,
of a kind. For every populated one-degree cell the model holds a decade of observed daily
maximum temperatures (2011 to 2020, from the MERRA-2 reanalysis through NASA POWER). Each
simulated year it draws one of those ten years by the run's seed and replays it, shifted
by the cell's warming since that decade, and counts heatwaves the way Aglas-Leitner and
colleagues count them in *Recent extreme heatwaves are no match for future events*
(Nature Communications, 5 November 2026, read on the 7th): three or more days over the
calendar-day 90th percentile, a severity in units of the threshold's own excess over the
climatology, the days, the cumulative heat, and the share of a country's people under
extreme heat on the worst day. A country's heatwave is judged on the mean over its people.

```esim
table 2100 heat.people_days=person_days_of_heatwave heat.people_month_long=people_in_a_month_long_heatwave heat.countries_month_long=countries_with_a_month_long_heatwave heat.longest_days=longest_heatwave_anywhere_days
```

What the paper finds, by global warming level, is the ladder this layer should be read
against. At 2 °C the chance that a projected heatwave beats the worst severity of Europe
2018 or of Northern Africa and the Middle East 2023 is about one in five; between 2.5 and
3 °C those chances take their steepest step; past 3 °C most regions beat their worst
observed event most of the time, and in 2060 to 2099 the model ensembles expect one or
more such heatwaves a year in nearly every region whatever the emissions. Australia's
2019 event is the exception, rare even then.

Read this layer as a floor, and for three reasons it states itself. The replayed decade
is the base, so "exceptional" here means exceptional for 2011 to 2020, a decade already a
degree warmer than the paper's 1961 to 1990. The weather of one year is moved whole, so
the spread of the days stays as it was while the mean climbs, and the paper's own result
is that heatwaves lengthen and widen faster than the mean alone would make them; a knob
exists to widen the days and is set to zero because no paper gives the number. And a year
is drawn independently of the last, which is not the climate's memory. The one thing the
layer does that no table could: by the 2050s on the held path some cells are above their
threshold every day of the year, which is the model saying that a tropical summer as it
was in the 2010s has become the whole year, and the paper's six-month maximum is the same
finding with a ceiling on it. The crossings below list, for each country, the first year
a month-long heatwave covered more than half its people, and the first season-long one.

## The world, scored

Anselm, 2026-11-05: the era of cheap externalities is ending, regions differ in what they have
left, in their ecosystems, air, water, wealth, fairness and institutions, and all of that
together is a total bottom line a person should be able to read at a glance, for every region,
on a globe, and watch move. The run pages now carry that globe, at the top. Every country is drawn by its own outline
three times, one ring per group of bands, human, nature and economy, each a pastel, wider and
brighter the higher the group's health; a country fills with red by the worst of its pressures, never
by its mean: the share of its people living past the heat thresholds that year, the weakest
of its three groups under a floor, or a fall under its own start. A rich country with a heart
attack is not averaged well. The nine bands are a click away, as a pole or a block. The bands are nine World Bank
indicators, each the latest value the Bank holds for the country: life expectancy, income per
person, the Gini index, the governance estimate of voice and accountability, fine-particle air
pollution, forest cover, protected land and sea, renewable fresh water per person, and CO₂ per
person. Each is turned into a score from 0 to 1 by its rank among the countries that report it.
So a band says where a country sits among its peers and never what a good number is, and a
country missing a band is not penalised. Norway stands near the top at about three quarters,
Canada and Brazil near seven tenths, the United States at six tenths with a low fairness band
and the lowest footprint band of all, Bangladesh and Nigeria near four tenths. Those are the
record's numbers, ranked, with nothing of ours in them.

What moves the glyphs after the run's first year is ours, and a placeholder: each January
the module reads the share of each country's people living in cells past the climate niche and
past the 31 °C wet-bulb limit, the run's own stress, and takes part of the living, means and
water bands in proportion; a wild card knocks one band for the world for some years by its
kind, a pandemic the living, a crash the means, a war the voice, an eruption the air. By 2100
with emissions held, 49 countries have a tenth or more of their people past a threshold: the
Gulf, the Sahel, the Caribbean islands. Bangladesh is not among them, which is a limit of the
model and not a finding: its annual mean sits under the niche's 29 °C, and the reanalysis
under the wet-bulb count reads humid heat mild. The rules are written in the module's parameters so they can be argued with. Drag the
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
table 2100 people.world_total=people energy.gmst_anom_c=warming shock.count=wild_cards people.shock_deaths=taken_by_wild_cards_and_conflicts
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

## The loop closes: derailment

Everything above runs one way. The emissions set the warming, the warming sets the heat, the
heat is counted, and nothing the heat does reaches back to the emissions: a path to zero by
2050 gets there whatever happens to the people on it. James Dyke, writing in The Conversation
on 5 November 2026, calls the missing loop derailment risk: the consequences of climate change
erode the capacity to address its causes. Storm damage and the debt it leaves crowd mitigation
out of a poor country's budget; conflict breaks the cooperation that cutting emissions needs,
more where inequality is high; a flood is followed by a backlash, Valencia in 2024 being his
case; and collective action is easier to destroy than to build. Behind it the tipping points
cascade, an Atlantic shutdown starving the Amazon, the ice sheets speeding each other.

Since 2026-11-07 the project carries that loop as a placeholder with the article's shape and
guessed numbers, declared in the module's parameters. A capacity module reads what the run
already knows each year, the share of the world's people under heat stress and the voice and
fairness bands weighted by people, and keeps a capacity to cut between one and zero: the
target falls with the stress, more where fairness is low; the capacity falls fast toward a
lower target and recovers slowly toward a higher one, the recovery scaled by voice, which is
the article's asymmetry; a wild card knocks it for some years by its kind; and where a
country's people are under heat stress a conflict may be drawn by the run's seed, which takes
one percent of that country's people and knocks the capacity by the country's share of the
world. In the derailed variants the capacity goes to the emissions module, which multiplies
each year's planned cut by it. A lost year is never made up, and when the plan has reached
zero the cutting goes on at the plan's last pace, still slowed, until the rate is zero. So a
plan is a plan, and what is achieved is what the heat and the shocks left of it. The same
module keeps the options: the carbon budget left to 2 and 3 °C under the assessment's response
to cumulative emissions and the cut rate now needed to hold each, so a chart can show what must
be done rising while what can be done falls.

```esim
table 2100 energy.gmst_anom_c=warming carbon.emissions_gtc_yr=emissions_in_2099 capacity.world=capacity_to_cut capacity.conflicts=conflicts_drawn tipping.elements_past_central=past_the_central_estimate carbon.tipped_cumulative_gtc=carbon_released_by_tipped_elements
```

What the loop does, with these numbers, is small on the zero path and large where the cut is
slow. Zero by 2050, derailed, reaches zero in 2056 and ends the century 0.05 °C warmer. The
3% a year path, the one Anderson and Bows examined, is derailed harder: the capacity slows a
cut that never ends, so by 2099 the world is still emitting 3.6 billion tonnes of carbon a
year against the plan's 1.2, thirty-one conflicts have been drawn in the Gulf and the Sahel,
and the century ends 0.18 °C warmer than the plan. On the held path, where there is no cut
to slow, the capacity itself is the finding: it falls to a fifth by 2100 as a sixth of
everyone comes to live past the heat thresholds. The world that most needs to cut is the
one the model leaves least able to, which is the article's claim as a number, and the
number is a placeholder.

Two more things the loop makes visible. The 2 °C budget, measured from the model's own
warming, which runs 0.2 °C ahead of the record, is spent by 2040 on every path, including
zero by 2050; the chart of the cut now needed climbs to a hundred percent and stays there,
and the 3 °C budget is the one still open. And the tipping elements now push on each other
in the cascade variants: Wunderling and colleagues found in 2021 that in a coupled model of
four elements the ice sheets lower the Atlantic circulation's threshold by about 55%, the
circulation and Greenland lower West Antarctica's by about 40%, and the circulation lowers the
Amazon's by about 10%, at the full strength of the interactions they considered. Carried into
the assessment's thresholds as fractions, that brings the Atlantic circulation's central
estimate from 4.0 to 1.8 °C, which every path passes in 2029, and the Amazon's from 3.5 to
3.15 °C, which the held path passes in 2090, so the Amazon begins to release its 30 billion
tonnes of carbon ten years before the century ends. The paper's own words are that its model
makes no predictions and its cascades are committed over long timescales; here a lowered
threshold is passed the year the warming reaches it, and the ladder above still draws the
unshifted rungs, so a cascade run's crossings sit below them.

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
  where nobody moves. Deaths, harvests, water and migration are not in these runs, and the
  one response by anyone is the derailed variants' capacity rule, a placeholder. This is the largest thing the page gets wrong, and it is wrong in a
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
