<!-- order: 2 -->
# Terminology

The words, fixed. Where two words have been used for one thing, the first is the one to
use from now on.

**Kernel.** The coupler: one virtual clock, the event queue, the shared world store, the
ledgers, the log, snapshots and replay. It knows nothing about water or deer. Also called
the harness in older notes; kernel is the word.

**Module.** One piece of science or one piece of someone else's engine, behind a small
contract: describe itself, build from parameters, initialise, handle events, snapshot and
restore. A module owns its private state and touches nothing else directly. "Compute
module" and "agent" in earlier notes mean this.

**Kind.** A module type, named like `water.basin` or `people.heat`. A manifest
instantiates a kind under a name of its own; one kind can appear several times in one
manifest under different names and parameters.

**Instance.** One named module in a running manifest, for example the `water.basin`
instance called `water`. Other modules address it by that name.

**Describe.** What a kind says about itself from parameters alone, without building or
fetching: parameters with defaults, units and provenance; topics published and
subscribed; fields owned and read; ledgers; outputs; data assets. The kernel checks a
manifest's wiring from the descriptions, and a model reads them instead of source.

**Layer.** A loose grouping of kinds by what part of the world they integrate: solar,
atmospheric composition, energy balance, abiotic earth systems, biogeochemistry, primary
producers, consumers, people and their systems. A layer is a way of talking, not a
construct in the code; a kind is "in" a layer because of what it reads and owns.

**Patch.** Anything bound to one place: a single module that handles an area (the
Amazon basin as one forest module), or a bundle of modules for an area (the Boone River
basin with its weather, water, crop, animals and farms). A module need not span the
planet; a manifest is the grouping. A bundle that includes many patches (Amazon, Congo,
the boreal belt) is a world-spanning layer another manifest can include for, say, forest
cover. A planetary simulation is patches on one clock, coupled through shared global
state and boundary fluxes; the country layer is 177 very coarse ones.

**Ledger.** The book a run keeps for one conserved quantity: water in cubic metres,
carbon in tonnes, energy in joules, animals in individuals, nitrogen in kilograms.
Every move is written twice, out of one account and into another, and on a cadence the
kernel closes the book and checks that nothing appeared or vanished; a leak above
tolerance fails the run. Several modules can hold one ledger, which is how emissions,
the carbon cycle and a future permafrost module would share one atmosphere. The auditor
reads the closes to see what was spent down and what built up.

**Coupling.** A relationship between two modules, in the only form the kernel has for
one: one module writes a named thing (a topic, a field, a ledger entry) and another
reads it and changes its own state because of it. "Ice sheets affect sea level" is a
coupling only when a module that knows the ice publishes something the sea module
subscribes to. A relationship the world has and the run lacks is a missing coupling.

**Feedback.** A coupling that runs back up the stack: warming changes the sinks, which
changes CO2, which changes warming. The basin has several; the planet stack, as of
2026-11-02, has one.

**Cascade.** A coupling with a consequence: a crossing that moves a stock someone reads,
rather than a flag. The first one is `carbon.tipped`, which turns a tipping element's
crossing into carbon in the carbon ledger, which the pools module turns into CO2, which
the energy module turns into warming, which the tipping module checks again. Each step is
a module's own, declared, auditable at its ledger; the cascade is the path through them,
and the wiring graph draws it as a loop.

**Wiring.** All the couplings of a manifest, derived from the describes before anything
runs. `esim check` reports the wiring as errors, warnings and information; the run page
draws it as a graph, modules as boxes, couplings as arrows, ledgers as shared books,
with arrows that end in air for outputs nothing reads. The word is from the check, not
from the literature; "coupling graph" means the same.

**Manifest.** The TOML document that fully states a simulation: the run window, seed and
cadences, an optional place, the modules with their parameters, and the audit's
scoreboards. A manifest may include other manifests. The resolved manifest text is the
whole state a run depends on, besides the records it fetches by hash.

**Bundle.** A manifest written to be included by others, such as `planet-core.toml`.
"Layer ensemble" and "nested manifest" mean this.

**Variant.** A named set of parameter overrides a project applies on top of its manifest,
such as emissions held or emissions to zero. A variant is how one project holds several
assumptions without several manifests.

**Scenario.** A manifest plus the assumptions written into it, as a thing a person wants
to see: "the Boone basin through a dry summer", "the planet with emissions held". The
word is for conversation; the artefact is the manifest.

**Project.** A folder under `projects/<area>/<name>/`: `project.toml` with the title, the
person's description in their own words, who and when, and any variants; the manifest;
and `runs/`. The folder is the project.

**Area.** The first path segment under `projects/` or `docs/`: a person's name or a
topic, claimed by whoever writes there first. Areas stand in for accounts until there are
accounts.

**Run.** One execution of a manifest, written to a folder named by the manifest's hash
and the log's digest: the resolved manifest, the event log, series, map frames, ledgers,
scores, the audit, and the variant if any. A run is replayable from its manifest and
seed; two runs of the same manifest and seed produce the same digest.

**Ensemble.** Many runs of one manifest with different seeds, kept as bands rather than
single lines, for the parts of a manifest that are stochastic (hazards, weather members).
This is the word's meaning in forecasting and it is kept to that here; a group of modules
is a bundle, not an ensemble, however natural the other use is.

**Record.** A real observation series or layer fetched through the registry: a gauge, a
station, a reanalysis, an ice core, a census. Never synthetic.

**Scoreboard.** A record reported by a module as a series so that a model series can be
scored against it, such as the gauge beside the simulated discharge.

**Score.** Skill of a model series against a scoreboard at matching times: Nash-Sutcliffe
efficiency, Kling-Gupta efficiency, correlation, bias, root mean square error.

**Audit.** What the independent auditor says about a finished run: stocks spent or
built, diversity, shocks and recovery, crossings, tears, and a verdict of holds,
stretching or tearing.

**Stretching.** A system absorbing a shock and returning: a stock that drops after a
shock and is back within a band of its pre-shock level before the next shock arrives.
Measured by the auditor on ledger stocks against shock episodes in the log; the measure
is recovery time against recurrence interval and needs no model of the system.

**Tearing.** A shock a system does not recover from: a stock that has not returned to
its band before the next shock, or a year on with no shock following; an extinction; a
ledger that fails to close. Recovery slower than recurrence is the definition, and the
verdict "tearing" is given only on that evidence. A crossing of a published threshold is
reported beside it, with its source, but is not itself called a tear until something in
the run fails to recover.

**Holds.** Neither: no shocks, or shocks that moved nothing.

**Crossing.** An event a module publishes the first time a stated threshold is passed: a
country leaving the human climate niche, the warming entering a tipping element's
range. Always with a source and a confidence.

**Ledger.** Double-entry accounts of one conserved quantity in one unit, closed on a
cadence; a module that holds a stock declares it and the kernel compares the declaration
with the flows. A leak above tolerance fails the run.

**Field.** A gridded quantity. A world field is shared state one module writes and others
read; a frame is a field snapshot written for maps.

**Event and topic.** A message at an instant, broadcast on a topic or sent to a named
instance, optionally about one grid cell. Topics and field names are the vocabulary; see
[Vocabulary](?doc=system/vocabulary).

**Registry and catalog.** The registry fetches and reads records; the catalog is its index
of sources with licence, verification date and notes; the lock file holds the hash of
every file a run used.

**Facade.** The conversational layer: a model that turns a description into a manifest
against the kinds' descriptions, checked by the kernel before anything runs.
