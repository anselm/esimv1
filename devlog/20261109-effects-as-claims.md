# Effects as claims

2026-11-09. Anselm, with a meme of Delhi's air (an AQI readout of 2052 captioned
"sarcasm", and a headline that 99 of the 100 most polluted cities are in India right now,
neither checked here): "here is a detail on the 2100 scenario that i didn't consider -
regarding AQI impacting longevity. Interestingly as well, birth rates are dropping around
the world - it's not entirely clear why - some people argue that our phones are a better
distraction, other people argue that microplastics are a concern. There's also some
thinking about heat making people think more slowly; we basically cannot shed the energy
required for thinking - so we tend to think less in hot climates (so the theory goes). I
wonder if its possible to try to include subtle effects or somewhat unproven effects like
this? Can 'effects' in general be more modular - like how are these managed right now - I
think we have a fairly large set of effects already being applied to the 2100 scenario."

This entry is the inventory he asked for and a design. Nothing is built.

## How effects reach the 2100 run today

Every effect is code inside the module that owns the quantity it moves, with the strength
declared as a parameter whose note says guess, recalled or checked. What crosses between
modules crosses on a few ports, and the ports have one pattern: a message carries a `name`
and a number, the receiver keeps the latest number per name, and combines them by sum or
by product. Nobody owns the name; a new sender needs no change at the receiver.

| port | carries | combined by | senders today | receiver |
|---|---|---|---|---|
| `forcing.extra` | W/m2 by name | sum | gases (methane, n2o), assessed (the AR6 components), wildcards (a pulse) | energy.twolayer |
| `carbon.emitted` | GtC | sum | emissions, tipped (permafrost, Amazon) | carbon.pools |
| `emissions.scale` | a factor by name | product | wildcards | carbon.emissions |
| `emissions.capacity` | 0 to 1 by name | product, on the planned fall only | capacity | carbon.emissions |
| `people.shock` | a fraction, world or one country | compounded | wildcards, capacity (conflicts) | people.countries |
| `shock.crossing` | a card by kind | a band multiplier per kind for some years | wildcards | score.regions, capacity |
| `region.pressure` | stress share and three bands | a fall and a slow recovery | regions | capacity |
| `tipping.crossing` | an element and a level | threshold shifts by the coupling rows | tipping | tipped, tipping itself |

And inside modules, not on any port:

- `score.regions` multiplies the living, water and means bands by one minus a declared
  factor times the country's heat stress (three parameters, all guesses).
- `tipping.elements` shifts thresholds by the `[[coupling]]` rows in `tipping.toml`.
- `policy.capacity` turns pressure into a capacity by its own fall and recovery rates.

What there is no port for at all: anything that changes how many people there are except
a one-off fraction removed. `people.countries` is a data layer. It reads the UN's medium
projection and places it; there are no births and no deaths in the run, so there is
nothing for air, heat or fertility to act on. The UN's path already contains the UN's own
fertility assumption, which is a fall; the file in the catalog carries every variant (the
catalog says so: "all variants") and the reader keeps Estimates and Medium only
(`crates/registry/src/countries.rs`, line 92).

So the honest summary: the effects are modular at the seams (the by-name ports), and not
modular inside (each rule is a few lines in the module that owns the stock, with its
strength as a parameter). A new effect today means a new parameter and a few lines in the
right module, or a new module publishing on an existing port. Nine modules, about a dozen
rules, no list of them anywhere but the wiring diagram and the describes.

## What the three effects would need

**Air and longevity.** The air band is the World Bank's PM2.5 exposure
(`EN.ATM.PM25.MC.M3`), one number per country, ranked. It never changes in a run, since
nothing emits particles. An effect would be an excess death rate per microgram above a
threshold, applied to a population that has deaths in it; the Global Burden of Disease
puts ambient PM2.5 at a few million deaths a year worldwide (recalled, order of magnitude,
not checked). Two things are missing before the effect: a mortality port on the people,
and something that moves PM2.5 over the run (the simplest honest driver is the fossil
share of the emissions path, which the emissions module knows; a crude link, declared as
one).

**Fertility.** The run does not need to know why births fall to carry the fact that they
might. The UN's Low variant is a sourced path for exactly that, and reading it is a
parameter on the people module (`variant = "Low"`), a day's work, no new science, and a
variant on the 2100 project (`held-low-fertility`). The causes Anselm lists (phones,
microplastics) have no number anyone could defend; the model should carry the outcome
with its source and leave the cause to the README. A heat link to births has some
literature (Barreca and others on temperature shocks and birth rates, recalled, not read
here) and would be a claim row, below.

**Heat and thinking.** There is a literature on heat and cognitive performance (exam
scores on hot days, labour productivity by wet-bulb; Graff Zivin, Hsiang and Neidell on
temperature and human capital, Park on high-stakes performance; both recalled, neither
read here). The energetic mechanism Anselm gives is one theory and I know no source for
it. In this model the natural stocks for it are the means band (output) and the capacity
to act (policy); a claim row would multiply one or both by a declared factor per
heatwave-day from `heat.pressure`, which is published every year and which nothing reads
yet. That port already exists for this purpose and has been waiting since 2026-11-07.

## Design: a claims table and three more ports

The move is the one `future-speculative.toml` made for wild cards: the uncertain thing is
a row in a table, with a source, a confidence and a switch, and one module reads the table
and translates rows into messages on ports the owners already accept. A subtle or
contested effect is then a row, not a module, and the page can list every row the run
carried with its confidence beside its size.

**The table**, `data/events/effects.toml`, one `[[claim]]` per effect:

```toml
[[claim]]
id = "heat-means"
title = "Hot days cut output"
on = "band.means"                 # the port and the name: band.<band>, people.deaths,
                                  # people.births, capacity, emissions.scale, forcing.extra
driver = "heat.pressure.days"     # a published topic and field, per country where it is
per = "country"                   # country | world
shape = "linear"                  # linear | threshold | lagged
strength = -0.002                 # per unit of the driver, on the stock named by `on`
cap = 0.3                         # the most it may move the stock, as a fraction
source = "Graff Zivin, Hsiang and Neidell 2018 (recalled, not read)"
confidence = "contested"          # checked | recalled | guess | contested
status = "off"                    # off by default; a variant turns it on
```

**The module**, `effects.claims`: reads the table, subscribes to every driver named,
and each January publishes one message per active row on the port named by `on`, with the
row's id as the `name`. Its describe lists the rows as a table (as the wild cards do), so
the run page shows them, and declares a publish per port it uses and a subscribe per
driver, so the wiring diagram draws the arrows and the check refuses a driver nothing
publishes. Variants switch rows on by id (`claims.on = ["heat-means", "air-deaths"]`).

**The ports.** Three the owners do not have:

- `people.births` and `people.deaths` on `people.countries`: `{ name, iso3?, factor }` or
  `{ name, iso3?, per_1000 }`, kept by name, product or sum. This needs the module to
  have births and deaths, which means reading the UN's crude rates (the WPP indicator
  file carries births, deaths and the rates per country per year, the same source and
  licence; a new catalog asset) and stepping population as the UN path plus the named
  changes to its own births and deaths. The UN total is then reproduced bit for bit with
  no claims on, which the regress ladder checks.
- `band.<name>` on `score.regions`: `{ name, iso3?, band, factor }`, a multiplier by name
  per band, the way the wild-card knocks are applied today, but from outside and by name.
- `capacity.scale` on `policy.capacity`: `{ name, factor }`, product.

`emissions.scale` and `forcing.extra` already take by-name messages, so an emissions or
forcing claim needs no change.

**What the page says.** A run's describe carries the claims table; the story module can
read `claims.crossing` (one sentence per row the first year it is on, with its
confidence) so the bottom line says "this run carries three contested effects" before
the numbers. The limits page gets a line: a claim with confidence `contested` or `guess`
is a question the run asks, not a finding.

## Rungs

1. `variant = "Low"` on the people module and a `held-low-fertility` variant. Sourced,
   no new ports, shows the fertility question at its honest size (the UN's own spread by
   2100 is about two billion people between Low and High, recalled, to be read off the
   file).
2. The three ports and `effects.claims` with the table empty but for `heat-means`, driven
   by `heat.pressure`, which is the one driver already published and unread.
3. Births and deaths in the people module from the UN's rates, then `air-deaths` as a row,
   with PM2.5 either held (a static toll, which is still a toll) or moved by the fossil
   share, declared as a crude link.
4. The capacity link (heat and thinking) as a second row on the same driver, so the two
   readings of one theory, less output and less will, sit side by side in one variant.

The rule that holds through all of it: a row's number has a source or says guess, the
switch is off until a variant names it, and the page lists what was on.
