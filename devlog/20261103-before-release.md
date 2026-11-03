# 2026-11-03 — Before release: beauty, and which scenarios to exercise

Anselm, after ae61643 was pushed and deployed: "i will audit the 2100 scenario - which is
the one i am most interested in today. i am eager to make this public as well. I think a
few things have to happen tho - i don't know if we have a global todo.md ? i tend to use
one even though it breaks my 'everything goes in the devlog/' philosophy."

His two points, in his words:

1. "if we look at 'intelligent place' applications such as urbanly - we see that they are
   _very beautiful_ - they focus very much on a rich and vivid somewhat consumer facing
   experience. i do feel our service is deeply technical - it may take substantial work for
   us to find a way to make this pretty."
2. "it makes sense to exercise a few other scenarios as well as the 2100 scenario. there
   are two axes in my mind - local,bioregional,global - and secondly slow speed - fast
   speed - where if a model is fast enough it can actually be an operator in the loop (the
   difference between advising about likely fire paths versus real time advice on where to
   deploy firefighters). local scenarios are the actually the ones where people are the
   most picky, although the fewest people care about local stuff versus global stuff - but
   a local scenario may make sense - maybe even a forest fire. The boone valley scenario is
   mildly interesting - a bit boring - but of course we can compare it to historical data"

Agent: Claude (Fable 5.1), the earth-simulator-1 room on canada1, session
`0bad2b71-90e5-4e88-978e-bd10ad8801f7`. This entry records the thinking; nothing is built
from it yet, and the 2100 project is left untouched while he audits it.

## The todo

There is one: `todo.md` at the root, asked for on 2026-11-01, one line per item pointing
at the devlog entry that holds the thinking. A "Before release" section is added to it
today. The todo and the devlog do not conflict if the todo stays an index: lines, not
arguments.

## 1. Beauty

Where it can go without breaking the house rule (sober, sciency; nothing a reader could
mistake for a claim). The README page is the right place, because its figures are drawn
from the runs and a figure can be beautiful without being less true. The run pages stay
the laboratory. Concretely, in the order it would be seen:

- The front page. Today it is a list of projects as text. Each project should lead with
  one picture made from its own runs (the ladder for planet-2100; a map at a date for a
  basin) and one sentence, so the first screen is a gallery of what the model has done,
  not a directory.
- Maps that look like places. The people and heat maps are flat fields in a run-wide
  stretch; a coastline, a graticule and a quantile stretch (already on the todo) would do
  most of the work. A basemap under the Boone fields.
- Motion. The compare page's cursor can play; a README figure that plays from 1990 to
  2100 when it scrolls into view is the one consumer-facing thing that costs nothing in
  honesty.
- Type and space. The pages are dense by design; the README needs a larger measure and
  more air than the run pages, which `app.css` can give it alone.

Not seen: Urbanly itself. The points above are from his description and the house style,
not from looking at it.

## 2. Scenarios on two axes

Where the project stands on his grid:

|  | slow (advice) | fast (operator in the loop) |
|---|---|---|
| local | Boone (M1 to M11, hindcast, the regression anchor) | nothing |
| bioregional | nothing run; Four Corners and the African Humid Period are on the todo | nothing |
| global | planet-2100 | not the point |

A forest fire is the right next scenario because it measures both axes at once. Local
readers are the picky ones, and a fire with a known ending (a perimeter record day by
day) scores the spread model the way the Boone year scores the basin. And a fire day is
the natural speed test: how long does the kernel take to simulate one day at the
resolution a fire needs? If that is seconds, the operator-in-the-loop product is a surface
question (live inputs, a clock that advances with wall time, decisions injected and
replayed), not a model question. If it is hours, it is a model question first.

What exists: `hazard.fire` is a cellular spread on the Boone grid (M7, cropland fire,
forced ignition, a `spread_base` tuned up for a what-if since the Iowa default does not
spread). It is not Rothermel; the brief names Rothermel as the published model to write
against. A forest fire scenario would need slope and fuel (the DEM is there; a fuel map
is a new asset) and wind at the hour (ERA5 is there).

Data doors, all to be checked before any is relied on (recalled, not fetched today): NASA
FIRMS active-fire detections (free key), the NIFC/WFIGS fire perimeter service, MTBS burn
severity for the final footprint. A fire chosen for a hindcast should have a perimeter
sequence, not just a final footprint, or the daily score is impossible.

Boone: agree it is boring as a story and keep it anyway. It is the regression ladder's
anchor and the only place the model is scored against a gauge every day. It is the lab,
not the showcase.

## Before release, as a list

In the todo under "Before release". The order I would take: the front page as a gallery
of run-made pictures; the fire hindcast as the second scenario, with the speed measured;
then accounts and the crumpled namespace (already on the todo); the held-low variant's
place on the public compare page is still his call.
