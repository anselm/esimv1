# Why the 2100 page reads mild, and what a wiring graph is for

2026-11-02. Anselm sent four posts from X in the small hours: Battambang rice farmers
going from drought to flood with 173,000 acres past harvesting per the province's deputy
agriculture director; a greenhouse-gas forcing chart on a 130,000-year axis with the
present as a vertical spike annotated "580 ppm CO2 equivalent" and "Eemian, sea level
+7.5 m"; "October hottest ever recorded, 1.76 °C above preindustrial"; and Al Jazeera's
map of each country's highest temperature ever recorded under the line "they do not
arrive one by one. They stack." His question: "What is it about articles and images like
this that seem to paint a more dire picture?" and then: "I actually don't think we really
do evaluate the knock on effects and cascade of effects at the same time."

Written by the project's Claude session (Fable 5.1) in the esimv1 room on canada1.

## Why they land and we do not

They are dire because of where they stand and what they point at, not their adjectives.

1. **The present tense.** A record is a measurement and cannot be argued with. Our page
   leads with 2099 under a named assumption, honest and deniable; the hindcast that ends
   in 2025 sits below the fold.
2. **The deep-time axis.** The forcing curve is a cliff only on a 130,000-year axis. On
   ours, 1850 to 2100, the same curve is a slope. Ice cores (EPICA Dome C, Vostok) are
   real records that would give us the axis.
3. **Paleo analogues beat unit translation.** "The last time forcing was here the seas
   stood 7.5 m higher" is an observation. The "billion Hiroshima bombs" line converts
   forcing rather than stored heat and overstates by a few times; skip it.
4. **Extremes, not means.** Drought then flood then drought. We model annual means per
   country; the harm is in the tail the mean drags along.
5. **One place, named people, a named source.** 173,000 acres in Battambang lands; 741
   million outside the niche is an abstraction.
6. **A dated next thing.** "Drought early 2027" is a calendar entry. Between 2025 and
   2100 we have smooth curves and a handful of crossing years.

What not to copy: "Earth is being cooked" and "100% chance". The house rule is to let the
reader feel it unassisted, and the dire feel survives without the editorial.

## Fan-out, not cascade

The basin stack cascades: weather moves water, water moves the crop, the crop moves the
herbivores, fire takes biomass, and the ledgers carry each consequence onward. The planet
stack is a fan-out. Every country layer reads one temperature line and writes a crossing,
and nothing reads a crossing. Tipping, niche and wet-bulb are three lenses on one number,
so the page reads as symptoms of one cause rather than as compounding. The kernel is not
the problem; the planet modules are terminal.

"They stack" is two claims. **Coincidence** is several limits crossed in one place at
once, which is a count per country and cheap. **Cascade** is one crossing changing
another module's inputs, which needs consequence modules, each a stated response with a
source, subscribing to crossing topics and moving a stock in a ledger someone else reads.
The couplings missing, ranked by effect on the bottom line against how well sourced:
permafrost and Amazon into the carbon ledger; ice sheets into sea level; AMOC into
regional temperature (sign differs, keep it for that); temperature into harvest; harvest
and heat into migration. Guards: include the feedbacks of the other sign too, and make
the fan-out visible rather than argued.

## The wiring graph

Anselm asked what a wiring graph and a ledger are, and later: "Ice-sheet crossing into
sea level rise is a way of saying 'ice-sheets have a relationship with sea level'. A
wiring graph is a way of saying 'relationship'?" Yes, with one sharpening: here a
relationship is a channel with two ends, one module writing a named quantity and another
reading it and changing its state because of it; the graph draws the channels the run
has, not the ones the world has, and the ice-to-sea arrow is absent. "Wiring" is the
kernel's word (from `esim check`), not the literature's; terminology.md now defines
ledger, coupling, feedback and wiring in plain words.

Built today: `web/src/lib/Wiring.svelte`, a section on every run page drawn from
`describe.json`. Rows by longest path over forward couplings, sources pulled down to sit
just above their first reader, barycentric ordering within rows, back edges found by
depth-first search and drawn in the brand colour climbing beside the forward arrow,
ledgers as a column of shared books at the right joined by dotted lines to their holders,
long edges bowed only as far as the boxes in their way require, labels placed where
edges have parted (near the target when a source fans out, near the source when a
target gathers, in a row gap for a long edge), outputs nothing reads as short red stubs
into air with a tooltip, and three counts under the drawing: couplings forward and
ledgers, feedbacks named, outputs nothing reads named, plus the sentence "Crossings only
the story listens to: people.crossing, tipping.crossing. A threshold passed changes a
sentence on this page and nothing else in the run."

The two drawings say it without argument. Planet-2100: 16 couplings, 1 feedback (energy
to carbon), 5 outputs nothing reads, both crossing topics terminal. The nitrogen basin
run (m11, 2013 season, rerun today for its describe.json): 14 couplings, 6 ledgers, 5
feedbacks (veg to water, deer to veg, coyote to deer, farms to veg, nitrogen to water).
Verified by screenshot in light and dark and at phone width (the drawing scrolls
sideways there; it does not shrink).

Not done: the records-to-date first screen, the ice-core axis, the extremes layer, the
coincidence count and the consequence modules, all on `todo.md` under a new heading.
Nothing deployed; push is his call.
