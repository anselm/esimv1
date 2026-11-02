<!-- order: 15 -->
# Adding a module

The recipe, in the order that keeps the rules intact.

1. **Find the record first.** What observation or base layer will this module read, and
   what scoreboard will it be judged against? If there is none, the module is a formula
   check and is labelled as one. Add the source to `data/catalog.toml` with its URL,
   format, units, licence, the date you verified it and from where, and its quirks. Write
   a reader in the registry crate; it fetches through the fetcher so the bytes are hashed
   and cached.
2. **Name what it owns and reads.** Which fields will it write (and with what unit), which
   will it read and from which kinds, which topics will it publish and subscribe, which
   ledger stocks will it hold. Use the names in [Vocabulary](?doc=system/vocabulary); add
   a new name only when none fits, and in its form.
3. **Write the kind** in `crates/modules/src/<layer or place>.rs`: a struct with its
   parameters and state, `new(name, params)` that reads parameters with defaults and
   fetches its data, the `Module` trait (`init`, `handle`, `snapshot`, `restore`), and
   `describe(name, params)` beside it. Every parameter in `new` appears in the describe
   with its default, unit and a provenance note: the paper, the devlog entry, or the word
   "guess".
4. **Account for what it moves.** Open or join a ledger in `init`, declare the stocks it
   holds, transfer every movement in the precision it publishes, and redeclare stocks
   after each change. A module that publishes an f32 to another module accounts the f32.
5. **Choose a priority** so that at an instant it runs after what it reads and before what
   reads it, and state the priority in the describe.
6. **Register** the kind in the `KINDS` table in `crates/modules/src/lib.rs`.
7. **Write a manifest** that uses it on a real stage, run `esim check`, read the
   information lines, run it, look at the ledgers and the maps, score it.
8. **Test.** Add the manifest to the describe test's list if it exercises new surface; the
   test compares the run's log against the declarations and fails on drift. Add a test
   for the module's own claim where it has one (a fire that spreads; a forecast that
   matches the record's mean).
9. **Write the devlog entry**: what it is, what the record says, what it gets wrong, what
   was left out. Lead with what is wrong.
10. **Update the describe when the module changes.** It is the module's word to the
    kernel, to the facade and to people; a stale describe is a lie the test will catch.

## What makes a good module

One idea with its own record. Reads declared, outputs owned, numbers accounted,
parameters sourced, omissions stated in the summary. Small enough that someone with a
different opinion about that one idea can write a competing kind in an afternoon and run
both against the same scoreboard.

## Later

A kind in another language runs over the wire binding in `docs/protocol-v0.md`: JSON
lines over stdio, the same lifecycle, the same fifteen requests. Nothing above changes
except where the code lives.
