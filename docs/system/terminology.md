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

**Patch.** A place-bound set of modules: the Boone River basin with its weather, water,
crop, animals and farms is a patch. A planetary simulation is patches on one clock,
coupled through shared global state and boundary fluxes. The country layer is 177 very
coarse patches.

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

**Record.** A real observation series or layer fetched through the registry: a gauge, a
station, a reanalysis, an ice core, a census. Never synthetic.

**Scoreboard.** A record reported by a module as a series so that a model series can be
scored against it, such as the gauge beside the simulated discharge.

**Score.** Skill of a model series against a scoreboard at matching times: Nash-Sutcliffe
efficiency, Kling-Gupta efficiency, correlation, bias, root mean square error.

**Audit.** What the independent auditor says about a finished run: stocks spent or
built, diversity, shocks and recovery, crossings, tears, and a verdict of holds,
stretching or tearing.

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
