# Portland in the year after a Cascadia earthquake

On 12 November 2026 the Portland City Council voted to require the fuel terminals on the
Willamette River to cut their storage capacity by at least a fifth. The terminals stand on ground that is
expected to liquefy in a great earthquake, and a 2022 study for the county and the city estimated
that such an earthquake would spill between 95 and 194 million gallons from their tanks. More than
90 percent of Oregon's liquid fuel passes through the same terminals, so the tanks also hold the
fuel that emergency crews and hospital generators would need in the months afterwards. Storing
less fuel means a smaller spill and a smaller reserve.

This project is a simulation built to look at both sides of that trade-off in one model. It takes
a magnitude 9 earthquake on the Cascadia subduction zone, sets it at 2 p.m. on Friday 26 March
2027, and follows Portland for a year: which buildings, fuel tanks, bridges, power structures and
roads are damaged, and then, day by day, who has electricity, which hospitals have power, how much
fuel is left and how many people die. The date is a scenario choice and not a prediction. Nobody
knows when the fault will next rupture.

The model is assembled from published studies of this earthquake, and it is early work. It does
put numbers on both sides of the storage argument. In its base run the terminals spill 60.6
million gallons and keep 172.9 million. With a fifth less fuel held they spill less and keep
138.3 million. With every tank retrofitted they spill a small fraction as much and keep 228.7
million. But several of the model's results disagree with the studies they can be checked
against, and many of its parameters are recalled from memory or guessed. The spill is smaller than
the 2022 study's minimum, electricity returns months sooner than the state expects, and the
model's two ways of counting the dead disagree. The sections below give the results, the reasons
for doubting each one, and what the model leaves out.

## Why this scenario

The Cascadia subduction zone is a fault off the coast of the Pacific Northwest. Its last great
earthquake was in January 1700, and the date is known from Japan. A tsunami reached the Japanese
coast that month with no earthquake felt beforehand, and villagers, merchants and samurai recorded
the flooding. In the mid-1990s researchers linked that tsunami to an earthquake on the northwest
coast of North America (US Geological Survey Professional Paper 1707). The Survey's catalogue
lists the earthquake at magnitude 9.0 on the evening of 26 January 1700, Pacific time.

Layers of sediment on the sea floor record earlier ruptures. They show 19 or 20 ruptures of the
whole fault, or nearly all of it, in the past 10,000 years, which is one every 500 to 530 years on
average (Goldfinger and others, 2012). In March 2027 the last one will be 327 years in the past.
Multnomah County's page on the fuel terminals gives a 26 percent chance of a major seismic event
in the next 50 years. The page does not say where that estimate comes from.

Most of Portland was built before any of this was known. When Oregon adopted a statewide building
code in 1974, it placed Portland in seismic Zone 2. The code moved the city up to Zone 2B in 1990
and to Zone 3 in 1993, and a higher zone requires stronger construction. The city's own history of
these changes says scientists did not fully understand the risk until the 1990s. Ten of the twelve
bridges over the Willamette River in this model were built between 1908 and 1973. At the fuel
terminals, 91 percent of the active tanks were built before 1993 or have no recorded year. The
city counts more than 1,600 known or suspected unreinforced masonry buildings. So the hazard is as
old as the fault, and the risk comes from what was built above it before anyone knew what the
fault could do.

### What the state expects

The Oregon Resilience Plan is a 2013 report to the legislature by the Oregon Seismic Safety Policy
Advisory Commission. It estimated how long services in the Willamette Valley, which includes
Portland, would take to recover from this earthquake if it struck under the conditions of the
time.

| Service | Estimated time to restore |
|---|---|
| Electricity | 1 to 3 months |
| Police and fire stations | 2 to 4 months |
| Drinking water and sewer | 1 month to 1 year |
| Top-priority highways, partly restored | 6 to 12 months |
| Healthcare facilities | 18 months |

The plan describes these as estimates with "a lot of uncertainty and little detail".

In 2018 the Oregon Department of Geology and Mineral Industries, known as DOGAMI, published a
study of the earthquake's effect on the three counties around Portland (Open-File Report O-18-02).
The study took simulated motion of the bedrock and worked out how the soils above would shake.
It mapped where the ground would liquefy, which is what loose, saturated soil does when shaking
makes it behave like a liquid, and where slopes would slide. It then ran the region's buildings
through Hazus, the loss-estimation software of the Federal Emergency Management Agency. It did
this for four cases: day or night, and dry or saturated soil.

The worst of the four is a daytime earthquake on saturated soil. For that case the study estimates
1,473 deaths in the three counties and 85,211 people displaced for the long term. Its summary for
the City of Portland gives 896 deaths, 3,654 injuries needing a hospital and 30,627 people needing
shelter. This project uses the same case as its base scenario, and DOGAMI's study supplies both
the ground shaking the model starts from and the casualty figures it is compared with.

### The fuel terminals

The Critical Energy Infrastructure Hub is a six-mile strip of fuel terminals on the west bank of
the Willamette in northwest Portland. This page calls it the hub. According to Multnomah County,
more than 90 percent of Oregon's liquid fuel passes through it. It is supplied by the Olympic
Pipeline from Puget Sound and, for about 30 percent of its fuel, by tanker barge.

In 2022 the county and the city published a seismic risk analysis of the hub. Its tanks could hold
about 333 million gallons and held about 234 million, so they were about two thirds full. The
study estimated that this earthquake would release between 95 and 194 million gallons, 57 percent
onto the ground and 43 percent toward the river. The county's summary of the study says the average
tank with a recorded date was built in 1954. It puts the costs that could be given a dollar value
at up to 2.6 billion dollars, and the full cost at many multiples of that.

Ordinance 2026-312, which the council passed on 12 November 2026, requires each bulk fuel terminal
to cut its in-service storage capacity by at least 20 percent by 1 December 2036. Tanks that hold
only aviation fuel are exempt. The vote was 10 to 0, with two councillors absent.

The rule limits tank capacity and leaves the amount of fuel held to the operators. That matters
because the tanks are not full. A 20 percent cut to 333 million gallons of capacity leaves about
266 million, which is still more than the 234 million gallons the 2022 study found in the tanks.
The Oregonian's report on the day of the vote made the same distinction. It also reported an
objection from Oregon's energy department: the move away from liquid fuels may be slower than the
city assumes, and stored fuel is the region's reserve as well as its spill risk. The reserve has
been needed before. In 2025 a leak shut the Olympic Pipeline for more than a week, and the governor
declared a fuel emergency.

This project began on the day of the vote, as an attempt to put numbers on both halves of that
trade-off within one simulation.

## How the simulation works

The model covers a rectangle about 30 kilometres wide and 26 kilometres tall. It contains the City
of Portland and neighbouring parts of Multnomah, Washington and Clackamas counties. DOGAMI's
inventory puts 935,195 residents and 297,803 buildings inside it. The rectangle is divided into
cells about 30 metres across.

The model does not simulate the fault or the shaking. It reads DOGAMI's maps for the
saturated-soil case, which give the strength of the shaking at every point and the probability
that the ground there liquefies or slides. Everything that follows is this project's own
calculation on that ground.

At the moment of the earthquake the model works out the damage. For buildings it uses a fragility
curve, which gives the probability that a building is damaged at a given strength of shaking, and
estimates for each cell what share of the buildings reach each level of damage. Deaths and
injuries follow from a casualty rate for each level. The larger pieces of infrastructure are
treated one at a time. Each of 587 fuel tanks, twelve bridges, 6,146 power transmission structures
and 137 segments of emergency route sits at its real location, reads the ground there, and is
assigned a damage state by a random draw against a fragility curve of its own. A run's seed is the
number that fixes those draws, so two runs with the same seed give the same result and runs with
different seeds show how much depends on chance.

The model then steps through the following year one day at a time, and here the parts depend on
one another. Take a hospital whose building survives. If the grid still reaches it, it stays open.
If the grid does not, it runs on its generators, and the model gives every hospital 72 hours of
generator fuel, which is a guess. After that it needs a delivery. A fuel truck has to find a route
from the hub to the hospital over road segments that are open and, if the hospital is across the
river, over a bridge that is open. The hub has to have fuel left after the spill. So whether one
hospital has power on the fourth day depends on power structures, roads, bridges and tanks that
were each damaged or spared in the first minutes. Following chains like this one is the reason
for putting all of these things in a single simulation.

The figure shows the relationships as a person might draw them. A solid line is a relationship the
model represents. A dashed line is one that exists in the world and is missing from the model.

```esim
actors
the ground: shaking, liquefaction and landslides
buildings: 297,803 buildings
people: 935,195 residents
hospitals: thirteen, with generators
electricity: 6,146 transmission structures
roads: 137 emergency route segments
bridges: twelve over the Willamette
fuel terminals: 587 tanks
water: an assumed schedule
the ground -> buildings: damage and collapse
the ground -> electricity: structures fall
the ground -> roads: segments close
the ground -> bridges: most close
the ground -> fuel terminals: tanks fail and spill
buildings -> people: deaths, injuries, displacement
hospitals -> people: treat the badly injured
electricity -> hospitals: grid power
electricity -> people: interrupted care
bridges -> roads: join the two banks
fuel terminals -> roads: fuel trucks
roads -> hospitals: generator fuel
fuel terminals -> water: fuel in the river (not modelled)
fuel terminals -> buildings: fire (not modelled)
water -> people: illness (not modelled)
```

Many of the model's numbers are weaker than this description may suggest. The values of the
fragility curves and casualty rates are recalled from memory of the Hazus manual and have not been
checked against it. Several parameters were then adjusted until the model's totals for Portland
matched DOGAMI's. Others, such as the 72 hours of generator fuel, are guesses. The section
[How each part is modelled](#how-each-part-is-modelled) gives the rule and the source for each
part.

A run can also be watched. Open one from the list of runs above this text and press play. The map
shows the bridges and hospitals as marks that change when they close or lose power, and a clock
gives the date and whether the moment is before, during or after the earthquake. The arrow keys
step from one event to the next.

## What the runs show

Each row in the tables below is one variant of the scenario.

| Variant | What it changes |
|---|---|
| `wet-day` | The base run: saturated soil, 2 p.m., seed 1 |
| `seed-2`, `seed-3` | The base run with different random draws |
| `wet-night` | The earthquake at 2 a.m. |
| `dry-day` | Dry soil, so almost no liquefaction |
| `cut-20` | 20 percent less fuel held at the hub |
| `retrofit` | Every tank meets the newer standard and releases at most 10 percent |
| `no-hub` | No fuel held at the hub |
| `ranges` | An earlier version of the model, described below |
| `burnside-ready` | Meant to keep the Burnside Bridge open, but has no effect, as explained below |

The `ranges` variant does not compute bridges, roads, electricity or the spill from individual
components. It draws their outcomes from the published ranges: the Resilience Plan's restoration
times, the county's statement on its bridges and the 2022 study's release range. It is kept so
that the computed results can be read beside the estimates they are checked against.

### The day after

```esim
table 2027-03-27T21:00Z quake.buildings_collapsed=buildings_collapsed quake.deaths=deaths_counted_by_cell agents.dead=deaths_counted_by_agents quake.displaced=people_displaced fuel.ground_gal=fuel_spilled_on_the_ground fuel.river_gal=fuel_spilled_toward_the_river
```

**Buildings.** In the base run 1,047 of the 297,803 buildings collapse, and 19,769 take extensive
or complete damage. The shaking injures 7,120 people badly enough to need a hospital, 610 of them
with life-threatening injuries, and displaces 42,925.

**Deaths.** The model counts the dead from the shaking in two ways, and the two counts disagree.
The first works cell by cell: it multiplies the people in each cell by a casualty rate. It gives
1,000 deaths in the base run, and 564 and 602 with the other two seeds. The second represents the
residents as 93,520 agents, each standing for ten people, places each agent in a building that
collapses or stands, and counts the agents who die. It gives 670, 460 and 550. The two methods use
the same rates and should agree. At night they do, at 631 and 630. By day the agents' count is
lower by between a tenth and a third, and the cause has not been found. Until it is, neither
figure should be quoted as this model's death toll. The total that a run reports as `deaths.total`
uses the agents' count.

Neither count is an independent estimate in any case. The model's casualty rates were adjusted
until its deaths in Portland matched DOGAMI's, and the largest adjustment triples every rate.
DOGAMI's loss table gives 810 deaths in Portland's nine districts for this case, and its summary
for the city gives 896. The cell-by-cell count for those districts is 748 in the base run, which
is close to DOGAMI's because it was made to be.

What the model adds to DOGAMI's figure is a view of how much the toll depends on chance. The three
seeds differ by almost a factor of two, although the number of collapses barely changes between
them, from 998 to 1,047. In the model 97 percent of the deaths happen in collapsed buildings, and
a collapsed building kills a fixed share of the people in it. The toll therefore depends on which
buildings collapse. A collapse in a cell full of daytime workers counts for far more than one on a
residential street.

The model handles night and dry soil poorly. In DOGAMI's study the night-time toll for the three
counties is about a quarter of the daytime toll. In the model's area it is nearly two thirds, and
the model's night-time deaths in Portland are twice DOGAMI's. DOGAMI's toll on dry soil is about
two thirds of its toll on saturated soil, while the model's two figures are almost the same. The
reason is a rule in the model: only shaking makes a building collapse. Ground failure destroys
buildings without collapsing them, and a destroyed building that has not collapsed kills few of
its occupants. Removing the liquefaction therefore removes many destroyed buildings and few
deaths.

**The spill.** In the base run 233 of the 587 tanks fail and release 60.6 million gallons: 34.4
million onto the ground and 26.2 million toward the river. The 2022 study estimates 95 to 194
million gallons, so the model's release is about two thirds of the study's minimum. The release
from each failed tank follows the study's own rule, so the shortfall lies in how many tanks fail.
That number comes from a tank fragility curve that is recalled from memory and unverified.

The variants move the spill in the direction one would expect. Holding 20 percent less fuel cuts
the release toward the river from 26.2 to 21.0 million gallons. Retrofitting every tank cuts it to
2.0 million. On dry soil 97 tanks fail and 3.7 million gallons go toward the river, which shows
how much of the spill in this model comes from liquefaction under the tanks.

### Three days after

```esim
table 2027-03-29T21:00Z lifeline.crossings_open=bridges_open service.power_share=share_of_people_with_electricity lifeline.hospitals_dark=hospitals_without_power fuel.hub_gal=fuel_left_at_the_hub
```

**Bridges.** Ten of the twelve bridges close in the base run. Only Tilikum Crossing and the
Sellwood Bridge, the two newest, still join the two banks of the river. The Broadway Bridge reopens after 21 days and the Marquam and Burnside bridges after 120
days. The other seven take the worst level of damage and stay closed for 365 days, the repair time
this project assigned to that level for an older bridge. They reopen on the last day of the run,
so a full year without them is as much as the model can say.

Multnomah County says none of the downtown bridges is expected to be usable immediately after a
major earthquake, and that it could be weeks to years before any is. The model's result resembles
that statement, but it is not an independent check, because the curve for the older bridges was
set by judgement from the county's own report.

The `burnside-ready` variant was meant to show the effect of a Burnside Bridge built to survive.
It changes a setting that only the earlier version of the model reads, so its results are
identical to the base run's.

**Electricity.** In the base run 90 percent of people keep their electricity at the moment of the
earthquake. The share is 95 percent on day 3 and 97 percent on day 7, and everyone has power by
day 22. The Resilience Plan expects one to three months. The model is this far from the plan
because transmission structures are the only part of the power system it contains. Of the 6,146
structures 586 fail, and crews repair twelve a day. The model has no substations, power lines or
neighbourhood distribution, so none of those can fail in it. This is the model's least credible
result, and two others depend on it: how many hospitals lose power, and how many people die in the
months after.

**Hospitals.** All thirteen hospital buildings stand in the base run. Three have no grid
electricity and cannot be reached by a fuel truck, so they lose power on day 3 when their
generators run out: Oregon Health & Science University Hospital, Doernbecher Children's Hospital
and Providence Milwaukie Hospital. Two are still without power on day 7 and none by day 14.

The `ranges` variant shows how much this depends on the electricity result. There, electricity
returns on the Resilience Plan's schedule, and eleven of the thirteen hospitals lose power on
day 3. The true figure for this scenario is unknown, and the model cannot narrow it until it
represents more of the power system. The model also does not represent hospital capacity. It has
no beds, staff or supplies, and no damage inside a hospital building that still stands.

### A year after

```esim
table 2028-03-26T21:00Z after.deaths=deaths_after_the_shaking fuel.consumed_gal=fuel_used_by_the_response fuel.hub_gal=fuel_at_the_hub
```

**Roads.** In the base run 22 percent of the emergency route length is open during the first week,
27 percent on day 30 and 61 percent on day 90. All of it is open by day 180. No hospital can be
reached by road from the hub until day 111. The three hospitals that lost power on day 3 got it
back from the grid, long before a fuel truck could have reached them.

**Fuel.** That delay exposes a fault in the fuel accounting. The model starts counting the
emergency response's fuel use only once a truck can reach a hospital. In the base run the response
therefore uses no fuel for 110 days, which is not a result to rely on. Meanwhile resupply resumes
on day 14, and the hub is back to its starting stock about two and a half months after the
earthquake.

The model does show the two sides of the storage argument as numbers. After the spill the hub
holds 172.9 million gallons in the base run, 138.3 million with 20 percent less fuel held, 228.7
million with every tank retrofitted, and nothing with no hub. In this model, holding less fuel
gives a smaller spill and a smaller reserve, and retrofitting the tanks gives the smallest spill
and the largest reserve.

Two cautions apply before these figures are used in the argument over the ordinance. The `cut-20`
variant removes a fifth of the fuel held, which is a stronger measure than the ordinance's cut to
capacity. And the model cannot say how long any of these reserves would last, because the
response's daily use and the date of resupply are guesses. Whether a smaller reserve would run
out before resupply is a question the storage argument depends on, and this model cannot yet
answer it.

**Water.** Water service is an assumption in every run, so it is an input and not a finding. In the
base run 10 percent of people have water on day 30 and half on day 150, and everyone has it by
day 300. That leaves 841,676 people without water for more than a month.

**Deaths after the shaking.** The model adds deaths in the following year by two rules. One applies
a death rate to every day a person spends without electricity, standing for medical care that is
interrupted. The other lets people with life-threatening injuries die on any day when no hospital
is open. The base run adds 2 deaths over the year, with a range of 0.3 to 3.7, all from
interrupted care. The `ranges` variant adds 66, with a range of 11 to 121. The whole difference is
the length of the power cut: 443,000 person-days without electricity in the base run against 14.5
million in the `ranges` variant. Nobody dies of untreated injuries in either run, because some
hospitals stay open throughout. Since the base run restores electricity far faster than the
Resilience Plan expects, its figure of 2 deaths is too low by the model's own logic.

The 2022 study also estimates deaths at the hub itself if the spill ignites: 0 to 7 among about
1,200 workers. The model draws a number in that range and scales it by the size of the release,
which adds about one death in the base run. It does not model the fire.

## What cannot be trusted yet

- **The death toll.** The toll from the shaking is fitted to DOGAMI's estimate, and the model's two
  ways of counting it disagree by up to a third in daytime runs for a reason not yet found.
- **Recalled values.** The fragility curves for buildings, tanks and the two modern bridges, and
  every casualty rate, are recalled from memory of the Hazus manual and have not been checked
  against it.
- **The spill.** The base run releases about two thirds of the 2022 study's minimum.
- **Electricity.** It returns in three weeks where the Resilience Plan expects one to three months,
  because the model contains only transmission structures.
- **Fuel use.** The response's daily use, the hospitals' generator fuel and the date and rate of
  resupply are guesses, and the response uses no fuel for the first 110 days.
- **Water.** The restoration schedule is assumed, not computed.
- **Deaths after the shaking.** They rest on one rate borrowed from another disaster and on the
  model's electricity result.
- **Bridges and roads.** The curve for older bridges and the closure times for roads are judgements
  made for this project.
- **The building count.** The run reports 297,803 buildings in the area. The project's note on how
  the source data was prepared gives 297,120 building outlines. The difference has not been
  explained.

The model leaves out several things entirely. It has no fire following the earthquake, including
fire in the spilled fuel. It has no debris on the roads, no movement of spilled fuel down the
river, no bridges over the Columbia River, no shelters, and no people leaving the city. It has no
illness from unsafe water and no deaths from cold.

There is no record of this earthquake to score the model against. Every check on this page is a
comparison with another study, and the ground shaking itself is the output of DOGAMI's model.

The faults of the simulator's other models are listed on [Limits today](/docs/system/limits).

## How each part is modelled

**The ground.** The model reads DOGAMI's maps for the saturated-soil case: the peak acceleration
and velocity of the ground, and the probability and extent of liquefaction and landslides. Peak
acceleration across the area runs from 0.15 to 0.32 g, where 1 g is the acceleration of gravity.

**Buildings.** For each cell the model estimates what share of the buildings reach each of four
levels of damage, using a fragility curve, and then adds the buildings lost to ground failure. It
draws at random how many buildings in the cell collapse outright. Deaths and injuries follow from
a casualty rate for each level of damage, applied to the people DOGAMI places in the cell at 2 p.m.
or 2 a.m. The curve and the casualty rates have the form Hazus uses, but the project has not yet
read the Hazus technical manual, so their values are recalled from memory and unverified. Six
parameters were then adjusted until the model's totals for Portland matched DOGAMI's.

**People.** The 93,520 agents each have a home cell and a work cell. At 2 p.m. an agent is at work,
in a building that either collapses or stands, and it dies at that building's casualty rate.
Afterwards it has electricity or not according to its home cell. The agents make no decisions.
They do not leave the city, seek care or move around.

**Fuel tanks.** The 587 tanks come from the per-tank table in the 2022 study. A tank that fails
releases the share of its contents the study assigns to tanks of its age: 50 to 100 percent for
tanks built before 1993 or of unknown year, and up to 10 percent for newer ones.

**Bridges.** The twelve bridges are put in two classes. The two newest, Tilikum Crossing and the
Sellwood Bridge, use a curve for modern bridges. The other ten use a curve for vulnerable bridges.
That curve is a judgement made for this project. It is anchored on a 2014 Multnomah County report
on four bridges the county owns, which found parts of them vulnerable at the shaking it expects
from a Cascadia earthquake, 0.20 g. A bridge with moderate damage or worse is closed, and reopens
after a repair time set by its damage.

**Electricity.** The 6,146 power structures are the transmission towers and poles in DOGAMI's data.
Each fails with the probability DOGAMI gives for ground deformation at its location. An area has
electricity when its two nearest structures stand. Crews repair twelve structures a day, which is
a guess. The model has no substations, no power lines and no local distribution network.

**Roads.** The 137 road segments are the area's designated emergency routes. A segment closes where
DOGAMI expects the ground to deform, for a number of days that is a guess. Each day the model
checks which hospitals a truck could reach from the hub over open segments and open bridges.

**Hospitals and fuel.** The area has thirteen hospitals. A hospital with electricity from the grid
stays open. One without it runs on its generators for 72 hours and then loses power unless a fuel
truck can reach it. The model keeps an account of the hub's fuel in gallons that must balance
every day. It starts with the 233.5 million gallons found by the 2022 study, and the spill leaves
it at once. The emergency response is assumed to use 300,000 gallons a day, and resupply by
pipeline and barge is assumed to resume on day 14 at up to 1 million gallons a day. Both rates and
the day are guesses.

**Water.** The model does not compute water service. It restores water along a straight line that
ends on a day drawn at random from the Resilience Plan's range of one month to one year.

**Deaths after the shaking.** The death rate per day without electricity is derived from a study of
Puerto Rico after Hurricane Maria (Kishore and others, 2018), which is an analogy and not a
measurement of Portland. The rate for untreated injuries is a guess. The model counts the people
who depend on electrically powered medical equipment at home, about 5,900 in this area, but
applies no death rate to them.

## How the modules are connected

The figure below is generated from the latest run. It shows the model's modules, the fields each
one writes and reads, and the events that pass between them.

```esim
wiring
```

## About this page

The text was drafted in November 2026 by Claude, an AI model, for Anselm Hook's esim project. The
tables are drawn from the latest runs each time the page loads. The numbers in the prose were read
from the runs of 12 November 2026.

## What this page has read

Every source was read on 12 November 2026 unless noted.

- City of Portland, council document 2026-312, *Bulk Fuel Terminal Capacity Reduction Regulations*.
  Taken: the 20 percent cut by 1 December 2036, the aviation fuel exemption and the vote history.
  The ordinance passed as amended on 12 November 2026 by 10 votes to 0, with two councillors absent.
- Gosia Wozniacka, *A Big One nightmare scenario drives Portland to cut fuel storage along the
  Willamette*, The Oregonian/OregonLive, 12 November 2026. Read from a pasted copy of the text.
  Taken: that the rule cuts capacity and not fuel held, the energy department's position and the
  2025 pipeline shutdown.
- r/Portland, the discussion thread on that story, 12 November 2026.
- *CEI Hub Seismic Risk Analysis*, ECONorthwest, Salus Resilience and Enduring Econometrics, for
  Multnomah County and the City of Portland, February 2022. Read in full. Taken: the tank counts
  and ages, capacity and contents, the release rules and release range, the split between ground
  and river, and the estimate of deaths at the terminals. Its Appendix B, a table of 621 tanks, is
  the source of the model's 587 tanks.
- Multnomah County's web pages on the hub and on the 2022 study. Taken: the share of Oregon's fuel,
  the average tank year, the 26 percent chance in 50 years and the cost figures.
- Monica Samayoa, *Fuel spill in Portland after a major earthquake could be bigger than Deepwater
  Horizon, study finds*, OPB, 7 February 2022. Taken: the cost figures, which it gives as the
  county's page does.
- DOGAMI Open-File Report O-18-02, *Earthquake regional impact analysis for Clackamas, Multnomah,
  and Washington counties, Oregon*, Bauer, Burns and Madin, March 2018. Read: the report, its
  community summary and its two data bundles. Taken: the ground maps, the building inventory and
  occupants, the loss tables, the transmission structures and the emergency route segments.
- Multnomah County, *DOGAMI: Study estimates Cascadia earthquake impacts for Portland region*, the
  county's page on the 2018 study, last reviewed 27 November 2024. Read, but nothing on this page
  is taken from it.
- *The Oregon Resilience Plan*, Oregon Seismic Safety Policy Advisory Commission, February 2013.
  Read: the executive summary and the Cascadia chapter. Taken: the restoration table.
- Multnomah County, *Willamette River Bridges Capital Improvement Plan, Seismic Vulnerabilities and
  Retrofit Report*, 9 November 2014. Read: the introduction, the performance criteria and the
  sections on each bridge. Taken: the shaking levels and the finding that the four county bridges
  are vulnerable at them.
- Multnomah County, *Earthquake Ready Burnside Bridge*, the questions page. Taken: the statement
  that no downtown bridge is expected to be usable immediately.
- Federal Highway Administration, National Bridge Inventory, Oregon file for 2024. Taken: each
  bridge's year built, type, length and condition.
- OpenStreetMap, through the Overpass service. Taken: 352 tank outlines, used to place the tanks
  for which the 2022 study gives no coordinates. The placement within a property is a guess.
- Kishore and others, *Mortality in Puerto Rico after Hurricane Maria*, New England Journal of
  Medicine 379:162-170, 2018. Read: the abstract and reporting on it, not the paper. Taken: the
  excess deaths and the share attributed to interrupted care, turned into a rate per person-day
  without electricity.
- US Department of Health and Human Services, emPOWER, county data. Taken: the number of Medicare
  beneficiaries who depend on electrically powered medical equipment in the three counties.
- Atwater and others, *The orphan tsunami of 1700*, US Geological Survey Professional Paper 1707,
  2005. Read: the abstract. Taken: the account of the tsunami and of how it was linked to Cascadia.
- Goldfinger and others, *Turbidite event history*, US Geological Survey Professional Paper 1661-F,
  2012. Read: the abstract. Taken: the count of ruptures and the average interval.
- City of Portland, *Overview of unreinforced masonry (URM) buildings*. Taken: the seismic zone
  history and the building count.
- US Geological Survey, *M9 Cascadia Earthquake Scenarios*, the median scenario. Taken: shaking at
  one point in downtown Portland as a cross-check. Not used in the runs.
- US Geological Survey earthquake catalogue, the entry for the 1700 earthquake. Only the entry's
  title, magnitude and time were retrieved.
- Elizabeth Urban, *The M9 Project, part 1: a history of large earthquakes in Cascadia*, Pacific
  Northwest Seismic Network blog, 13 December 2017. Read, but nothing on this page is taken from
  it.
- Satake and others, 1996, in *Nature*, on the 1700 tsunami. Cited by the sources above. Not read.
- FEMA, *Hazus Earthquake Model Technical Manual*. Not read. The fragility curves for buildings,
  tanks and the two modern bridges, and every casualty rate, are recalled from it and unverified.
  The curve for the older bridges is this project's judgement and does not come from the manual.
