# RFC 0001: a common transport for simulations

Anselm, 2026-11-07 (afternoon, after the derailment work shipped): "I would like a document
that is a public schema for a manifest. An RFC. It does not have to be something by we use
here. It is more a theory. I want to define a protocol or interoperability standard between
different implementations of this engine. Although we don't use json I think this document
should be json." And, while the first draft was being validated: "the role of this document
is to argue for a standard transport for simulations between competing vendor engines. It
has to persuade. It doesn't have to be completely comprehensive and exhaustively tiring in
detail."

Written in this session (Claude, esim room) as `docs/rfc/0001-manifest.md`, rendered at
`/docs/rfc/0001-manifest` under a new docs area `rfc` and section "Standards", with the
exhaustive shape in `docs/rfc/manifest.schema.json` (JSON Schema draft 2020-12, served as
a file). The first draft was a 500-line specification with normative tables for every
block; his note came in while it was being checked, and it was rewritten as a 260-line
argument: the case first (the arrangement is the only part of a simulation worth
exchanging; five things a transport changes), one page of shape around a complete example,
the three rules that make it a transport rather than a configuration file (resolution, the
resolved form carrying `kinds` and `locks`, canonical text by RFC 8785 and an id), checking
without running, the five levels of sameness, one rule about honesty (provenance on every
parameter), what it leaves out on purpose, the mapping from esim's TOML, security, open
questions. The detail moved into the schema, where it is machine-checkable.

Decisions in the design, all abstracted from what esim does rather than invented fresh:

- Variants live in the manifest, not the project, because a family of assumptions is the
  thing worth sending. esim keeps them in `project.toml`; the RFC says so.
- The resolved manifest carries the describes of its kinds and the hashes of its records
  (`kinds`, `locks`), so a reader with no engine can check the wiring and draw the graph.
  esim keeps these beside the run; the RFC calls that the same information in a shape a
  reader elsewhere cannot use.
- Sameness is a ladder of five levels (readable, checkable, runnable, comparable,
  reproducible) and the standard promises nothing at the fifth; that is the honest claim,
  and most of the value is at the first four.
- Provenance (`checked`, `recalled`, `guess`, `fitted`) is the one conduct rule in the
  shape, because a guessed number that travels as a checked one defeats the exchange.
- `place` stays a geographic window; a general grid declaration is the first open question.
- `ext`, namespaced by implementation, for everything else; nothing under it may change a
  result.

Checked: the schema validates against the 2020-12 metaschema; the complete example in the
text validates against the schema with no errors (the locks example fails only on its
shortened hashes, which the text says are shortened); the page renders light and dark; the
heckler report finds no paragraph-level tells. esim does not read this form, and the RFC
says that is deliberate. Not pushed, not deployed.

## Draft 2, the same afternoon: prior art, and variables and loops

Anselm, after draft 1 shipped: "Please do some research actually - who else has sim
grammars? Of course there are many grammars for 3d - vrml for example. Oh Oh I want to be
able to encode lightweight math. The grammar for GitHub.com/orbitalfoundation was actually
pure JavaScript used in a declarative way! We need to be able to express variables and
loops - I realize that a fully unrolled declarative grammar is not good enough! What do we
do?"

What was read (2026-11-07, search results and the pages fetched; secondary where noted):
SED-ML's site (Level 1 Version 5; models, simulations, tasks, repeated tasks over ranges,
data generators; MathML), Tellurium's Antimony reference ("SBML was not designed to be
readable or writable by humans"), the prostep SSP fact sheet (SSP 1.0, March 2019) and a
Modelica conference paper on importing SSP into Dymola, the CERFACS OASIS3-MCT user guide's
namcouple chapter, the WGCM paper on ES-DOC for CMIP6 and PCMDI slides on the CMIP6 JSON
vocabularies, Wikipedia and NATO notes on HLA IEEE 1516 (secondary), the ODD 2020 update in
JASSS 23(2) 7, the LEMS paper (PMC4174883), ASAM's press release on splitting OpenSCENARIO
XML and DSL and the 2.0 concept paper (first 100,000 characters: "a low-level and concrete
specification format, primarily designed to be read by simulation tools"; `keep(...)`
constraints; the "concrete scenario" output), Vega's expressions page ("a restricted
subset of JavaScript"; no assignment, no `new`, no control flow), NVIDIA's Learn OpenUSD
page on composition strength ordering (secondary), Wikipedia on VRML97 ISO/IEC 14772 and
X3D ISO/IEC 19775 (secondary), Jsonnet's design article (pure functional, hermetic, Turing
complete by choice), the Starlark spec as quoted in search results ("deterministic and
hermetic", "execution is finite", "no recursion or unbounded loops"; the GitHub page itself
would not render through the fetcher), cuelang.org, and orbital-sim's manifests on disk
(`public/orbital/*/manifest.js`: exported objects with `inherits`; hormuz reads
`process.env.BLOCKADE`).

The answer, now section 8 of the RFC: three layers. The transport stays concrete and
unrolled, because that is what a reader without an interpreter can check and what hashes
the same everywhere; OpenSCENARIO's concrete scenario, SED-ML's experiment and Vega's
compiled grammar are the same layer. Authoring is a program that writes the transport; the
standard does not pick the language and requires only that it be hermetic (no clock,
environment, network or undeclared file; randomness only from the seed), and the resolved
manifest carries a `from` block with the program's hash and its sources' hashes so the
generation is reproducible. JavaScript in a sandbox is orbital-sim's form and runs in the
viewer; Starlark and Jsonnet are hermetic by construction; the choice is an open question.
One declarative loop stays in the transport, a `range` on a variant for ensembles and
sweeps (SED-ML's repeated task), because the engine needs to know those runs belong
together. Math at run time, for what no generator can unroll (emissions as a function of
the year and the warming so far), is an expression string with Vega's restrictions, typed
`expr` by the kind's describe with the names in scope, total and bounded, checked by the
checker; a parsed form (MathJSON is one encoding) may travel beside it but the string is
the transport. The example replaces esim's `after_record = "zero_by:2050"` mini-grammar
with `max(0, last * (1 - (year - 2024) / 26))`. VRML's lesson is the line: once a script
node is in the file, interchange means whatever the viewer runs.

Schema: `from`, `variant.range` (uniform or explicit values), `describe.params[].type`
with `expr` and `vars`. Every example in the text validates (the shortened hashes excused).
esim reads none of this yet; `events.shock`, `carbon.emissions`'s `after_record` and the
project variants are the three places it would first apply.

## Draft 3, later the same afternoon: two levels, and the entity component pattern

Anselm: "The grammar exists at 2 levels. There is a meta level - variables, loops, includes,
layout. Also I used an ecs pattern for a lot of my work - may take you a bit of research to
find that. This is above the use case - basically 'how do we express manifests or state
succinctly?' Then there are the structures we are building in the grammar - related to
modeling the world."

Found on disk (2026-11-07): the 2024 orbital website docs in
`~/projects/2027/orbital/reference/orbital-website-2024/docs/` (`34-entities.js`: "Instead you
have only instances, and instances are decorated with components"; `33-manifests.js`: "a
manifest is only a series of events over time, that are inhaled by the system, that observers
watch"; the argument that JavaScript is a "good enough" meta-grammar and that the base grammar
matters less than the separation of declarations from procedure; `37-manifests.js`: "transport
grammars for reflecting system state must be human editable"), the orbital bus `SPEC.md`
(filters match on key presence; manifests are `{ load }` of ESM modules whose named exports get
the export name as id) and `PRIMER.md` ("The bus is the message substrate. ECS is a layer above
it. Manifests are an authoring convention above that."; ECS from the blackboard pedigree), the
`orbital-sys` readme, and the harness memory notes on the orbital conventions (entities are
component bags with presence-as-type).

The RFC is now in three parts. Part I, the meta grammar: a manifest is a list of entities, each
an `id` and a bag of components that are plain data; presence is type (an entity with `kind` is
a module instance, with `run` the run, with `variant` a variant); a later entity with an
earlier id replaces it component by component, which is the whole include and variant rule; a
component a reader does not know is carried, never rejected, and the checker reports "no system
observes it", which replaces the namespaced `ext` escape hatch; order is layout; the authoring
program, ranges, expressions, resolution and identity follow. Part II names the components for
the world (run, place, kind, params, ref, audit, variant) and says why ledgers, fields and
topics are not components: they are declared by kinds, derived by the checker, governed by the
vocabulary. Part III is what an engine promises (the five levels, honesty, what is left out,
practice, security, open questions). The prior-art table gained the entity component system
(its game-engine origin marked recalled) and the orbital row now quotes the 2024 docs.

The schema is rewritten in the entity form: `entities[]` with the named components under
`$defs/components`, `additionalProperties` allowed on an entity (data only, never a function),
`params` and `ref` dependent on `kind`, `variant` requiring `set` or `range`, `ext` gone. Every
example validates and five negative cases behave as the text says (unknown version, params
without kind, upper-case id and a variant with neither set nor range are rejected; an unknown
component passes). The snapshot and the log as entities, which would make a run folder one kind
of document, is the new open question; orbital already does this.
