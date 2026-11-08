# Three notations for a manifest, and the manifest as events

Anselm, 2026-11-07, in the RFC 0001 session, after draft 3: "How do reconcile plain json with
my competing desire to have variables", then "And includes", then "what do you mean by keep the
variables in the hand that write it? ... my goal for having variables in declarative documents
is because declarative transport grammars like html are super long largely because they
completely unroll everything - they lack any way to say '10 of' and so on. I'm sorely tempted
to use actual javascript or some subset of the grammar. I think an absolutely purely
declarative notation is simple to parse, but hard for humans to understand." Then, on the
parser cost: "it moves the parser out of the realm of trivial embedded systems like say a
raspberry pi". Then the orbital account: the `.js` manifests' exports were events on a bus,
the bus registered listeners dynamically, submodules fell out of that. The ask:

> 1) if needed - ie - if not done - refactor the core engine to be an event bus and to handle
> dynamic registration of listeners. 2) i want to support a .js notation (we can presume
> quickjs - so maybe a subset of full blown js). i want js to allow for variables, loops and
> compression of the expression of a scene. 3) also support json 4) continue to support
> existing notation.

And, while it was being built: "one nice thing about javascript is that it is a bit easier to
add rich comments to the manifests."

Done by Claude (Fable 5.1) in this session, the evening of 2026-11-07; the work is in commits
a8636ac, 67826b4 and the one carrying this entry.

## The argument, as settled

Variables belong to the hand that writes the document, not the hand that reads it. A variable
in a transport exists only to be resolved, and every competing engine would have to agree on
the resolver; in the program, no agreement is needed, only the output. So the standard has two
forms: a source form (JavaScript, hermetic by subtraction) and an object form (the entity
list); an engine must read the object form and may read the source form; the id is of the
object form; two resolvers must produce identical bytes. The Pi reads JSON. Includes go with
variables, for the same reason and one more: an include is a dependency the reader must
resolve to read the document at all, and hashing the text hashes a pointer. The replacement
rule stays. Two constructs stay in the object form, by one test: the range (unrolling it
destroys the ensemble as one thing) and the expression (it cannot be evaluated before the
run). The next candidate is a conditional and the test says no.

## What was built

- `crates/kernel/src/entities.rs`: the entity form in and out of the TOML-shaped `Manifest`
  (which gained `audit`, `variants`, `vocabulary`, `from`, `unknown`); presence is type;
  replacement by id; `range` unrolls to variants (`seeds-01` ... `seeds-30`); unknown components
  carried and reported by the check as information; canonical text in the JCS manner with
  ECMAScript number formatting (`es_number`, tested against the cases that bite: 1e21, 1e-7,
  0.1+0.2); the id as sha256 of it. `Manifest::load` dispatches on extension.
- `crates/kernel/src/js.rs`: QuickJS through rquickjs 0.14. Prelude deletes `Date`, makes
  `Math.random` throw and freezes `Math`; a resolver that allows only `./` and `../` under
  the entry's directory; a loader that hashes every file read, turns `.json` into its value
  and `.toml` into that manifest's entity list; a 10 s interrupt and a 256 MB limit; exports
  read through JSON.stringify so functions vanish; the export order recovered from the text
  because the module namespace is alphabetical; the short form for `run`, `place`, `audit`
  and a variant (`set` or `range` without a `variant` wrapper).
- `crates/kernel/src/kernel.rs`: the manifest is played onto the bus. Each module entry is a
  `manifest.entity` event at the start, delivered in the constructor so construction errors
  stay construction errors and every module is initialised before the first event; a module's
  publish on the same topic births a module mid-run; `manifest.refused` logs a refusal; the
  snapshot carries `born` and `from_snapshot` rebuilds base then born. The start births take no
  sequence numbers, so every digest in every run folder stands (checked: p1 3121bc12 before and
  after; the boone replay verifies). A kernel test births a module at hour three, refuses a
  duplicate and an unknown kind, and replays from a snapshot after the birth.
- CLI: `esim manifest <file> [--toml|--js|--id]`; run and project folders keep `manifest.json`
  with `from`; `project new` copies a program and its imports to `source/` and carries the
  document's variants into project.toml.
- `manifests/p1-planet-co2-1850-2025.js` (the TOML's twin: same id, same digest),
  `manifests/j2-planet-members.js` (import, loop, range), `manifests/j3-planet-wild-members.js`
  and the project `anselm/planet-wild-members` (eight seeds of the deck on the held planet).
- RFC 0001 draft 4 (section 5 rewritten around the two forms; `include` gone from the
  schema; section 13 now from practice), `docs/system/manifests.md` (Three notations, Births),
  `docs/protocol-v0.md` (the birth event).

## Decisions and the honest parts

- The internal canonical stays the TOML text: a `.json` or `.js` source is rendered to TOML
  for the override and run-id path, so no existing run id moved. The entity form is a first
  class input and output, not yet the internal form.
- Components are data only. Orbital allowed functions in manifests; between strangers a
  function is code the receiver may not run, so behaviour goes through `ref`, and a
  `code: { language, source }` component would be carried and reported, not run.
- The check is static. A mid-run birth is checked for id and kind by the factory, not for its
  wiring. `events.shock` has modes off, draw, named; the check did not catch `mode = "on"`,
  the build did, which is a gap in that module's describe, noted, not fixed tonight.
- The JavaScript export order is recovered by scanning the text for `export` statements, since
  the module namespace is alphabetical by the language's rules. A destructuring export and the
  brace form are handled; an export inside a comment would be counted. Good enough for layout,
  which is all it decides.
- QuickJS compiles into the kernel crate as C; the deploy VM has Node only and serves files, so
  nothing there changes.
