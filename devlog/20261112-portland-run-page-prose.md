# Portland: the run page's prose rewritten

12 November 2026, late. Claude Code session in the esimv1 room on canada1
(session_018TGCFiQHbUYn12HkNti2xk; begun on Opus 5.5, finished on Fable 5.2).

Anselm: "The prose here needs to be rewritten in light of new prose guidance", with the link
to the `ranges` run page on esimv1.exe.xyz. The guidance is the harness writing-style skill as
revised that evening.

## What a run page's prose is

None of it is in a document. The page draws each module's summary, captions, notes and
"leaves out" text from the module's `describe` in Rust, the story sentences and their sources
from the events the modules publish during the run, and the claims table from
`data/events/portland-cascadia/aftermath.toml`. The map key is in `ModuleViews.svelte`.

## What changed

- `lifelines.rs`, `quake.rs`, `components.rs`, `agents.rs`, `aftermath.rs`: every summary,
  parameter note, view title and caption, note, story sentence, source and confidence string
  rewritten for a reader who has not seen the code. The coined words are gone from the page:
  "the window" is the modelled area, "crossings" are bridges, "the hub" is introduced as the
  Critical Energy Infrastructure Hub, "by the seed" is a random draw, "ramp" is a straight line
  to a drawn day, DOGAMI and Hazus are said in full where a module first uses them. Fragment
  chains joined by semicolons are sentences. Numbers in sentences carry thousands separators.
- The lifelines summary says which mode the run uses, and the electricity chart's caption
  follows the mode. Before, the caption described the ranges mode in every run.
- The toll sentence now says beside its figure that it is the count by map cell and that the
  page's total uses the agents' count, which is lower by day for a reason not yet found. On the
  page the sentence said 1,000 dead beside a counter reading 711 with nothing to connect them.
- The first deaths-after sentence named only interrupted care; it now names both rules that
  can add deaths.
- `aftermath.toml`: the claims' titles, sources and confidence notes. Two were stale: the cold
  claim said "late January" and the Puerto Rico analogy said "in winter"; the event is in March.
- The map key: "bridge closed", "hospital without power", "fuel hub, tanks failed".
- "what this does not measure" and "what this does not compute" are both "what this module
  leaves out".

Report names, topics, fields and parameters are unchanged; `crates/modules/tests/describe.rs`
passes. The project's frozen manifest was not updated, so its notes are as they were.

## The runs

The sentences are events, so the log digest and with it each run id changed. All ten variants
were rerun and the old run folders pruned. Every series and ledger is identical to the runs
of earlier that day except `agents.per_second`, which is a measured speed. The page Anselm
linked is now `/anselm/portland-cascadia/q1-portland-cascadia-bf1b2fcb-c4a4d1af` locally; the old
address will stop resolving on the live copy at the next deploy.

Anselm had said earlier that night not to run the simulator on canada1. The run page cannot
change without a run, so the ten variants were run (about three seconds each).

## Not done

- Not pushed and not deployed. The live copy shows the old page until `deploy/deploy.sh esimv1`.
- `cover.png` and `card.png` are still stills of the old page.
- The project README was written under the guidance before this revision and was not touched.
- The other projects' modules (fire, planet, coast) have the same compressed style in their
  describes and were not touched.
- `retrofit` means "built after 2004" in `lifelines.recovery` and "built after 1993" in
  `hazard.components`. The notes now say what each module does; the disagreement stands.
