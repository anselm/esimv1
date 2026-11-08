<!-- order: 1 -->
<!-- section: Standards -->
# RFC 0001: a common transport for simulations

Status: draft 3, 2026-11-07. Author: Anselm Hook, drafted with Claude in the esim session
of that day. Adopted by no engine yet, including this one.

This is an argument, with a schema attached. It proposes that engines which couple
modules of science on one clock should exchange their simulations as a single document,
the manifest, in a shape they all read. The full shape is the JSON Schema at
[`/docs/rfc/manifest.schema.json`](/docs/rfc/manifest.schema.json); this text gives the
reasons, the outline and the one page of rules that make it a transport rather than a
configuration file. esim writes its manifests in TOML, and the shape here is abstracted
from that practice, not copied from it.

## 1. The case

Everyone who builds a whole-system simulator builds the same four things. A clock. A set
of modules, each owning one piece of the world, a river, a carbon cycle, a crop, the
people of a country. Some way for the modules to affect each other. And a way to say what
is in the run and what it assumes. The first three are the engine, and engines will
compete: on speed, on the fidelity of their modules, on their records, on their pictures.
The fourth is a document, and today each engine writes it in a dialect of its own.

The document is the only part worth exchanging. A scenario is a claim: these modules, with
these parameters, over this window, coupled this way, say the planet is 2.1 °C warmer in
2100 with emissions at zero by 2050. Everything that makes the claim arguable is in the
arrangement. Which modules were included and which left out. Which numbers were set and
which were the author's guesses. What the modules were allowed to tell each other. A
reader who has the arrangement can argue with the claim; a reader who has only the chart
cannot.

Today the arrangement is locked inside the engine that ran it. To criticise a scenario you
must install the vendor's software, learn its dialect, and trust that what it shows you is
what it ran. A common transport changes five things:

- **A scenario can be read before it is run.** Someone with no engine at all can open the
  document, see the modules and the couplings, and draw the graph.
- **It can be checked without being run.** Whether every coupling has both ends, whether
  two modules fight over one field, whether a required input is missing, follows from the
  document and the modules' own self-descriptions.
- **It can be run elsewhere.** A second engine with the same kinds, or with its own kinds
  under the same names, runs the same arrangement, and the two results can be put side by
  side.
- **A disagreement becomes a finding.** When two engines run one manifest and differ, the
  difference is about the models, not about the file format. That is the comparison the
  field keeps saying it wants and never gets, because no two groups ever ran one
  arrangement.
- **Guesses travel as guesses.** The transport carries the provenance of every number, so
  a parameter nobody has sourced cannot arrive at the next engine looking like one that
  was measured.

Formats have done this before for other arts. Images, 3D assets and musical performances
each had a decade in which every vendor's files were an island, and each got a transport
that did not try to standardise the tool, only what passed between tools. Simulation of
the whole system is at the start of that decade.

## 2. Two levels

The grammar has two levels, and most of the confusion in this field comes from writing
them as one.

The lower level is the **meta grammar**: how to express a collection of state succinctly
at all. Entities, components, identity, order, includes, inheritance, variables, loops,
the expressions a document may carry, how it is made canonical and hashed. Nothing at this
level knows what a river is. It is the same question a scene description, a game level, a
configuration system and a user interface layout all answer, and the answers are old.

The upper level is the **world-modelling components**: the run window, the place, the
module and its kind, the ledgers of conserved quantities, the shared fields, the topics,
the records read, the variants of an assumption. This is the vocabulary of a simulation of
the world, and it is the part that competing engines have reason to argue about.

Part I of this document fixes the meta grammar. Part II names the components. An engine
that adopts Part I and only some of Part II still interoperates at the lower level: it can
read, draw, check and hash the document, and say which components it has no system for.

## Part I: the meta grammar

### 3. Entities and components

A manifest is a list of **entities**. An entity is a JSON object with an `id` and any
number of **components**, each a named key whose value is plain data. There are no classes
and no types: an entity is what its components say it is. One with a `kind` component is a
module instance; one with a `run` component is the run; one with a `variant` component is
a variant. A system, in the engine, acts on every entity that has the components it needs
and ignores the rest.

```json
{
  "manifest": "manifest/1",
  "vocabulary": "https://esimv1.exe.xyz/docs/system/vocabulary",
  "entities": [
    { "id": "run", "run": {
        "start": "1850-01-01", "end": "2100-01-01", "seed": 1, "close_every": "30d",
        "notes": "The planet core with people on half-degree cells, 1850 to 2100. Emissions after the record follow the variant's assumption. An assumption, never a forecast." } },
    { "id": "story", "kind": "story.bottomline" },
    { "id": "emissions", "kind": "carbon.emissions", "params": { "after_record": "hold" } },
    { "id": "pools", "kind": "carbon.pools" },
    { "id": "forcing", "kind": "forcing.ar6" },
    { "id": "energy", "kind": "energy.twolayer" },
    { "id": "people", "kind": "people.countries", "params": { "res_deg": 0.5, "distribution": "worldpop" } },
    { "id": "heat", "kind": "people.heat", "params": { "res_deg": 0.5, "niche_c": 29.0 } },
    { "id": "tipping", "kind": "tipping.elements", "params": { "coupling": 0.0 } },
    { "id": "audit", "audit": {
        "scores": [["energy.gmst_anom_c", "obs.hadcrut5_anom_c"]],
        "shock_topics": ["shock.", "people.crossing"] } },
    { "id": "held", "variant": {
        "title": "Emissions held at the last recorded rate",
        "set": { "emissions.after_record": "hold" } } },
    { "id": "zero-by-2050", "variant": {
        "title": "Emissions to zero by 2050",
        "set": { "emissions.after_record": "zero_by:2050" } } },
    { "id": "hindcast", "variant": {
        "title": "The record only",
        "set": { "run.end": "2025-01-01" } } }
  ]
}
```

Four rules make this a transport rather than a pile of objects.

**Identity.** `id` is unique in the document, lower case, a letter first. Other entities
address an entity by it. A later entity with an earlier entity's `id` replaces it
component by component, which is how includes and variants work (section 5).

**Presence is type.** A reader asks "does this entity have a `kind`", never "is this a
module". A component the reader does not know is carried, never rejected; the checker
reports it as information, "no system observes `x`". This is the whole extension mechanism
and it replaces a namespaced escape hatch: an engine's private component is just a
component, and another engine sees it, keeps it, and says it did nothing with it.

**Order is layout.** The order of entities is the order a result is presented in. It must
not affect the result; an engine whose results depend on entity order is not conforming.

**Components are data.** A component's value is a JSON object, array, number, string or
boolean. No functions, no references by pointer, no scripts. What a component means is
Part II's business, or the engine's; what it is, is always data a stranger can print.

The same grammar expresses state. A snapshot is a list of entities with state components
(a stock, a field's version, a module's private state); an event is an entity-shaped
message with a topic. This document fixes the manifest and leaves the snapshot and the
log as an open question, but the reason to choose entities here is that one grammar can
carry all three, as orbital's manifests already do.

### 4. Who has done this before

Every field that exchanges simulations has met this problem, and the ones that solved it
made the same cut: a concrete document travels, and something richer writes it. Read on
2026-11-07, through search results and the pages named; where only secondary sources were
found the row says so.

| where | what travels | the lesson |
|---|---|---|
| Systems biology: SBML, [SED-ML](https://sed-ml.org/), Antimony | SBML is the model as XML with MathML for the equations; SED-ML (Level 1 Version 5) is the experiment: models, simulations, tasks, repeated tasks over ranges, data generators, outputs. [Antimony](https://tellurium.readthedocs.io/en/latest/antimony.html) is the human text that converts to and from SBML, because SBML "was not designed to be readable or writable by humans". | The experiment is a document separate from the model, and it has one declarative loop: a repeated task over a range. People write a language; machines exchange the XML. |
| Engineering co-simulation: FMI and [SSP](https://www.prostep.org/mediathek/fact-sheets/system-structure-and-parameterization-ssp) (Modelica Association, SSP 1.0 in March 2019) | A unit of model as a container with a self-description, and an XML system structure of components, their connections and their parameters. | The self-description is what makes the structure file checkable; one paper notes importers trust the unit's own ports over the structure's. Connections are scalar. |
| Climate: [OASIS3-MCT](https://www.cerfacs.fr/oa4web/oasis3-mct/oasis3mct_UserGuide/node32.html), [ES-DOC](https://wcrp-cmip.github.io/WGCM_Infrastructure_Panel/Papers/CMIP6_ESDOC_documentation.pdf), CMIP6 vocabularies | The coupler's text file names the fields exchanged, their periods and remappings. ES-DOC's CIM documents models and experiments after the fact, not runnably. CMIP6 moved its controlled vocabularies and variable tables to JSON. | The field has a coupling file per coupler and a documentation standard, and no transport between engines; the vocabulary problem was solved by committee in JSON. |
| Distributed simulation: HLA, IEEE 1516 | The Federation Object Model: the classes, interactions and data types a federation exchanges. | A standard can govern the vocabulary of a federation without governing its members. |
| Agent-based models: the [ODD protocol](https://www.jasss.org/23/2/7.html) (2006, updated 2010 and 2020) | Prose under seven headings. The 2020 update argues for standard descriptions of simulation experiments beyond the model. | Prose did not make models comparable, and its own authors say so. |
| Neuroscience: [LEMS](https://pmc.ncbi.nlm.nih.gov/articles/PMC4174883) under NeuroML 2 | Component types with parameters, dynamics as equations and structure, with dimensions on every quantity. | Definitions separate from instances; units in the core. |
| Driving scenarios: ASAM OpenSCENARIO, [XML then DSL](https://www.asam.net/news-media/press-releases/detail/news/asam-openscenarior-1-and-2-split-into-separate-standards/) | 1.x was XML, "a low-level and concrete specification format, primarily designed to be read by simulation tools". 2.0 became a declarative language with parameters, constraints (`keep(t in [7..12.5]s)`) and composition, whose output is a "concrete scenario". The two are now separate standards. | The field found the concrete format too low to author in and too useful to drop. It kept both. |
| Visualisation: [Vega](https://vega.github.io/vega/docs) and Vega-Lite | A JSON grammar; Vega-Lite compiles to Vega; inside, expressions are "a restricted subset of JavaScript" with no assignment, no `new`, no control flow, and a fixed set of functions. | A declarative JSON document with a small expression language in strings is enough for a whole discipline, and a higher language compiles down to it. |
| 3D: VRML97 (ISO/IEC 14772) and X3D, [OpenUSD](https://nvidia-omniverse.github.io/LearnOpenUSD/composition-basics/strength-ordering.html) | VRML had a scene graph, `ROUTE`s between fields and a `Script` node in ECMAScript. USD has layers, references, payloads and variant sets with a fixed strength order. | Once a script node is in the file, interchange means "whatever the viewer runs". USD put composition and variants into the data model instead, and that is what made it a transport. |
| Configuration: [Jsonnet](https://jsonnet.org/articles/design.html), [Starlark](https://github.com/bazelbuild/starlark/blob/master/spec.md), CUE | Jsonnet is a pure functional language that emits JSON; every JSON document is a Jsonnet program; Turing complete by choice. Starlark is a Python dialect that is "deterministic and hermetic", with finite execution and "no recursion or unbounded loops". CUE unifies types and values as constraints. | The industry settled on hermetic programs that generate JSON. None of them put the program into the JSON. |
| Games: the entity component system | A pattern from game engines (its origin in the early 2000s is recalled, not checked): no classes, only instances; an instance is a bag of named components, plain data; systems act on every instance that has the components they need. Unity's DOTS, Bevy and flecs are current engines built on it. | Type is presence. A thing is what its components say it is, and a new kind of thing needs no change to the grammar. |
| This author's own: [orbital-sim](https://github.com/orbitalfoundation/orbital-sim) and the [orbital bus](https://github.com/orbitalfoundation/orbital-bus) (2024 to 2026) | A manifest is a JavaScript module whose named exports are entities, each a bag of components, with an `inherits` path; the loader turns exports into messages, and listeners match a message by the presence of keys, not their values. The 2024 docs: "you have only instances, and instances are decorated with components"; "a manifest is only a series of events over time". The bus primer names the layers: the bus is the message substrate, ECS a layer above it, manifests an authoring convention above that. | The same grammar expresses the arrangement, the state and the messages. Variables and loops come free from the host language, and one manifest reads an environment variable, which is the hermeticity leak the others legislate against. |

### 5. Succinctness: authoring, includes and ranges

A fully unrolled list of entities is the right thing to exchange and the wrong thing to
write. A manifest with 177 countries, thirty seeds and a sweep over three parameters is a
program's output, not a person's. The meta grammar answers at three levels of power, and
keeps the transport at the lowest.

**Includes and replacement, in the document.** `include` names other manifests, folded in
first and in order; the including document's entities come after; an entity that restates
an earlier `id` replaces it component by component. That is the whole composition rule,
the same one OpenUSD and the orbital loader use, and it lets a short manifest change one
parameter on top of a shared bundle without any language at all.

**Ranges, in the document.** An ensemble of thirty seeds or a sweep over a parameter is
not thirty documents; the engine needs to know the runs belong together, as SED-ML's
repeated task does. A variant may carry a `range` instead of a fixed value:

```json
{ "id": "seeds", "variant": { "title": "Thirty members", "range": { "run.seed": { "from": 1, "to": 30, "steps": 30 } } } }
```

A range is the only loop the transport carries, and it unrolls trivially.

**Everything else, in a program that writes the document.** Variables for shared numbers,
loops over countries, functions that stamp out patches, inheritance: these live in an
authoring language, and the standard does not pick it. It requires one thing: the program
is hermetic. It reads no clock, no environment, no network and no file outside the sources
it declares, draws no randomness beyond the run seed, and given the same sources produces
the same document. Starlark and Jsonnet have this by construction; JavaScript has it when
run in a sandbox that provides nothing else, which is orbital's form and the one the
reference implementation will take first: a module whose named exports are the entities,
evaluated with no globals but the seed and the declared sources. The resolved manifest
records where it came from:

```json
{
  "from": {
    "language": "javascript",
    "entry": "planet.manifest.js",
    "hash": "sha256:4c7a…",
    "sources": { "countries.json": "sha256:a0b1…" }
  }
}
```

With that block anyone with the same interpreter regenerates the same document and
checks the id; anyone without it still has the document.

### 6. Math at run time

Some numbers cannot be unrolled by any generator because they depend on the run:
emissions as a function of the year and the warming so far, a rule that acts once a
threshold is passed, a parameter that follows a series another module publishes. For
these a component may carry an expression as a string, in a language with Vega's
restrictions: arithmetic, comparison, conditionals, a fixed set of functions (`min`,
`max`, `abs`, `exp`, `log`, `pow`, `clamp`, `step`), named inputs, and nothing else. No
assignment, no loops, no recursion, no calls into the host; every expression is total and
evaluates in bounded time in IEEE double precision. The owner of the component says which
values are expressions and which names are in scope; for a module's parameters that is the
kind's describe:

```json
{
  "name": "after_record",
  "type": "expr",
  "vars": ["year", "last", "gmst_c"],
  "default": "last",
  "unit": "GtC/yr",
  "note": "emissions each year after the record; last is the last recorded rate",
  "provenance": "checked"
}
```

and a manifest writes, in place of the reference implementation's `zero_by:2050`:

```json
{
  "id": "emissions",
  "kind": "carbon.emissions",
  "params": { "after_record": "max(0, last * (1 - (year - 2024) / 26))" }
}
```

The checker verifies that every name an expression uses is in scope and that the string
parses. An implementation may keep the parsed form as a JSON tree beside the string
(MathJSON is one such encoding), but the string is the transport, because a person has to
read it. This is as far as the document goes toward being a program. VRML put the script
in the file and lost the transport; USD and Vega kept it out and kept theirs.

### 7. Resolution, canonical text and identity

A configuration file depends on the machine that reads it: its defaults, its library, its
copy of the data. A transport cannot.

**Resolution.** Before a run the engine folds the includes, replaces entities by `id`,
copies the place down into every module that does not set its keys, fills every parameter
from the kind's default, applies the variant, and writes the result as the run's own
manifest. That document is complete: the engine must be able to run from it alone, and a
reader must see every number the run used without knowing any engine's defaults.

**The resolved form carries what it rests on.** `kinds` holds the self-description of
every kind used (section 9), so a reader can check the wiring with no engine. `locks`
holds the content hash of every record read. `from` holds the program that wrote it.

```json
{
  "locks": {
    "owid-co2": { "url": "https://example.org/owid/co2.csv", "hash": "sha256:9b1e…", "bytes": 4120377, "fetched": "2026-11-01" },
    "hadcrut5-annual": { "hash": "sha256:51aa…" }
  }
}
```

(Hashes are shortened in the examples.)

**Canonical text and identity.** The resolved manifest is serialised by the JSON
Canonicalization Scheme, RFC 8785, and its id is the SHA-256 of that text. Two resolved
manifests with one id are one manifest, whatever files they came from and however their
keys were ordered. A run is identified by its manifest id and the digest of its event log.

## Part II: the components for modelling the world

### 8. The components this standard names

Part I lets any engine carry any component. Part II names the ones a simulation of the
world needs, so that two engines mean the same thing by them. The schema has the full
shape; this is the list.

| component | on an entity that is | what it says |
|---|---|---|
| `run` | the run, one per manifest | `start`, `end` as UTC dates or RFC 3339 with `Z`; `seed`, the only source of randomness, each module's stream derived from it and the module's `id`; `close_every`, the ledger closing cadence; `snapshot_every`; `notes` for a person |
| `place` | the place, at most one | a window on the WGS 84 graticule, `lat_s`, `lat_n`, `lon_w`, `lon_e`, and `res_arcsec`; copied at resolution into every module that does not set the same keys, so spatial modules share one grid |
| `kind` | a module instance | the kind, `family.kind`, in the shared vocabulary |
| `params` | a module instance | only the parameters the kind's describe lists, in its units; numbers, strings, booleans, arrays, or expressions where the describe says so |
| `ref` | a module instance whose kind the engine lacks | a URL and a content hash of an implementation, with a binding hint: `native`, `wasm` or `process` |
| `audit` | the audit, at most one | `scores`, pairs of series names, simulated then observed; `shock_topics`, the topics that count as shocks; advisory, never affecting the run |
| `variant` | a variant | a `title`, and `set` (overrides, `run.<field>` or `<id>.<parameter>`) or `range` (section 5); a variant may not add an entity or change a kind |

Three things in the engine model are not components because they are not stated in a
manifest: **ledgers** (a module opens one by name and unit, and two modules that name one
ledger share it), **fields** (one writer, any readers, on the place's grid) and **topics**
(events between modules). They are declared by kinds in their describes, derived by the
checker as the wiring, and governed by the vocabulary, which is where the real
interoperability argument between engines will happen. A manifest names kinds; kinds name
the rest.

### 9. The describe and the check

Checking depends on what each kind says about itself, so the describe's shape is part of
the standard. An engine must produce one for every kind in its library from a name and
parameters alone, without building the module or fetching a record: the kind and a
summary; its priority among modules acting at one instant; its parameters with defaults,
units, provenance and, for expressions, the names in scope; the topics it publishes and
subscribes to; the fields it owns and reads; its ledgers; its outputs; the records it
reads.

From the entities and the describes a checker derives the wiring and reports three grades.
Errors, which make the manifest unrunnable: an unknown kind with no `ref`, a parameter the
kind does not list, an expression over a name not in scope, two writers of one field, a
required read nothing writes, a read in one unit of a field written in another, a message
to an entity not present, one ledger in two units, two entities with one `id` after
resolution. Warnings: a reader whose priority puts it before its writer at the same
instant, so it sees last tick's value; a guessed parameter the manifest left at its
default. Information: optional couplings not present, subscriptions nobody publishes,
outputs nobody reads, components nothing observes. The information grade is the
manifest's own account of what it could gain, and it is the line a reviewer reads first.

## Part III: what an engine promises

### 10. In what sense the same run

The hardest question about any transport is what it promises, and the honest answer is a
ladder. Two engines agree about a manifest at one of five levels, each including those
before it.

1. **Readable.** Both parse it, resolve it the same way and compute the same id. Testable
   with no modules at all.
2. **Checkable.** Both derive the same wiring from the `kinds` block and report the same
   errors and warnings. Also testable without modules.
3. **Runnable.** Both have, or fetch, every kind, and run to the end with every ledger
   closing within tolerance.
4. **Comparable.** Both report the series the audit names and the crossings the kinds
   publish, and a person can put them side by side. The numbers need not agree. The
   shape of the record does.
5. **Reproducible.** Both produce the same event log digest. This needs bit-identical
   module semantics, the same random generator and the same floating point behaviour,
   and is a property of two engines that share modules, not of the standard. No engine
   may claim it on the standard's authority.

Most of the value is at levels one to four. A manifest written in one shop can be read,
drawn and criticised in another before any code moves; when both run it, what differs is
the science. Level five is where competing vendors will never meet, and the standard does
not need them to.

### 11. One rule about honesty

Every parameter in a kind's description carries a provenance: `checked`, read from its
source and compared; `recalled`, from memory and not yet checked; `guess`, sourced by
nobody; `fitted`, set by a stated method against a record. The transport requires it for
every number with a physical meaning. This is the one place the document legislates
conduct rather than shape, because a manifest that lets a guessed number travel as a
checked one defeats the purpose of exchanging manifests at all. A chart does not carry
this; a document can.

### 12. What it leaves out, on purpose

Not the authoring language: any hermetic program may write a manifest and the manifest
says which one did. Not the engine: how a kernel orders events, stores fields or closes
ledgers is the vendor's. Not the protocol between a kernel and a module, which is a
second document and lets a module be compiled in, loaded as WebAssembly or run as a
process. Not the vocabulary: whether two engines mean the same thing by
`energy.gmst_anom_c` is the real interoperability problem and belongs in a shared,
versioned list a manifest names by URL; this document only insists that the name be
carried. Not the run folder, the log, the frames or the scores. Not, yet, the snapshot,
though section 3 says why it should be the same grammar.

### 13. From practice

esim's TOML manifests map onto this form: `[run]` is the run entity, `[place]` the place,
each `[[module]]` with its `[module.params]` an entity with `kind` and `params`, `[audit]`
the audit; the instance name becomes the `id`. Three things differ. esim keeps variants in
the project beside the manifest, where this document puts them in it, because a family of
assumptions is the thing worth sending. esim keeps the kind descriptions and the data
locks in files beside a run, which is the same information in a shape a reader elsewhere
cannot use. And esim's include can be spliced at a point in the module order, which this
document does not yet have.

The reference implementation does not yet read this form. That is deliberate: the
argument should stand before the code follows it.

### 14. Security

A manifest can name code and data by URL. An engine verifies the content hash before
using either, and runs a fetched module with no more authority than the module protocol
grants: its parameters, its seed, its events, the fields it reads. Components are never
evaluated; expressions are evaluated only by the expression language of section 6, never
by the host's. A manifest from a stranger is a document to check, not a program to run;
levels one and two are safe on any input, level three is not.

### 15. Open questions

- Whether `place` should be a general grid declaration rather than a geographic window,
  so that a tank or an abstract lattice is first class.
- Whether a variant may add an entity, which would make "with and without this coupling"
  one document at the cost of variants no longer being a table of differences.
- The expression language's exact grammar and function list, and whether a parsed form
  travels beside the string.
- Whether the reference implementation's first authoring language should be JavaScript in a
  sandbox (orbital's form, runnable in the viewer) or Starlark (hermetic by construction).
- The snapshot and the log as entities: the same grammar for arrangement, state and
  messages, which is orbital's bet and would make a run folder one kind of document.
- Where the vocabulary lives and how a version of it is named.
- Whether the audit belongs in the manifest or in a document that names one.

## References

- The schema: [`manifest.schema.json`](/docs/rfc/manifest.schema.json).
- [The module protocol, version 0](/docs/protocol-v0), the second document.
- [Manifests](/docs/system/manifests), [Vocabulary](/docs/system/vocabulary),
  [Terminology](/docs/system/terminology): the practice this was abstracted from.
- The orbital pattern: [orbital-bus](https://github.com/orbitalfoundation/orbital-bus) (SPEC.md and PRIMER.md) and
  [orbital-sim](https://github.com/orbitalfoundation/orbital-sim); the 2024 orbital docs on entities and manifests, read on disk.
- RFC 3339 (timestamps), RFC 8785 (JSON canonicalization), JSON Schema draft 2020-12.

## Changes

- Draft 1, 2026-11-07: first text.
- Draft 2, 2026-11-07: prior art (section 7); authoring, ranges and run-time expressions
  (section 8) after Anselm's note that a fully unrolled grammar cannot express variables and
  loops and that orbital-sim's manifests were JavaScript used declaratively; `from`, `range`
  and the `expr` parameter type in the schema.
- Draft 3, 2026-11-07: two levels (Anselm: "there is a meta level - variables, loops, includes,
  layout. Also I used an ecs pattern ... then there are the structures we are building in the
  grammar - related to modeling the world"). Part I is the meta grammar: a manifest is a list of
  entities, each a bag of components, presence is type, unknown components are carried not
  rejected; Part II names the components for the world. The schema follows.
