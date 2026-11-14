# Todos, after the shaping passes of 14 November 2026

Anselm: "Looking back at recent comments - let's log issues to the todo. You observed several
good points to fix." This gathers every rough edge and open point reported in this session's
replies since the morning's list (`devlog/20261114-critical-todos.md`), which still stands for
the model's faults. Where an item is already there it is only named. Each line says where the
thing is.

## The site and its pages

1. **One `run.html` serves every variant of a project**, so a variant cannot have words of its
   own. Allow `run-<variant>.html`, or a tag that shows text only for named variants.
   (`web/src/lib/RunView.svelte`, `Notebook.svelte`)
2. **Flat maps of Portland are cluttered** with OpenStreetMap place names since `roads.routes`
   began carrying the road context. Thin the names by kind and by zoom. (`FieldMap.svelte`)
3. **The sun module's picture is an elevation map**, its only map output. Give `sun.irradiance`
   an irradiance frame, or draw its picture from a series.
4. **Kind pictures go stale.** `web/scripts/kind-pictures.mjs` is run by hand. Run it in the
   deploy. Ten of 53 kinds have no picture because no run on disk has an output of theirs that
   varies.
5. **The notebook page has not been seen on a phone**, and the effects have been seen only in
   one still at the instant of the earthquake, never in motion or on a real device.
6. **The live front page was not looked at after the globe went on**, only the local one.
7. **HTML pages are trusted code.** Fine while scenarios arrive through GitHub. Before anyone can
   upload a page from the site, it needs a sandbox. (`docs/system/pages.md` says so.)
8. **Every other project lacks the 3D view** until its runs are redone: only Portland and one
   Palisades variant have been rerun since modules carried layers.
9. **The older `Scene3D` is still used** inside the fire modules' panels, beside the general view.
10. **A run's `describe.json` is 1.5 MB** with road and bridge lines in it; the notebook and the
    scene load it whole.
11. **The front page's counts are literal.** "81 runs on record" counts run folders, most of them
    variants of a few scenarios. A funder may read it as more than it is.
12. **Six projects are featured, by this session's choice** (Portland, the Palisades cells, Earth
    2100, Half Moon Bay, Fort McMurray, the Boone predators). His to confirm or change.
13. **Docs and kinds lists have no paging**; not needed at their size today.

## Manifests and the RFC

14. **Thirty-seven manifests are still TOML**, and so are the RFC's examples. Convert by hand as
    each is touched; `esim manifest --js` output is longer than the TOML it replaces.
15. **The RFC needs a draft that leads with the source form**, since the project now writes
    scenarios in it.

## The engine

16. **Snapshots are 1.4 GB a run** for Portland. Left out of the deploy; not fixed. Find the
    module whose saved state is large and mark what can be rebuilt on restore as not saved.
17. **A new place's first preparation waits on OpenStreetMap**, up to five minutes a request, and
    the engine cannot list what a scenario needs without creating its modules. (critical todos 6)
18. **Several modules each read the same file** (the residents map four times, the elevation
    window by every module). One provider module per shared dataset. (critical todos 7)
19. **The generalized Portland modules still fall back to Portland's coordinates** when a
    manifest states no place (`people.aftermath`, `people.agents`, `quake.ground`,
    `quake.buildings`, `portland.lifelines`); the newer ones default to zero.
20. **`people.aftermath` is general across earthquakes only**: it listens for `quake.toll`.
21. **`portland.lifelines` still holds drinking water's schedule, the story and the older
    `ranges` version.** (critical todos 8; the primer's "water system failure after earthquake"
    entry is the design for the replacement)
22. **`water.surface` is not hydrology**: no flow, level or tide, so spilled fuel stays where it
    lands.
23. **Roads and bridges on the map**: spans come from OpenStreetMap ways within 180 m of a
    bridge's point, which can pick up a neighbouring ramp. Not checked bridge by bridge.

## Portland's standing small faults (unchanged, listed so they are in one place)

24. `retrofit` means built after 2004 in the lifelines' ranges mode and after 1993 in `fuel.tanks`.
25. The story gives a bridge's reopening one day early; `service.routes_open` reads 1 in computed runs.
26. The `burnside-ready` variant has no effect in computed mode.
27. A run reports 297,803 buildings; the data folder's note says 297,120 outlines.
28. The page's account of the ordinance rests on KOIN's report, not on the ordinance's own text;
    the 2012 DOGAMI study and the McCullough Research report are unread.

## Sources and scenarios

29. **Compare data sources** across this engine's catalog, the primer's appendix and
    digitaltwins.wiki. (`devlog/20261114-scenario-todos.md`, section D)
30. **The earlier attempt's scenarios** are set out as todos in the same entry, with a suggested
    order: the AR6 variants, then water pipes and a power network for Portland, then Hormuz.

## Done in the same sitting as this list

- **The project page has one list where it had two.** Variants and runs were separate lists and
  the runs, which are what a visitor wants, sat under the variants. Now each row is a version of
  the scenario with its newest run and an "Open run" button on it; what it changes is in small
  print, earlier runs of the same version fold under it, and a run of no variant ("as written",
  or custom) gets a row of its own. (Anselm: "Can this be one list? ... I am lacking visual
  clarity on 'runs'. That is mostly what I as a user wants to do. It is buried.")
