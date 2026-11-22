# A refactor pass: how to approach it

22 November 2026. Written by Claude (Fable 5.2, the esimv1 room on canada1) in answer to
Anselm. Nothing is moved or renamed by this entry. It follows
`20261122-playable-sausalito-as-an-exercise.md`, which has the parts table.

## What he asked

> i need to refactor esimv1 - i want to do a refactor pass on the repo. how to best
> approach? move kernel to esimv1-kernel repo? orbital-foundation had a pattern of many
> small separate repos with an overarching user experience, in this case esimv1
> effectively takes on the role of the multiuser hub. but i am thinking that each of the
> scenarios could also be a separate repo. if we prefix all repos with esimv1- then that
> may help reduce noise level since these are mixed into github.com/anselm which may not
> be right resting place ultimately. arguably a new org or maybe even pushing these into
> orbitalfoundation may be best? thinking about durable patterns for moving forward here
> - smaller modules seems best. thoughts?

## What the code says today

Checked in the Cargo files today: `kernel`, `registry` and `audit` depend on no other
crate of ours; `modules` depends on `kernel` and `registry`; `cli` depends on all four.
The viewer shares no code with any of them and reads run folders. The server starts the
command line as a process.

So there are two kinds of boundary already. Between the Rust crates the boundary is a
crate interface, checked by the compiler. Between the engine, the viewer and the site it
is a file format (the run folder, the project folder, the manifest).

## Recommendation

**Split repositories where the boundary is a file format. Keep the Rust crates together.**

A new kind usually touches `modules`, `registry`, a describe test and sometimes the
command line in one change. In separate repositories that is three releases in order. The
crates are already small modules; a workspace gives that without the release cost.
three.js, the comparison he made, is one repository. A separate `esimv1-kernel`
repository would be worth it only when someone outside wants the kernel without our
modules, and nobody does yet.

Repositories, with the prefix he proposed:

| repository | holds | depends on |
|---|---|---|
| `esimv1` | the hub: server, accounts, documentation site, the deploy | viewer package, a built engine |
| `esimv1-core` | the five crates, the catalog, the ladder manifests, regress, the protocol and RFC | nothing of ours |
| `esimv1-viewer` | the Svelte components that draw a run folder, as an npm package | the run folder format |
| one per scenario with a life of its own | project folder, record folder, manifest, page | a named engine version |

This differs from the earlier entry in one place: there I had the engine keep the name
`esimv1`. His version, where the hub keeps it, costs less. The deployed site, the local
folder, the room and the memory index all key on that name already, and the hub is where
a visitor arrives. `-core` rather than `-kernel` because the repository holds the
registry, the modules and the command line too, and `kernel` is the name of one crate.

Not every project needs a repository. Fourteen repositories each want a `CLAUDE.md`, a
devlog, a manifest and a room. The scenarios someone outside will care about (Sausalito,
Portland, the Palisades, Earth 2100) earn one. The ladder and the small examples stay in
the engine as its examples and its regression fixtures.

## The organisation

GitHub redirects a transferred repository, so this does not have to be settled before
the split. Use the prefix now under `anselm`; transfer later. The question that decides
it is who receives the grants. If the money goes into a foundation, the repositories
belong in that foundation's organisation, because funders look for the work under the
name they paid. `orbitalfoundation` has 31 public repositories today (counted through the
GitHub API), so the prefix is needed there as much as under `anselm`. A new organisation
is the cleanest listing and one more thing to administer. One caution on names: `v1` in a
prefix that is meant to last will read oddly when there is a v2.

## Order of work

1. **Write down the seams before cutting them.** The run folder gets a format version in
   `run.json`. The manifest has RFC 0001. The module contract has `protocol-v0.md`. The
   command line's arguments are the fourth.
2. **A project by path.** `esim project run <folder>` on a folder anywhere, with its
   record folder inside it, separate from the engine's own data root. This is the one
   code change that makes a scenario repository possible. Prove it by moving one small
   existing project out and back (Half Moon Bay: one kind, a small record, short runs),
   since Sausalito has no manifest yet.
3. **Rearrange inside this repository first**: `core/`, `viewer/`, `site/`, `scenarios/`,
   with the build, the regression ladder and the deploy all passing. A wrong cut is cheap
   to undo while it is one repository.
4. **The viewer as a package**: components that take the address of a run folder and
   nothing else from the site.
5. **Cut the repositories** with `git filter-repo`, which keeps each part's history.
   Scenarios one at a time, each rerun to the same log digest before and after.

## What stays awkward

- Kinds written for one place (`portland.lifelines`) are Rust and stay in the engine
  until a module host exists. A scenario repository is content only until then.
- The regression ladder runs real projects (the planet, the Boone year). Those stay with
  the engine.
- The fetched cache is shared by every scenario on a machine. It belongs to the engine's
  data root, not to any scenario.
- Client-side runs (`20261122-storage-compute-and-repos-from-a-cloud-session.md`) want a
  steppable kernel and a run that never touches disk. That is a change inside `core`, and
  is easier after step 3 than before it.

## Later the same day: his answer, and steps 1 and 2 done

Anselm:

> esimv1 repo naming - yes i just felt it was easier to leave esimv1 as the user facing
> repo - true the naming is not perfect, but it feels like we break less stuff if esimv1
> acts as the hub. esimv1-core sounds good. esimv1-viewer sounds good. repos only for
> truly significant scenarios makes sense esimv1-pallisades, esimv1-portland,
> esimv1-earth2100, esimv1-playable-sausalito and leave rest. we will move githubs later -
> can stay in /anselm . your recommended order is good. i see there is substantial work
> here - please take your time but yes go for it.

So the names are decided: `esimv1` (hub), `esimv1-core`, `esimv1-viewer`, and four
scenario repositories, all under `anselm` for now.

**Step 1, the run folder has a format number.** `run.json` now begins with `format` (1) and
`engine` (the version that wrote it). The command line adds them; the kernel is unchanged.
`docs/system/projects-and-runs.md` describes format 1 and says when the number moves.

**Step 2, a project by path.** Three changes, none in the kernel:

- `esim_registry::paths` is the one place that knows where the record folders are. Every
  module that built `ESIM_ROOT/data/events/...` by hand (sixteen files) now asks
  `paths::event(rel)`. When a project is being run and its own `data/events/` holds the
  first segment of `rel`, the project's copy is read.
- `esim project run|prepare|update|prune|new` take a folder as well as `<area>/<name>`.
  The command line sets `ESIM_PROJECT` to the folder before any module is made. A project
  by folder is left out of the workspace's index.
- `engine = "0.3.0"` in `project.toml` is the oldest engine the project runs on. The
  workspace version went from 0.2.0 to 0.3.0 so there is something to name.

The test: Half Moon Bay's project folder copied to a scratch folder outside the
repository with `data/events/halfmoonbay` inside it, the engine's own copy of that record
moved aside, and `esim project run ./hmb --variant as-built --offline` run from the scratch
folder. It gave log digest `70bece767182b643`, the same as the run in the tree, and its
`describe.json` carries the record's rows. The same run in the tree with the record moved
aside gives the same digest and a describe without the rows, because `coast.oneline` reads
a missing record as an empty one. That is a fault of its own (a missing record should be
an error) and is not fixed here. A project naming engine 9.0.0 was refused with a sentence
saying so. The regression ladder shows no drift and the tests pass. Everything was put
back; Half Moon Bay stays in the tree, as agreed.

A mistake on the way: the first builds ran in a shell without `cargo` on its path and my
filter hid the error, so the first regression pass and the first outside run were on the
old binary. Both were redone on the new one.

Not done by this step:

- A scenario's repository usually holds more than the project (a view, a devlog, the
  harness's own `project.toml`, which is a different file with the same name). The
  project folder will have to be a subfolder of the repository, such as `scenario/`.
- An answer from OpenStreetMap fetched for an outside project is still kept in the
  engine's `data/osm/`.
- `esim fit`, `ensemble` and `sweep` take a manifest and do not set `ESIM_PROJECT`.

## Step 3, first half: the engine is in `core/`

`Cargo.toml`, `Cargo.lock`, `crates/`, `data/`, `manifests/` and `regress/` moved into
`core/` with `git mv`, and the built `target/` and the fetched cache with them. That folder
is what will be cut out as `esimv1-core`. The top folder keeps `projects/`, `docs/`,
`runs/`, `web/`, `deploy/` and the devlog: the hub.

The command line has two roots where it had one. `ESIM_ROOT` is the engine's folder and
`ESIM_WORKSPACE` is where `projects/`, `docs/` and loose `runs/` are. With neither set,
`esim` run from the top folder takes `./core` as the engine when the catalog is there, so
the habit of running from the top folder still works. The server passes both. The ladder's
manifests are read relative to the engine's folder.

Checked: the build, the tests and the regression ladder (37 values, no drift) from
`core/`; Half Moon Bay rerun from the top folder to the same digest; the local server
restarted on the new binary and a run page looked at in a browser.

What a person has to change: commands that named `manifests/x` from the top folder now
say `core/manifests/x`, and the binary is `core/target/release/esim`. The README and
`docs/system/workspace.md` say so. Older devlog entries and the long list in `CLAUDE.md`
still name `crates/`, `data/` and `manifests/` without the prefix; they are history and
are left as written.

Left in the hub for now, to be decided at the cut: the engine's own documentation
(`docs/system`, the RFC, `protocol-v0.md`), which the hub's site serves and the command
line writes two pages of. The deploy script was edited for the new paths and has not been
run.

## Step 4: the viewer is a package beside the site

Twenty-one components and seven helpers that draw a run folder moved from `web/src/lib/`
to `viewer/src/`, with a `package.json` named `esimv1-viewer` and an index that exports
them. The split fell cleanly by file: none of the moved files imported the router, the
socket or the account code. What stayed in `web/` is the site: the pages (project list,
project, run, compare, docs, kinds, new project), the router, the socket, accounts and
the server.

Three things the drawing code took from the site without saying so are now stated in one
place, `viewer/src/config.js`: where project folders are served, and the two outline files
for maps and the globe. The defaults are the site's addresses, so the site is unchanged.

The site reads the package from source through a Vite alias while both are in one
repository. After the cut it becomes an ordinary dependency.

Checked: the site builds; a Palisades run page with the 3D layers view, the Earth 2100
project page and the front page were looked at in a browser on the rebuilt site, with no
page errors.

Limits of this cut are in `viewer/README.md`: it has only ever been used by this site, it
needs Tailwind 4 in the host page, and its components' props are not documented. The
Sausalito view does not use it yet, and the three small modules written there today
(tide, sky, water) would move here once it does.

## What is left before the cut (step 5)

- Records for the four scenarios move into their project folders (`data/events/<name>`
  inside the project), so each project folder is the whole of its scenario. The two
  Palisades projects share one record folder, which needs a rule.
- Decide where the engine's own documentation lives.
- The cut itself makes four or more new repositories and pushes them. That is outward
  and is Anselm's to say go on, with the names as he gave them.

## Portland's record moves into its project

`core/data/events/portland-cascadia` (17 MB: DOGAMI's ground layers, the tanks, bridges,
poles, routes, the loss tables) is now `projects/anselm/portland-cascadia/data/events/portland-cascadia`.
The project folder is the whole scenario: description, manifest and its source, page,
README, record. Nothing in the manifest changed; `folder = "portland-cascadia"` is found
in the project first.

Checked by running `wet-day` under a scratch name before and after the move: the same log
digest (`3cdfbe0d1c103a92`), an identical `describe.json`, and every series identical
except `agents.per_second`, which is a wall-clock speed and differs between any two runs.
The scratch runs were deleted; the standing runs were not touched, and `esim project
update` was not run, so the frozen manifest is as it was.

The source manifest is in two places for now: `core/manifests/q1-portland-cascadia.js` and
the project's `source/`. The copy in `core/` goes at the cut.

Not moved: the Palisades record, because two projects read it (`palisades-2025` and
`palisades-cells`) and one folder can only be inside one of them. Earth 2100 has no record
of its own; its tables (`tipping.toml`, the two future-event lists) are shared and stay
with the engine.

## Three questions for Anselm before the cut

1. **Palisades.** One repository, `esimv1-palisades`, holding both projects and their
   shared record? Then a scenario repository is a folder of projects with one
   `data/events` beside them, and the record is looked for beside the project as well as
   inside it. Recommended, because the Rothermel run is the baseline the page compares
   our engine against.
2. **How the hub shows a scenario kept elsewhere.** Recommended: the hub keeps a short
   list (repository, version, where it sits under `projects/`), and a script clones each
   at its named version before a build or a deploy. Addresses such as
   `/anselm/portland-cascadia` do not change. Git submodules would do the same with more
   to go wrong.
3. **The engine's own documentation** (`docs/system`, the RFC, the protocol). Recommended:
   it moves to `esimv1-core` with the code it describes, and the hub's site serves it
   from its checkout of the engine, at the same addresses.
