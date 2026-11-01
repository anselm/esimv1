# 2026-11-01 — This effort against the digitaltwins.wiki engines shelf

Anselm: "On the digitaltwins.wiki site I recently sorted the engines page so we can clearly
see what the most popular engines are - how does this nascent effort compare? What can we
grow into here?" Agent's assessment, read from the twin-compendium repo's
`devlog/20261101-engines-by-standing.md` (tiers by evidence, 2026-11-01) and
`devlog/20261031-engines-by-method.md` (sort by method, 2026-10-31).

## Where it sits today, by his own yardstick

The standing tiers rest on releases, citations, named users and access. This repo is two
days old, private, has no release, no citation and no user but its author. On that
evidence it is tier 4, below "New or unproven". That is the correct reading and no
discouragement; it is the same tier Generative Agents' code sits in.

Module by module, each of the twenty Widely used engines beats the matching module here
by decades:

| domain | widely used engine | here |
|---|---|---|
| river and basin water | National Water Model, Delft3D | bucket + D8 routing, NSE 0.47, structural ceiling |
| crops | DSSAT | GDD phenology + LUE, canopy a month late, a third of real biomass |
| food webs | Ecopath with Ecosim | two cohorts, Holling II |
| ecosystem services | InVEST | none |
| traffic and people | MATSim, SUMO, UrbanSim | none |
| energy and economy | GCAM, PyPSA | money ledger on one farm block type |
| agents, general | NetLogo, Mesa, AnyLogic | kernel agents on a grid |
| system dynamics | Vensim | ledgers, informally |

Nothing here should be presented as competing with any row of that table.

## What the sort by method says about the gap

The method sort has a section F, "context-to-forecast whole-system engines", the class
Anselm named as the mission. Read strictly it holds one entry, MiroFish, a population of
language-model agents producing a report; nothing physical. The Widely used band holds
single-domain process models and agent toolkits and no whole-system engine at all.
Destination Earth is whole-Earth physics a user cannot feed context to. The coupling
frameworks in section A (CSDMS Basic Model Interface, OpenMI, Modelica and
ModelingToolkit) couple equations or process models but do not carry populations, agents
and rules on one clock with accounts that must close, and none is consumer-facing.

So the slot the brief describes is empty on the shelf, and the parts of this repo that
point at the slot are not the modules. They are:

- the kernel: one clock for five representations, conservative multi-rate, ledgers at
  every seam, append-only log with bitwise replay, ensembles as a first-class run;
- the registry: real terrain and observations fetched once by hash, with event tables;
- the validation habit: every rung scored against a gauge, a satellite or a station;
- speed: fifteen seconds a basin-year makes thousand-branch ensembles and what-ifs cheap,
  which Delft3D or DSSAT at full fidelity cannot offer;
- the facade still to come: English request to manifest to forecast with the surprises.

## What to grow into

Three paths, not exclusive.

1. **Harness, not zoo.** Treat the tier 1 engines as modules to wrap, not rivals to
   rewrite. The Basic Model Interface (CSDMS) is the established contract for exactly this,
   and several of the twenty already speak it or something close: WRF-Hydro behind the
   National Water Model, DSSAT's cropping system model, Ecopath, SUMO and MATSim through
   their own APIs. The kernel's contribution is then the clock, the ledgers, the data, the
   ensembles, the what-ifs and the facade. Cost: legacy Fortran and Java, licence mix,
   each wrapper its own project.
2. **Own light modules, composable and fast.** Keep the native modules as the "model
   library" layer, the way NetLogo ships models, accepting lower per-domain fidelity in
   exchange for whole-system coupling and speed. This is what the ladder did and it reached
   a real forecast in a day. Its ceiling is that no single module will ever be cited.
3. **Product class F.** Build the thing the shelf lacks: a person supplies a place and a
   question in English and gets a near-term whole-system forecast with the unexpected
   consequences, grounded in physics and observations rather than language-model agents.
   Per the wiki's own sort, no physical engine occupies this today.

Recommendation: 1 and 3 with 2 as the scaffolding. Modules are of two kinds, native for
speed and coupling, wrapped for fidelity where it matters. The first wrapper should be
chosen to kill a known weakness and prove the seam at once: DSSAT for the crop would fix
the canopy and most of the nitrate error, and SUMO would make "traffic in my town" real.
A BMI adapter is one kernel-side piece of work that serves both.

How this project climbs the wiki's own tiers is also its roadmap: tier 3 needs a public
repo and a release; tier 2 needs one outside user and one validation someone else can
rerun; tier 1 is years away and needs the wrappers. Whether and when it appears on the
shelf is Anselm's call; today it would be an honest tier 4 entry, or a Twin entry for the
Boone River, which the twins' bands would place under "Pilot or prototype".
