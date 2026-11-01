# 2026-10-31 — Design brief, pasted by Anselm

Anselm pasted this into the room on 2026-10-31. His framing in the same message, verbatim:

> i have written, or attempted to write, several 'simulate the world' type engines - see
> https://simulate.world for original thinking, and then https://github.com/orbitalfoundation for
> more extensive thinking. Also see https://digitaltwins.wiki for a very very detailed list of
> tools, techniques, use cases and so on.
>
> I am thinking of taking yet another shot at building a simulation appliance, something similar
> to ESRI, but with a stronger consumer facing facade - although today any ux probably means
> largely a conversational interface rather than a rich graphical interface.

The brief itself was distilled from a claude.ai chat (not this session) in October 2026. It is
reproduced below unedited. Everything in it is a hypothesis to be derisked by code.

---

Whole-system simulator: design brief

Distilled from a design conversation, October 2026. This file is the starting context for the code project. Everything here is a hypothesis to be derisked by code, in the order given at the end.

Goal

A general purpose simulator of near-term future outcomes at three scales: local (a parcel, a basin, a city), bioregional, and planetary. Forward in time, over rich collections of independent behaviors: irradiance, volumetric weather, hydrology, soils, flora, fauna, built landscape, people, law and markets. Output is not one prediction but an ensemble of trajectories with the mechanism visible at every step, so a person can inspect a run, rerun it, and intervene.

The thing that has never been built as a whole is the coupler: many representations, many rates, one shared world state, with conservation across every seam. Any single domain already exists in a mature tool. The risk is almost entirely at the seams.

Out of scope, deliberately

User accounts, identity, access control, decentralization, hosting, and any operating-system ambitions. Also module loading and packaging: how a behavior gets into the process (wasm, dynamic library, custom grammar) is a surrounding problem to solve later. The core question is simulating forward correctly.

Five representations, five clocks

Every behavior on the list has one of five mathematical shapes, each with its own natural timestep.

Fields: irradiance, weather, soil moisture, surface and ground water, ocean. Grids or volumes, PDE-like, timestep of seconds to minutes set by stability. Only a few of these cost real compute.

Networks: rivers, roads, pipes, power grids, trade. Flow routed along edges, hours to days.

Populations: flora as plant functional types with fractional cover per cell, plankton as nutrient-phytoplankton-zooplankton-detritus fields, soil microbes as rate constants on carbon pools, fauna as cohorts (functional group, body mass, count, age) attached to cells, in the manner of the Madingley model. Seasons to years, with stochastic disturbance.

Agents: people, households, firms, cities. Discrete events, decision rules, days to years. Entity-component layout with parallel typed arrays.

Rules: zoning, water rights, tariffs, treaties, markets. Constraints that mutate the other layers, enforced at events. The jurisdiction layer is load-bearing and routinely underestimated.

Explicit objects throughout. An implicit learned world model cannot be audited, so it is ruled out for decision support, though learned components may sit inside a module.

Core abstractions

Shared world state. Parallel arrays and graphs with explicit units. Three kinds of state kept apart: module-private (opaque, snapshotted), shared world state (entities and attributes, written only through timestamped events), and reference data (large, read-mostly, reached through a query interface).

Scheduler. One virtual clock. Each behavior advances at its own rate under a conservative rule: nobody advances past the slowest neighbor who might write what you read. Order is a tuple (virtual time, priority, sequence), so the same inputs and seed give the same run. Modules never sleep or poll; they react to events and schedule more.

Translators. Conserving conversions between representations: field to network, cohort to agent, agent to field, coarse to fine and back. Scale switching is the genuinely unsolved mechanism: a wolf population is a density field at continental scale and individuals in one basin, and the switch has to conserve the count. Same mechanism later lets households be a density at planet scale and agents in a city.

Ledgers. Mass, water, energy, carbon, nitrogen, money, people. Every ledger closes, every month, across every representation boundary and every scale level. The ledgers are both the test of the coupler and the scoreboard: net preserved diversity is a population ledger, cost of living and quality of life are agent ledgers.

Thresholds. Communities and ecosystems stretch, then tear. Cloth simulators fail at tearing because the representation has no tear. State variables carry explicit thresholds; the scheduler detects a crossing and fires a discrete event (zero-crossing detection, as in hybrid simulation). Regime shifts in ecology (forest to savanna) and a town that stops being a town use the same mechanism.

Event sources. Two flavors of one table: event, consequence, module addressed, source, confidence band, and either a date or a rate.

The calendar holds known future events: celestial mechanics (eclipses, tides, launch windows, satellite deorbits), demography (everyone who will be sixty in 2086 is already born), commitments and law (treaty deadlines, emissions targets, zoning effective dates, debt maturities, sunset clauses), built capital with lead times (plants, dams, transmission, transit; retirement years), recurring rhythms (elections, census, Olympics, harvests, religious calendars), and climate already in the pipeline. The calendar seeds the event queue before a run starts and gives the hindcast its honest 1990 knowledge.

Hazards hold possible random events with rates: solar storms (roughly one percent per year for a Carrington-class event), earthquakes and eruptions from recurrence tables, floods and hurricanes with drifting return periods, pandemics and crop pathogens, impactors and gamma-ray bursts with rates near zero, and human hazards (wars, coups, financial crises, grid failures) where the rates are guesses. A hazard is a module that reads world state and emits a rate; conflict rises with food prices, grid failure with heat waves. Each hazard has three settings in the manifest: off, on at base rate, or forced at a chosen date for a what-if.

Ensembles. The native unit of output is a thousand seeded branches, reported as a distribution with each branch's event log attached. Any branch replays exactly from its seed.

Interventions. A manifest edit at a date (a treaty in 2031, a cover-crop subsidy in 2028) is a one-line diff and a rerun.

Manifest. Declarative wiring: modules by version, sockets bound, parameters, data sources, clock mode, calendar and hazard settings. Heavy code lives in modules; the arrangement is a document.

Event log. Append-only, with periodic snapshots. Replay from any snapshot, fork a branch at any instant, diff two branches on one topic. The log is a dated event list, which is also the output format: an LLM narrates a branch as prose with each sentence linked to the ledger that produced it.

Validation is the scarce resource

Agents make code cheap, so the frontier moves to scoring. Every chain needs a scoreboard reality keeps on its own. Weather forecasting works because of data assimilation (pull model state toward observations) and verification (score every forecast). Both belong in the core, beside the scheduler.

Hindcast harness: start in 1990 with 1990 knowledge and calendar, run to 2020, check whether 2020 lands inside the bands. World3 (1972) is the precedent for a coarse whole-system model holding for fifty years.

Scoreboards available now: USGS May nitrate loads and the July hypoxia cruise (since 1985); MODIS leaf area and greenness (since 2000); GEDI canopy height; forest loss alerts; eBird; FAO stocks and prices with three food price crises (2008, 2011, 2022); shrimp size-class prices (1990 to 2010).

Reference models to port, not reinvent

Each is a module an agent writes against the published equations, proven by matching the original's outputs. SWMM (drains), InVEST (urban cooling, stormwater, habitat), SPARROW (nutrient delivery by reach), NOAA's hypoxia ensemble, LPJ-GUESS or CLM plant functional types (vegetation), NPZD (plankton), Madingley (fauna cohorts), Rothermel (fire spread), SUMO or MATSim (traffic), UrbanSim (buildout), Whitebox (hydrology tools, already Rust), DSSAT (crops), FMI 3.0 for importing existing engineering models.

Derisk sequence

Seam tests first, each a week or two, each ending in a ledger that closes:

1. Two fields at different rates (sun and surface temperature), energy conserved.
2. Field to network: rain to runoff to river reaches, water conserved.
3. Network to population: river stage to a floodplain forest cohort.
4. Population to agents: forest to a logging firm whose decision changes the forest.
5. Agents to rules to fields: a zoning change alters impervious surface, which alters runoff. The hard one.
6. Nested scales: the same code at parcel, basin and continent, coarse results driving fine boundaries, counts conserved.

Life layer, continuing from the abiotic stack already explored (planet-wide irradiance, hydrology, DEM-derived layers):

1. Plant functional types as fields driven by climate fields, scored against MODIS. Proves reaction coupling with two-way feedback (albedo, evapotranspiration, runoff).
2. Madingley-style cohorts in cells. Proves a second representation in the same store.
3. Zoom one basin to individuals and back, count conserved. Proves nesting.
4. A land-use agent that writes a field. Proves the human seam.

Worked chains with real scoreboards:

Gulf hypoxia. County fertilizer and manure, crop calendar, tile drainage and runoff, SPARROW reach routing, USGS outlet loads (calibration), summer weather as two hundred branches, hypoxia ensemble, July cruise (scoreboard), shrimp size-class prices, and a crop disposition ledger (feed, ethanol, export, food). Forty seasons to hindcast. The 2026 forecast miss (7,027 sq mi forecast, 1,332 measured after a storm) is the argument for ensembles.

Food system. Multi-breadbasket failure, stock drawdown, price spike, export bans (observed 2008, 2011, 2022), consumption loss in import-dependent countries, mortality. Every link has a model and data. This is the "Hunger Years" mechanism from fiction, with the model saying how much shock it takes.

Open questions

Determinism under parallel execution (event order from the tuple, never from finish order; property tests from day one). Numerics placement (which modules need native or GPU). Scale-switching with conservation (no general solution exists). Whether the social layers can be scored at all before a hindcast harness exists.
