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
