# The Portland page rewritten under the consolidated writing guidance

2026-11-12, evening. Claude (Claude Code session 018TGCFhEpYs9hm68xDLd5Xy, room earth-simulator-1 on canada1).

Anselm, after the jubilee on writing guidance: "Ok, commit this, switch to it, and I want to try
rewrite the portland cascadia fault scenario readme again."

## What was done

`projects/anselm/portland-cascadia/README.md` was rewritten from nothing under the harness
writing-style skill as it stood that evening, about 5,000 words. The old text is in git before this
commit. Order: what the page is, why this scenario, what the state expects, the fuel terminals and
the council's vote, what the simulation does, what the runs show (the day after, three days after,
a year after), what cannot be trusted yet, the wiring figure, the reading list.

Every number in the prose was read from the runs of 12 November 2026 with a small script that
prints a series at a time for every variant. Every mechanism described was read in the code
(`quake.rs`, `components.rs`, `lifelines.rs`, `aftermath.rs`, `agents.rs`) and in
`data/events/portland-cascadia/fragility.toml`. The actors figure was cut to nine actors and
fifteen arrows because the first version, with twelve and twenty, drew its labels on top of each
other. Checked by screenshot, light and dark.

Newly read for the page: the council's document page for Ordinance 2026-312 (passed as amended
12 November 2026, 10 to 0, two absent), the USGS pages for Professional Paper 1707 and for
Goldfinger and others 2012, and the city's page on unreinforced masonry buildings.

## Faults found in the model while checking the text

None is fixed. The page states each one that affects a number it shows.

1. The two death counts disagree by day. The cell count gives 1,000, 564 and 602 for seeds 1 to 3
   and the agents 670, 460 and 550. At night they agree (631 and 630). Lead: the agents put 6,020
   people in collapsed buildings and 670 die, a rate of 0.111. The cell count's 974 deaths in
   collapses imply about 8,770 people in collapsed buildings at that rate. So the difference is in
   how many people are inside collapsed buildings, not in the rate. The agents total 935,200 people
   against 1,013,450 daytime occupants, which explains a part. The agents also record no deaths
   outside collapses, where the cells have 26. The agents' view text still says the two agree in
   expectation.
2. `burnside-ready` does nothing in computed mode: `bridge_ready` is applied only in the ranges draw.
3. The response's fuel use starts only when a hospital is reachable from the hub, day 111 in the
   base run.
4. Electricity returns in 22 days against the Resilience Plan's one to three months, because
   transmission structures are the whole grid.
5. The base spill, 60.6 million gallons, is two thirds of the 2022 study's minimum. The per-tank
   release follows the study, so the shortfall is in how many tanks fail (233 of 587).
6. Night and dry soil: only shaking collapses a building, so the dry-soil toll barely differs and
   the night toll is too high against DOGAMI's ratios.
7. The seven older bridges at complete damage reopen on the run's last day (365 days, a judgement).
8. Story sentences that contradict the computed run: "The grid is down everywhere" while 90 percent
   keep power; "Day 1: power is back for half the people"; Broadway on day 20 for a 21-day repair.
   `service.routes_open` reads 1.000 in computed mode. `tanks.release_gal` in the ranges run shows
   the components' figure.
9. Stale words elsewhere: the `lifelines.recovery` summary describes the ranges mode; the
   `quake.ground` summary says "the best anyone has"; variant titles, `project.toml`'s description
   and the manifest notes; the ordinance rows in `hub.toml` and `record.toml` still say the vote was
   not recorded; the root README's Portland bullet and its "Where things stand" paragraph; the
   Portland entry in `CLAUDE.md` (765 deaths, eleven of twelve bridges).
10. The run reports 297,803 buildings and the data folder's README 297,120 outlines. Not reconciled.

## Next

Anselm brought a second revision of the writing guidance from ChatGPT the same evening. It loosens
the one-job-per-sentence rule and adds a section on holding the reader's attention. The results
section of this page is the part it would change most: it goes topic by topic, and the chain from
bridges to roads to fuel trucks to generators to hospitals is split across five paragraphs.
