# Critical todos, as of 14 November 2026

Anselm: "lets track critical todos." This is the list as it stands at the end of the session of
12 to 14 November (Claude Code, esimv1 room, session_018TGCFiQHbUYn12HkNti2xk). It is a dated
snapshot: a later list supersedes it with a new entry. Each item says where its detail lives.

## Critical: results a reader would be misled by

1. **The two death counts disagree by day.** The buildings module's count by cell and the agents'
   count differ by a tenth to a third in daytime runs (1,000 against 670 in the base run) and
   agree at night. Cause not found. The page's total uses the agents'. Until this is found,
   neither figure should be quoted. (`devlog/20261112-portland-readme-rewritten.md`)
2. **Electricity comes back in three weeks where the state expects one to three months.** The
   model has transmission structures only: no substations, lines or neighbourhood distribution.
   How many hospitals lose power and how many die afterwards both follow from this, so both are
   too low. (`power.structures`; the page, *Electricity*)
3. **Every fragility value is recalled from memory.** The curves for buildings, tanks and modern
   bridges and every casualty rate are from memory of the Hazus manual, unread. The spill comes
   out at half to three quarters of the 2022 study's minimum for this reason. Read the manual
   and replace the values. (`data/events/portland-cascadia/fragility.toml`)
4. **The response uses no fuel for 118 days.** Its draw starts only when a truck can reach a
   hospital. The reserve half of the storage argument cannot be read off the model until this
   is fixed and the daily use and the resupply day are sourced. (`fuel.depot`)
5. **A river forecast is overdue for checking.** The ten-day Boone River forecast issued on
   1 November 2026 was to be scored against USGS on or after 11 November. Not done.
   (`devlog/20261101-f1-first-forecast.md`)

## Critical: the engine

6. **A new place's first preparation waits on OpenStreetMap inside the step**, up to five minutes
   a request. The engine cannot yet say what a scenario needs without creating its modules.
   (`devlog/20261114-preparing-a-scenario.md`)
7. **Several modules read the same file.** Four read the residents map and every module asks for
   the elevation window. One provider module per shared dataset would end it, with no kernel
   change. (same entry)
8. **`portland.lifelines` still holds drinking water's schedule, the story and the whole older
   `ranges` version.** Water as its own module is the next piece out.
   (`devlog/20261113-layers-and-portland-modules.md`)

## Important: what the model does not have that the argument is about

9. **Fire at the fuel terminals and evacuation.** The ordinance's own plan (due 30 September
   2027) is about both; the model has neither. A fire engine exists in this repository from the
   Palisades work.
10. **Hydrology.** `water.surface` says where the water is and sets the spill down on it. It has
    no flow, river level or tide, so it cannot say where the fuel goes.
11. **Night and dry soil.** Only shaking makes a building collapse, so the toll barely changes
    between those cases, where DOGAMI's falls sharply.

## Smaller, known and standing

- `retrofit` means built after 2004 in the lifelines' ranges mode and after 1993 in `fuel.tanks`.
- The story gives a bridge's reopening one day early; `service.routes_open` reads 1 in computed runs.
- The `burnside-ready` variant has no effect in computed mode.
- A run reports 297,803 buildings; the data folder's note says 297,120 outlines.
- The KOIN article's account of the ordinance is on the page unchecked against the ordinance's
  own text; the 2012 DOGAMI study and the McCullough Research report are unread.

## The 3D view

- The effects were seen in one still at the instant of the earthquake, not in motion, and not on
  a phone.
- Only Portland and one Palisades variant have been rerun; every other project keeps its cover
  picture until its runs are redone.
- The older `Scene3D` is still used inside the fire modules' panels.
- A run's `describe.json` is 1.5 MB with road and bridge lines in it.

## Added the same day, at the deploy

- **A run's `snapshots.json` is very large**: 1.4 GB for one Portland run, 15 GB for the project's
  ten. They filled the VM's 25 GB disk and stopped the deploy. The deploy now leaves the file
  behind (the read-only copy never resumes a run and the viewer never reads it), which took the
  VM's projects from 18 GB to 6 GB. The size itself is not fixed: some module's saved state is
  about 120 MB per snapshot, twelve times a run. Find which, and mark what can be rebuilt on
  restore as not saved. It also fills canada1 at 1.5 GB a run.
