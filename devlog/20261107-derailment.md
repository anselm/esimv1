# Knock-on effects in the 2100 scenario: derailment

Anselm, 2026-11-07: "Thinking again about the 2100 scenario - I feel we are still failing to
really consider knock on effects. What are some ways we can improve this scenario? Here is an
article discussing how some impacts reduce optionality", with James Dyke's piece in The
Conversation of 2026-11-05, "The worse climate change gets, the harder it becomes to stop it"
(Global Systems Institute, Exeter; read that morning through the fetcher, the coauthors not
named on the page).

The article's argument, as read: warming has effectively doubled its rate in a decade;
tipping points may cascade (an Atlantic shutdown starving the Amazon of water vapour, the
Amazon's dieback moving the carbon cycle, both speeding the ice); and, its own contribution,
"derailment risk": the consequences of climate change erode the capacity to address its
causes. Its mechanisms: a sovereign debt doom loop (extreme weather damages vulnerable
economies, debt service crowds out mitigation and adaptation finance); conflict, which
undermines the cooperation mitigation needs, stronger where inequality and polarisation are
high; political backlash (Vox after the 2024 Valencia floods); and the asymmetry that
"collective action can be easier to destroy than build". Its prescription: map the feedback
loops, build resilience against disruption, make governance responsive in unstable periods.

## What the model has, and the loop it lacks

Every knock-on the article names is a feedback from impacts to capacity, and the model has no
such loop. Emissions after 2024 are an assumption chosen by the variant and nothing in the
run can move them. The sinks feed back (Millar), three tipping elements feed carbon back,
and that is the whole set of feedbacks; the heat exposure, the wild cards and the region
scores are all downstream and nothing reads them upstream. The actors figure on the page
draws this as the dashed arrow from people to emissions, and the article is a paper about
that one arrow.

## Five ways in, in the order I would build them

1. **Derailment as a module, through a port that exists.** `carbon.emissions` already takes
   `emissions.scale { name, factor }` from the wild cards. A `policy.capacity` module computes
   a world capacity to cut, 0 to 1, each year from what the run already knows: the share of
   people under heat stress (score.regions), the damage of the year's wild cards, the voice
   band (institutions) and the means band weighted by population, and the asymmetry the
   article names (capacity falls fast on a bad year and recovers slowly). It publishes a
   factor that slows the planned decline: with capacity c and a planned path that cuts by
   r a year, the effective cut is c·r, so `zero_by:2050` becomes zero by later or never. One
   parameter set, declared as placeholders, and a variant `zero-by-2050-derailed` beside the
   clean one. The page then shows the gap between the plan and what the model's own impacts
   let happen, which is the article's claim in a chart.
2. **Cascading tipping.** `tipping.elements` checks each element against the warming alone.
   Add a coupling table to `data/events/tipping.toml`: when the AMOC passes its central
   estimate, the Amazon's thresholds drop by a stated amount; when Greenland passes, the
   AMOC's drop; West Antarctic and Greenland raise each other's chance. The interaction
   signs and magnitudes are in Wunderling et al. 2021 (Earth System Dynamics), recalled and
   to be read before any number is typed. Then a run can tip a chain, and the ladder shows
   the chain.
3. **A place for the pressure.** Debt, conflict and backlash are national, not global. The
   region scores already hold per-country stress and bands; give the capacity module a
   per-country capacity, let the wild cards take a region (the field exists, unused), and
   let conflict's annual chance rise with a region's stress and fall with its voice band,
   which is the article's conflict mechanism with its inequality modifier. The globe's red
   then has a cause beyond heat.
4. **The people's response.** Deaths and movers from the heat and the sea
   (devlog 2026-11-05), so that the population contracts where the stress is and the
   capacity module has a loss to read rather than an exposure count.
5. **Optionality as a number on the page.** The article's word is options. For each year,
   the remaining budget to a warming level under the current path, and the cut rate now
   needed to hold it, both of which the run can compute from its own series; drawn beside
   the capacity, the reader sees the gap open: what must be done rising while what can be
   done falls. That is the picture the piece is asking for and no module is needed, only a
   view.

Everything above is a placeholder in its numbers and honest in its shape; the first and the
fifth are a day, the second an afternoon once the paper is read, the third and fourth are
the larger work already planned. Not built this morning.

## Built, the same day (Anselm: "Let's try hard to add this level of detail to the 2100 scenario")

Four of the five, in the order above, as placeholders that say so; the fourth (deaths and
movers) stays planned. The work is in `crates/modules/src/capacity.rs` (kind `policy.capacity`),
`planet.rs` (`emissions.capacity` on `carbon.emissions`), `score.rs` (`region.pressure` yearly:
the share of people under heat stress and the people-weighted voice, fairness and means bands;
`people` per country in the scored table), `countries.rs` (`people.shock` with an `iso3` takes one
country; `tipping.elements` with `coupling` and the `[[coupling]]` rows; the story's new ports and
counters), `data/events/tipping.toml` (three coupling rows with the paper's provenance block),
the manifest (`capacity` in `mode = "watch"`, `tipping.coupling = 0.0`), five project variants,
the compare page's label, the docs and the README.

Decisions on the way:

- The derailment goes through a new port, not `emissions.scale`. A factor on the rate cannot keep
  a zero plan above zero; what derails is the cut, so the emissions module walks the plan year by
  year from the record's last year and multiplies each planned fall by that year's capacity, never
  a rise. With no capacity ever published the walk is skipped and the plan is returned bit for bit,
  so every older run and the regression ladder are untouched.
- The first run had zero-by-2050 stall at 1.9 GtC a year for ever, because after 2050 the plan has
  no fall left to scale. That is not the article's picture (later, not never), so after the plan
  reaches zero the cutting continues at the plan's last pace, still scaled, until the rate is zero:
  zero by 2050 becomes zero by 2056 under the default numbers.
- The conflict gain came down from five to two percent a year per unit of stress: fifty-four
  conflicts in a run buried the thresholds in the story. Fourteen on the zero path, thirty-one on
  the 3% path, drawn in the Gulf and the Sahel where the heat stress is.
- `watch` mode so every variant reports the capacity and the options and only the derailed ones
  act on them. The budget's TCRE is AR6's 0.45 °C per 1000 GtCO2, recalled, and declared so.
- The couplings carry Wunderling et al. 2021's ensemble-mean net lowering per element (55, 40 and
  10 percent at full interaction strength, read from the publisher's page this morning; its two
  parameter tables are images and were not read) as fractions of our thresholds, applied once when
  any of the pushing elements has passed; the split per link is not in the paper's text and is
  marked as ours; the raised Greenland threshold has no number and is not applied. The year's check
  loops until nothing new passes, so a chain can close in a year.

What the runs say (all seventeen variants rerun after the manifest change, 5 s each):

| variant | warming 2100 | emissions 2099, GtC/yr | capacity 2099 | conflicts |
|---|---|---|---|---|
| zero-by-2050 | 2.11 | 0 | 0.70 (not applied) | |
| zero-by-2050-derailed | 2.16 | 0 (zero in 2056) | 0.68 | 14 |
| zero-by-2050-derailed-wild | 2.18 | 0 | 0.58 | 15 |
| falling (3%/yr) | 2.48 | 1.2 | 0.58 (not applied) | |
| falling-derailed | 2.65 | 3.6 | 0.53 | 31 |
| held | 3.36 | 11.8 | 0.21 (not applied) | |
| held-cascade | 3.37 | 11.8 | | AMOC flagged 2029, Amazon releases from 2090, 18.6 GtC tipped against 15.8 |

The honest reading: with these placeholders the loop is worth a few hundredths of a degree on the
zero path and two tenths on the slow path, and the capacity's own fall on the held path (to a
fifth) is the article's claim as a number, where there is no cut for it to slow. The 2 °C budget
closes by 2040 on every path because the model runs 0.2 °C warm; the 3 °C budget is the one that
moves. The surface of the everything run was checked against the describes by a script (no
undeclared topic); the describe test passes; the regression ladder is unchanged.

Not done: the per-country capacity table is published (`capacity.scored`) and nothing draws it
yet; the globe could read it as a fourth ring or in the hover. Deaths and movers (the fourth way)
are still the planned impact layer. The conflicts have no duration, no spread and no place beyond
the country. Nothing in the capacity is finance or politics; it is one number with the article's
shape.
