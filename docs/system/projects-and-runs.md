<!-- order: 7 -->
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
in a label become spaces). The figures come from the runs and from nothing else: the
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

## A run is a folder

```
runs/<name>-<manifest hash>-<log digest>/
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

The folder name carries the manifest's hash and the log's digest, so two runs of the same
manifest and seed land in the same folder and two different manifests never collide.
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
