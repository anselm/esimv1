# 2026-11-01 — One project with variants, nested manifests, and the first layers above the climate

Anselm: a single project should be parameterised rather than split into one project per
assumption; build the higher layers, coarse is fine at planetary scale; more layers, not
fewer (infrastructure, supply chains, grids), with cross-layer feedback, regression-tested
against history; bioregional tipping points rather than collapse dates, with borders as
the places where people's options change. Then, mid-work: AMOC and MENA, the paleo water
events, patches not layers, nested manifests. Agent: Claude (Fable 5.1), earth-simulator-1
room on canada1. The research threads are in `20261101-research-agenda.md`.

## Variants

A project is still one manifest. `project.toml` may now carry `[[variant]]` tables, each a
name, a title and a `set` of overrides (`run.end`, `emissions.after_record`, any
`<module>.<param>`); `esim project run <path> --variant <name> [--set k=v ...]` applies
them, writes the resolved manifest into the run folder (so a run still encapsulates its
own state), records the variant and the overrides in `variant.json`, and the project page
lists variants with a Run button each and labels every run with its variant. The two
2100 projects are gone; `anselm/planet` has three variants: `hindcast` (to 2025), `held`
(the 2024 rate to 2100) and `zero-by-2050`.

## Nested manifests

`include = ["planet-core.toml"]` at the top of a manifest folds another file's modules,
place and audit tables in first; the including file's `[run]` stands, and a module that
restates an included name replaces it. `manifests/planet-core.toml` is the first bundle;
`p4-planet-people.toml` is eleven lines of its own on top. A project folder gets the
resolved text, so it never depends on another file. This is what "layer ensembles" means
here: a bundle is a file.

## The layers

Six kinds, all in `crates/modules/src/planet.rs` and `countries.rs`, all with sources
that answered anonymously today:

- `carbon.gases`: methane and N2O from Law Dome until the NOAA global means begin, as
  forcing by the Myhre 1998 expressions, published to the energy module on a new
  `forcing.extra` topic. 0.54 and 0.22 W/m² in 2024.
- `sea.thermal`: sea level from the energy ledger's stored heat, thermal expansion only.
- `people.countries`: Natural Earth outlines rasterised to a 2° world grid; people per
  country per year from the UN (estimates to 2023, the medium projection after, labelled;
  1950 held flat before 1950, labelled a placeholder).
- `climate.countries`: each of 171 countries' CRU temperature record regressed on the
  global anomaly (both smoothed over eleven years), so a country's share of global
  warming is its own observed slope (median 1.04); the run's anomaly times the slope gives
  its future, and its hottest month moves with it. Pattern scaling from the record.
- `people.heat`: people in countries whose annual mean has left the human climate niche
  (29 °C, Xu et al. 2020, recalled) and whose hottest month passes a marker that has no
  paper behind it (32 °C, a guess, kept to show the shape). A `people.crossing` event the
  first year a country crosses; a country already over a marker in 1850 is a state of
  its climate, not a crossing, and is not announced.
- `tipping.elements`: `data/events/tipping.toml`, fourteen elements with the assessed
  warming range and timescale, every row marked `recalled` from Armstrong McKay et al.
  2022 and not yet checked against the paper; a `tipping.crossing` event when the run
  passes an element's low end and its central estimate.

The auditor lists every `*.crossing` event and counts them in its reasons.

## What the three variants say, and the one thing to read first

**The model now runs half a degree warm.** Adding methane and N2O with no aerosols on top
of a carbon cycle that keeps too much CO2 airborne puts the 2024 anomaly at +2.05 °C
against the measured +1.54 (HadCRUT5, re-zeroed to 1850 to 1900). The temperature scores
went from passable to negative skill. Every crossing year below is therefore early, by
roughly the twenty years it takes the model to gain half a degree. The two missing
pieces are the aerosol series and a carbon cycle with real sinks, both on the list; until
they land, read the years as "when the model's thermometer says", not as dates.

| 1 July | held: warming | outside the niche | countries | tipping past central | zero by 2050: warming | outside the niche | countries |
|---|---|---|---|---|---|---|---|
| 2024 | +2.05 | 152 M (1.9%) | 10 | 7 | +2.05 | 152 M | 10 |
| 2050 | +2.58 | 386 M (4.0%) | 16 | 7 | +2.35 | 308 M | 12 |
| 2075 | +3.01 | 615 M (6.0%) | 18 | 9 | +2.32 | 416 M | 11 |
| 2099 | +3.39 | 741 M (7.3%) | 22 | 9 | +2.31 | 484 M | 11 |

The crossings list is the part that reads like the story Anselm wants told, and it is
the part to be most careful with. Under held emissions: Niger leaves the niche in 2035,
Chad in 2045, Kuwait in 2048; Egypt's hottest month passes the marker in 2038, India's
in 2051 with 1.7 billion people, Pakistan's in 2061. Under zero by 2050 the list stops at
Egypt in 2046, and the count of people outside the niche in 2099 is 484 million against
741. The gap between the two columns is the decision; the countries in both columns are
the commitment.

Three honest caveats besides the warm bias. A country crosses whole, because its
temperature is one number; India at 1.7 billion is the loudest example of a mean hiding
a continent. The niche threshold is on the annual mean and says nothing about humidity,
which is what kills; the hottest-month marker is a guess and is labelled one. And the
UN's population after 2023 is the UN's projection, which assumes a world in which these
crossings do not change where people live.

## Moving maps

Fifty frames per field over 250 years, 16,200 cells each: the country mean temperature,
the hottest month, people per square kilometre, and in or out of the niche, all under the
run page's one cursor. The maps are country-shaped blocks and look it; that is the
resolution of the data behind them, not a rendering choice, and the devlog says so for
the person who asks why Russia is one colour.

## The hormuz precedent

Anselm passed on what the hormuz room (github.com/orbitalfoundation/hormuz) built for
country tipping points: a stress board, one row per country with eight sourced layers,
a mark where a layer has crossed a stated threshold at the week scrubbed to, rows ranked
by the count of marks; and a breaking-points panel, each row a flow, the function that
needs it, a sourced threshold and the consequence past it, with dated country readings
appearing only once the record reaches their date. Two lessons carry over directly. The
tipping table and the crossing events built today are the breaking-points row shape
(threshold, consequence, source, confidence) and should keep it as more layers arrive
(wet-bulb, harvests, grids). And the stress board is the missing view: the audit lists
crossings by year, but a board that counts a country's crossed markers at the cursor,
ranked, is the through-line a reader can scrub. It is a viewer addition over data the
run already writes; it is on `todo.md`.

## Not done

- Aerosols; the box carbon cycle; checking the tipping table against the paper.
- Humidity and wet-bulb, which replaces the guessed marker; the Gulf first.
- A population that responds (migration, mortality) instead of the UN's path.
- Coastlines and a pin on the world maps; the paired two-scenario view.
- The AMOC record and the Stommel box; the two paleo hindcasts. `todo.md`.
