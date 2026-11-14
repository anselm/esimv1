# Shaping the site for funding: first pass

14 November 2026. Claude Code session in the esimv1 room on canada1
(session_018TGCFiQHbUYn12HkNti2xk, Fable 5.2).

Anselm: "i want to do a ton of changes to shape this for funding. you can commit, push, deploy
after." Nine numbered items followed (0 to 8). This entry says what was done for each, what was
answered without building, and what is left.

## Done

- **0. The front page lists workspaces**, the top folders (`/anselm`, `/guest`), each with its
  count of projects and runs and up to three covers. `/anselm` lists its projects ten to a page.
  `Pager.svelte` is on the workspace list, a workspace's projects, a project's runs and the
  loose runs. `/all` still lists every project. Contributors still come through GitHub.
- **0.1. The corner badge links to the front page; the version is 0.2.0** (`Cargo.toml`,
  `web/package.json`).
- **1. The docs page has three levels that look like three levels.** An area ("The system",
  "The future", "Standards") is a large heading over a heavy rule with a line saying what it is;
  a section is small coloured capitals in a left column; a page is a plain row beside it. Three
  reference cards sit on top: module kinds, data sources, limits.
- **2. Layers are numbered** 1000 (sun) to 10000 (people), a thousand apart; disturbances and the
  record have no number. In `LAYERS` (lib.rs, layers.js), the layer panel of the 3D view, the
  kinds page and `docs/system/layers.md`, which was already rewritten on the 13th to the current
  order (built structures, flows and law and money inside the not-alive half).
- **3. JavaScript is the lead notation.** `manifests/q1-portland-cascadia.js` is written by hand
  (75 lines with comments, one `folder` and one `soils` stated once) and replaces the TOML. Its
  manifest id is identical to the TOML's, so it is the same scenario; all ten variants were rerun
  from it with the same results. `docs/system/manifests.md` now leads with it and says what the
  file is: a description evaluated once before the run, with no part in the simulation and no
  state. That is the answer to his question about "local state": the program is sealed (no clock,
  no random numbers, no files, no network) so that one file always means one scenario; what
  changes during a run lives in modules. `esim manifest --js` exists but its output is longer
  than TOML (209 lines for Portland), so conversion that is worth having is by hand.
- **6. `docs/system/goals.md`, "Goals and milestones"**, in a new docs section "Where this is
  going": the goal (a service ordinary people use and others deploy for their region), four
  phases (discovery, refinement, experience, deployment), how things work today, where the
  project stands. Organised from his notes; his joke about time is left out for want of a source.
- **7. The kinds page is cards by layer**, each layer a large coloured heading with its number,
  empty layers shown, a card per kind with the first sentence of its own summary, opening to the
  full detail. It is reached from the docs page and returns to it.
- **8. A data sources page exists**: `docs/system/sources.md`, written by `esim sources --write`
  from the catalog, and `docs/system/data.md` on how data is handled. Both were under "Projects
  and the record"; the sources page is now also a card at the top of the docs.

## Answered, not built

- **4. What `/person/project` shows.** The project's `project.toml` (title, description, cover,
  variants), now the 3D view of its newest run, the variants, the runs, and then its `README.md`
  rendered from Markdown, with fenced `esim` blocks drawn as live figures from the runs.
- **5. Run pages as notebooks in HTML.** Not hard, and the pieces exist. The plan: each view the
  site can draw becomes a tag (`<esim-scene>`, `<esim-map field="...">`, `<esim-chart
  series="...">`, `<esim-counter>`, `<esim-story>`, `<esim-table>`), a project or a run may carry
  a `page.html` that lays those out with any HTML around them (tabs included), and the page as
  it is now becomes the default when there is none. The Markdown README keeps working. The care
  needed is that HTML from a contributor is code: fine while scenarios arrive through GitHub and
  are reviewed, and to be sandboxed before accounts can upload. Not built in this pass.
- **7, images.** Stock photographs were not used on the kinds cards. A stock picture of a river
  beside `water.basin` shows nothing the module does. The proposal is a small picture made from
  a real run of each kind, which the engine can produce. Not built.

## Left for the next pass

- The other 37 manifests and the RFC's examples are still TOML.
- `page.html` and the view tags (item 5).
- Depictions on the kinds cards.
- Pagination on the docs list and the kinds page was not needed at their present sizes.
- The front page no longer shows featured projects first; a funder landing there sees two
  folders. A short statement of what the project is, above the workspaces, is the obvious next
  addition.
