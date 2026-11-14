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

## Second pass, the same day

Anselm: "Ok make home page a bit richer ... Make pages as html notebooks please. Let's not worry
about sandbox risks. Ok use real runs for card depiction. Make front page prettier yes." And a
question: "Does JavaScript local state have the bounding region of that run in itself or is that
elsewhere?"

- **The answer on the bounding region.** It is in the JavaScript file itself: `export const
  place = { lat_s, lat_n, lon_w, lon_e, res_arcsec }`. When the manifest is resolved, the place
  is copied into every module's settings. Nothing about the region lives elsewhere, except that
  the scenario's tables (tanks, bridges, hospitals) are in the folder the file names.
- **HTML pages.** `page.html` and `run.html` beside a project's manifest, drawn by
  `Notebook.svelte`: the HTML is set as written, each `esim-*` tag is filled by `NbView.svelte`
  with the view it names, `<esim-tabs>` makes tabs of its sections, and the page's scripts run.
  Tags of one run share one moment (`RunCtx` in `notebook.svelte.js`). Eleven tags; documented in
  `docs/system/pages.md`. Portland's `run.html` is the first: the scene, a timeline, five live
  counters and seven tabs (what happens, buildings and people, fuel, bridges and roads, power and
  hospitals, how far to trust it, everything recorded), with commentary that holds for every
  variant and says where each result is weak. The engine's own layout is at `?raw`.
- **Kind pictures from real runs.** `web/scripts/kind-pictures.mjs` draws, for each kind, its
  first map frame at the end of a run that uses it, or a series over the run when it has no map,
  in its layer's colour: 43 of 53 kinds. The other ten have no output that varies in any run on
  disk. `docs/kinds/index.json` says which project and which output each came from, and the card
  says so too.
- **The front page**: a statement of what this is over the Portland scene, three counts read
  from the repository (scenarios, kinds, runs), six featured scenarios, then the workspaces.
  `featured` was true on all fourteen projects; it is now true on six (Portland, the Palisades
  cells, Earth 2100, Half Moon Bay, Fort McMurray, the Boone predators).

Looked at locally: the front page at desktop width, the kinds page, the Portland run page with
its Fuel tab open. Not looked at: the notebook on a phone.

Known rough edges: the 2D maps now show OpenStreetMap place names for Portland and there are
too many of them; `run.html` is one template for all variants, so a variant cannot yet have
words of its own; the kind pictures are made by hand-run script and go stale when runs change.
