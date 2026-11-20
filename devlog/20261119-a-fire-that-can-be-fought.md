# A fire that can be fought

19 November 2026, afternoon. Claude (Fable 5.2) in the esimv1 room on canada1, session
`1cc66e9f-2882-4c70-b8d3-6d9d3cf1beb1`. Follows `20261119-the-record-and-a-fit.md`, which ended on
"the engine needs a way for a fire to stop". Anselm: "yes we definitely should do stopping!"

## What was built

`fire.suppression` has a third mode, `resources` (`crates/modules/src/suppression.rs`). It reads a
table of status reports (`reports`, here `palisades-2025/calfire/updates.csv`, local time, so
`reports_tz_offset_h = -8`) and each hour:

1. reads `fire.burned_frac` and finds the edge: unburned land cells beside burned ones (sea is
   elevation at or below zero);
2. measures how far the fire came to reach each edge cell in the hour, as the distance to the
   ground burned an hour before (a two-pass chamfer transform), less the one cell an unmoved edge
   already stands off;
3. takes the hour's capacity in metres of edge, `share` times the sum of count times rate over
   engines, dozers, hand crews and helicopters in the latest report;
4. spends it on edge cells that moved no faster than `hold_rate_m_h`, most active first, and holds
   the unburned ground within `depth_m` of each;
5. publishes the held cells as one `ground.break`, which both fire engines already obey.

It writes the frame `edge_held`, reports `suppression.containment_pct` (held edge over all edge)
beside the incident's reported figure, and sits in the People layer. Its snapshot now carries the
held cells, which the older `hold` mode lost. The kernel was not touched.

Closing a perimeter at a production rate is Fried and Fried's idea (Forest Science 1996, recalled,
not reread). Working the most active holdable edge first is ours. The rates (engine 240 m/h, dozer
600, hand crew 100) are recalled from the NWCG Fireline Handbook's Appendix A and unchecked; the
helicopter's 200 is a guess.

## What the fit says

Two constants fitted, `share` and `hold_rate_m_h`, on `vector-5`, trained on the five mappings up
to 2025-01-10T04:00Z and tested on the six after (`runs/f3-palisades-cells-fit-7973bc87`, 48 runs
in 105 s; a second search in the low hold-rate region, `fit-48712f9a`).

| | train | test | acres at the end |
|---|---|---|---|
| nobody fights it | 0.545 | 0.818 | 64,600 |
| `vector-5-crews` (share 0.2463, hold 997 m/h) | 0.215 | 0.173 | 19,000 |

Mapped at the end of the window: 23,343.

The loss is a cliff in both constants. Share under about 0.2, or hold rate under about 150 m/h:
the fire escapes. Above: caught within hours, and the loss is flat (0.215 to 0.225). The fitted
run holds its whole edge by 45 hours; the incident reported 0 percent until 18:45 on 9 January
local and 11 percent at the window's end, and the real fire grew 7,500 acres in those days. So
the size is right and the course is wrong, and the containment series, which the fit never saw,
is what shows it.

## Limits and what is next

- Held ground never fails. A line that can be lost (by wind, by an ember count, by the fire's
  rate beside it) is the next thing, and is what would let a fought fire creep.
- The engine's unfought second day is vigorous everywhere (64,600 acres). The crews hide that
  fault; they do not repair it. A marginal-burning rule (a fire that goes out when its heat
  falls under a floor in damp air and light wind) is the engine-side half of stopping.
- Nothing is counted before the first report that lists resources (13:20 local on 8 January),
  so the first 27 hours are unfought. In fact engines were there and defending houses.
- `fire.rothermel` does not publish `fire.burned_frac`, so its runs cannot be fought this way.
- One fire. The share absorbs the cell-following edge's length, so it will not carry to another
  resolution unchanged.
- All 20 variants of `anselm/palisades-cells` were rerun on the changed manifest (the module
  gained two parameters, so the manifest's id changed); the unfought runs' numbers did not move.
