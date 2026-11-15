# No place in the reader

14 November 2026. Claude (Fable 5.2), esimv1 room, session 1cc66e9f, with Anselm.

## The ask

Anselm, reading the Portland page, asked how the ground "knows that it is the area bounded by
the place". The answer was that the manifest reader copied every key of a `place` table into
every module. His replies, in order:

> why does the reader understand that place is a special concept to begin with? ... shouldn't
> the javascript - which is the high level scripting glue for us - be responsible for this -
> rather than the reader implicitly understanding anything? on a principle that the reader
> should be tiny and generic - and not be subject matter aware.

> i think we should fix everything everywhere as best we can, all older manifests, and so on.
> there should really be nothing special about place and there shouldn't need to be special
> documentation about it. as far as the engine is concerned each module gets some props...
> from the manifest... if we want, for simplicity, it is nice to be able to pass the entire
> place structure as a single property.

## What changed

- **The reader.** `Manifest` has no `place`; `fold_place` is gone; the TOML include merge, the
  entity form and the JavaScript short form no longer name it. A module gets the parameters the
  manifest gives it. This is the kernel getting smaller.
- **The module library.** A module that works on a window takes `place` as one parameter, a
  table. `with_place` in `crates/modules/src/lib.rs` reads its keys as the module's own before
  `build` and `describe`; a key the module sets itself wins. This is a convention of these
  modules, which are the subject-aware level, and the reader does not know it.
- **The manifests.** The six scenario manifests with a `[place]` table (Half Moon Bay, both
  Palisades, Fort McMurray, both Nicasio) were rewritten as JavaScript by a line-based converter
  that kept their comments, and the TOML files deleted. Portland's script now hands `place` to
  its modules. Each new manifest was checked against the old binary's resolution: same
  entities in the same order, and each module's parameters equal once `place` is spread.
- **The runs.** A manifest's id is the hash of its resolved form, which changed (a nested
  `place` in place of five copied keys, and no place entity), so the run ids of seven projects
  changed. Every variant that had a run was rerun offline and its log digest compared with
  the old run's before the old folder was removed. The result is in the commit message.
- **The RFC and schema.** `place` is no longer a named component; the open question about it
  became a question about how engines agree that modules share a grid.

## What is left

- The earthquake family refuses to start without a window. About thirty older modules (the
  Boone basin, the fires, the coast, the staked places) still fall back to the coordinates of
  the first place they were built for. Requiring the window there means stating it in about
  twenty-five Boone manifests first.
- An old document that still carries a `place` table or entity is read without complaint and
  its modules get no window from it. The check lists the unknown component; nothing refuses.
- Thirty-one manifests (the Boone ladder, the planet) are still TOML. None of them had a
  `[place]` table.
- Kinds' describes list the window as five flat parameters, not as `place`.

## The reruns, and run names

Forty-nine runs across seven projects were redone offline against the new manifests. Twenty-nine came
back with the same log digest. The other twenty, all of Fort McMurray and all but one of
Palisades cells, had been made on 4 November; each differs from its rerun in one log line,
`ground.loaded`, which gained `wind_at` and `wind_start` on 8 November. Every other event is
identical, so the results on those pages stand. The old binary on the old manifest gives the
new digest too, which is how the cause was separated from this change.

The run ids changed because an id carried the manifest's hash. Anselm, on seeing that:

> oh thats why we have the unique urls... can we stabilize the urls without having them be a
> manifest hash? ... we don't need backwards compatability - i think each run should just have
> a unique name

So a project's run is now `runs/<name>`: the variant's name, `base`, or `--name`. A rerun
replaces it. All eighty project run folders were renamed. Loose runs keep the hashed form.
