# The clock as a film

2026-11-12, evening. Anselm, on the Portland page: "what can we do to make this fault line sim much
more informative / dramatic for lay people? It's just very dry. We should start before the event -
show the transition visually in a compelling way. We need to convey this better." Session: the
esim room (Claude Fable 5.1), this folder.

## What was dry about it

The run page was a lab bench. The run started two hours before the rupture, so there was no
before: the first frame already carried the shaking map, which never changed, and the damage
appeared whole in one frame at the instant. The slider walked the year in four hundred steps of
twenty-two hours, so the four minutes that matter were never under the cursor. The sentences
were a list under the counters, and the counters sat under the maps. A reader had to know what to
look at.

## The rung built tonight

- **A week before.** The run starts on 19 January 2027. Seven days of an ordinary city: the people
  map, twelve crossings open, thirteen hospitals, the hub full. The lifelines module says so in its
  first sentence, with the 1700 date beside it.
- **The motion at the instant.** `quake.ground` writes its acceleration and intensity maps at the
  rupture, not at the start, so before it there is nothing to map and at it the ground appears. A
  first version pulsed the map through the four minutes under an envelope; Anselm, watching:
  "Don't shake! Let's not be that dramatic!" Removed within the hour. The map is DOGAMI's, once.
- **The damage settles.** `quake.buildings` now schedules its reckoning for the end of the shaking
  rather than the instant, and writes zero frames at the start so every map exists before the event.
- **Moments.** The time cursor walks the run's moments, the union of every frame time, every
  crossing's time and four hundred even steps, and dwells a third of a second on a moment less than
  an hour from the next, so the four minutes play as a scene and the year as a sweep. The label beside
  the clock reads relative to the event when the run has one: "6 days before", "2 min in", "day 25".
- **The film.** A new view kind, `film`: one panel at the top of a module's section with the map on
  the left and, on the right, the moment, the newest sentence with the two before it faded, and the
  module's counters. The view's source is a list of fields; the film shows the latest one that has
  begun, so the people are on the map until the rupture, the ground motion while the shaking lasts,
  then the damage. `lifelines.recovery` declares it first.
- **A moment in the URL.** `?t=<unix seconds>` on a run page puts the cursor there and holds it, so a
  moment can be shared and a cover can be shot.

## What it is not yet

The film is still a map, a sentence and numbers. Not built: a 3D scene (the inventory has no
heights; OpenStreetMap's would do for the centre), the crossings and hospitals as marks on the map
that change colour with their state (the `lifeline.marked` and `lifeline.failed` events are
there to draw from, the way `Scene3D` draws the record), the spill as a plume on the river, the
sound of it, a narration track that reads the sentences at the right moment. Each is a rung, and
the marks are the next.

## Sober, and with and without (later that evening)

Anselm, three messages while this was being built. "What I mean is that an observer should simply
be able to see a before and after. 'The big one' will cause deaths - magnified by these fuel
depots. Maybe there can be multiple sims runs - some with and without - this would more clearly
paint outcomes." Then: "This is a lay person science tool - we have to be sober - drama should be
in the results. It is ok to show deaths, suffering, lack of access to basic services, power
outages, transport outages and whatever we can in a clear way like on a map - but we are not
making a kids cartoon. I expect we will have to do more research on core data here and will
continue to enrich the module system." And the county's page on the DOGAMI study, with the note
that articles on the issue show a large number of deaths and criticism around the fuel depots and
the bridges failing.

What was built on that:

- **Marks on the map.** The film draws every component the lifelines module marked, with its
  state at the cursor, from the run's own log: squares for the crossings, circles for the hospitals,
  a diamond for the hub; a ring when open, filled when closed, dark or spilled, in a status palette
  the key beside the map names with counts. The module now publishes a hospital's turns to dark and
  back as `lifeline.failed` and `lifeline.restored`, so the marks have something to read.
- **One deaths number.** `quake.buildings` publishes `quake.toll` as numbers when the shaking ends;
  the lifelines module keeps `toll.deaths`, the buildings' dead plus the hub's own, and the film's
  first counter is that.
- **The hub's own toll, from the record.** The county's 2022 study assumes an explosion and applies
  the Texas City and Baton Rouge rates to the hub's twelve hundred workers: none to seven dead, two
  to eighty hurt (its Table 11). The run draws in that range and scales by the release. DOGAMI's run
  models no fire following the earthquake and says so in section 4.4.4; neither does this one. So on
  the record the hub adds about three dead to 765 in the base run and none in the no-hub run, and
  the page says that plainly rather than inventing a magnifier. The large death counts in the
  articles are DOGAMI's own, which the buildings module carries (and the county's summary page gives
  as 27,000 injuries, 85,000 needing shelter and 37 billion dollars); the criticism of the depots is
  about the spill and the fuel, which is where the model does show a difference.
- **No-hub.** A variant with no stock on the bank (`storage_cut = 1.0`); resupply from day fourteen
  keeps a week's deliveries as the working stock the trucks draw from. At day thirty the base run has
  317 days of the response's fuel in the tanks, the no-hub run 22. The river gets nothing.
- **The county page.** Added to the record and the reading list; it carries no bridge or hub numbers.

Next for the core data, as he expects: fire following earthquake (Scawthorn's model, which Hazus 4.2
restored and DOGAMI could not run in time), which is the only route to a hub toll beyond the site; a
hospital inventory with beds and generators; the water and power networks from DOGAMI's own gdb
instead of ramps; the bridges' real fragilities from ODOT.

## Deaths after, other impacts, the marks, the human model (night)

Anselm: "Extend sim to model deaths better. Extend sim to model other impacts. Can map visualization
more clearly indicate impact points such as bridges down? Paint human model into readme on that
scenario. I believe we also do render computational model on each run?" And then, on the page's
"a scenario, not a forecast": "I want to brute force simulate near term outcomes - I do not want
just a report. I want something similar to the palisades example - the goal of this engine is to
build out capability to simulate the future and playtest outcomes."

Built:

- **`people.aftermath`** (`crates/modules/src/aftermath.rs`), the deaths and harm after the shaking as
  claims in `data/events/portland-cascadia/aftermath.toml`, each with a rate, a source, a confidence
  and a switch, listed on the run page on or off. On: care interrupted by the loss of power at the
  Puerto Rico 2017 rate (Kishore et al. 2018: 4,645 excess deaths over 102 days, a third from
  interrupted care, over 3.34 million people, recalled, gives 4.55e-6 per person-day; the paper's
  interval as low and high factors); the life-threatening injured with no open hospital, at a guessed
  daily rate, treated the first day any hospital is open; the people on power-dependent medical
  devices without grid power, from HHS emPOWER's county counts (Multnomah 4,825, Washington 3,593,
  Clackamas 3,152 Medicare beneficiaries; 0.63 percent of the three counties' people on recalled 2020
  populations), counted and not killed, since the Puerto Rico rate already covers them. Off, no source:
  cold homes, waterborne illness. It reads `quake.toll` (the buildings now publish the severity-3
  count separately) and `lifeline.state` (new, daily) and owns `deaths.total`. Base run: 66 after
  (11 to 121) on 765 in the shaking; seed 3 with a slow grid, 145; untreated zero everywhere because
  the generators cover the first days. Impacts reported: person-days without power (14.5 million)
  and water (140 million), the people past the one-month rule (58,450 without power, 841,676 without
  water), the devices without power at the cursor.
- **The marks** are larger, crossed when down, named on hover, and listed by state under the key, so
  "crossing closed: Sellwood, Ross Island, ..." reads without the map.
- **The human model** is in the README as prose, "How the pieces hang together", above an extended
  actors figure (the people, the aftermath edges), and the run's wiring is embedded in the README
  (a new `wiring` embed, drawn from the latest run's describe), with a paragraph that says plainly
  what is computed from the ground cell by cell and what is arranged from published ranges. Yes, the
  computational model is drawn on every run page, as the wiring at the bottom.

**The brute-force ladder.** Anselm's point stands: the buildings are computed the way the Palisades
fire is, and the lifelines and the aftermath are not. DOGAMI's own impact bundle carries what the
next rungs need, cut tonight into the record folder: `poles.csv`, the transmission structures in
the window with DOGAMI's chance of failure and deformation at each, wet and dry; `routes.geojson`,
the emergency transportation route segments with DOGAMI's risk class at each. The rungs, in order:

1. `power.poles`: every pole fails or stands by its own probability and the seed; a cell's power is
   its nearest poles'; crews repair a stated number a day in an order (hospitals first); the power
   share per cell is computed and the map shows the dark parts of the city coming back. The lifelines
   read it instead of ramping. The Resilience Plan's one to three months becomes a check, not an input.
2. `roads.segments`: each segment closes by its class and reopens by crews; reachability of each
   hospital and of the hub along open segments and open crossings, so the trucks' day is computed.
3. `fire.following`: ignitions by the Hazus rate on the damage, on the `fire.cells` engine with
   DOGAMI's footprints as fuel, roads as breaks, no water pressure, the hub's ground release as a fuel
   bed; the first route to a hub toll beyond the site. Needs the manual's ignition rate, which is
   still not downloadable from here (the file that came was 620 bytes), or Scawthorn's.
4. The spill as a flow: the ground release down the slope on the DEM, the river's share carried by the
   county study's seasonal currents.
5. People as queues: hospitals with beds, shelters with places, the injured arriving by reachability,
   the untreated dying by a rate per day waited.
6. Ensembles as the playtest: `esim ensemble` over seeds for each variant, the compare page under one
   cursor, so a cut, a retrofit, a bridge or a no-hub is read as a distribution and not a run.
