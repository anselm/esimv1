# Preparing a scenario: what the engine is, and where data gets fetched

14 November 2026. Claude Code session in the esimv1 room on canada1
(session_018TGCFiQHbUYn12HkNti2xk, Fable 5.2), in conversation with Anselm.

## The conversation

It began with a stall: the first OpenStreetMap fetch for Portland held a three-second run for
ten minutes. Anselm: "that slow osm issue is a concern. we may need to cache." Then he stepped
back: "i do want to step back and think about the engine as a whole".

Where it settled, in his words and the answers:

- "esim1 is a simulation engine - it loads a scenario, a scenario has modules, and it runs that
  scenario". Yes: a manifest is a place, a window, a seed and a list of modules with settings;
  the kernel is a clock and a bus; a run is a batch that writes a folder; the site plays folders
  back.
- "I do want to avoid polluting the kernel with special case behavior as a general principle. We
  currently have an elegant abstraction where we have just a message bus and an ability to
  register listeners." The kernel crate is about 2,300 lines, 430 of them the run loop; besides
  delivery it gives modules a clock, ledgers, shared fields, reports and determinism. A proposal
  to have the kernel collect data declarations and run a fetch queue was withdrawn on this
  principle. `enabled = false` in the manifest reader, added the day before, he let stand:
  "enabled is probably ok in the kernel - generally we have to remember somehow that we want to
  keep the kernel small."
- "does that mean a module is 'compiled' for a given scenario? because if a module is somewhat
  generic then it would not know if it is seattle or portland." A module is generic code; an
  instance is created at the start of a run and handed the scenario's place and its settings.
  That is the first moment it can know what data it needs.
- "so we do have a kind of compilation phase - the reading of a manifest. that makes sense. that
  is a reasonable time to guarantee that the data exists - prior to a run." The phase already
  existed: resolve, describe, check the wiring, create the modules, start the clock. The stall
  was in "create the modules", before the clock. What was missing was the step having a name.
- "we can always pass extra hints in the manifest to a module."
- On fifty Portland scenarios: data is kept by what it is, so it is fetched once. On two runs at
  once: a real race in the fetcher (a shared temporary name, a hash list rewritten whole).

Not taken up: a message on the bus that warms every module (a module needs its data before it
can declare itself, and a message is inside a run); each kind stating its exact files in its
describe, so the engine could say what a scenario needs without creating anything (cleaner, but
new code in every module that fetches); one provider module per shared dataset, so that four
modules do not each read the same residents map (needs no kernel change; not done).

## What was built

- **`esim prepare <manifest>`** and **`esim project prepare <area/name> [--variant v]`**: resolve,
  check the wiring, create each module in turn, print each as it is ready and every file fetched,
  and stop. No run folder.
- **`--offline`** on any command, or `ESIM_OFFLINE=1`: nothing is fetched; a missing file is an
  error naming the file and its URL.
- **The fetcher** (`crates/registry/src/fetch.rs`), safe and bounded:
  one process at a time fetches a given file, held by a lock file under `.locks/` that the
  system releases if the process dies; a download goes to `<name>.<pid>.part` and is renamed;
  the hash list is re-read under a hold before it is rewritten, so two processes adding different
  entries both keep theirs; thirty seconds to connect, two minutes for a read to make progress,
  five minutes for a whole answer; requests to one host a fifth of a second apart. The MODIS
  search and token calls use the same client.
- `data/osm/` is found beside the hash list, not beside the cache, so pointing `ESIM_CACHE`
  elsewhere does not lose it.

## Checked

- `esim prepare manifests/q1-portland-cascadia.toml` with everything present: thirteen modules,
  nothing fetched, 3 seconds.
- `--offline` against an empty cache: refused at the first module, naming the elevation tile.
- Two preparations started together on an empty cache: one fetched the elevation tile (46 MB),
  the other the land-cover tile (65 MB), neither fetched twice, no `.part` left, both finished in
  13 seconds, and the hash list was unchanged because both files matched it.
- `cargo test` (describe) and `esim regress`: pass, no drift.

A first version of the race test pointed the cache elsewhere and so lost `data/osm/`, asked
Overpass again, waited five minutes, and was refused by the hash list because OpenStreetMap had
changed since the first fetch. The refusal is right (the list protects what earlier runs used);
the lesson is that an OpenStreetMap answer is not reproducible, which is one more reason it is
kept in git.

## Still open

- A new place's first preparation still waits on Overpass for up to five minutes a request.
- The engine cannot yet list what a scenario needs without creating its modules.
- Repeated reading of the same file by several modules in one run is unchanged.

Committed, not pushed, not deployed.
