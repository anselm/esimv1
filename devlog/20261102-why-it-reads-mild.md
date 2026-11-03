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

## Addendum, later the same day: a 2009 article as a regression test

Anselm sent George Monbiot's "A self-fulfilling prophecy" (Guardian, 2009-03-16; "take
with a grain of salt"). The merit of a seventeen-year-old piece is that its claims have
matured into checkable ones. Against the record and against this model:

- "Four degrees is a best case on current trends." Emissions grew from 10.4 GtC in 2009
  to 11.8 in 2024 in the OWID record, about 0.85% a year, not the 3% the piece implies.
  In this model, emissions growing 1% a year to 2100 give +3.9 °C and emissions held give
  +3.3 °C, so four degrees was a fair reading of the trend continued for ninety years.
  "Best case" was wrong: emissions to zero by 2050 give +2.1 °C here.
- Anderson and Bows (2008): cuts of 3% a year from 2020 still lead to about four
  degrees. Added as the project variant `falling` (3% a year from 2025, since the record
  now runs to 2024): +2.45 °C in 2099, 504 ppm, 7 tipping elements past their central
  estimate. This model disagrees with their four degrees by a wide margin. I recall their
  figure sat at the high end of the sensitivity range and carried non-CO2 assumptions,
  but I have not read the paper; it is on the list to check before saying who is right.
- Solomon et al. (2009, PNAS): temperature stays roughly constant for a thousand years
  after emissions stop. In the zero-by-2050 variant the warming is 2.06 °C in 2050 and
  2.09 °C in 2099 with CO2 falling from 473 to 448 ppm: flat, as the paper says. That is
  the zero-emissions-commitment result and this model reproduces it without having been
  built to; a good sign about the carbon response and the two-layer balance together.
- Sea level "two or three times the IPCC's predictions". Out of reach here: this model
  has thermal expansion only, a known fault on the limits page.
- Vegetation a net carbon source above about 3 °C, food production down above about
  3 °C. The first is the land-sink reversal, which this model has only as a smooth
  feedback (Millar) and as the Amazon row of the tipping table; the second is the
  harvest layer on the todo. Both are couplings, not framings.
- The aid, reparations and politics are out of scope for a simulator.

One thing the `falling` run exposed: the count of people whose capital passes the
wet-bulb limit jumps from 34 million in 2050 to 1.7 billion in 2075 in a single step,
because one very populous country's capital crosses and the layer counts the whole
country by its capital's station. The step is an artefact of one station per country,
not a cliff in the world; the 1 km population layer and more than one station per
country are the fix and were already on the todo. The page should say so until then.

## Addendum, evening: cells instead of capitals

Anselm, on the one-station step: "When you say a populous capital crosses are you saying
that the daily temperature in a large capital goes past the wet bulb point long enough
for the country as a whole to be tipped? Yes more granular population would help. Do we
really need stations?" Then: "What you're saying we need is a more granular map of both
people and a granular map of estimated surface temperature?" and "Well I think that we
should have layers like this."

The shape agreed: people on a grid, temperature and humidity on the same grid for a
baseline of real years, exposure computed per cell and summed by country, no threshold
applied to a country as a whole.

**Data doors tried (all anonymous):**
- WorldPop 2020 1 km global mosaic: 870 MB GeoTIFF, tiled 256, Float32, LZW, CC BY 4.0,
  found through the hub API (the directory paths guessed first were 404s). The server
  claims byte ranges and ignores them, so GDAL over HTTP fails; downloaded whole. Our
  reader gained TIFF LZW and a tile walker so the raster is never held in memory; the
  cell sums are cached beside the file. GDAL's statistics on the same file give 7.969
  billion people, ours 7.970: the decoder is right.
- ERA5, the public Google copy: hourly 2 m temperature is there on a quarter degree, but
  2 m dew point exists only in the raw-grid store chunked one hour of the whole globe
  per chunk, so a baseline decade would be several hundred gigabytes. Out for now; a
  bulk path (its own job on a machine near the bucket) is the way back to it.
- NASA POWER: MERRA-2 at 0.5 by 0.625 degrees, daily maximum temperature and dew point
  and a 2001 to 2025 climatology, by regional request. Limits found by trying: 10
  degrees a side, 366 days, one parameter per regional request; no stated rate limit;
  six back-to-back requests answered in 0.4 s each. 343 ten-degree boxes hold people, so
  a decade of two parameters plus the climatology is 7,203 requests; `esim warm cells`
  makes them once, resumably, at about one a second. The JSON (2 MB per answer) is
  reduced to a table of tenths of a degree (230 KB) and the JSON's hash kept in the lock,
  which is a new fetcher mode, `fetch_reduced`. POWER's referencing page asks for an
  acknowledgement and states no restriction; no licence text exists on the site, which
  the catalog says.
- Natural Earth 1:50m outlines, because at half a degree the 1:110m file loses the small
  island states.

**Modules.** Each country kind kept its country mode as the default and gained a cells
mode by parameter, so every older run still reproduces: `people.countries` with
`distribution = "worldpop"` places each country's UN count over its cells in proportion
to WorldPop and owns `people.count`; `people.heat` with `baseline = "cells"` adds the
country's warming since the climatology to each populated cell's own annual mean;
`people.wetbulb` with `source = "cells"` builds, per populated cell, the days per year
over each limit against the warming, from the baseline decade, by solving for the warming
each day needs to reach the limit (a Newton step on Stull, monotone) and binning it, so a
run's year costs one lookup per cell. A country "crosses" when half its people live past
a limit; the counts and person-days are continuous. Frames are folded to a coarser grid
(`frame_res_deg`) so a half-degree run does not write a megabyte per map per frame.
Climate publishes a second anomaly, since POWER's climatology window, beside the first.

**Honesty notes written into the describes and the limits page.** The warming applied
to a cell is still its country's one number; MERRA-2 is a reconstruction, not a
measurement, and the station form stays as the check; the day's dew point stands for the
afternoon's; the 2020 settlement pattern is held for every year; the half is a choice.

**Open.** The lock file gains one entry per POWER answer (about 7,200, 1.4 MB) and is
rewritten on each; a per-source lock shard would be tidier. The runs of the planet-2100
project with the cells manifest, and their numbers against the capital-based ones, follow
once the cache is warm.
