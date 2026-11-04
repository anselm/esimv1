# 2026-11-04 — The vector form: the first day right, the second day everywhere

Anselm, first thing: "lets rev the custom engine again for accuracy." Last night's entry (*The slope*)
had named the next form: drop the e-folding length as the carrier of direction and write the heat a
burning cell delivers as I·f(d)·(1 + φ), with wind and slope added as vectors the way Rothermel adds
them, and the ellipse's backing rate in place of a downhill division. This is that, and what it found.

## What was built

`fire.cells` has a declared `form`. `reach`, the default, is the engine as it was, every earlier run
unchanged. `vector` writes the heat delivered from a burning cell to a neighbour at bearing θ as

    q = I · exp(−d/λ0) · (1 + |φ|) · (1 − e) / (1 − e cos(θ − θ_head))

where φ is the vector sum of a wind term along the wind (φ_w = `wind_phi` · U^`wind_phi_exp` on the
10 m HRRR wind; 1.9 and 0.9, the shape Rothermel's C U^B gives a shrub bed under a 0.4 wind
adjustment, recalled and rounded, so about 15 at 10 m/s) and a slope term up the receiving cell's
aspect (`slope_rate_gain` · tan²φ, 20 as Rothermel's 5.275 β^-0.3 for a shrub bed, recalled). θ_head
is the sum's bearing, and e is Anderson's ellipse eccentricity from the wind that would give |φ| on
its own, so the flanks and the back follow the ellipse from the head: at 10 m/s the flank runs at
about 1.4 times the flat rate and the back at 0.9, the head at 16. `wind_gain` and `slope_gain`, the
old stretches of the reach, are not used in this form. `phi_cap` (30, a guess) stands in for
Rothermel's effective-wind limit.

The first run of it stalled at 900 acres, and the reason is worth keeping. The heat a cell holds
decays between doses with `cool_min`, so under steady heating it settles at dose times cool_min and
a direction whose dose is below the threshold never lights, however long it is heated. In the reach
form that was the cliff of yesterday morning; in the vector form it starves the flanks, since the
ellipse gives them a tenth of the head. A spread rate is not like that: time to ignite is threshold
over rate, and a slow direction is slow, not closed. So `cooling = "idle"` (declared; `always` is the
old behaviour) decays the heat only over the time no dose arrived. With that the fire moves.

Two more things were built to test the second day, below: `lines = "hold"` in the engine makes every
completed line in the record (`obs.record`'s events of kind line, category Completed …, status not
Archive) hold from its time on, the cells under it to `line_width_m` taking no heat and no ember; and
`obs.record` takes a `time_shift_h`, hours taken off every feature's time for a field known to lag
the work. Both are declared and both are guesses where they are guesses.

## What the record says

Acres and Sørensen overlap at four of the mapped perimeters (first infrared map at 27 hours, then
33, 49 and the last at 101 hours; mapped 15,824, 15,832, 19,629, 23,343):

| variant | 27 h | 33 h | 49 h | end |
|---|---|---|---|---|
| near-4-slope-6 (last night's best) | 7,015 · 0.60 | 11,364 · 0.75 | 18,885 · 0.79 | 31,062 · 0.70 |
| vector-4 | 23,589 · 0.74 | 39,432 · 0.54 | 63,741 · 0.45 | 68,453 · 0.49 |
| **vector-5** | **16,525 · 0.79** | 28,890 · 0.67 | 53,907 · 0.51 | 64,618 · 0.51 |
| vector-6 | 12,370 · 0.78 | 21,545 · 0.77 | 41,498 · 0.61 | 65,011 · 0.51 |
| vector-8 | 6,839 · 0.58 | 10,964 · 0.76 | 22,699 · 0.82 | 58,293 · 0.54 |
| vector-12 | 3,026 · 0.31 | 4,418 · 0.42 | 6,538 · 0.49 | 26,304 · 0.66 |
| vector-5-held (lines hold) | 16,525 · 0.79 | 28,890 · 0.67 | 53,907 · 0.51 | 64,618 · 0.51 |
| vector-5, lines 24 h early | 16,525 · 0.79 | 28,524 · 0.68 | 53,296 · 0.52 | 64,612 · 0.51 |
| vector-5, slope gain 6 | 14,340 · 0.78 | 26,374 · 0.69 | 52,770 · 0.52 | 66,595 · 0.50 |
| vector-5, φ capped at 12 | 27,432 · 0.69 | 42,259 · 0.52 | 62,971 · 0.46 | 66,379 · 0.50 |
| vector-5, moisture gain ×3 | 7,230 · 0.60 | 12,353 · 0.80 | 25,249 · 0.80 | 58,353 · 0.54 |
| ×10 thresholds, ×2 wind factor | 14,142 · 0.79 | 25,437 · 0.71 | 49,816 · 0.54 | 67,310 · 0.49 |
| ×40 thresholds, ×8 wind factor | 12,627 · 0.76 | 22,839 · 0.72 | 46,671 · 0.56 | 64,622 · 0.51 |

Three of these stay in the project (vector-5, vector-8, vector-5-held); the rest were run this
morning and removed, their numbers kept here. Runs are deterministic.

The first day is right. vector-5 has 16,525 acres at the first infrared perimeter against 15,824
mapped, with an overlap of 0.79, the first run on either engine to have both the pace and the shape
of the first day; last night's best had the shape at half the area, and the morning's near-4 the
area at 0.59. The arrival map is a lobe running west along the coast to Malibu and north to Topanga,
with the eastern edge at Pacific Palisades and Brentwood, which is where the fire was.

The second day is wrong, and in a way that no knob reaches. Between the 27-hour and the 49-hour
perimeters the record grew by 3,800 acres and vector-5 by 37,000: by 49 hours the model has filled
the whole mountain block north to Calabasas. The table is the list of what was tried against it.
Raising the thresholds slows both days together (vector-8 has the second day and loses the first;
there is no scale with both). The slope gain cut to 6 does nothing to it. Capping the rate factor
makes it worse, since a lower cap lowers the ellipse's eccentricity and widens the flanks. Tripling
the moisture gain slows both days alike. The record's completed lines, held from their recorded
time, change nothing, because the first completed line in the record is at 19:40Z on the 9th, after
the overrun; held a day early, with 203 lines and 3,905 cells, they take a few hundred acres off it,
because the fire has already passed where most of them are. And the test that settles what it is:
the thresholds times eight under a wind factor times eight, the same head rate and an eighth of the
flat rate, leaves the second day at 46,671 against 53,907. The flanks and the back are not the
overrun. The overrun is heads: on the second day, under a wind reading 8 m/s falling to 1 at the
ignition point and the slope term at its full strength wherever the ground rises, the fire has a
head in every direction the hour's wind field and the next ridge give it, and runs them all at once.

## What it means

The vector form is kept, with idle cooling, as the engine's form from here, and yesterday's rings
and fingers are the reach form's. The second day is the next question and it is not a constant.
Two things the model does not have are the candidates. The real fire's growth on the 8th and 9th
was into the mountains to the north and east, slowly, and toward Encino on the 10th and 11th; the
model's is the same places two days early and all at once. Either the HRRR wind over the mountains
on the 8th and 9th is not what blew (3 km, a Santa Ana, no terrain in the engine's reading of it;
the terrain-following wind of item 1 would change exactly the second day's heads), or the fire was
held there by what the record does not place: aircraft on the 8th and 9th once the wind eased, and
crews whose lines the GIS unit drew a day later. The record's resource counts by day (CAL FIRE's
reports) are the second candidate's measure; the wind is the first's. Both are in the todo under
items 1 and 4.

One lesson about the budget form itself: a heat budget with decay is a rate model with a floor, and
the floor is a cliff. Every "cliff" this project has met (reach 3 cells yesterday morning, the ridge
tops last night, the flanks this morning) was the same thing, the steady state of dose against decay
falling under the threshold. Idle cooling removes it, and with it removed the engine's rate is
threshold over dose and the ellipse does what Anderson's does. What is left of the budget is the
threshold (fuel and moisture), the burn time (how long a cell is a source) and the reach of the
heat (f(d)), which is what it should have been.

## What Anselm's sources say about the interventions (added the same morning)

While this was being written Anselm sent two summaries (machine-made, with their sources; the
sources are his to have checked and are listed as he sent them, not fetched here): the Los Angeles
Times of 2025-01-13 on whether brush clearance could have slowed the fire, LAist and Fire Engineering
on the state's analysis of the 2025 fires, the LAFD after-action review of 2025-10-08
(`lafd.org/sites/default/files/pdf_files/Palisades_AARR_10_08_2025_final.pdf`), the UC San Diego
WIFIRE programme's account of its real-time maps, OroraTech's visualisation, a Hoover piece, and the
Times of 2025-10-30 on the crews ordered off the smouldering Lachman burn on 2 January. Their burden:
on the first day nothing worked. Fuel breaks were overrun ("a 10-lane freeway in front of that fire"
would not have slowed it, in the Orange County chief's words as the Times quotes him), aircraft were
grounded by the wind, the fire went house to house, and the crews turned from containment to
evacuation; 40 engines sat in repair yards, mutual aid was stretched across fourteen fires, and the
fire had rekindled from the Lachman burn that crews left on 2 January.

That is the model's first day: nothing stopped it and the vector form, with nothing stopping it, has
the day. It also says the lines in the record were never the second day's answer, which is what the
held runs found from the other side. What the summaries do not settle is the second day. The record
grew by 3,800 acres between the 27-hour and 49-hour perimeters while the model grew by 37,000, and
whether that is the wind easing over the mountains or the air attack once aircraft could fly (recalled:
aircraft returned on the 8th as the wind dropped; not in these summaries, to be checked in the
after-action review) is the open question. The after-action review is the document to copy into
`data/events/palisades-2025/` next, since it is the incident's own account of when resources were
where; CAL FIRE's reports by time, already there, carry the counts.

Session: Claude Fable 5.1, the esimv1 room on canada1, 2026-11-04 morning. Also this morning: a
`prune --keep 0` with a flag I guessed wiped the project's fourteen runs; they were rerun (deterministic,
same numbers) before anything else.
