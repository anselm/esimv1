# Scenarios to bring over from the earlier attempt: detailed todos

14 November 2026. Claude Code session in the esimv1 room on canada1
(session_018TGCFiQHbUYn12HkNti2xk, Fable 5.2).

Anselm: "Bring over scenarios but just as detailed todos - let's not do yet. Although I must say
it is tempting!" Nothing here is built. Each item says what the earlier attempt had (under
`~/projects/2027/orbital/`, mapped in `devlog/20261114-orbital-raw-materials.md`), what this
engine already has that would carry it, what is missing, and a first rung small enough to
finish. What was read for this entry: each earlier scenario's `info.json` or README, the Hormuz
manifest, and from the primer's scenario library its criteria, its priority list and five entries
(power grid cascade, water system after an earthquake, Tuvalu, Hormuz, MENA). The rest of the
library was read as headings only.

The primer's criteria for a scenario are worth keeping as the test for this list: specific (a
real place), intersectional (the interest comes from coupled parts), counterfactual leverage (it
shows something an argument cannot), an audience with a stake, and buildable.

## A. The earlier attempt's scenarios

### A1. Strait of Hormuz: a supply-chain cascade
- **What it was.** An agent-based model of commodity flow through the strait and its knock-on
  effects: energy prices, fertiliser, crop yields, food insecurity, helium. One knob, the share
  of the strait closed. Seven modules (`packages/agents/hormuz/`: flows, energy, fertilizer,
  agriculture, helium, food-security, observe) and six data tables (`public/orbital/hormuz/data/`:
  countries, oil fields, refineries, fertiliser plants, LNG and helium, routes). The library's
  entry names the data it wanted: IEA energy flows, the EXIOBASE input-output matrix, OEC trade.
- **What this engine has.** Countries as a layer (Natural Earth outlines, UN population, World
  Bank indicators in `score.regions`), ledgers that must close, interventions as parameters run
  over seeds, `events.shock` for a dated shock, the globe view.
- **What is missing.** Any trade or flow network: there is no module in which a quantity moves
  between countries. No prices. No crops at country scale.
- **First rung.** One commodity (crude oil) as a ledger between exporting and importing
  countries along the six routes in the old data, with the closure as a parameter, and the only
  output the shortfall by importing country. No prices yet. Record to check against: published
  transit volumes through the strait.
- **Layer.** Flows through the built, and law and money once prices arrive. It would be this
  engine's first scenario with no map cells at all.
- **Care.** His note on it: "Here I have emotions around non public data." Much of what is
  written about Hormuz is not public data; the rung above uses only published volumes.

### A2. IPCC AR6 warming scenarios
- **What it was.** An energy balance model driven by AR6 SSP CO₂ pathways, mapping temperature
  anomaly under four pathways from 2024 to 2100.
- **What this engine has.** Nearly all of it: `carbon.emissions`, `carbon.pools`,
  `energy.twolayer`, `forcing.ar6`, `climate.countries`, and Earth 2100's six emission variants.
- **What is missing.** The four SSPs as named variants (the current ones are this project's own
  assumptions), and a map of the anomaly by cell (the gridded pattern is the noted open problem:
  the MERRA-2 pattern fails over South Asia and a CMIP6 pattern is the next route).
- **First rung.** Four variants of the planet manifest with emissions set to SSP1-2.6 through
  SSP5-8.5, checked against the AR6 assessed warming ranges for each. This is the smallest item
  on the list.

### A3. MENA warming to 2049
- **What it was.** A stub: a README resting on Malik and others 2024 (JGR Atmospheres) on
  accelerated warming in the Middle East and North Africa. No model.
- **What this engine has.** Per-country warming, heat and wet-bulb exposure on cells
  (`people.heat`, `people.wetbulb`, `weather.heatwaves`), WorldPop people.
- **What is missing.** A regional warming pattern that can be trusted there. The per-country
  slopes already run high for Gulf states, which CLAUDE.md records as a fault, and this paper is
  a published figure to hold them against.
- **First rung.** A regional window of Earth 2100 over MENA with the paper's regional rates as
  the record, reporting where the model's country slopes sit against them. That is a check
  before it is a scenario.
- **Read first.** The paper itself; only the earlier README's summary of it has been seen.

### A4. Tuvalu: the freshwater lens
- **What it was.** A folder with a baseline and nothing in it. The library's entry: sea-level
  intrusion, storm overwash and whether the island's freshwater supply stays viable; small
  domain, well-characterised physics, existing literature; "good for demonstrating tipping point
  dynamics".
- **What this engine has.** Sea level from thermal expansion (`sea.thermal`), a shoreline model
  (`coast.oneline`), people, ledgers.
- **What is missing.** Groundwater of any kind. A freshwater lens is a new module.
- **First rung.** One island as a single lens with the classical lens relation, recharge from
  rain and loss to sea-level rise, as a water ledger; no map. Record: the published lens surveys
  the library points to, not yet found or read.

### A5. World events on a globe
- **What it was.** A viewer, not a simulation: conflict events from GDELT, UCDP and ACLED on a
  globe with a time slider.
- **What this engine has.** `obs.record` for placed, dated events, the globe view, and
  `policy.capacity`, which draws conflicts with no record behind them.
- **What it would be here.** A record module that gives the capacity model something to be
  checked against. ACLED's licence needs reading before it is fetched.
- **First rung.** UCDP's public georeferenced events as a record layer on the planet runs.

### A6. Tenerife futures, and the microeconomy
- Tenerife futures is a design fiction (an imagined 1976 conference), by its own README, and
  not a simulation. Not a candidate for the engine as it stands.
- The microeconomy folder says of itself that it is unused.

## B. From the primer's scenario library, closest to what exists

These two are in the library under "Resilience" and are exactly Portland's two weakest parts.

### B1. Water system failure after an earthquake
- **The library's entry.** Pipe break probability from ground motion, pressure loss, hospital
  risk; seismic hazard maps and the utility's water system GIS.
- **Here.** Drinking water in Portland is an assumed schedule, the last thing still inside
  `portland.lifelines`. A `water.pipes` module on the same ground maps would replace it. The
  Portland Water Bureau's estimate of 1,500 to 3,000 breaks is the record to hold it against,
  and has so far reached this project only through a search summary.
- **First rung.** Breaks per kilometre from ground deformation on whatever pipe network is
  public, counted against the bureau's range. Service and repair afterwards.

### B2. Power grid failure cascade
- **The library's entry.** N-1 contingency, blackout spread, restoration sequence; grid topology
  partly available from regulators. Its backlog names Texas in Winter Storm Uri (2021) as an
  instance with rich data for validation.
- **Here.** `power.structures` has no network, which is why electricity returns in three weeks
  where the state expects one to three months. This is item 2 of the critical todos.
- **First rung.** Substations and lines as a graph over the existing structures, service by
  connectivity. Uri as a second place to prove the module is not Portland's.

### B3. Already done here, for the record
- Wildfire spread up canyon terrain: the Palisades (two engines) and Nicasio.
- Fort McMurray 2016 as a historical reconstruction: done, with the faults the limits page lists.
- Tipping elements (the library has the Amazon, permafrost and AMOC as separate entries): held
  in `tipping.elements` and `carbon.tipped`, with a coupling table.

### B4. The library's own priority order, against this engine
1. Urban heat island: not started. `sun.irradiance` and land cover (WorldCover is now read for
   water) are the two inputs it names.
2. Bike lane and street reallocation: not started; nothing here models travel.
3. Wildfire spread: done.
4. Hormuz closure: A1 above.
5. California groundwater: not started; no groundwater (the same gap as Tuvalu).
6. MENA warming: A3 above.

The other library entries (suburb approval, urban growth boundary, stormwater, tree planting,
air quality, dam removal and salmon, dead zones, Chesapeake nutrients, debris flow, rewilding,
coral, Suez and Panama, semiconductors, export bans, data-centre demand, sea-level inundation,
India heat mortality, migration, and the smaller ones) were seen as headings only and are not
assessed here.

## C. A suggested order, when the time comes

1. A2, the SSP variants: a day's work, and it makes Earth 2100 speak the language funders know.
2. B1 and B2, water pipes and a power network: they fix Portland's two least credible results
   and each is a general module.
3. A1, Hormuz, first rung: the first scenario with no map, which would test whether the engine
   is more than a spatial one.
4. A3, the MENA check, since it tests a known fault.
5. A4, Tuvalu, which needs groundwater and so opens California groundwater too.

## D. Data sources: a future todo

Anselm: "Data sources is interesting - maybe as a future todo? The digitaltwins.wiki tries hard
to enumerate." Three lists exist and have not been compared:

- this engine's catalog, `data/catalog.toml`, rendered as the Data sources page;
- the primer's appendix, `orbital-primer/chapters/91-appendix-data-sources.md`, and the short
  list at the end of its scenario library (Global Forest Watch, LANDFIRE, the IPCC Interactive
  Atlas, EXIOBASE, Comtrade Plus);
- digitaltwins.wiki, which sets out to enumerate the field's data and tools.

The todo: compare the three, list the sources the other two name that the catalog lacks, and
say for each which scenario above would use it. The wiki is the right home for the enumeration;
this engine's page should list only what it actually reads and link to the wiki for the rest.
