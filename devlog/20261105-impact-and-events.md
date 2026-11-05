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
