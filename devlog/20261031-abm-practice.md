# 2026-10-31 — How others do agent-based models, and whether they scale and couple

Anselm asked, same day: "how do others do agent based models? what are best practices?",
"can agent based models tackle bioregional models with potentially millions of agents?",
"can we couple agent based models with continuous data models such as irradiance, elevation,
hydrology?" Written by the agent from general knowledge of the field, not from a fresh
literature search; names of tools and methods are ones the agent is confident exist, numbers
are kept to what it is confident of.

## How the field does it

Frameworks, by lineage:
- Teaching and small research: NetLogo (the lingua franca since the late 1990s), Mesa
  (Python), Agents.jl (Julia), GAMA (GIS-native, good at space).
- Serious research: MASON (Java), Repast and Repast HPC (C++, MPI, distributed), FLAME GPU
  (millions of agents on one GPU).
- Domain-specific and large: MATSim (transport, whole-city populations, every person has a
  daily plan and replans between iterations), UrbanSim (land use and real estate), Covasim
  and OpenABM-Covid19 (epidemics, national populations), EpiSimdemics and its descendants
  (epidemics at the scale of the whole United States on supercomputers), Madingley (global
  ecology, every animal on Earth as cohorts).

Practices that are now standard among the people who do this well:
- Describe the model with the ODD protocol (Overview, Design concepts, Details; Grimm and
  colleagues, 2006, revised since). It is the reason two ecologists can read each other's
  ABMs. Our manifest plus module docs should be ODD-shaped.
- Pattern-oriented modelling (Grimm and Railsback): calibrate and validate against several
  observed patterns at once, at different scales, not one time series. A model that matches
  three unrelated patterns is believed; one that matches one is a fit.
- Separate the schedule from the agents. Time-stepped (everyone acts each tick) for simple
  models; discrete-event (agents schedule their next action, a priority queue runs them in
  time order) once rates differ. MATSim and the epidemic models are event-driven.
- Entity-component layout, parallel typed arrays, no object per agent once counts pass a few
  hundred thousand. This is the game-engine ECS idea and FLAME GPU's whole design.
- Seeded randomness, deterministic ordering, and replay. Treated as hygiene, not a feature.
- Sensitivity analysis and ensembles over single runs. One run of a stochastic ABM is an
  anecdote.
- Calibrate against data with approximate Bayesian computation or history matching, not by
  hand. Hand tuning is the field's chronic disease.
- Keep agents stupid. Rich cognition inside agents is where ABMs go to die; decision rules
  of a few lines, checked against behavioural data, outperform clever ones.

The field's known failures, which the brief should avoid: models nobody can reproduce, models
validated against the data they were tuned on, "toy" models that never meet an observation,
and the opposite, giant models with thousands of parameters that fit anything.

## Millions of agents at bioregional scale

Yes, and it is done routinely, but with three tricks, not brute force.

- Brute force is possible to a point. MATSim runs full metropolitan populations, FLAME GPU
  and Repast HPC run millions, the epidemic models run hundreds of millions. In Rust with
  typed arrays, ten million simple agents stepping daily is a laptop problem; the memory is a
  few gigabytes and the step is seconds. The limit is not the agents, it is what they read:
  if every agent queries a field, a spatial index is mandatory.
- Super-individuals (Scheffer and colleagues, 1990s): one record stands for n identical
  individuals, with n as a state variable. Madingley's cohorts are exactly this. A bioregion
  with a billion insects is a few thousand cohorts. This is the brief's population
  representation.
- Sampling: a synthetic population of one in ten or one in a hundred, reweighted. The
  epidemic and transport models do this when the full population is unnecessary.
- Nesting, which is the brief's scale switch: densities at the coarse level, individuals
  only where the question needs them, with the count conserved across the switch. No
  framework gives this for free; it is the mechanism this project has to build.

So the honest answer: millions of agents are not the risk. Millions of agents coupled to
fields with conservation, replayable, is the risk, and that is why the brief is about seams.

## Coupling agents to continuous fields

Yes. This is the oldest coupling in the trade and the best understood of the seams.

- Lagrangian on Eulerian: agents carry a position, fields live on a grid; agents read the
  field by interpolation, write to it by depositing into the cell. Every particle-in-cell
  physics code and every fish-larvae-in-ocean-current model works this way.
- Cell-attached agents: the agent belongs to a cell and reads the cell's state directly.
  Land-use change models (the CLUE family, SLEUTH, the GIS-coupled ABMs of the 2000s) and
  Madingley do this. Cheapest, and right for most of our cases.
- Rates as the interface: the field hands the agent a rate (mm of soil water per day, W/m2
  of irradiance), the agent hands back a flux (tonnes removed, litres pumped). Ledgers close
  because the interface is in conserved quantities, not in states.
- Co-simulation standards: FMI (engineering), ESMF and OASIS (climate model component
  coupling), HLA (defence). All solve the many-rates, one-clock problem with a coupler that
  owns the clock and interpolates in time. The brief's conservative scheduler is the same
  idea as Chandy-Misra-Bryant null messages and the conservative branch of parallel
  discrete-event simulation (the optimistic branch being Jefferson's Time Warp).

The elevation layer is the easy case: static reference data, read-only, queried by cell.
Irradiance is a field with a daily and annual cycle and almost no feedback from agents, so it
is a boundary condition. Hydrology is the real seam: agents change land cover, land cover
changes runoff, runoff changes what agents can do, and the water ledger has to close across
cells, reaches and withdrawals. Seam test 2 and seam test 5 in the brief are that coupling.

## What this means for the design

Nothing in the brief is novel in parts; every piece has a precedent and a name. The
unprecedented part is having all five representations in one store under one clock with
ledgers. That is a systems-integration bet, not a research bet, which is the good kind for a
product. The research-grade unknown remains scale switching with conservation.
