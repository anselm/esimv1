# The module protocol, version 0

This is the contract between the kernel (the coupler: one clock, ledgers, the log, the
world store) and a module (one piece of science, or one piece of someone else's engine).
It is written so that a module can be compiled into the binary, loaded as a wasm
component, or run as a separate process in any language, and the kernel cannot tell
which. Today only the in-process Rust binding exists (`crates/kernel/src/module.rs`,
`describe.rs`). The wire binding described here is the next piece of work; where it says
"will", that is what is meant.

The protocol is deliberately small. The things that carry meaning are not the messages
but the **vocabulary** they name: topics, field names and units, ledger names and
accounts, cell addressing. The vocabulary is governed separately (see the end).

## Model

- **Virtual time** is an integer count of seconds, UTC. Events are delivered in order of
  (time, priority, sequence). Lower priority runs first at the same instant. A module that
  reads at an instant what another writes at that instant takes the higher priority
  number.
- A **module** owns private state, reacts to events, and asks the kernel to do things. It
  never touches another module. It is a pure function of its parameters, its seed, the
  events it receives and the fields it reads; given the same of each it must produce the
  same requests. That is what makes runs replayable and ensembles honest.
- An **event** is `{ t, priority, seq, src, to, topic, at, data }`. `to` is a module name
  for a directed message or absent for a broadcast to the topic's subscribers. `at` is an
  optional grid cell for spatial delivery. `data` is JSON, loosely typed on purpose; the
  topic says what shape to expect.
- The **world** is a store of named fields, each a flat `f32` array on a declared
  latitude-longitude grid with a unit. The first module to write a field owns it; nobody
  else may write it. Writes are versioned and logged by digest, not by data.
- A **ledger** is double entry in one unit with a tolerance. Modules move amounts between
  accounts and declare the stocks they hold; the kernel closes every ledger on a cadence
  and a leak larger than the tolerance fails the run.
- A **cell** is an index into a grid, row major from the north-west corner. Grids are
  declared with a field; a cell address is only meaningful against the grid it names.
- A **birth** is an event on the topic `manifest.entity` with data `{ id, kind, params }`.
  The manifest's own modules arrive this way at the start instant, ahead of everything
  else; a module may publish one later and the kernel builds, initialises and registers
  the module when it is delivered, then passes the event on to the topic's subscribers. A
  refused birth is logged on `manifest.refused` with its reason (since 2026-11-07).

## Lifecycle

```
describe(name, params)  -> Describe            no state, no data, may be called any time
build(name, params)     -> module instance     fetches what it needs through the registry
init(ctx)                                      subscribe, open ledgers, declare stocks, first schedules
handle(event, ctx)      ...                    one event at a time, in order
snapshot()              -> state               private state, exact round trip
restore(state)                                 same instance type, resumes bitwise
```

`describe` is the verb added on 2026-11-01. It states the module's surface from the
parameters alone, so that the kernel can check a manifest's wiring before anything is
fetched or run, and so that a facade can read what a kind is for and needs without
reading its source.

## Describe

The shape (`esim kinds --json` prints every kind's under default parameters):

| field | meaning |
|---|---|
| `kind`, `name`, `summary` | what it is, in a sentence a person can read |
| `priority`, `dt` | its ordering at an instant, its own tick |
| `params[]` | `name`, `default`, `unit`, `note`; the note carries provenance, and "guess" means nobody has sourced the number |
| `publishes[]` | broadcast topics it emits |
| `sends[]` | `{ to, topic }` directed messages |
| `accepts[]` | directed topics it handles when addressed |
| `subscribes[]` | topics it wants delivered |
| `fields_owned[]` | world fields it writes, with units |
| `fields_read[]` | world fields it reads, with `required`, and `on` when the read is triggered by an event rather than its tick |
| `ledgers[]` | `name`, `unit`, `tolerance` |
| `frames[]`, `reports[]` | output rasters and scalar series, for maps and charts |
| `assets[]` | catalog asset ids or `events/<table>` it fetches |
| `notes[]` | anything else, usually where the parameters came from |

The kernel's check (`esim check`) derives from these:

- errors: unknown kind; two modules with one name; two writers of one field; a required
  read with no writer; a read in one unit of a field written in another; a directed send
  to a module not present or not accepting the topic; one ledger in two units; an asset
  not in the catalog or an event table not on disk;
- warnings: a required tick read of a field whose owner has the same or a later priority
  (the reader sees the previous write); ledger tolerances that disagree;
- information: optional couplings not present; subscriptions nobody publishes; outputs
  nobody reads.

`esim run` refuses a manifest with errors. A test runs real manifests and compares every
publish, send, field write and subscription in the log against the declarations, so a
description that drifts from the code fails the build.

## Requests a module may make during init or handle

These are the fifteen verbs of `Ctx` today and the message set of the wire binding.

| request | arguments | meaning |
|---|---|---|
| `schedule` | after, priority?, topic, data | an event to itself |
| `send_at` | t, to, topic, data | a directed event at an absolute time |
| `send` | to, topic, data | directed, now (after the current instant's earlier events) |
| `publish` | topic, data | broadcast, now |
| `publish_at` | cell, topic, data | broadcast about a cell; delivered to topic subscribers and to modules subscribed at that cell |
| `send_cell` | to, cell, topic, data | directed, about a cell |
| `subscribe` | topic | |
| `subscribe_at` | topic, cells | only events at these cells |
| `ledger` | name, unit, tolerance | open or join a ledger; the first opener's unit and tolerance stand |
| `transfer` | ledger, from, to, amount | |
| `declare_stock` | ledger, account, value | the first declaration is the opening balance |
| `report` | series, unit, value | a scalar at the current time |
| `field` | name, unit, grid, data | an output frame |
| `world_write` | name, unit, grid, data | write the shared field this module owns |
| `world_read` | name | the field as last written: unit, grid, version, data |

Randomness: in process, a module draws from a kernel-held xoshiro256** stream seeded from
the run seed and the module name. Over the wire the kernel will hand the module that seed
at `init` and the module owns its generator; both are deterministic, which is all that
matters.

## The wire binding (next)

JSON lines over stdio or a socket. The host speaks first.

```
host  -> {"protocol":"esim/0","describe":{"name":..,"params":{..}}}
module-> {"describe":{..Describe..}}

host  -> {"build":{"name":..,"params":{..},"seed":u64,"root":"<ESIM_ROOT or cache url>"}}
module-> {"ready":true} | {"error":".."}

host  -> {"init":{"t":i64}}
module-> zero or more request lines, then {"done":true}

host  -> {"event":{..Event..}}
module-> zero or more request lines, then {"done":true}

host  -> {"snapshot":true}          module-> {"state":<json>}
host  -> {"restore":{"state":..}}   module-> {"done":true}
```

A request line is one of the fifteen verbs as `{"verb":{args}}`. `world_read` is the one
synchronous call: the module sends `{"world_read":"name"}` and waits for
`{"field":{"unit":..,"grid":{..},"version":..,"data":"<base64 f32 LE>"}}` before
continuing. Fields over the wire are the cost of running out of process; a shared-memory
or Arrow binding is a later optimisation, not a change to the protocol.

A module reference in a manifest will be a kind name from the canonical library, or a
URL (a release, an OCI image, a wasm file) locked by content hash in the project, the way
the data catalog already locks assets. The kernel resolves the reference to one of the
bindings and proceeds identically.

## What this does not fix

The protocol does not say what topics mean, what fields are called or what units they
carry. That vocabulary is the real interoperability surface and is kept as a separate,
versioned list: the names the first-party modules use today are the seed of it
(`esim kinds --json` is the current enumeration), and the digitaltwins.wiki Ontologies
shelf is where its definitions belong. Changing a name there is a change to every module
that uses it; changing a message here is a change to the kernel only.

The manifest that arranges modules under this protocol, as a transport between engines, is
proposed separately in [RFC 0001](/docs/rfc/0001-manifest) (2026-11-07).

## Status

- 2026-11-01: v0 written. In-process Rust binding complete, `describe` on all thirteen
  kinds, `esim check`, `esim describe`, `esim kinds --json`. Wire binding not started.
