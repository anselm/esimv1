# 2026-11-01 — The describe verb and the protocol document

Anselm, after the roots entry: "ok, lets go ahead with what you suggest - the describe
verb, and a protocol document - i think also after this lets push to github". This entry
records what landed. Agent: Claude (Fable 5.1) in the earth-simulator-1 room.

## What landed

- `crates/kernel/src/describe.rs`: the `Describe` shape (params with defaults, units and
  provenance notes; publishes, sends, accepts, subscribes; fields owned and read, with
  `required` and `on`; ledgers; frames; reports; assets; notes), a builder, and `check()`,
  which derives errors, warnings and information from a manifest's descriptions alone.
- Every one of the thirteen kinds has a `describe(name, params)` beside its `new`. The
  registry in `crates/modules/src/lib.rs` is now one table, `KINDS`, with `build` and
  `describe` per kind; the three hand-kept lists it replaced were the compiled-in surface
  the roots entry called a corner.
- `esim check <manifest>` prints the wiring report and exits 1 on errors. `esim describe
  <manifest>` prints descriptions and findings as JSON. `esim kinds --json` prints every
  kind under default parameters: the toolchain's surface for a facade. `esim run`,
  `verify`, `ensemble` and `sweep` all check first and refuse a manifest with errors.
- The check also covers data: an asset a module names must be in `data/catalog.toml`, an
  event table must be on disk. Three sources the modules already used had no catalog
  entry (Open-Meteo ERA5, Open-Meteo GEFS, MODIS MOD13A1 on Planetary Computer); they
  have them now, with licence notes marked for verification before promotion.
- `crates/modules/tests/describe.rs`, seven tests: every manifest checks clean; a broken
  manifest produces the expected errors; and real runs (M0, M7 forced fire, M10 season)
  have every logged publish, send, field write and subscription compared against the
  declarations. The 15 earlier tests still pass.
- `docs/protocol-v0.md`: the contract between kernel and module written so that a module
  can be in process, wasm or a separate process; the fifteen requests; the describe shape
  and the check rules; the JSON-lines wire binding to build next; and the statement that
  the vocabulary, not the messages, is the real interoperability surface.

## What the check says about the existing manifests

All 26 check clean. The information lines are true and useful: in the M11 manifest the
water module would read `rules.no_crop` but no rules module is present, the weather
module writes humidity, shortwave and wind that nothing reads, and the deer subscribe to
fire topics with no fire. The one warning class, a tick read of a field whose owner has the same or a later
priority, fires on none of the 26 manifests: the farms read the water module's soil at
the same priority, but that read is declared optional, so it is information, not a
warning. The class exists for the next module that gets the ordering wrong.

## Not done

- Place once in the manifest (the bounding box is still repeated per module). The
  describe output makes the repetition visible; fixing it is a manifest schema change.
- The wire binding and a stdio module host. The document says exactly what to build.
- Module references by URL and hash.
- Pushed to GitHub at Anselm's request after this entry; first push of the repo.
