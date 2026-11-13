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

## Afterwards: he meant the project page

Anselm: "I'm sorry I mostly actually meant https://esimv1.exe.xyz/anselm/portland-cascadia. It is
ok to adjust the detailed prose." So the project README was rewritten as well, under the same
revision of the guidance. The run-page work above stands.

What changed in `projects/anselm/portland-cascadia/README.md`:

- The opening starts from the council's vote and the trade-off it poses (the tanks are the spill
  and the reserve), then says what the project is, then gives the headline numbers and the three
  main reasons to doubt them. The old opening described the model's inputs.
- "How the simulation works" is now a short account that reads straight through and is built
  around one dependency chain (a hospital, its generator, a fuel truck, a bridge, the hub). The
  rule and source for each part moved to a supporting section after the limits, "How each part is
  modelled".
- In the results, each finding now carries its reason or its consequence where the old text
  listed figures: why the two death counts should agree, what the seeds show, why dry soil barely
  changes the toll, what the `ranges` variant says about the hospitals, what the model cannot yet
  say about the reserve.
- The authorship note moved to the end.

Every number is the old page's, which was read from the runs of 12 November; the reruns above are
numerically identical. The reading list is unchanged. Not pushed, not deployed.

## The dates moved again

Anselm: "Make the sim start Jan 1. Move fault to Feb 1." The event is Monday 1 February 2027 at
2 PM PST (`at = 2027-02-01T22:00:00Z`, `tz_offset_h = -8`, `tz_name = "PST"`); the run is
2027-01-01T22:00Z to 2028-02-01T22:00Z; the wet-night variant is at `2027-02-01T10:00:00Z`.
Changed in the manifest, `project.toml`, the project README (its opening, the 327 years, the
three table dates), the root README, the Puerto Rico claim's note and CLAUDE.md. `esim project
update` was run, the runs pruned and all ten variants rerun.

Against the March runs every series ends on the same value except `fuel.consumed_gal` and
`fuel.delivered_gal`, which are one day's worth lower (300,000 gallons each in the base run).
The March year contained 29 February 2028 and ran 366 days after the event; this one runs 365.
No figure in the README's prose depends on either.

Seen while checking and not fixed: on a computed-mode run page the map's key counts the ten
closed bridges as open and draws them as open marks; only the ranges mode marks a closed bridge.
The counter beside the map is right (2 open).
