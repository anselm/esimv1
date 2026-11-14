# Where the earlier attempt's raw materials are

14 November 2026. Claude Code session in the esimv1 room on canada1
(session_018TGCFiQHbUYn12HkNti2xk, Fable 5.2).

Anselm, after the globe was taken from orbital-sim: "The raw materials are local somewhere. This
was an earlier attempt but built on the older js core." So orbital-sim is the predecessor of this
engine: the same aim, on the earlier JavaScript core. This entry is a map of what is on canada1
under `~/projects/2027/orbital/`, for whoever wants to bring something across. Nothing there was
changed; another project's folder is read-only to this room.

| What | Where | Size, state |
|---|---|---|
| The primer's source, *Simulate the World* | `orbital-primer/chapters/*.md`, built with `Makefile`, `book.yaml`, `book-style.tex`, `refs.bib` | 21 chapter files, 41,000 words; version 0.4, June 2026; its preface asks not to be circulated yet |
| A library of scenarios to model | `orbital-primer/chapters/80-reference-scenario-library.md` | 3,100 words |
| Data sources, as the primer lists them | `orbital-primer/chapters/91-appendix-data-sources.md` | 1,000 words |
| Scenario thinking | `orbital-wiki/theory/20260531-scenarios-to-model-abstract.md`, `20260604-scenarios-to-model-concrete.md`, and three more theory notes | short notes |
| Technical notes | `orbital-wiki/technical/` (model parts, interface goals, identity and signing) | three notes |
| The earlier scenarios | `orbital-sim/public/orbital/{hormuz,ipcc,planetary,world-events}`, `orbital-sim/public/anselm/{tuvalu,tenerife-futures,africa_middle_east_warming_2049,microeconomy}` | each a folder with a JavaScript manifest and its data |
| The earlier engine's modules | `orbital-sim/packages/agents/` (atmosphere, elevation, insolation, spatial, ingestion, hormuz) | JavaScript, on the older core |
| The site and its globe | `orbital-sim/website/`, `orbital-sim/viz/` | live at orbital-sim.exe.xyz |
| The Earth texture | `orbital-sim/public/assets/earth.jpg` | NASA's Blue Marble (Anselm, 14 November 2026: "Earth texture is def blue marble"); the folders do not record it |

What looks most useful here, not yet acted on:

- The scenario library and the two scenario notes, as a list of what to model next in the
  discovery phase.
- The primer's data-sources appendix, to compare with this engine's catalog.
- The Hormuz, Tuvalu and MENA scenarios, as candidates to rebuild on this engine's modules. Their
  JavaScript manifests are the earlier core's, not this engine's notation, so each would be a
  rebuild and not a port.
- The primer's chapters on publication and on participation, as the long form of the goals page.
