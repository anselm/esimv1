<!-- order: 7 -->
<!-- section: Projects and the record -->
# Projects and runs

## A project is a folder

```
projects/<area>/<name>/
  project.toml      title, the person's description in their words, who, when, variants
  manifest.toml     the resolved manifest
  README.md         the author's account of the scenario, with live figures (optional)
  runs/             one folder per run, and index.json
```

The folder is the project. The command line writes it (`esim project new`), runs it
(`esim project run`, with `--variant` or `--set`), and lists projects (`esim projects`);
the viewer and the server only read and write the same files. Anyone can copy the folder
and run it elsewhere.

The description matters more than it looks. It is what the person asked for, in their
own words, kept beside the manifest a model drafted from it. When the manifest and the
description disagree, the description is the claim and the manifest is the attempt.

## A depiction, and the front page

A project may carry a picture: a file beside the manifest, named in `project.toml` as
`cover = "cover.png"`, with `cover_caption` saying what it is and where it came from. It is
shown at the top of the project page and on the front page's card. The house rule for
images holds: a picture drawn from the runs, or a real photograph, or a piece of art made
for the scenario; never a stock image standing in for the place. A README may also show a
picture that sits in the folder, `![what it is](file.png)`.

`featured = true` in `project.toml` puts the project on the front page, which shows only the
featured scenarios as cards; `?all` lists every project under its area.

## Accounts, on a public copy

A public, writable copy (the server started with `ESIM_PUBLIC=1`) asks who you are before it
writes anything. Identity comes from [attest.monster](https://attest.monster): you sign in
there with a passkey, and your attest handle is your area. You can create, run and write
only under `/<handle>`; the first time you do, `projects/<handle>/area.toml` records the
handle and the account's DID, and from then on the area is yours. attest checks a handle
against its reserved and offensive word lists when it is made, so this server does not; it
keeps only a short list of its own words (`docs`, `runs`, `kinds`, `compare`, and so on)
that cannot be areas. A private copy, run without `ESIM_PUBLIC`, asks nobody who they are.

## A project's README

A project folder may hold a `README.md`, and the project page renders it above the
variants and runs: the author's own account of the scenario, in their words, beside the
manifest a model drafted and the runs the kernel made. It is Markdown, and a fenced block
whose language is `esim` is a live figure drawn from the project's runs:

```
    ```esim
    ladder
    ```
```

draws the tipping elements' thresholds as rungs on a warming axis with each variant's
path climbing past them, the observed record as dots, and a table of the year each path
passes each rung; and

```
    ```esim
    table 2100 energy.gmst_anom_c=warming carbon.co2_ppm=CO₂ people.outside_niche=people_outside_the_niche
    ```
```

prints each variant's value of the named series at the start of that year (underscores
in a label become spaces); and

```
    ```esim
    actors
    weather: wind, dry air, heat, the sun
    fire
    weather -> fire: drives the head downwind
    fire -> weather: makes its own wind (not in the model)
    ```
```

draws the actors and the relationships between them in the author's own words: a line
`name: words` is an actor with a caption, `a -> b: words` an arrow read along its
direction, and words ending in a parenthesis that begins "not" mark a relationship the
world has and the model lacks, drawn dashed with the words kept. The actor with the most
arrows stands in the middle. It is the one figure an author draws by hand, and it is for
the picture a person carries of the system, which the wiring diagram on a run page (modules,
topics, fields) does not give. The figures with numbers in them come from the runs and from nothing else: the
rungs are the table the run's own describe records, the paths and values are the runs'
series, the years are the runs' crossing events. An author writes the story and cannot
type a number into a figure. Prose may of course quote numbers, and a quoted number goes
stale when the runs change, so the honest habit is to quote few and let the figures
carry the rest. `projects/anselm/planet-2100/README.md` is the first one.

Other materials belong in the folder too: images (real ones), a bibliography, the
sources' PDFs where their licence allows; the server serves the folder as it is. The
folder is the namespace: first come first served per area, a project's own things under
its own path.

## Variants

`[[variant]]` tables in `project.toml` name parameter sets: a title, a name, and a `set`
of overrides. One project holds several assumptions this way (emissions held; emissions
to zero) and its runs are labelled by variant. Running a project "as written" applies
none.

## Preparing a scenario

A module is generic code. It learns where it is only when a run creates it and hands it the
scenario's place and its own settings, and that is when it works out which data it needs and
fetches what is missing. So everything a run does before its clock starts is a preparation: the
manifest is resolved, its wiring is checked, and each module is created in turn. Data is on disk
before the first simulated instant.

`esim prepare <manifest>` does that much and stops. It prints each module as it becomes ready and
every file that was fetched, and it writes no run. `esim project prepare <area/name>` does it for
each of a project's variants. After it, a run of the same scenario fetches nothing.

Any command takes `--offline`: nothing may be fetched, and a missing file is an error that names
the file and where it would have come from. Use it where a run must not wait on a network, such
as an ensemble or a server.

Fetched data is shared. A file is kept by what it is, not by which scenario asked, so fifty
scenarios of one place fetch each file once. Two preparations started at the same moment are
safe: one fetches a file while the other waits and then finds it. Requests have time limits, and
requests to one server are spaced. Answers from OpenStreetMap are slow and unreliable to fetch,
so once reduced they are kept in the repository under `data/osm/`.

## A run is a folder

```
runs/<run name>/
  manifest.toml     the resolved text this run used
  variant.json      the variant and overrides, when any
  describe.json     every module as it described itself for this run, with the small tables it read
  run.json          window, seed, ledgers with every close, series, field metadata, timings
  log.jsonl         every event, in order, with its data
  fields/<name>/    one f32 frame per written snapshot, row-major from the north edge
  scores.json       skill of each named model series against its scoreboard
  audit.json        the auditor's reading
  snapshots.json    kernel snapshots, when the manifest asked for them
```

A project's run has a plain name, and the name is its address: `/anselm/portland-cascadia/wet-day`.
The name is the variant's, or `base` for the manifest as written, or whatever `--name` gives.
Running a variant again replaces the run of that name, so the address stays the same while the
scenario is revised. What exactly a run was is recorded inside the folder: `manifest.json`
carries the manifest's id (a hash of its resolved form) and `run.json` the digest of the log.
Runs made from a manifest file outside any project have no name to go by, and their folder is
named for those two hashes: `<manifest name>-<manifest hash>-<log digest>`.
Runs are large (a basin year is about 45 MB, mostly frames) and are not kept in git;
projects and their descriptions are.

## Determinism and replay

`esim verify` runs a manifest twice and from a snapshot and requires the same digest and
the same ledgers. Modules draw randomness only from a seeded stream the kernel gives
them, keyed by the run seed and the module's name, so a change of seed changes exactly
the stochastic parts.

## Ensembles and sweeps

`esim ensemble` runs branches with consecutive seeds in parallel and keeps quantile
bands per series; it warms the data cache once before spawning, so branches never
download. `esim sweep` samples parameter ranges and scores each sample against a
scoreboard. Both write under a project's `runs/` when run for a project.

## Loose runs

Runs made from a manifest file rather than a project land in the repository's `runs/`
folder and show under "loose runs" in the viewer. They are the development scratch
space; anything worth keeping becomes a project.
