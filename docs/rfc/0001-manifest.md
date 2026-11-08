<!-- order: 1 -->
<!-- section: Standards -->
# RFC 0001: a common transport for simulations

Status: draft 2, 2026-11-07. Author: Anselm Hook, drafted with Claude in the esim session
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

## 2. What a manifest is

One JSON object. Here is a complete one, in the vocabulary of the reference
implementation; another engine's kind names would be its own.

```json
{
  "manifest": "manifest/1",
  "vocabulary": "https://esimv1.exe.xyz/docs/system/vocabulary",
  "run": {
    "name": "planet_people",
    "start": "1850-01-01",
    "end": "2100-01-01",
    "seed": 1,
    "close_every": "30d",
    "notes": "The planet core with people on half-degree cells, 1850 to 2100. Emissions after the record follow the variant's assumption; the default holds them at the last recorded rate. An assumption, never a forecast."
  },
  "modules": [
    { "name": "story", "kind": "story.bottomline" },
    { "name": "emissions", "kind": "carbon.emissions", "params": { "after_record": "hold" } },
    { "name": "pools", "kind": "carbon.pools" },
    { "name": "forcing", "kind": "forcing.ar6" },
    { "name": "energy", "kind": "energy.twolayer" },
    { "name": "people", "kind": "people.countries", "params": { "res_deg": 0.5, "distribution": "worldpop" } },
    { "name": "heat", "kind": "people.heat", "params": { "res_deg": 0.5, "niche_c": 29.0 } },
    { "name": "tipping", "kind": "tipping.elements", "params": { "coupling": 0.0 } }
  ],
  "audit": {
    "scores": [["energy.gmst_anom_c", "obs.hadcrut5_anom_c"]],
    "shock_topics": ["shock.", "people.crossing"]
  },
  "variants": [
    { "name": "held", "title": "Emissions held at the last recorded rate", "set": { "emissions.after_record": "hold" } },
    { "name": "zero-by-2050", "title": "Emissions to zero by 2050", "set": { "emissions.after_record": "zero_by:2050" } },
    { "name": "hindcast", "title": "The record only", "set": { "run.end": "2025-01-01" } }
  ]
}
```

Five blocks, and that is the whole grammar.

**run** is the window, the seed and the cadences. Times are UTC, dates or RFC 3339 with a
`Z`; a run that means a local time has a hidden dependency on where it was written. The
seed is the only source of randomness, and each module's stream derives from the seed and
the module's name, so adding a module changes nobody else's draws.

**place**, absent above, states a window and a resolution once for every spatial module,
so that they share one grid and can read each other's fields. It is copied into each
module's parameters at resolution time, and the copy is kept, so the rule need never be
applied twice.

**modules** are the instances, in order. Each has a name other modules address, a kind,
and only the parameters it means to set; defaults live in the kinds. A kind the engine
does not have may carry a `ref`, a URL and a content hash, to an implementation it can
fetch. The order is the layout of the result and must not affect the result: delivery
order is time, priority and sequence, never position in the file.

**audit** names the pairs of series to score, simulated beside observed, and the topics
that count as shocks. It advises the auditor and touches nothing in the run.

**variants** are named override sets, `run.end` or `instance.parameter`, so that one
document holds a family of assumptions, emissions held, falling, zero by 2050, as a table
of differences. A variant may not add a module or change a kind; a scenario that needs
that is a different manifest.

A document may `include` others, which fold in first; a module that restates an included
module's name replaces it, so a short manifest can change one parameter on top of a
shared bundle.

## 3. What makes it a transport

A configuration file depends on the machine that reads it: its defaults, its library, its
copy of the data. A transport cannot. Three rules close the gap.

**Resolution.** Before a run, the engine folds the includes, copies `place` down, fills
every parameter from the kind's default, applies the variant, and writes the result as the
run's own manifest. That resolved document is complete: the engine must be able to run
from it alone, and a reader must be able to see every number the run used without
knowing any engine's defaults.

**The resolved form carries what it rests on.** Two blocks appear only after resolution.
`kinds` holds the self-description of every kind used: its parameters with units and
provenance, the topics it publishes and subscribes to, the fields it owns and reads, its
ledgers, its outputs, the records it reads. `locks` holds the content hash of every record.
With these a reader anywhere can check the wiring and know which data was read, with no
access to the engine or its cache.

```json
{
  "locks": {
    "owid-co2": { "url": "https://example.org/owid/co2.csv", "hash": "sha256:9b1e…", "bytes": 4120377, "fetched": "2026-11-01" },
    "hadcrut5-annual": { "hash": "sha256:51aa…" }
  }
}
```

(Hashes are shortened in the examples.)

**Canonical text and identity.** For hashing and comparison the resolved manifest is
serialised by the JSON Canonicalization Scheme, RFC 8785, and its id is the SHA-256 of
that text. Two resolved manifests with one id are one manifest, whatever files they came
from and however their keys were ordered. A run is then identified by its manifest id and
the digest of its event log, and two people on two continents can say whether they ran
the same thing.

## 4. Checking without running

Because every kind describes itself, the couplings of a manifest follow from the document
and the descriptions, before anything is fetched or run. A checker reports three grades.
Errors, which make the manifest unrunnable: an unknown kind with no `ref`, a parameter
the kind does not list, two writers of one field, a required read nothing writes, a read in
one unit of a field written in another, a message to a module not present, one ledger in
two units. Warnings: a reader whose priority puts it before its writer at the same
instant, so it sees last tick's value; a guessed parameter the manifest left at its
default. Information: optional couplings not present, subscriptions nobody publishes,
outputs nobody reads. The information grade is the manifest's own account of what it
could gain, and it is the line a reviewer reads first.

## 5. In what sense the same run

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

## 6. One rule about honesty

Every parameter in a kind's description carries a provenance: `checked`, read from its
source and compared; `recalled`, from memory and not yet checked; `guess`, sourced by
nobody; `fitted`, set by a stated method against a record. The transport requires it for
every number with a physical meaning. This is the one place the document legislates
conduct rather than shape, because a manifest that lets a guessed number travel as a
checked one defeats the purpose of exchanging manifests at all. A chart does not carry
this; a document can.

## 7. Who has done this before

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
| This author's own: [orbital-sim](https://github.com/orbitalfoundation/orbital-sim) (2026) | A manifest is a JavaScript module: exported objects with an `inherits` path, run by Node. | Variables and loops for free, and one manifest reads an environment variable, which is the hermeticity leak the others legislate against. |

## 8. Authoring, transport and math at run time

A fully unrolled document is the right thing to exchange and the wrong thing to write.
A manifest with 177 countries, thirty seeds and a sweep over three parameters is a
program's output, not a person's. The prior art answers with one cut and one addition,
and this draft adopts both.

**The transport stays concrete.** What an engine runs, hashes and compares is the resolved
JSON of sections 2 and 3. No variables, no loops, no scripts: that is what makes it
checkable by a reader with no interpreter and the same on every machine. OpenSCENARIO's
concrete scenario, SED-ML's experiment and Vega's compiled grammar are this layer.

**Authoring is a program that writes it, and the standard does not pick the language.** A
manifest may be generated by anything that satisfies one rule: the program is hermetic. It
reads no clock, no environment, no network and no file outside the sources it declares,
draws no randomness beyond the run seed, and given the same sources produces the same
document. Starlark and Jsonnet have this by construction; JavaScript has it when run in a
sandbox that provides nothing else, which is how the reference implementation will take
orbital-sim's form: a module that exports the manifest, evaluated with no globals but the
seed and the sources. The resolved manifest records where it came from:

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
checks the id; anyone without it still has the document. Loops over countries, variables
for shared numbers, functions that stamp out patches: all live here, and the transport
never sees them.

**One declarative loop in the transport: ranges.** An ensemble of thirty seeds or a sweep
over a parameter is not thirty documents; the engine needs to know the runs belong
together, as SED-ML's repeated task does. A variant may carry a `range` instead of a fixed
value:

```json
{ "name": "seeds", "title": "Thirty members", "range": { "run.seed": { "from": 1, "to": 30, "steps": 30 } } }
```

A range is the only place the transport does not unroll, and it unrolls trivially.

**Math at run time is a small expression language, not a script.** Some numbers cannot be
unrolled by any generator because they depend on the run: emissions as a function of the
year and the warming so far, a rule that acts once a threshold is passed, a parameter that
follows a series another module publishes. For these the transport carries an expression
as a string, in a language with Vega's restrictions: arithmetic, comparison, conditionals,
a fixed set of functions (`min`, `max`, `abs`, `exp`, `log`, `pow`, `clamp`, `step`), named
inputs, and nothing else. No assignment, no loops, no recursion, no calls into the host;
every expression is total and evaluates in bounded time in IEEE double precision. A kind
declares which parameters take an expression and which names are in scope:

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
{ "name": "emissions", "kind": "carbon.emissions", "params": { "after_record": "max(0, last * (1 - (year - 2024) / 26))" } }
```

The checker verifies that every name an expression uses is in the kind's scope and that
the string parses. An implementation may keep the parsed form as a JSON tree in the
resolved manifest (MathJSON is one such encoding), but the string is the transport,
because a person has to read it. This is the lightweight math; it is as far as the
document goes toward being a program, and the line is drawn where Vega drew it.

Three layers, then: a language to write in, which the standard leaves open but requires
to be hermetic; a concrete document to exchange, which it fixes; and expressions inside
the document for what only the run can know, which it keeps small. VRML put the script in
the file and lost the transport; USD and Vega kept it out and kept theirs.

## 9. What it leaves out, on purpose

Not the authoring language (section 8): any hermetic program may write a manifest and the
manifest says which one did. Not the engine: how a kernel orders events, stores fields or closes ledgers is the
vendor's. Not the protocol between a kernel and a module, which is a second document and
lets a module be compiled in, loaded as WebAssembly or run as a process. Not the
vocabulary: whether two engines mean the same thing by `energy.gmst_anom_c` is the real
interoperability problem and belongs in a shared, versioned list a manifest names by URL;
this document only insists that the name be carried. Not the run folder, the log, the
frames or the scores.

An engine that needs more puts it under `ext`, keyed by its own name, and nothing under
`ext` may change a result. Readers ignore what they do not know there and preserve it
through resolution.

## 10. From practice

esim's TOML manifests map onto this one for one: `[run]`, `[place]`, `[[module]]` with its
`[module.params]`, `[audit]`. Three things differ. esim keeps variants in the project
beside the manifest, where this document puts them in it, because a family of assumptions
is the thing worth sending. esim keeps the kind descriptions and the data locks in files
beside a run, which is the same information in a shape a reader elsewhere cannot use. And
esim's include can be spliced at a point in the module order, which this document does not
yet have.

The reference implementation does not yet read this form. That is deliberate: the
argument should stand before the code follows it.

## 11. Security

A manifest can name code and data by URL. An engine verifies the content hash before
using either, and runs a fetched module with no more authority than the module protocol
grants: its parameters, its seed, its events, the fields it reads. Parameters are never
evaluated as expressions. A manifest from a stranger is a document to check, not a program
to run; levels one and two are safe on any input, level three is not.

## 12. Open questions

- Whether `place` should be a general grid declaration rather than a geographic window,
  so that a tank or an abstract lattice is first class.
- Whether a variant may add a module, which would make "with and without this coupling"
  one document at the cost of variants no longer being a table of differences.
- The expression language's exact grammar and function list, and whether a parsed form
  travels beside the string.
- Whether the reference implementation's first authoring language should be JavaScript in a
  sandbox (orbital-sim's form, runnable in the viewer) or Starlark (hermetic by construction).
- Where the vocabulary lives and how a version of it is named.
- Whether the audit belongs in the manifest or in a document that names one.

## References

- The schema: [`manifest.schema.json`](/docs/rfc/manifest.schema.json).
- [The module protocol, version 0](/docs/protocol-v0), the second document.
- [Manifests](/docs/system/manifests), [Vocabulary](/docs/system/vocabulary),
  [Terminology](/docs/system/terminology): the practice this was abstracted from.
- RFC 3339 (timestamps), RFC 8785 (JSON canonicalization), JSON Schema draft 2020-12.

## Changes

- Draft 1, 2026-11-07: first text.
- Draft 2, 2026-11-07: prior art (section 7); authoring, ranges and run-time expressions
  (section 8) after Anselm's note that a fully unrolled grammar cannot express variables and
  loops and that orbital-sim's manifests were JavaScript used declaratively; `from`, `range`
  and the `expr` parameter type in the schema.
