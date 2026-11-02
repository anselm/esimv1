# esim

A whole-system simulator built as a bucket of tools: small modules of science on one
clock, each reading real records, exchanging events and shared fields, moving conserved
quantities through ledgers that must balance at every seam. A manifest arranges modules
into a scenario; a run is a folder anyone can replay, score, audit and look at. The
first stages are a river basin in Iowa and the planet to 2100.

**Live, read-only: https://esimv1.exe.xyz/** · version 0.1.0 · code MIT

The planet pages to start with:

- [The planet, with people on it](https://esimv1.exe.xyz/?project=anselm/planet-2100): one
  project, five assumptions about emissions after 2024, each a variant.
- [Two futures, one clock](https://esimv1.exe.xyz/?compare=anselm/planet-2100): emissions
  held at today's rate beside emissions to zero by 2050, under one time cursor.
- [Documentation](https://esimv1.exe.xyz/?docs) and [the module kinds](https://esimv1.exe.xyz/?kinds).
- [Limits today](https://esimv1.exe.xyz/?doc=system/limits), to read before any number.

Nothing on those pages is a forecast. Every run past 2024 rests on a stated assumption,
every threshold carries its source and whether it has been checked, and the page says
what the run does not yet measure.

## What it does

- **Kernel** (`crates/kernel`): a virtual clock in seconds, events ordered by time,
  priority and sequence, shared gridded fields with one owner each, cell-addressed events,
  double-entry ledgers closed on a cadence, an append-only log with a digest, snapshots
  and exact replay, TOML manifests with includes and a place stated once.
- **Registry** (`crates/registry`): a catalog of sources with licence and verification
  notes, a fetcher that hashes and caches every file, and readers for elevation tiles,
  gauges, stations, reanalyses, satellite vegetation, emissions, ice cores, temperature
  records, population, country outlines and the station network. Nothing synthetic,
  including in tests.
- **Modules** (`crates/modules`): twenty-six kinds today. For a basin: solar irradiance
  on terrain, a surface heat slab, reanalysis weather, a water balance routed to the
  gauge, a crop, herbivore and predator cohorts, fire, farm agents, calendar rules, soil
  nitrogen to the river. For the planet: observed emissions into a carbon ledger, a carbon
  cycle with sink feedback, methane and nitrous oxide, the IPCC assessed forcings, a
  two-layer energy balance, thermal sea level, the records as scoreboards; then countries
  as the coarse unit: people per country, each country's temperature from its own record,
  the human climate niche, wet-bulb days at each capital, the tipping elements' thresholds,
  and a bottom-line module that tells the run as sentences with sources.
- **Auditor** (`crates/audit`): reads a finished run and never the live one: every stock
  spent or built, diversity, shocks and recovery against recurrence, crossings, tears,
  and a verdict of holds, stretching or tearing. It lists what it could not measure.
- **Command line** (`crates/cli`, binary `esim`): run, verify, ensemble, sweep, score,
  audit, check, describe, kinds, projects, sources.
- **Viewer and server** (`web/`): Svelte pages composed from each module's own declared
  views in the manifest's order, maps under one time cursor; a Node server that serves
  the pages and folders and, when not read-only, lists, drafts (with a model), checks,
  creates and runs projects over one socket.

## Honesty rules

They are the point of the system and are enforced by code where code can:

- Real records or nothing; a formula check is labelled as one.
- Every parameter is named, defaulted and sourced; a guess says "guess"; a number recalled
  from memory and not yet checked against its paper says "recalled".
- Nothing is tuned by hand; the hindcast harness moves parameters by method.
- A run past a record is an assumption, named in the manifest, never a forecast.
- Accounts balance: a ledger leak above tolerance fails the run.
- Runs are reproducible: the same manifest and seed give the same digest; a run folder
  keeps its resolved manifest and the hashes of every file it read.
- Lead with what is wrong. `docs/system/limits.md` is rewritten as it changes.

## Try it

Rust stable (rustup) and Node 20 or newer. First runs fetch a few hundred megabytes of
records into `data/cache/` and pin their hashes in `data/catalog.lock.toml`.

```sh
cargo build --release
./target/release/esim kinds                                        # the module kinds
./target/release/esim check manifests/p4-planet-people.toml        # wiring from the describes
./target/release/esim run manifests/p1-planet-co2-1850-2025.toml   # 175 years of planet, about a second
./target/release/esim run manifests/m3-water-boone.toml            # a year of basin hydrology, about 40 s
./target/release/esim verify manifests/m2-heat-boone-week.toml     # determinism and replay
cargo test --release                                               # about 40 s with the data cached

# projects: a folder with a manifest, a description and runs; variants are named overrides
./target/release/esim project new me/planet --manifest manifests/p4-planet-people.toml
./target/release/esim project run me/planet
./target/release/esim project run me/planet --set emissions.after_record=zero_by:2050

# the viewer and the server, one process
cd web && npm install && npm run build && node server.mjs          # http://localhost:5181
cd web && npm run dev                                              # development: hot reload, socket proxied to :5181
```

A manifest is a short TOML document; `docs/system/manifests.md` has the grammar. The
conversational way to make one is the "+ project" form in the viewer, which asks a model
to draft a manifest from a description against the kinds' own descriptions and has the
kernel check it before anything runs.

## Layout

```
crates/        kernel, registry, modules, audit, cli
manifests/     the ladder, from two buckets to the planet; bundles to include
projects/      <area>/<name>/{project.toml, manifest.toml, runs/}   (runs are not in git)
docs/          documentation pages, docs/<area>/<name>.md; docs/system/ is the simulator's own
data/          catalog.toml, catalog.lock.toml, events/ tables      (cache/ is not in git)
devlog/        dated entries: the design brief and every decision since
deploy/        provision and deploy a read-only copy to an exe.dev VM
todo.md        open research projects and layers, one line each
```

## Where things stand

The basin stack reproduces the shape of the gauge record and runs high on nitrate; the
planet hindcast is within a few percent of Mauna Loa and about 0.15 °C warm in 2024
against HadCRUT5; the country layers count people living where stated limits are
passed, with one temperature per country and one station per capital. The known faults
are on the Limits page and in the devlog, which is the project's memory: start with
`devlog/20261031-design-brief.md`, then the milestone entries, then the latest day.

## Contributing a module

`docs/system/adding-a-module.md` is the recipe: find the record first, name what the
module owns and reads in the shared vocabulary, write the kind with its `describe` beside
its constructor, account for what it moves, register it, run it on a real stage, and add
the manifest to the test that compares declarations against run logs. Competing kinds
for the same thing are welcome; the record and the describes decide.

## Licence

The code is MIT (`LICENSE`). The records the simulator reads are not in this repository;
each is fetched from its publisher under its own terms, listed in `data/catalog.toml` and
on the Sources page, and several are free for non-commercial use only.
