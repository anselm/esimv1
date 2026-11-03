<!-- order: 4 -->
# Modules

A module is one piece of science behind a small contract, and the contract is the only
thing the kernel knows about it.

## The contract

```
describe(name, params) -> Describe     no state, no data; may be called any time
build(name, params)    -> instance     fetches what it needs through the registry
init(ctx)                              subscribe, open ledgers, declare stocks, schedule
handle(event, ctx)                     one event at a time, in order
snapshot() / restore(state)            exact round trip of private state
```

During `init` and `handle` a module makes requests of the kernel and nothing else:
schedule an event to itself; send to a named instance or publish on a topic, now or at a
time, optionally about one grid cell; subscribe to topics or to topics at cells; open a
ledger, transfer between accounts, declare a stock; report a scalar; write a frame; write
the world field it owns; read any world field. Fifteen verbs. The full contract is
`docs/protocol-v0.md`.

A module is a pure function of its parameters, its seed, the events it receives and the
fields it reads. That is what makes runs replayable and ensembles honest, and it is why a
module may not reach around the kernel to another module or to the clock.

## What a module owns

Its private state. The world fields it was first to write; nobody else may write them.
The stocks it declares in a ledger; it must keep them exactly, because the kernel
compares its declaration with the flows every close. Its own data: a module fetches what
it needs through the registry at build time, by catalog id, and the describe names those
assets so a manifest can be checked for data before anything is fetched.

## Why describe exists

The describe is the module's surface in words a program can read: parameters with
defaults, units and provenance notes; topics published, sent, accepted and subscribed;
fields owned and read, with units and whether the read is required; ledgers with units
and tolerances; output frames and series; data assets; notes. From the descriptions of a
manifest's modules the kernel derives errors (a required read nobody writes, two writers
of one field, a ledger in two units, a send to nobody, an asset not in the catalog),
warnings (an ordering that reads last tick's value) and information (optional couplings
not present). `esim check` prints it; `esim run` refuses a manifest with errors; `esim
kinds --json` prints every kind's describe, which is what a model reads to build a
manifest. A test runs real manifests and compares every publish, send, field write and
subscription in the log against the declarations, so a describe that drifts from its
code fails the build.

The same declarations drawn as a graph are the **wiring** section of every run page:
modules as boxes in rows by dependency, topics as solid arrows and fields as dashed ones
from the module that writes to each module that reads, ledgers as shared books beside
the flow, feedbacks as arrows that climb back up, and outputs nothing in the manifest
reads as short red arrows into air. It computes nothing new; it is the check made
visible. For the planet stack it shows one temperature line fanning out to three
threshold modules whose crossings reach only the story, which is why that page reads as
symptoms of one cause rather than as compounding, and which couplings would change
that. `devlog/20261102-why-it-reads-mild.md` has the reasoning.

## The pool

There are twenty-three kinds today, all in one repository. There will be hundreds, then
thousands, from many hands, and they will live at URLs: a release, an OCI image, a wasm
file, locked by content hash in the manifest the way the catalog locks data. The kernel
cannot tell an in-process module from one over a pipe, and the protocol document
specifies the wire binding for that.

Expect the pool to be heterogeneous and a little adversarial. Two kinds will claim to be
the right way to grow a crop or route a river or move a herd, with different parameters,
different sources and different values about what matters. That is intended. A person,
or a model acting for one, mixes and matches what they judge best for a scenario, and
the system's job is not to pick the winner but to make the choice checkable: every kind
says what it reads, what it owns, where its numbers come from and what it leaves out;
every run can be scored against the record; two manifests that differ in one module can
be run side by side under one cursor. Provenance and scoreboards are how a pool like
this stays honest without a gatekeeper. See [Evolution](?doc=system/evolution).

## Views: how a module asks to be shown

A run page is composed from the manifest, not written by hand: the kernel's header and
time cursor first, then each module's section in the manifest's order, then the
auditor's reading, the scores, the ledgers, and the raw events and manifest behind
disclosures. Each module's section holds the views its describe declared:

| view | source | shown as |
|---|---|---|
| `counter` | a report | one number at the cursor, with a title and a note |
| `map` | a frame | the frame at or before the cursor, under the page's one cursor |
| `chart` | a report, optionally with an observed report beside it | a line, the cursor marked |
| `crossings` | a topic ending in `.crossing` | the crossings so far, newest first |
| `story` | the word `crossing` | the sentence each crossing event carries, newest first, with its source |
| `text` | the text itself | prose |

A module that declares no views gets a map per frame and a chart per report. A module
that computes nothing but exists to be first on the page is allowed, and
`story.bottomline` is one: the manifest's order is the page's order, so the bottom line
goes at the top by being listed first. A crossing event carries its own sentence,
written by the module that knows the source, so the viewer never invents words. Every
run writes `describe.json` beside `run.json`, and that is what the page is built from;
there is no module-specific code in the viewer.

## Granularity

A module should be the size of one idea with its own record: a water balance, a crop's
phenology and growth, a predator cohort, a fire's spread, a country's temperature from
its record, the tipping elements table. Modules that are too small chatter through the
event queue; modules that are too large hide the seams where ledgers should close. When
in doubt, split at the point where two different people would hold different opinions.

## Where modules live today

`crates/modules/src/`, one file per layer or per place, registered in a table of kinds.
[Adding a module](?doc=system/adding-a-module) is the recipe.
