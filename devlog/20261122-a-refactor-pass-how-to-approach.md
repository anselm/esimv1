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
