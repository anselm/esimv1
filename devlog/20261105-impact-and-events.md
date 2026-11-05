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
