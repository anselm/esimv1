# Impact, contraction, and the events that could come

Anselm, 2026-11-05 morning, three things on the 2100 scenario:

> the 2100 scenario maps really don't communicate anything. Weirdly the readme map clearly
> shows red areas for wet bulb but the live sim maps just seem to do nothing over the run?

> in general I feel like the 2100 scenario really needs a much more clear sense of population
> contractions due to system stresses.

> does the site have an enumeration of known future events (launches, Olympics, country
> reunification anything macro and known)... and also speculative possible events?
> (Asteroids, gamma ray bursts, cholera, war, other macro scale unexpected or black swan
> events). It would be nice to run 2100 scenarios with impacts from speculative events
> optionally - effectively a seeded random triggering one of the speculative events at a time
> and potentially having large impacts. I would like these lists to be reusable I may add
> them to digitaltwins.wiki as a timeline of future known events and future speculative
> events - effectively a data source used by other tools.

## The maps: a linear ramp over five decades of people

The frames were fine. In the held run the people-outside-the-niche field goes from 24
populated cells at the start to 1,716 at the end, with a maximum of 34 million people in one
cell; the wet-bulb days field from 29 cells to 377, maximum 106 days. The map drew them on a
linear ramp from zero to the run's maximum, so a cell with half a million people sat at one
and a half percent of the ramp, the first shade above the land colour, and the eye read
nothing. The cover on the project page had been drawn by hand on a log scale, which is why
it showed what the live map did not. `FieldMap` now has a third stretch, `log`, a
logarithmic ramp over the run's range with a floor four decades under the maximum so zero
cells stay blank, and it is the default when a field's positive values span three decades
or more. The run-wide and per-frame linear stretches remain as buttons.

## Contraction: the layer the model lacks

Said on the 2100 page since last night: the people counts are exposure and the population
is the UN's path, so nobody dies and nobody moves. What Anselm wants to see is the opposite
sign, population falling where the stresses pile up, and he is right that nothing on the
page shows it because nothing in the model makes it. The design, not built:

- `people.impact`, a module reading the fields the runs already make (annual mean past
  29 °C, wet-bulb days over 31 °C, and, when it exists, the sea's reach) and the country
  population, and owning two new fields and two ledgers: deaths per cell per year from heat
  by a published exposure-response (the Lancet Countdown and Vicedo-Cabrera et al. 2021 are
  the candidates to read; nothing recalled is good enough to type), and movers per cell per
  year, people leaving cells past a stated limit for the nearest cells within the same
  country that are not, with a fraction crossing borders. Both go into the people ledger
  as declared stocks so the auditor sees them; the UN path becomes the baseline the module
  departs from rather than the answer.
- The non-linear turn is the point. Heat deaths rise steeply past the threshold rather than
  linearly, and a delta that floods is a step. Bangladesh is the case to score against: low,
  crowded, in the path of the sea and the humid heat. The scoreboard would be the IDMC's
  displacement record and the WHO's heat mortality estimates, both to be fetched, not
  recalled.
- The map then has something to show: a `people_change` field, the departure from the UN
  path per cell, signed, on a diverging ramp, with the country totals as a chart. That is the
  "population contraction" picture, and it would be dashed on the actors figure until it
  runs.

## The events: two lists on disk, two modules to come

`data/events/future-known.toml` (24 rows) and `data/events/future-speculative.toml` (17
rows) exist as of this morning, written from memory and marked so on every row; the schema
is flat so the files can travel. The modules:

- `events.timeline`: reads the known list, publishes `event.known` at each date, writes the
  sentence into the story. No effect on anything. Cheap, and it gives the time axis its
  landmarks.
- `events.shock`: reads the speculative list; with the run's seed, draws at most one event
  over the run's horizon by the rows' annual probabilities (or takes a named one from the
  manifest, `event = "vei7-eruption"`, for a scenario); publishes `shock.forcing` (a pulse in
  W/m² for n years), `shock.emissions_scale` and `shock.mortality`; `energy.twolayer`,
  `carbon.emissions` and `people.countries` subscribe and apply them, each in its own ledger
  so the audit sees the pulse and its decay. One event at a time, as asked; an ensemble over
  seeds then gives the spread of futures the list implies, which is the picture the
  cascade map draws by hand.

Before either runs, every row wants a source beside it. The known list is checkable in an
hour against the organisers' pages; the speculative list's probabilities are the order of
magnitude commonly quoted and should be read as placeholders until the papers named in each
row are read.

## Later the same morning: the lists read against Wikipedia

Anselm pointed at Wikipedia's *2030s*, *21st century* and *Timelines of the future* pages and
a Google answer with its own black-swan list. The known list was rebuilt from the pages
(plus *List of future astronomical events* and *3rd millennium*, which the timelines page
links): 88 rows, 82 read on Wikipedia and 6 still recalled (the 2026 and 2045 eclipses, the
US election and census, the two Paris stocktakes). The `confidence` vocabulary now has
"wikipedia" for a row read there, which is a tertiary source and not the organiser's page;
"checked" is kept for a row confirmed against the primary. The speculative list gained five
rows from the Google answer (an engineered pandemic, subsea cable severance, a sovereign
debt cascade, geoengineering termination shock, an unseen asteroid), with no probability,
since the answer gave none and the session will not invent one; Carrington-plus and the AI
runaway were already covered. 22 rows.

## Later: the wild cards built

Anselm: "i don't care if the module is accurate - i just want a placeholder, it is ok to have a
black swan event happen more than once. the fact is _some_ black swan event will happen - we just
had a pandemic for example - and that is actually likely. we will probably see a big change due
to ai as well. these almost leave the realm of black swan. the module or modules should be able
to be added to a scenario - in particular i want to plug it into the 2100 scenario, and have runs
that maybe end with unexpected outcomes. think of this like a video game with wild card factors
optionally."

Built the same morning as `events.shock`. Each January from `from` it tries every card in the
deck once against the module's seeded stream (so `run.seed` changes the draw and a card can
come twice), or plays one named card in a named year. A card fires as a `shock.crossing` with a
sentence (the auditor collects `.crossing` topics into the story, so nothing in the viewer
changed except the compare page's label) and as up to three effects: `forcing.extra` named
shock, which the energy balance already summed by name; `emissions.scale`, a new port on
`carbon.emissions` that multiplies the rate by the product of named factors; `people.shock`, a
new port on `people.countries` that removes a fraction of every country's people from that
year on and counts them. Off by default; the 2100 manifest carries it off and five variants
turn it on. Describe test and regress both clean (the hindcast is untouched because nothing is
drawn before 2025).

What the runs say, emissions held, at 2100: the baseline 10.18 billion people and +3.36 °C;
seed 1 draws 17 cards (four financial crises, three cholera decades, three automation
reorderings that apply nothing, an AMOC card, a Pinatubo, a great-power war, a 1918 pandemic
in 2090) and ends at 9.93 billion, 241 million taken, +3.35 °C; seed 2, 13 cards, 9.97 billion;
seed 3, 17 cards, 9.96 billion; the deck drawn four times as often, 63 cards, 9.61 billion, 562
million taken, +3.24 °C; the named Tambora in 2040 dips the warming for two years and leaves
2100 unchanged. The finding is the smallness: with the emissions path held, the slow things
decide the century and the shocks are weather on top, because no card in the deck changes the
path for good. A card that did (a war that halves emissions for a generation, a technology
that zeroes them, a collapse that does both) would be the first to add, and the levers for it
exist. The README has the section and the table.

## Afternoon: the world scored, on a globe

Anselm: "We live on a planet that is incrementally spending down or privatizing natural
resources - effectively an era of cheap or abundant externalities, natural resources is
ending ... All of these factors and others constitute a total bottom line; some use the term
triple bottom line, but there are many independent factors, arguably rankable. We could try to
identify scoring or values that distinguish or typify different regions of the world - and
then we could score the world ... It may be possible to make a glyph that is a gestalt of all
of these factors ... effectively scattering these totem poles or single glance markers ... if we
simulate the world forward in time - to 2100 - we can visually see where there are knockbacks
... I want to improve the 2100 page with a rendering of the state on a globe, with graphics
like this per region. Is that possible?" He pointed at his globe widget at orbital-sim.exe.xyz
(a textured three.js sphere with dot markers; read in the local checkout, not reused: the
house style wants outlines on a plain sphere, and the widget's texture is a file of unknown
origin).

Built the same afternoon. The data: nine World Bank indicators through the public API,
checked anonymous that morning (life expectancy, income per person PPP, Gini over 2012 to
2025, WGI voice and accountability under its `GOV_WGI_` prefix with `source=3`, which took
four tries to find, PM2.5, forest share, protected share, renewable water per person, CO2 per
person). The score: each band a percentile rank among the countries that report it, so the
page never has to say what a good number is; the total the mean of the bands present. The
motion: placeholders declared as parameters, a quarter off living and water past the niche, a
quarter more off living past a wet-bulb limit, a global knock per wild-card kind for some
years. The picture: `Globe.svelte`, a plain sphere with the Natural Earth outlines as line
segments and one instanced mesh per band, each country's bands stacked outward at its
centroid (largest ring, shoelace), OrbitControls, hover for the bands, the table at the cursor
from the run's `region.scored` events. 242 countries scored; Norway about 0.75, Canada 0.72,
Brazil 0.62, the United States 0.60 (fairness 0.19, footprint 0.07), Bangladesh 0.39, Nigeria
0.35. The mean over countries drifts from 0.503 to 0.501 by 2100 under held emissions: the
rules are mild and global, which the README says.

What this asks for next, in order: a place for the wild cards (a region field that names a
country or continent so a card can take one region's air and water, the reactor leak Anselm
named); the people's response layer feeding the living band from deaths and movers rather
than from a crossing flag; a population-weighted world mean beside the plain one; and a
choice of band set per project, since nine Bank indicators are one reading of a bottom line
and the module should take its bands from a table rather than a constant.

## Evening: a ChatGPT answer on bioregional scorecards, and what it says about ours

Anselm shared a ChatGPT conversation titled "Community health measures" and asked what I
thought and whether it changes the strategy. Only the final answer renders without a login,
so his prompt is unread. The answer, as read: track what a region depends on, what it depletes
and how it copes when a supply route fails, rather than net inflows; two established measures,
physical trade balance (imports minus exports by weight) and ecological deficit (demand against
regenerative capacity); a scorecard of three questions, what must come in, what is being drawn
down, what happens when a route fails (close the port or road in a scenario and count the days
each community can meet essential needs); Bangladesh after the 1998 floods as the case (more
than two million tonnes of rice imported, almost all from India, which it cites to FAO and I
have not checked); for communication, one journey a viewer follows before and after (Cyclone
Amphan 2020 imagery, then a road in Koyra broken two months on, a 20 km detour, salt water back
with each tide), replay the storm against two sea levels with the view labelled modeled and the
satellite labelled observed, and one concrete question on screen: "At what sea level does this
road become impassable?"

What I think. It is a fair critique of the globe built this afternoon. The nine bands are
stocks ranked among peers: what a region has. The answer's three questions are flows and
counterfactuals: what it depends on, what it is drawing down, what breaks. Those are what a
simulator can do and a ranked indicator cannot, and a scorecard that is all stocks will show
Norway high and Bangladesh low in 2100 much as in 2025, which is what ours does. On
communication it says, in other words, what the telling page says: the story carries the
reader, the modelled and the observed are labelled apart, and a page should pose one concrete
question and answer it from the runs. The journey at road level is the scale the fire pages
already have (Palisades Drive at 11:25) and the planet page lacks.

What changes, in order, none built tonight: the bands become three kinds, has, draws down,
depends on, with the drawdown bands from the Bank's own time series (forest area change,
water stress ER.H2O.FWST.ZS as a candidate) and the dependence bands from FAO food balance
sheets and energy import shares, to be checked for anonymous access; the wild cards get a
place and a "route fails" kind, so a card can close one region's port for a season and the
globe can show the days of reserve; every project page opens with one concrete question; and
the 2100 page follows one place, the Bangladesh delta, through the run, which is the impact
layer's first job. Memory: the physical trade balance and ecological deficit measures are the
chat's naming and were not looked up here.

## The bioregion as a patient (Anselm, the same evening)

"Human Population itself and growth rate is a measure hmm - I was looking for biodiversity
measures more than human centric but it is meaningful too. If we treat a bioregion as a
patient - how does a doctor get a holistic read on that patient? How does we know when the
patient dies?"

Checked that evening: the UN SDG API (unstats.un.org/sdgapi, anonymous JSON) serves the IUCN
Red List Index per country and year from 1980 (series ER_RSK_LST, 12,322 rows; Afghanistan
1980 = 0.929), the annual forest area change rate (AG_LND_FRSTCHG), the Mountain Green Cover
Index and the share of terrestrial, freshwater, marine and mountain Key Biodiversity Areas
under protection. So a biodiversity band with a trend is one fetcher away, where today's nine
bands are human-centric stocks with forest and protected area as the only nature in them.

The reading of the question is in the reply to Anselm; in short: a doctor reads a patient by a
few vital signs taken often, a history, and a prognosis against the normal range for that
body; the signs for a bioregion would be greenness and primary production from orbit each
month, river flow and groundwater, the fire regime against its own history, the Red List
Index trend and species abundance where a record exists, and soil and water quality; the
normal range is the region's own past, not the world's rank. Death has a published
definition: the IUCN Red List of Ecosystems calls it collapse, a transition past which the
defining features are lost and the characteristic biota gone, with criteria for decline in
extent, environmental degradation and disruption of processes (recalled; the criteria paper is
Keith et al. 2013, to be read). The Keeley finding of 2026-11-04 is a local case: chaparral
burned too often becomes grass that burns more often, a regime shift, a death with a
successor. For a people-and-land region the second death is dependence without reserve: the
region cannot feed or water itself and the route fails. Nothing built tonight.

## "The Living Region" (Anselm's ChatGPT-built page, the same evening)

Anselm: "How about this: https://the-living-region.anselm52491.chatgpt.site". Read in full by
rendering it (the fetcher choked on an embedded Landsat image). A single page, "How healthy is
a place?", four questions with a Bangladesh field case and a sources section:

- Community well-being, "What does GDP leave out?": room to live (income left after housing,
  transport and childcare, lowest fifth), health and opportunity (life expectancy by
  neighbourhood, mental distress, reading, young people's access to work), time and access
  (commuting hours, reach to clinic, groceries, park), connection and voice (a neighbour to
  call, life worthwhile, decisions reflect views). Show current value, five-year trend, the gap
  between groups: "Who experienced the gains?"
- Ecology and biodiversity, "Could the people be thriving while the river is dying?": native
  life (abundance, breeding success, genetic diversity, range), habitat connections (spawning
  habitat reachable, floodplain the river can enter), renewal (sediment moving, young streamside
  trees, a wetland watered when its species need it), resilience to a shock (name the system
  and the disturbance). "Record its current condition, the locally appropriate reference range,
  and its direction of change." The Umatilla Tribes' River Vision as the working example: flow,
  channel and floodplain form, connectivity, streamside plants, aquatic life.
- The Bangladesh case: a before-and-after Sentinel-1 radar slider around Nazirpur (16 and 22
  May 2020, Amphan), labelled observed, then Koyra in July with the road to Uttor Bedkashi broken
  and the tide still coming through; then "What if Amphan came back in 2100?", labelled modeled,
  three sea-level scenarios (32, 50, 84 cm) from Mitchell and colleagues 2022, exposed population
  up 50 to 80 percent in the high one.
- Material flows and resilience, "Should a healthy bioregion be a net producer?": physical trade
  balance, material footprint, ecological deficit as three different questions; an account per
  essential commodity (what arrives, by which routes, what can be made locally, what happens at
  the source); an interactive grain exercise (100 t a week needed, 60 local, 40 by one route,
  120 in reserve; halve the harvest, close the route); the 1998 flood (2.2 million tonnes of
  rice lost, over two million imported, mainly from India; FAO and World Bank cited).
- Sources and methods: "The indicators are prompts for a local assessment. Selecting the
  geography, reference conditions, time windows and ecological limits is part of the work."

Read against the project: it is the human model for a region, as the actors figure is for a
fire, and it is better than the globe's nine bands in three ways that matter: it reads a place
against its own reference range and direction of change, not a rank; it keeps nature's claim
separate from people's and names the river's five features; and it carries one place a reader
can stand in (Koyra's road) and one knob a reader can turn (the grain reserve). What it lacks
is what we have: a clock and a run. Its grain exercise is a one-week arithmetic; ours could be
the same account per region per year, with a flood or a wild card closing the route, scored
against the 1998 record. The design matters too: a page of four questions with a case and a
knob, not a page of panels, and the questions are the headings. That is the shape the 2100
page should grow toward, and the telling page should say so.

## The glyph grammar (Anselm's revised page, the same evening) and an icon block on the globe

Anselm: "I'm less concerned about factual accuracy at the moment. What I am searching for is
an implicit value of place. The metric is how can a person look at a globe and see bioregional
health? Now I am leaning towards an idea of a glyph that has child emoji representing a gestalt
understanding of place." And: "ignore the style. ChatGPT just wanted to make it pretty. I am
really just exploring how do we communicate information about regions?"

The page's new section, read in full: two glyphs on the same data. A portrait, "read from
crown to roots": leaves count thriving populations, a doorway shows affordability, roots are
grain routes, a mark travelling along a root is a delivery arriving, the portrait can be turned
45 degrees; two fictional estuaries (Alder, one route, one week in reserve; Reed, three routes,
four weeks) with a flood-the-crossing switch and a delivery timeline to play. And an icon
block: six measures for the United States and Bangladesh in fixed slots, land on top (forest,
fields), water and energy in the middle, household services below; area carries the amount
against a pale silhouette that is 100 percent, colour identifies the row and can be switched
off, a crossed slot is a measured zero and a hatched one a missing figure, a year slider
2010 to 2024. Its grammar, in its words: "A symbol supplies the noun. Position gives it a
consistent place in the whole. Size can express an amount, hue can identify a category, and
motion can describe a process." Bertin's visual variables and Ward on multivariate glyphs are
the references it names; the music aside (a voice keeping its colour through the score) is the
same idea for time.

Built that evening as a prototype: `Globe.svelte` has a second glyph, `icons`, now the
default, beside `bars`. Each country is a billboard sprite with a three-by-three block of
emoji in fixed slots, one per band (heart, money bag, scales, ballot box, wind, tree, park,
drop, footprints), each icon scaled so its area carries the band, the whole block growing a
little with the total; a missing band is a dot. Hover reads the bands. At the Americas' scale
Canada, the United States, Mexico and Brazil read at a glance (the United States: a small
scales, a small footprint); the Caribbean and Central America pile up, which any glyph map
does, and the zoom separates them. The emoji are the device's font, not fixed artwork, as the
page itself warns. The bars remain a click away.

What I think, for the reply: the icon block is the better of the two for a globe, because a
glyph that is read by position and size survives being small and being many, and a portrait
that is read as a picture does not; the portrait is for one region on a page. The thing
neither does yet, and the thing a run can do, is motion with one meaning: the block in 2050
beside the block in 2025, the same slots, and a mark that moves when something happens.

## The emoji totem (Anselm: "What if we had emoji on a cylinder?")

"Our own approach earlier was a totem pole. What if we had emoji on a cylinder? The problem is
the 9 block grid overlaps noisily in areas that are clustered together." Built as the globe's
third glyph and its default: a thin cylinder standing on each country's centroid along the
surface normal, the nine emoji printed up its skin in fixed slots, living at the foot and
footprint at the top, each scaled so its area carries the band, the strip printed twice round
so it reads from any side, the pole's height growing with the total, a territory with no band
at all getting no pole. The crowding is solved in the screenshot: the Caribbean, a pile of
blocks an hour earlier, is a fan of poles, thin where they stand and apart where they rise.
What is lost is the reading at a distance: at the whole-hemisphere zoom the icons are coloured
beads and the pole reads by its colour sequence, not its nouns; at a closer zoom the emoji
read. That trade is the right one for a globe, since the overview is for seeing which regions
differ and the zoom is for reading why. Not yet done: a wheel-driven close-up in the headless
check (the synthetic wheel event did not reach the controls), fixed artwork in place of the
device's emoji, and the pale silhouette behind each icon that makes a half-size icon read as
half.

## Later: three groups, a rainbow outline, land, the red, and the globe at the top

Anselm, in order: "we can split these into human factors, nature - specifically resilience,
and hmm maybe a third one could just be vanilla gdp / economic output ... I'm not happy with
glyphs or icons. Maybe we should try another option (can have a switch). Try summing up each
grouping into a health score from 0.0 to 1.0 and draw the outline of the country as a rainbow
of those summed bands, with some variation in thickness per band and or brightness. Each band
has to choose a nice primary pastel color." Then: "The globe should be near the top and also
it should have some kind of backdrop right now it seems to have no geography." Then: "Do we
have a way of showing when entire countries tilt into red - like when they are really
struggling? We need some total sum. And we should render that as a semi transparent red
overlay on the whole country."

Built, all in `Globe.svelte` and the viewer, nothing in the module:

- Groups, in the viewer from the nine bands: human (living, fairness, voice, air), nature
  (forest, protected, water, footprint), economy (means); a group's health the mean of the
  bands it has. The grouping is a constant in the component for now.
- The outline glyph, the new default with the switch keeping totem, icons and bars: each
  country's own rings (`docs/country-outlines.json`, written by `web/scripts/country-outlines.mjs`
  from the cached Natural Earth 50m file, outer rings thinned, 232 countries, 784 KB) drawn once
  per group as fat lines (three's LineSegments2), stacked outward, the weakest group innermost,
  width 1.2 to 5 px and opacity 0.35 to 1 by the score, pastels rose, green and blue with
  lighter variants for dark mode.
- Geography: an equirectangular canvas painted from the same rings, land on sea, wrapped onto
  the sphere, in both themes.
- The red: a second transparent sphere carrying a canvas where each country is filled red at
  an opacity set by its distress, the larger of how far its total sits under a floor of 0.35
  and how far it has fallen under its own total at the run's start, over 0.15; the legend
  counts the countries in the red and the hover says how deep. Both constants are the
  viewer's and are placeholders.
- The globe at the top: the story module now carries the `globe` view (it already carried the
  other modules' maps), the regions module keeps its chart; the 2100 variants rerun so their
  describe carries it.

Fat lines under software GL are slow enough that the headless screenshot timed out once; a
real GPU will not notice. What is not done: the grouping and the red thresholds as parameters
of the module rather than constants of the viewer, and a run in which something actually
tilts a country into the red, since under held emissions the placeholder rules move totals by
hundredths.
