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
