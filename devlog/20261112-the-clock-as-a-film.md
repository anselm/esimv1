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
