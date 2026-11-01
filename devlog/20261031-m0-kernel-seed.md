# 2026-10-31 — M0: kernel seed runs, ledger closes, replay works

Anselm: "Feel free to start coding now. I do need a way to visualize so we will need some
kind of minimalist web facade." Work by Claude (Fable 5.1) in this room, session
https://claude.ai/code/session_018TGCFhEpYs9hm68xDLd5Xy, on canada1.

## Done

- Rust toolchain installed for this user (rustup, stable 1.99.1). No Rust was on the box.
- Cargo workspace with three crates:
  - `esim-kernel`: virtual time (i64 seconds, no date dependency), events ordered by
    (time, priority, sequence), xoshiro256** per-module RNG seeded from run seed and
    module name, double-entry ledgers with declared-stock closing, append-only log with
    an FNV digest, module trait with a context, TOML manifest, a local projected tile
    grid, and the run loop with snapshot and resume.
  - `esim-modules`: a water source and a bucket, the M0 proof modules. A bucket with
    `leak` set misreports its stock on purpose.
  - `esim` (CLI): `run` writes a run folder (manifest, log.jsonl, run.json, snapshots,
    and runs/index.json), `verify` proves determinism and replay, `kinds` lists modules.
- `manifests/m0-buckets.toml`: source every hour, upper bucket every six hours, lower
  bucket every day, to the sea. Thirty days, 1,743 events, 7 ms. Water ledger closes
  thirty times with zero leak. `manifests/m0-leaky.toml` fails to close with a 2,549 m3
  discrepancy, which is the ledger doing its job.
- `cargo test`: seven tests. Determinism (two runs, one digest), seed independence when
  no module draws randomness, closure across two rates, leak detection, and replay from
  every snapshot reproducing the log tail exactly.
- The web viewer is being built by a subagent in `web/` against the real run folder;
  its result is reported separately.
- `data/events/`: the calendar and hazard tables seeded, with a schema README. See below.

## Design choices made in code

- Ledger closing compares a module's declared stock with the flow-derived balance. Double
  entry alone cannot catch a module lying about itself; the declaration can.
- Kernel housekeeping (close, snapshot) is itself events at extreme priorities, so the
  log records when ledgers closed and a resumed run keeps the cadence.
- A snapshot records the log length at the instant it was taken; replay is checked by
  comparing the resumed log with the original log from that index. The first attempt
  compared by sequence number and was wrong; sequence is scheduling order, not delivery.
- Modules receive `kind` folded into their params by the CLI because the kernel's factory
  is kind-agnostic. Slightly odd; revisit when the module registry grows.
- Time serialises as a Unix integer in JSON. Viewers convert.

## Not done, on purpose

Conservative multi-rate parallelism (the single-threaded order is the one it must
reproduce), the regional and global grids, the world-state store, the registry, and any
science. All M1 and later.

## Goals Anselm added during the session

- "the way people will use this product ultimately is to specify requests in English -
  like 'I want to model traffic in my town with changes to speed limits'". So the facade
  does more than diff a manifest: it finds the place in the registry, picks modules from
  the catalog, writes the whole manifest, runs, narrates. The manifest being a document
  is what makes that possible; the module catalog needs machine-readable descriptions
  (ODD-shaped) for the same reason. Added to M13.
- "Let's have our own future events timeline and our own chaos / black swans as well ...
  what are known events of the future and possible risks or events good and bad?" Seeded
  in `data/events/calendar.toml` and `hazards.toml` with a schema in the README: date or
  rate, consequence, module, valence (good, bad, mixed), confidence (certain, committed,
  projected, speculative), source, known_since, and for hazards the state they depend on
  and a default setting. He photographed a book's table of when AI replaces jobs
  (chapter "AI and Bitcoin", Table 14.1, book unidentified); its nine rows are in the
  calendar as speculative claims citing the photograph, which is the schema working as
  intended. Every seeded row carries the agent's honest confidence; several say "check"
  or "guess" and must be verified before promotion. The tables are data only; the
  kernel reads them at V4.

## Fitness against the use cases

Anselm: "You can measure the fitness of the framework against all of our use cases." A
matrix lives in `20261031-fitness.md` and is updated as rungs land.

## Next

M1 Sun on a real elevation tile, which needs the first registry fetch (a global DEM
product chosen and recorded) and the regional grid. The viewer gets its first map then.

## Addendum, same day: data layer lives here for now

Anselm: "we will have our own data layer for now or ways of fetching elevation data and
so on - later can move to other repos." So the registry (catalog, fetcher, mirror rules,
event tables) is built inside this repo under `data/` and a `registry` crate, with the
intent recorded in the data devlog entry of moving the text half to the wiki repo later.
Nothing in the kernel may assume where the data lives; it goes through the fetcher.

He also pointed out that "there are encyclopedias of the future out there". A web search
found Wikipedia's timelines-of-the-future hub and lists, and futuretimeline.net; they are
recorded in `data/events/README.md` as sources to mine into draft rows by an agent, with
promotion by review.
