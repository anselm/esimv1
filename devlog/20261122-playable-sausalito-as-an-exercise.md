# Playable Sausalito, as an exercise for the engine

22 November 2026. Claude (Fable 5.2) in the esimv1 room on canada1, session
`session_018TGCFiQHbUYn12HkNti2xk`. A point-in-time note. No engine code was changed today.

## The ask

Anselm wants to build a playable model of Sausalito, California, and to use it to exercise
how esimv1 is organised. His words:

- "my main goal is to build a playable sausalito. it can be a separate but public repo. a
  separate repo begins to help us exercise esimv1 as a simulation core, package or service -
  someday used by thousands of developers - similar to say threejs. this also implies that we
  will move the other projects out of esimv1 at some point soon also."
- "i realize that esimv1 is a research project and this use case is helping us exercise our
  thinking"
- On play: "the user will be at a certain point in time, set conditions, hit play, that can run
  by default for some maximum period and then stop, unless stopped by the player ... do we show
  users a day by day evaluation or do we just leap to the end of the run? this user experience
  is not defined yet."
- On the view: "more of a solid style - perhaps with sun position controls. showing buildings
  is important."
- "today we are just sketching however."

It follows the cloud session's note of the same day
(`20261122-storage-compute-and-repos-from-a-cloud-session.md`), which recommended a separate
repository for scenario content that needs no Rust.

## What was done

- A new public repository, `anselm/playable-sausalito`, with its folder at
  `~/projects/2027/playable-sausalito`. It holds scenario content only: a first essay on how a
  town approaches a playable model of itself, a chore list, and the ask. It has no manifest and
  no simulation. Nothing was pushed.
- The first question the model answers is not chosen. I started toward tides and sea level
  without being asked and was stopped; nothing came of it.

## What the exercise asks of esimv1

Each item is something the engine cannot do today or does only from inside its own tree. None
is built.

1. **A project outside the tree.** `esim project` commands resolve `<area>/<name>` under
   `ESIM_ROOT/projects`, and the viewer server reads `../projects`. A scenario in another
   repository needs a path form, or a list of project roots.
2. **A scenario's own record folder.** Modules read record tables from `data/events/<event>/`
   under `ESIM_ROOT` (about 49 places in the crates name that folder or the cache). A scenario
   elsewhere has to bring its tables with it, so the lookup needs a second root that the
   project supplies, while the shared cache stays where it is.
3. **A version to name.** An outside repository has to say which engine it runs on: a tag, a
   manifest schema version (RFC 0001), a list of the kinds it needs. Today a kind can change
   under a project without notice; inside one repository the regress ladder catches it, and
   outside nothing does.
4. **New kinds without a fork.** The decision today is that new Rust modules stay in esimv1.
   That holds while one team writes both. For the three.js comparison to become true, a
   scenario must be able to add a kind without changing the engine's repository:
   `docs/protocol-v0.md`'s wire binding and module host are the designed route, still unbuilt.
5. **Play.** Start at a chosen moment, set conditions, run to a limit, stop early, change a
   condition and continue. The kernel has snapshots and takes births mid-run from the bus, so
   the parts exist; no command holds a run open or resumes one with a changed parameter. Runs
   of the other scenarios take 2 to 10 seconds, so one answer to his question is to compute to
   the limit and play the result back day by day, with a stop that pauses the playback and a
   change that reruns from that moment. Untested, and it stops working when a run is slow.
6. **The prepared bundle** from the cloud session's note: one file per scenario that a run
   loads without fetching.
7. **Finer ground.** Copernicus at 30 m is too coarse for a town of about two square miles
   where a foot of water matters. A lidar-derived elevation reader would be new registry work.
8. **The viewer as a part.** A solid, sunlit view with buildings is closer to `Scene3D` (boxes
   lit by the sun at the cursor) than to `Layers3D`. A scenario repository needs to use the
   viewer's components without copying them, which the viewer is not packaged for.

## Moving the other projects out

He expects the existing projects to leave this repository too. Before that: items 1 to 3
above, and a decision on where project runs are published from, since the deploy now rsyncs
them from `projects/`. The regress ladder and the describe test read real project runs and
would need their own fixtures.

## Open

- The first question for Sausalito, which decides which of the items above is needed first.
- Whether play is computed ahead and played back, or stepped live.
- Whether the scenario repository vendors esimv1 (a submodule or a pinned checkout) or expects
  it installed.

## Later the same day: modularity as a whole

Anselm, after the first view tests: "should [the other demos] also move to separate packages?
How modular is visualization? ... are those libraries deeply interwoven with the core simulation
engine? ... the main esimv1 website is effectively a multi user space - and also a simulator -
and also a bunch of other stuff ... Almost like the simulator core should be the piece that moves
to a new repo?"

What is in this repository, counted that day:

| Part | Size | Depends on |
|---|---|---|
| Kernel (clock, bus, ledgers, manifest readers) | 2,300 lines of Rust | nothing of ours |
| Registry (fetching and reading data) | 3,100 | kernel |
| Modules (the kinds) | 12,100 | kernel, registry |
| Command line and auditor | 1,600 | all three |
| Viewer components and pages | about 5,700 lines of Svelte and JavaScript | run folders, as files |
| Server (projects, accounts, drafting, runs) | 485 lines | the command line, by spawning it |
| Content | 14 projects, 41 manifests, 5 record folders, docs, devlog | the kinds by name |

The viewer shares no code with the engine. It reads what a run wrote (`run.json`,
`describe.json`, `log.jsonl`, the field frames), and the server starts the command line as a
process. So the seam between simulation and picture is a file format, and it already exists.
What is not separate is the viewer from the site: `Scene3D`, `Layers3D`, `FieldMap`, `Chart`
and `Timeline` live inside the app with its router and project pages, and some know topics of
particular scenarios (fire ignitions, lifeline marks). I did not audit each component for this.

Recommended shape, four kinds of repository:

1. **The core**: the five crates, the protocol, the manifest RFC, the regress ladder, the guide.
2. **The viewer**, a package of components that draw a run folder, with no site around them.
3. **The site**: the multi-user workspace (server, accounts, project pages, front page), which
   uses the other two.
4. **Scenarios**, one repository each, content only.

Which piece keeps the name is his call. My recommendation is that `esimv1` stays the core,
because the core is what other developers will name as a dependency, and the site and the
scenarios move out. Moving the core out instead leaves `esimv1` meaning the website.

Order: Sausalito first, since it proves a project can run from outside. Then one existing
scenario with a record folder (the Palisades), which tests items 1 and 2 of the list above.
Then the rest. The viewer becomes a package when a second site needs it, which Sausalito's
solid view may be. Scenario-specific kinds (`portland.lifelines`) cannot leave with their
scenario until a kind can be added from outside the core.

Not decided. Nothing was moved.
