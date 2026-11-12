# The year after

On the evening of 26 January 1700 the sea floor off the Pacific Northwest slipped by twenty
metres along a thousand kilometres of fault, and nobody in Portland wrote it down, because
there was no Portland. The date comes from Japan: a tsunami with no earthquake behind it arrived
on the coast of Honshu that night, and in 1996 Kenji Satake and his colleagues matched it to the
drowned forests and the buried marshes that Brian Atwater had been reading along the Washington
coast since the 1980s. The trees that died standing in salt water at the mouths of the rivers,
the layers of sand on top of the marsh soils, the stories of the Makah and the Cowichan: all of it
one night, now dated to the day. The fault does this every few hundred years. The last time was
327 years before the afternoon this page is about.

What the State of Oregon plans for is a magnitude 9.0 on that fault, and since 2013 it has said in
public what it expects. The Oregon Resilience Plan, written for the legislature by the Seismic
Safety Policy Advisory Commission, put it in a table: in the Willamette Valley, electricity back
in one to three months, police and fire stations in two to four, water and sewer in one month to a
year, the main highways partly open in six to twelve, hospitals in eighteen months. On the coast
every line is longer. The plan's own word for these is sobering, and it adds that a business that
loses its services for a month tends to leave, so that a region with those numbers faces not a
bad season but a lost generation.

In 2018 the state's geologists ran the earthquake under the three counties around Portland,
building by building. DOGAMI's Open-File Report O-18-02 took synthetic bedrock motions from the
USGS, computed what the soils would do to them, mapped where the ground would liquefy and where
the slopes would fail, and put 615,852 footprints with their occupants through FEMA's Hazus loss
model, at two in the afternoon and two in the morning, on dry soils and on saturated ones. For the
City of Portland on a wet afternoon it came to 896 dead, 3,654 hurt badly enough for a hospital,
30,627 people needing shelter, and 18.9 billion dollars of building damage. For the three counties,
1,473 dead and 85,211 displaced. The county's own summary of the study, the page it still keeps,
puts it as up to 27,000 injuries from minor to fatal, about two percent of the three counties'
people, 85,000 needing shelter and 37 billion dollars of building damage, and quotes the study's
lead investigator: a Cascadia earthquake will happen, and how well we prepare will make all the
difference in how well we bounce back. The report says what every scenario says, that these are
planning figures with a lot of uncertainty and little detail, and it is the best anyone has.

## Why now

The thing Portland decided this week is about a six-mile strip of the west bank of the Willamette
in the north of the city, between the Fremont Bridge and Linnton, which the state calls the
Critical Energy Infrastructure Hub. More than ninety percent of the liquid fuel Oregon uses comes
through it, by pipeline from Puget Sound and by barge, into about four hundred tanks on fill and
river silt that DOGAMI maps as among the likeliest ground in the region to liquefy. The average
tank with a known date was built in 1954. In 2022 the county and the city published a study of
what the tanks would do in the earthquake: between 95 and 194 million gallons would leave them in
the first minutes, more than half onto the ground and the rest toward the river, a volume the
county chair compared to the Deepwater Horizon. The study put the costs it could count at 2.6
billion dollars and said the real figure would be many multiples of that.

On 12 November 2026 the city council took its second reading of an ordinance that requires each of
the hub's terminals to cut its tank capacity by at least a fifth by 2036. Gosia Wozniacka's story
in the Oregonian that morning is the one the r/Portland thread was about, and it says what the cut
is and is not. The fifth is of capacity, terminal by terminal, not of the fuel in the tanks; the
county's 2022 study found the tanks about two thirds full, 234 million gallons held in 333 million
of room, so a cut to capacity could leave the fuel where it is. Utility storage and sustainable
aviation fuel are exempt, and the jet-fuel storage may grow by a twentieth. The city has refused
new fossil-fuel storage at the hub since 2016; the drawdown is the first rule on what is already
there. The activists who pushed for it call it a stopgap and wanted a drawdown tied to inventories
and to Oregon's consumption, which the ordinance begins to collect; the city administrator owes
an alternative by June 2027. The state's energy department answered with the other side of the
argument: in an emergency the hub's storage is the buffer, as it was for a week in 2025 when a leak
shut the Olympic Pipeline and the governor declared a fuel emergency, the hub cannot be moved
because the pipeline, the rail, the river and the roads meet there, and the department's own
seismic programme for the tanks is the tool that reduces the risk. Councilor Steve Novick, voting
for it, said that unless demand falls the state will not let the city carry it out.

The thread is the same argument. One side: the tanks sit on the worst ground beside the river and
should go. The other: the hub holds the region's jet fuel and most of its diesel, there is nowhere
else to put a river terminal, and a fifth less storage is a fifth less of the fuel the recovery
will run on. Both are right, and the question is what the numbers do in time. The model's cut-20
variant takes the fifth off the fuel held, which is the stronger reading of the ordinance than the
one that passed.

## What this is, and is not

There is no record of this earthquake, so there is nothing to score a model against except other
models. This page does not pretend otherwise. The ground motion is DOGAMI's, cut to the window at
one arc-second: the peak acceleration and velocity at the soil surface, the chance and the amount
of liquefaction and landslide. The buildings are DOGAMI's footprints and the people are DOGAMI's
own occupant counts per neighbourhood unit, spread over the unit's buildings by floor area. On top
of that, our own damage model: a lognormal fragility in peak acceleration for a generic building
mix, ground failure that moves a building to complete damage, and casualty rates by damage state in
the Hazus manner. FEMA's Hazus manual could not be downloaded on the day, so the rates are recalled
and declared so, and six numbers were fitted so that Portland's extensive and complete damage, its
displaced and its deaths come out near DOGAMI's for the same afternoon. The toll is therefore the
state's by construction; what the model adds is the pattern on the map and the contrasts between
the cases, and the fit is reported on every run so a reader can see how far it is off.

Then the year after, which nobody has drawn for a reader, and which this page now computes
rather than arranges. A components module holds every physical thing as an agent with a place, a
fragility, a state and a repair path: the hub's 587 active tanks from the county study's own
per-tank table, placed on the study's coordinates where it gives them and on OpenStreetMap tank
footprints where it does not; the twelve Willamette crossings; the 6,146 transmission structures
DOGAMI mapped in the window with the state's chance of failure at each; the 137 emergency route
segments with the state's ground-deformation class at each. On the rupture each reads the ground
at its own cell and draws its state. A failed tank releases by the study's age rule, to the river
where the study's zone says so. Crews repair poles at a stated rate, most people first; segments
reopen by class; crossings by damage state. From what stands, the run computes each day the power
service cell by cell, the people-weighted share, and whether each hospital can be reached from the
hub along open segments and open crossings. The fragilities are recalled from Hazus and declared
so; the study's release total and the county's words on its bridges are the checks, reported on
every run, and the first runs fail them in instructive ways, below. The arranged version, with
the crossings' days, the routes' day and the power ramp drawn from the published ranges, runs
beside as the `ranges` variant. The hospitals stand or fall by the fragility at their cells, run
on seventy-two hours of generator fuel, and are refuelled once trucks reach them. The clock itself says on every moment whether it is before, during or after
the earthquake and gives its date and time. The water is
still a ramp; the pipes are not in the state's bundle. The hub is a ledger in gallons
that must close at every daily close: what the tanks held, what left them in the first minutes by
the county study's range, what the response and the generators burn, and what comes back up the
pipeline. Cutting the hub's storage by a fifth and retrofitting its tanks are variants, so the
two sides of the thread can be run.

## What to show people

Not the shaking map; the state has drawn that. The clock, with a before and an after. A run
starts two weeks before the afternoon, on 12 March 2027, with the people on the map, every crossing open, every hospital
on the grid and the hub full, and says so in its first sentence. At the instant the map becomes
the ground motion, four minutes later the damage, and from then on the crossings and the hospitals
sit on it as marks that change state: a square for a crossing, a circle for a hospital, filled when
closed or dark, with the key beside the map. Beside the map, the moment, the latest sentence and
the counts: the dead, the crossings open, the hospitals open and dark, the people with power and
water. Press play and the two weeks before go by in a few seconds, the afternoon in a sentence, and then
the days: the generators run dry on the third, trucks move in the third week, the first bridge
comes back in the fourth, power in the second month, water in the tenth. The arrow keys step from
moment to moment, and `?t=` with a time in the address holds the clock there, so a moment can be
sent to someone. Then change one thing and run it again: the variants put the hub's storage cut by
a fifth, every tank retrofitted, and no hub at all beside the base run.

```esim
actors
fault: the Cascadia subduction zone, 1700 and next
ground: DOGAMI's synthetic motion and the soils that liquefy
buildings: 615,852 footprints and the people in them
hub: 587 tanks, 233 million gallons, each on its own ground
bridges: twelve crossings, built 1908 to 2016
hospitals: thirteen, on generators
people: 935,195 in the window; 5,900 on medical devices that need power
power: 6,146 transmission structures, crews at twelve a day
water: pipes, back in a month to a year
pipeline: the Olympic line from Puget Sound
council: Ordinance 2026-312
fault -> ground: slips, four minutes of shaking
ground -> buildings: shakes and liquefies them
ground -> hub: each tank fails or stands on the ground at its cell
ground -> bridges: each crossing by its class, the county's 2014 report and the bridge inventory
ground -> power: each pole by the state's chance at it
hub -> hospitals: diesel for the generators, once trucks move
bridges -> hospitals: the east bank waits for a crossing
power -> hospitals: the grid comes back, hospital by hospital
pipeline -> hub: fuel again from day fourteen
council -> hub: a fifth less storage by 2036
hub -> buildings: fire following the spill (not modelled)
hub -> water: oil in the river at the intakes (not modelled)
buildings -> power: debris and poles, the lumpy recovery (not modelled)
buildings -> people: the dead and the injured in the shaking
hospitals -> people: the life-threatening injured, treated or not
power -> people: care interrupted, at the Puerto Rico rate; devices without power
water -> people: a month without water, the Resilience Plan's rule; illness (not modelled)
people -> people: leaving, shelters, cold (not modelled)
```

## How the pieces hang together

The way a person holds this in mind is a chain with waits in it. The fault slips and the ground
moves; the ground breaks buildings and kills people at once; the same ground breaks the crossings,
the poles and the pipes, and those decide whether the injured reach a hospital, whether the sick
stay alive at home, whether the water is safe, and whether anyone can get across the river to fix
any of it. The hub sits on the worst of that ground and holds the fuel the fixing runs on, so it
is both the spill and the stock. Above the ground sit the council, which decides storage, the
state, which calls the hub the buffer, the federal response, which decides when the trucks come,
and the people, who decide whether to stay. Each layer waits on the one before, and the waits are
measured in minutes, days, months and years. The figure above draws that model in the author's
words, with the arrows the engine lacks dashed.

The engine's own model is narrower and the run page draws it below each run as the wiring: who
writes which field, who reads it, which events pass between them, which ledger they share. It is
drawn from what each module declares, so a relationship the world has and the run lacks shows as a
missing arrow, and the check that refuses a manifest whose wiring does not close is the same
drawing. Here it is for this project's latest run:

```esim
wiring
```

What is computed from the ground, cell by cell, the way the Palisades fire is: the buildings'
damage, deaths, injuries and displaced, from DOGAMI's motion through a fragility at every cell. What
is arranged rather than computed: the lifelines, whose days are drawn from published ranges, and
the aftermath, whose deaths are rates from other disasters applied to the counts the run has. The
difference matters, and the next rungs are the ones that turn the arranged into the computed: the
power as its 18,098 transmission poles with DOGAMI's own chance of failure at each, the routes as
the 238 emergency segments with DOGAMI's risk class at each, fire following the earthquake on the
cell engine that ran the Palisades, the spill as a flow on the slope and the river, and people as
queues at hospitals and shelters rather than as rates.

## What the runs say

The scenario is Friday 26 March 2027 at two in the afternoon, Pacific daylight time, in a run that
begins two weeks before and ends a year after the event, on saturated
winter soils, which DOGAMI calls its highest-impact case. The table reads each run the day after
and at the end of the year.

```esim
table 2027-03-27 deaths.total=dead_all_causes quake.buildings_collapsed=buildings_collapsed quake.deaths_in_collapses=dead_in_them quake.portland_deaths=deaths_in_Portland quake.injuries_hospital=injuries_needing_a_hospital quake.displaced=people_displaced quake.buildings_extensive=buildings_extensive_or_complete quake.buildings_complete=buildings_complete fuel.river_gal=gallons_toward_the_river
```

```esim
table 2027-04-25 fuel.days_left=days_of_fuel_at_day_30 lifeline.hospitals_open=hospitals_open lifeline.crossings_open=crossings_open service.power_share=share_with_power
```

```esim
table 2028-03-26 lifeline.crossings_open=crossings_open_after_a_year fuel.consumed_gal=gallons_burned_by_the_response fuel.delivered_gal=gallons_resupplied fuel.hub_gal=gallons_at_the_hub service.water_share=share_with_water
```

What stands out:

- **Computed against arranged.** The base run is now the computed one, and the ranges run is
  beside it. On the tanks: 233 of the 587 fail on the ground at their cells and release 61 million
  gallons, 26 million to the river, against the study's 95 to 194 million, so the recalled tank
  fragility is too strong or the study's rule that every old tank on deforming ground fails is
  stronger than a curve. On the crossings: the first computed run had eight of twelve standing on
  the first day, where the county expects none usable at once, because the curves were generic
  highway bridges. Each crossing now carries the federal bridge inventory's facts (year, type,
  length, condition) and a seismic class: the county's own 2014 vulnerability report says the four
  downtown bridges it owns, un-retrofitted, do not meet no-collapse at the 500-year Cascadia level
  of 0.20 g or the 1,000-year level of 0.28 g, and names the columns, footings, bearing seats and
  the liquefiable approaches; the curve for that class is a judgement anchored on those two levels,
  and the older state and railway bridges share it by age; Sellwood and Tilikum, built in 2015 and
  2016 under current code, get a modern class. With that, eleven of twelve close on the first day,
  three are back by day thirty and five by day 120, which is the county's "weeks to years" computed
  rather than drawn. On the power: 586
  poles fail and twelve crews a day have the whole window back in thirteen days, against the Plan's
  one to three months, because a pole is all the grid this model has; the substations, the lines and
  the distribution are not in the state's bundle. On the routes: a fifth of the segment length is
  open on the first day and the hospitals on the hub's bank are reached at once. Each of these is
  the engine being wrong in a way a record catches, which is what it is for; the arranged run lands
  inside the ranges because it was drawn from them, and says nothing. The deaths after the shaking
  fall to a handful in the computed run because the power is back in two weeks, which is the same
  error seen from the people's side.
- **The collapses, building by building.** The buildings module no longer reports an expected
  toll alone: at every cell it draws how many of the cell's buildings collapse outright, a binomial
  draw at the cell's collapse share by the seed, and the dead in them follow at the collapse rate.
  In the base run 1,047 buildings collapse and 974 of the 1,000 dead are in them; the second seed
  has 998 collapses and 564 dead, the third 1,030 and 602. The expected toll is unchanged at 765,
  and what the draw adds is the truth that one office tower falling or standing moves the count by
  hundreds, which is why DOGAMI's own number carries the uncertainty it states. The map of collapsed
  buildings is on the run page.
- **The people as agents.** The run carries 93,520 super-persons of ten people each, drawn by the
  seed from DOGAMI's occupant rasters: a home cell by the residents, a work cell by the daytime
  occupants, one in 160 on an electricity-dependent medical device. At the instant each is in a
  building that collapsed or one that stood, by the cell's realised share, and dies at that
  building's rate; the agents' dead carry the total on the page (670 in the base run against the
  cell model's 1,000, the difference being that each agent is ten people and a collapse of a few
  buildings is a handful of agents). Each day each reads the power service at its home cell. They decide nothing yet; what they establish is the cost: the daily tick
  runs at about twenty million agent-days a second on one core, so every person in the window
  separately is a change of one parameter, and the work is the behaviour, not the compute.
- **The deaths after the shaking.** The aftermath module adds about 66 dead over the year in the
  base run, a range of 11 to 121, nearly all from care interrupted by the loss of power at the rate
  Puerto Rico recorded after Maria, applied to 14.5 million person-days without power here; none
  from untreated injuries, because every hospital here runs on generators through the first three
  days and the grid returns to each on its own day, so the 468 life-threatening injured are treated
  on the first day. The third seed, with a slower grid, has 145. That is the model's first honest
  reading of the later dead: a tenth of the shaking's toll, from one analogue, with cold homes,
  illness from the water and the people on medical devices counted but not killed. The 5,900
  people on power-dependent devices in the window, from HHS emPOWER's county counts, are reported
  as an exposure. A month on, 841,676 people are still without water and 58,450 without power, past
  the Resilience Plan's one-month rule.
- **With and without the hub.** The county's 2022 study puts the hub's own toll, assuming the
  spill ignites, at none to seven dead and two to eighty hurt among its twelve hundred workers,
  the rates of Texas City in 2005 and Baton Rouge in 2016 applied to the site; the run draws in
  that range and scales it by the release, so the base run adds about three dead to the 765 the
  buildings kill, and the no-hub run adds none. On the record, then, the hub does not magnify the
  earthquake's dead on the day; DOGAMI's run models no fire following the earthquake and says so,
  and neither does this one, and that is the gap most worth closing. What the hub decides is the
  year: at day thirty the base run has 317 days of the response's fuel in the tanks and the no-hub
  run has 22, a week of deliveries, with the pipeline assumed back on day fourteen in both. The
  state's argument and the activists' are both in the numbers; the spill is the activists' and the
  stock is the state's.
- **The spill is the fastest thing that happens.** In the base run about 150 million gallons leave
  the tanks in the first minutes, 64 million of them toward the river, a draw at the middle of the
  county study's range; the three seeds span 134 to 171 million. Cutting storage by a fifth before
  the day takes a fifth off: 52 million toward the river instead of 64. Retrofitting every tank to
  the post-2004 rule, so that at most a tenth of its contents leaves, takes it to 13 million. The
  ordinance is the smaller lever and the one the city can pass; the tanks' age is the larger one.
- **The fuel that spills is the fuel the recovery wanted.** The hub starts with 233 million gallons,
  the county study's two-thirds fill, and ends the first afternoon with 84 million. The response in
  this run draws 300,000 gallons a day once trucks move, and the pipeline is assumed back on day
  fourteen, so the stock never runs out here; it would in a run where resupply takes longer, and
  the hub chart is the place to watch.
- **Three days.** Every hospital stands in the base run, on the generic fragility, and every one
  goes dark on the third day when the generators run dry, because no truck moves before day ten
  to thirty. They come back on fuel from the hub when the routes open. That rule is a guess; the
  seventy-two hours is a common planning figure, and what the real deliveries would be is exactly
  the question the fuel hub debate is about.
- **The bridges set the shape of the year.** The first crossing is back in three to four weeks on
  every seed, the last after one to two years. An earthquake-ready Burnside Bridge, built, means one
  crossing from the first minute, and in this model it changes the hospitals' refuelling and nothing
  else, because nothing else here knows the river has two sides.
- **Day and night, wet and dry.** By night the run puts about 560 dead against 765 by day;
  DOGAMI's own contrast is sharper. Dry soils give nearly the same deaths as wet here, where
  DOGAMI's wet case has half again as many, because the extra buildings the wet ground destroys
  are carried away rather than collapsed, and this model does not let the ground kill. Those two
  gaps are the model's, and they are the first things to fix.
- **The pattern holds.** Across DOGAMI's 876 neighbourhood units the run's damage correlates 0.9
  with the state's in the wet case: the river flats, the industrial north, the fill along the
  Columbia Slough. On dry soils the correlation falls to 0.6, which says the generic fragility is a
  poor stand-in for the mix of wood, masonry and concrete that decides who dies when the ground
  holds.

## What is wrong with it

The fragility and casualty rates are recalled, not read, and two of the six fitted numbers (a 13 to
25 percent collapse share, every casualty rate tripled) are large for what they stand in for, which
is an inventory with no structure types; with the collapses now drawn building by building, the
dead in the window run from 560 to 1,000 across three seeds around that fit, and the agents at ten
people each are lumpier still. The night occupants sit in every building by floor area,
which is why the night toll runs 1.7 times DOGAMI's. The displaced are a fitted share of residents
in badly damaged buildings and come out double DOGAMI's on dry soils. The components' curves are
recalled or judged too: the crossings' curves are anchored on the county's stated levels and not
published, and the first computed runs fail two of their three checks: too few tanks fail, and the
power is back too soon because a pole is the whole grid here; the crossings now close as the county
expects. The water is a ramp; the
pipes are not in the bundle. No fire follows the earthquake, no debris lies on the routes, no
tsunami because Portland has none, no Columbia bridges, no one leaves. The hospitals have no beds.
The deaths after the shaking are rates from other disasters applied to the counts here. The ground
is a model's and so is the record it is read against. What this page can honestly claim is that
the state's own inventory, run as components on the state's own ground with the guesses named,
produces a year that can be argued with piece by piece, which the thread was having without it.

## What this page has read

- DOGAMI Open-File Report O-18-02, *Earthquake regional impact analysis for Clackamas, Multnomah,
  and Washington counties, Oregon*, Bauer, Burns and Madin, March 2018. Read 2026-11-12: the
  report, the community summary and both GIS bundles. Taken: the ground motion and ground failure
  layers, the inventory, the loss tables, the scenario definitions and casualty levels.
- *The Oregon Resilience Plan*, Oregon Seismic Safety Policy Advisory Commission, February 2013.
  Read 2026-11-12 (the executive summary and the Cascadia chapter). Taken: the restoration table,
  the fatality and loss ranges, the one-month business rule, the retail fuel days.
- *CEI Hub Seismic Risk Analysis*, ECONorthwest, Salus Resilience and Enduring Econometrics for
  Multnomah County and the City of Portland, February 2022. Read 2026-11-12 in full. Taken: the
  tank counts and ages, the capacity and contents, the release rules by age class and the release
  ranges, the ground and river split, the pipeline.
- Portland Ordinance 2026-312, Bulk Fuel Terminal Capacity Reduction Regulations, the council's
  document page, read 2026-11-12. Taken: the 20 percent by 1 December 2036, the eleven terminals,
  the exemption, the utilization figures, the vote history.
- USGS, *M9 Cascadia Earthquake Scenarios*, the median ensemble ShakeMap (cszm9ensemble_se) and its
  ground-failure product, read 2026-11-12. Taken: the Portland grid point as a cross-check; not
  used in the run.
- Multnomah County, *Earthquake Ready Burnside Bridge*, the FAQ page, read 2026-11-12. Taken: the
  statements on the downtown bridges, the dates and the budget.
- OPB, *Fuel spill in Portland after a major earthquake could be bigger than Deepwater Horizon,
  study finds*, Monica Samayoa, 7 February 2022. Read 2026-11-12. Taken: the county chair's words.
- Multnomah County, *Willamette River Bridges Capital Improvement Plan, Seismic Vulnerabilities
  and Retrofit Report*, 9 November 2014. Read 2026-11-12, the introduction, the performance
  criteria and the per-bridge vulnerability sections. Taken: the earthquake levels (100-year 0.07 g,
  500-year Cascadia 0.20 g, 1,000-year 0.28 g), the performance targets, the statements that the
  four county bridges' columns, footings and bearing seats are vulnerable at the Cascadia and
  1,000-year levels and that liquefaction and lateral spreading under the approaches are
  unmitigated, Burnside's designation as the regional lifeline route.
- FHWA, the National Bridge Inventory, Oregon 2024 delimited file, read 2026-11-12. Taken: each
  crossing's structure number, year built and reconstructed, kind, type, length, condition ratings
  and traffic, in the record's bridge table.
- The county study's Appendix B, the per-tank table (621 rows with capacity, year, type, the
  study's zone and its own release range for each), read 2026-11-12 from the report's PDF. Taken
  whole, as `tanks.csv` in the record folder; 228 tanks carry the study's coordinates.
- OpenStreetMap, the storage tanks and named terminals along the hub through the Overpass API,
  read 2026-11-12. Taken: 352 tank footprints, on which the study's unlocated tanks are placed by
  property, which is a guess within the property.
- DOGAMI O-18-02's impact bundle, the transmission structures (18,098, 6,146 in the window, with the
  state's deformation chance at each) and the emergency transportation routes (238 segments, 137 in
  the window, with the deformation class at each), cut 2026-11-12 with GDAL.
- Kishore et al., *Mortality in Puerto Rico after Hurricane Maria*, New England Journal of Medicine
  379:162-170, 2018. The abstract and the reporting on it read 2026-11-12, not the paper. Taken:
  4,645 excess deaths over 102 days with a wide interval, one third attributed to interrupted care;
  turned into a rate per person-day without power in the claims table.
- HHS emPOWER, the county layer of its public REST service, read 2026-11-12. Taken: Medicare
  beneficiaries on electricity-dependent equipment in Multnomah, Washington and Clackamas counties
  (4,825, 3,593 and 3,152), and the beneficiaries counted.
- Multnomah County, *DOGAMI: Study estimates Cascadia earthquake impacts for Portland region*,
  the county's page on the 2018 study (last reviewed 27 November 2024), sent by Anselm, read
  2026-11-12. Taken: the county's summary numbers (27,000 injuries, 85,000 needing shelter, 37
  billion dollars), the lead investigator's words. The page says nothing of the bridges or the hub.
- Gosia Wozniacka, *A Big One nightmare scenario drives Portland to cut fuel storage along the
  Willamette*, The Oregonian/OregonLive, 12 November 2026. Read 2026-11-12 from Anselm's paste of
  the text, the site having refused this machine. Taken: what the ordinance cuts and exempts, the
  state's and the activists' positions, the Olympic Pipeline shutdown, the June 2027 review, and
  the correction to the hub's contents below.
- r/Portland, the thread on that story, 12 November 2026, the comments through Reddit's RSS feed.
- Satake, Shimazaki, Tsuji and Ueda 1996 (Nature) on the 1700 orphan tsunami, and Atwater's work
  on the coastal marshes: recalled, not read here.
- FEMA's Hazus Earthquake Model Technical Manual: not read; every fragility and casualty number
  here is recalled from it and marked so.
