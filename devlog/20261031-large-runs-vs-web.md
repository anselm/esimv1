# 2026-10-31 — Extremely large simulations versus a web app

Anselm, same day: "I'm thinking about the tension between running extremely [large]
simulations and web based apps". Agent's assessment.

The tension: a thousand branches over thirty years at basin scale is minutes to hours on a
server; a web app is expected to answer in seconds. These do not meet in the middle. They
meet by separating the run from the view, the way every forecasting service already does:
ECMWF runs for hours, the app shows the forecast. ESRI's answer is the same, heavy
geoprocessing is a server job and the map is a client of its output.

Five mechanisms, all of which the brief's design already pays for:

1. Runs are jobs, the app is a client of results. The event log plus snapshots make a
   finished ensemble a dataset: distributions, ledgers, per-branch logs. Browsing it is
   instant and needs no simulator in the loop. Most of what a user does is browse.
2. A standing baseline. Keep a business-as-usual ensemble always run and kept current by
   assimilation (Anselm's June 2026 note already asked for a general scenario with Hormuz as
   a perturbation). Most questions are then diffs against the baseline, not fresh runs.
3. Fork, do not restart. An intervention is a manifest diff at a date. Fork every branch
   from its snapshot nearest that date and run only forward from there. A what-if in 2031 on
   a run to 2050 costs the years after 2031, not the years before. The conservative
   scheduler and module-private snapshots exist for exactly this.
4. Coarse first, fine later. Ensembles are embarrassingly parallel, so the first ten branches
   return in seconds with a wide band, and the band narrows in the background as the other
   nine hundred and ninety finish. A coarser grid or longer step for the first pass is the
   same trick on the other axis. The user sees an honest answer immediately and a better one
   on return.
5. Stream the log. The log is append-only, so the browser can tail a running branch and the
   narrator can narrate as it runs. Progress is the content, not a spinner.

Consequences:
- The browser never runs the big thing. A small wasm preview for the parcel case is a toy
  with a label saying so, and is optional.
- Compute is a real cost, which is why billing by branch-year fits: the economics of the
  product match the physics of the tension.
- The job runner, the snapshot store and the result store are kernel-adjacent deliverables,
  not product afterthoughts. Add to stage 2: V5 job runner with progress and notification,
  and a result store the viewer reads. Fork-from-snapshot is already D3.
- The sober viewer (P4) is a client of the result store, with one socket for the live tail.
