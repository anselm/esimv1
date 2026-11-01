# 2026-11-01 — Projects, the auditor, and a viewer built around maps and time

Anselm, after the far-field entry: "Ok tackle the user facing issues. Projects focused
views. Letting users add new projects. Map centric views. Scoring and tearing. We don't
need to worry about cloud run yet." Agent: Claude (Fable 5.1), earth-simulator-1 room on
canada1, the same afternoon as P1.

## Projects are folders

`projects/<area>/<name>/` holds `project.toml` (title, the person's description in their
words, who and when), `manifest.toml`, and `runs/`. The CLI owns it: `esim project new
<area/name> --manifest f [--title] [--description] [--by]`, `esim project run <area/name>`,
`esim projects [--json]` (which also refreshes `projects/index.json`). An area is a word
a person chooses today and an account later; there are no accounts. Five projects exist
under `anselm/` and one under `guest/` (below). The run folders are gitignored like the
loose `runs/`; the manifests and descriptions are committed.

## The auditor

`crates/audit`, an independent crate that reads a finished run folder (`run.json`,
`log.jsonl`, `scores.json`, `manifest.toml`) and never the live run. `esim run` calls it
at the end and writes `audit.json`; `esim audit <run_dir>` reruns it. It holds no theory
of any system; it measures what every run with the right ingredients exposes, and it
lists what it could not measure instead of inventing a number.

- **Spend-down.** Every ledger stock's balance at every close is already in the run.
  Change over the run and over its last quarter, per account: spent down, accumulating,
  steady, or accumulating from nothing.
- **Diversity.** Shannon evenness and effective number over the population stocks (a
  ledger in `individuals`). With one population it says so.
- **Shocks and recovery.** Shock events from the log (`fire.burn` and `hazard.*` by
  default; `[audit] shock_topics` in the manifest to change) grouped into episodes. For
  every stock, the drop after the shock and the time to return within 5% of its
  pre-shock mean, against the interval to the next shock. Recovery shorter than
  recurrence is stretching; no recovery before the next shock, or a year on, is a tear.
  That is Anselm's own definition from the pasted text and it needs no model of the
  system, only its record.
- **Tears.** Extinctions, failed closes, unrecovered shocks. A verdict for the run:
  holds, stretching, tearing, with reasons.

A manifest may also name its scoreboards, `[audit] scores = [["sim", "obs"]]`, and the
run scores them; P1 does.

What the first real audit said, on the September fire project: the fire took 4.5% of
the deer and the herd was back within the band in nine days; biomass and carbon were
unmoved; and the water stocks "fell 66% and did not recover", which is autumn, not the
fire. The auditor measured a season and called its drying a shock response. Two
limitations follow and are stated in the code: a season-long run confounds recovery
with seasonality, so the recurrence test wants multi-year runs; and the first version
found populations by series name and counted flow series (`herb_fled`) as populations,
so populations and watched stocks now come from ledger declarations only. A module that
wants the auditor to see its population must keep it as a declared stock.

## The viewer

Routes are URLs: `/` the projects, `?project=a/n` one project, `?project=a/n&run=id` a
run, `?new` the form, `?runs` the loose runs. The run page leads with the maps in a
grid under one time cursor with a play button; every map shows the frame at or before
the cursor and every chart marks it with the value under it. Then the audit, the scores,
the charts, the ledgers (the close grid collapses to a strip past 120 closes, which the
175-year planet run needed), and the raw events and manifest behind disclosures.
Verified by screenshot in light and dark. Not done: an overview map on the project page
and the projects list, which wants a place in the manifest first.

## Adding a project

`web/server.mjs` is a thin socket.io layer over the CLI (the house prefers one socket
with structured requests to REST): list, get, draft, check, create, run, with log lines
streamed back. The form asks for an area, a name, a title and a description in the
person's words. "Draft a manifest" sends the description with `esim kinds --json` and a
short grammar to a model; the answer's TOML goes through `esim check`; errors go back to
the model for up to three rounds; the person can edit the TOML or paste their own; Create
writes the folder through `esim project new`; Run streams `esim project run`.

The Anthropic key in `~/.bashrc` answered "credit balance is too low" on every call
today, so the facade could not be tested on Claude. The server falls to a local model
through ollama and labels the tier; `gpt-oss:120b` on canada1 drafted the test project
("the Boone River basin through the 2025 growing season, with the river checked against
the USGS gauge") in one round, five minutes, and the kernel accepted it unchanged. It
missed one thing: it said the flow could not be scored, when the basin module reports
the gauge series beside its own. The grammar now lists the scorable pairs. The project
`guest/boone-dry-summer-2025` was created and run through the server end to end.

## Place once, forced by the first drafted manifest

The drafted project crashed the basin module with an index out of bounds. The draft had
left every module on its defaults, and the defaults disagreed: the sun module's window
and resolution were not the basin's. The hand-written manifests never hit this because
they repeat the window on every module, which is the repetition the roots entry called a
corner. Three fixes landed together: every spatial kind now has the same default window
and resolution (the Boone window at 30 arcseconds); a manifest may state a `[place]`
table once, and the parser copies its keys into every module that does not set them,
before the check and the build; and the kernel gained a grid-checked read that names
both grids and both modules instead of panicking on an index. The drafting grammar now
asks for `[place]` in every manifest.

## Not done

- An overview map per project (the place table makes it possible now).
- The conversation is one turn; a second turn to refine a draft is a small addition.
- Auditing ensembles (bands, not single trajectories).
- Credit on the Anthropic account, which is Anselm's.
