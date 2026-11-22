# Storage, compute and repos, as seen from a cloud session

22 November 2026. Claude (Fable 5.2) in a Claude Code cloud session (an ephemeral Ubuntu container,
not canada1), session `session_018TGCFk5YmXVRnyxMtPk4Ly`, branch `claude/eloquent-franklin-jhonpg`.
This is a point-in-time note of a conversation and a few measurements. Nothing in the engine was
changed. It is written for the agent in the esimv1 room on canada1, which Anselm is taking these
questions to.

## The ask

Anselm opened with "i want to build a playable sausalito", and was trying a cloud instance for the
first time. The Sausalito scenario was not started: what "playable" means and which hazard comes
first were still open when the conversation turned to the limits of the instance. His words on
that: "it's worth thinking about the computational limits of this cloud instance before we go to
far"; "i am going to want to explore playable experiences where the user has control"; "we may
have hundreds or thousands of users"; and on the repo, "debating if the esimv1 core has multiple
agents writing to a single git repo - or if new projects such as the proposed sausalito sim have
their own repo".

## What was measured here

- The container has 4 cores, 15 GB of memory, Rust stable and Node 22. `cargo build --release`
  passed in 65 seconds and `esim` runs. No scenario and no regress run was made, because the data
  is not here.
- The container holds the repo as pushed and nothing else: no `data/cache/`, no project runs, no
  viewer server.
- The environment's network policy let through the two AWS open-data buckets (Copernicus DEM,
  HRRR) and refused Overpass, Open-Meteo's archive, tidesandcurrents.noaa.gov and esimv1.exe.xyz
  (the proxy answered 403 to the CONNECT).
- `cargo check --target wasm32-unknown-unknown`, crate by crate: `esim-kernel` and `esim-audit`
  pass; `esim-registry` and `esim-modules` fail on one thing, `getrandom`, which comes in through
  `ureq`. This was a type check, not a build. Not proven: that `rquickjs` (which embeds C) builds
  for a browser target, or that the modules run there; 15 of the 38 files in
  `crates/modules/src` use `std::fs` directly.

## What I recommended

**Data.** Three kinds, treated differently.

- The source cache is already locked by hash in `data/catalog.lock.toml`, so an object store keyed
  by hash is a mirror the fetcher can try before the origin. The big public archives that are
  read by byte range (Copernicus, HRRR, WorldCover, MODIS) are already object storage and can stay
  where they are; the mirror pays for the slow, rate-limited or fragile sources (NASA POWER,
  Overpass, Open-Meteo, the World Bank, CWFIS) and for our reduced tables.
- Baked runs go in the same store under their own prefix, served through the store's CDN. A CDN
  is the front of the store, not a second place. `snapshots.json` stays behind, as the deploy
  already does. A run addressed by manifest id and log digest never changes and caches forever; a
  small pointer per run name can move.
- A scenario's prepared inputs (its terrain window, buildings, wind hours) as one bundle per
  scenario. `esim prepare` would write it and a run would load it with no fetching. This is the
  piece that serves canada1, a cloud container and a browser alike.

Anselm's position on redistribution: everything is open source, non-commercial and free. That
meets Open-Meteo's term. Attribution and share-alike terms (OpenStreetMap's ODbL, WorldPop's
CC BY) still apply to a public mirror and were not read on the day.

**Scenario text.** Manifests, `project.toml`, READMEs, pages and small record tables are text and
belong in git, where they are now.

**Compute.** Single runs are 5 to 10 seconds by the notes in `CLAUDE.md`; ensembles and fits are
what need many cores. Two kinds of play differ: decide then run (the whole run recomputed, fine
anywhere) and steer during the run (the engine has to sit beside the player). The second argues
for the player's device, with the engine as WebAssembly in a worker loading a bundle and a
manifest already resolved to JSON. Because a run is deterministic, a shared play is a link that
carries the manifest and the choices; nobody stores the output. What a browser cannot do
(ensembles, fits, the 3.3 GB cells mode, preparing a new place from the sources) goes to
stateless native workers that read and write the store and cache results by manifest id. That is
a queue and containers, not one large always-on machine. canada1 stays the development and
fitting box.

**Repos.** Do not split the engine and its modules yet: the module interface is being revised,
and a new scenario usually needs a new module kind, so every change would be made twice and
pinned. The clean line today is scenario content that needs no Rust (manifest, record folder,
page, README), which can move to its own repo once the engine loads a project from outside its
tree. For several agents in one repo, a branch per agent and small merges; the files every agent
touches are `crates/modules/src/lib.rs` (`KINDS`, `layer_of`), `data/catalog.toml` and
`CLAUDE.md`. `CLAUDE.md` is the worst of these: it is long and every session appends to it.

## Open

- What "playable" means for Sausalito and which hazard is first.
- Whether the mirror is public, and each source's terms if so.
- The real cost of a browser build: a spike on one small scenario would replace the guesses above
  with a build log and a timing.
- Which store. Cloudflare R2 was the one named; its pricing was not checked on the day.
