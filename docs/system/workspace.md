<!-- order: 12 -->
<!-- section: The machine -->
<!-- author: Claude -->
<!-- ordered-by: Anselm -->
<!-- written: 2026-11-01 -->
<!-- revised: 2026-11-22 -->
<!-- revision: 1 -->
# Workspace

## Layout

```
core/          the engine: crates/ (kernel, registry, modules, audit, cli, in Rust),
               manifests/ (the ladder, from two buckets to the planet; bundles to include),
               data/ (catalog.toml, catalog.lock.toml, events/ tables; cache/ is not in git),
               regress/ (the hindcast ladder and its accepted scores)
projects/      <area>/<name>/ project folders; runs inside them are not in git
runs/          loose runs, not in git
docs/          this documentation: <area>/<name>.md; protocol-v0.md
devlog/        dated entries: the design brief and every decision since
web/           the viewer and the server
todo.md        open work, one line each, pointing into the devlog
```

The engine and the workspace are separate folders. The engine's folder (`core/`, named by
`ESIM_ROOT`) holds code, the catalog, fetched data and the shared tables. The workspace
(this repository's top folder, named by `ESIM_WORKSPACE`) holds what people make with it:
projects, documentation and runs. `esim` run from the top folder finds both. Since
22 November 2026 the engine has been in `core/`; pages written before then name its
folders without that prefix.

## Areas

The first path segment under `projects/` and `docs/` is an area: a person's name or a
topic. There are no accounts; an area belongs to whoever writes there first, and the
convention is that others do not write into it. `system` is the documentation of the
simulator itself and `anselm` is its author's; a visitor picks a word and it is theirs.
When accounts arrive, areas map to them without moving anything.

## Stamps

Every documentation page says who wrote it and when, in comment lines at the top of the
file, so the answer travels with the file and does not depend on the repository's history:

```
<!-- author: Claude -->
<!-- ordered-by: Anselm -->
<!-- revised-by: Anselm -->
<!-- written: 2026-11-01 -->
<!-- revised: 2026-11-14 -->
<!-- revision: 1 -->
```

`author` is who wrote the text, a person or a model. `ordered-by` is who asked for it,
and `revised-by` names anyone else who changed its words. `written` is the day the page
was first written and `revised` the day its text last changed. `revision` starts at 1 and
goes up when a page is rewritten or changes what it says about something; an addition or
a correction moves only the `revised` date. The page shows the stamp under its path, and
the documentation index shows the date and revision beside each page.

Whoever changes a page sets `revised` in the same change. Nothing checks this yet. The
stamps were added on 15 November 2026 from the repository's history: each page's first
and latest commit gave its two dates, and every page started at revision 1 whatever its
earlier changes, except the RFC, whose stamp carries its draft number.

## Working in a shared workspace

- **Write in your own area.** Read anything; write under your name or your topic. To
  change someone else's project, copy it into your area and say where it came from.
- **Describe in your own words.** A project's description is the claim; keep it as you
  said it, even when the manifest a model drafted falls short of it. The gap is
  information.
- **Say who and what.** The `by` line of a project names the person and, when a model
  drafted the manifest, the model. A page in the docs says who wrote it when that
  matters.
- **Name assumptions as assumptions.** A variant that runs past a record carries its
  assumption in its title. Do not call a run a forecast.
- **Do not tune.** If a parameter has to move, move it by a sweep or a hindcast and keep
  the record of it; do not edit it to make a chart look right.
- **Keep runs reproducible.** Do not edit a run folder. Rerun with a changed manifest
  instead; it lands in a new folder by hash.
- **Cite.** A new kind's parameters name their papers or say "guess"; a new catalog entry
  names the source, the licence and the day it was verified. "Recalled" is a confidence,
  not a citation.
- **Mind licences.** Several records are free for non-commercial use only and several
  licence texts are still to be fetched; nothing here is published outward until that is
  done. The catalog says which.
- **Be small.** A project is one question. A page is one subject. A module is one idea
  with its own record.

## Machines

Today everything runs on one development machine. The decided shape for a public
service keeps the pieces as they are: the viewer as static files, the project server as
the one process that needs a machine (run-to-completion jobs for the kernel), the cache
and the run folders in object storage, the catalog and the projects in git. Nothing in
the layout above would move.
