<!-- order: 7 -->
# Projects and runs

## A project is a folder

```
projects/<area>/<name>/
  project.toml      title, the person's description in their words, who, when, variants
  manifest.toml     the resolved manifest
  runs/             one folder per run, and index.json
```

The folder is the project. The command line writes it (`esim project new`), runs it
(`esim project run`, with `--variant` or `--set`), and lists projects (`esim projects`);
the viewer and the server only read and write the same files. Anyone can copy the folder
and run it elsewhere.

The description matters more than it looks. It is what the person asked for, in their
own words, kept beside the manifest a model drafted from it. When the manifest and the
description disagree, the description is the claim and the manifest is the attempt.

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
