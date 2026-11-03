# 2026-11-03 — A project's README, and the 2100 runs read against the thresholds

Anselm, this morning, after the night's commits were pushed and deployed: "i want to
deeply examine the 2100 scenario and measure it against a clear sense of any phase
transitions - that lay people can understand." And while that was under way: "can this
page include a readme? imagine how somebody who makes a scenario wants to communicate it?
we do already effectively treat manifests like a poor persons html layout... it is worth
thinking about hmm, if we even want to just use html? a lot of my thinking on other
projects was not just around raw sims but around communications, storytelling. generally
imagine that we are allowing users to make scenarios - they may want to document them -
generally i imagine a scenario does exist on the filesystem in a first come first serve
directed acyclic namespace (ie the filesystem)? in this case things like documents or
other materials can be in that area? thoughts?"

Agent: Claude (Fable 5.1), the earth-simulator-1 room on canada1, session
`0bad2b71-90e5-4e88-978e-bd10ad8801f7`. The two asks turned out to be one: the lay
reader's examination of the 2100 runs is exactly what a scenario's author wants to say
about it, so it was written as the first project README and the README machinery was
built to carry it. Nothing is pushed since 4a5c255; the commits that follow this entry
are local.

## 1. The answer to the README question

The project folder already is the namespace he describes: `projects/<area>/<name>/`,
first come first served per area, the server serving the folder as it is. What it lacked
was the author's own account. Now `README.md` in the folder renders on the project page
above the variants and runs.

Markdown, not HTML, and for one reason that is the honesty rule applied to storytelling:
a figure in the README is a fenced block the viewer fills from the runs, and an author
cannot type a number into it. `ladder` draws the thresholds against the paths, `table
<year> series=label …` prints each variant's value at a year. Both read the runs' own
files: the rungs come from the tipping module's describe (which now carries its table),
the paths and values from `run.json`, the years from the crossing events. Prose can still
quote a number, and a quoted number goes stale when the runs change, so the habit the
docs now recommend is to quote few and let the figures carry the rest. The vocabulary of
embeds should grow from the modules' declared views (counter, chart, map, crossings,
story), so that a README can only show what a module said it would show. That is the
"poor person's HTML layout" made deliberate: the manifest arranges modules, the README
arranges their views around an argument, and raw HTML would let the argument escape the
runs.

Materials belong in the folder (images, a bibliography, source PDFs where licensed) and
`docs/<area>/` should fold into the area's own folder once the namespace is crumpled.
Both are in the todo.

## 2. What was built

- `Describe.tables`: a module can record small tables it read, as JSON, so a run folder
  carries the exact table it used. `tipping.elements` records its sixteen rows;
  `people.countries` records the countries in the outline file's order, read from the
  cache only (describe never fetches).
- `web/src/lib/runs.js` (the latest run per variant, with audit and describe; a series at
  a time), `Readme.svelte` (prose and embeds), `Ladder.svelte` (the figure and its years
  table), the prose styles moved from the docs page to `app.css` so both pages share them,
  the README fetched and rendered on `ProjectView.svelte`.
- The Markdown renderer split a bullet when a wrapped line began with a year and a period
  ("2040. The first decades…" read as an ordered item). Indented lines now continue the
  item above them.
- A bug found on the way: the compare page's country picker mapped `country_index` values
  through `docs/countries.json`, which the server writes from the 110 m outline file, while
  the cells runs use the 50 m file with 242 countries in a different order. Delhi read as
  Iceland. The picker now takes the run's own list from its describe and falls back to the
  server's file for older runs.

Seven variants rerun (the describe is part of the run folder; the digests and folder names
did not change), `cargo test --release` passes, `esim regress` reports no drift.

## 3. The examination

The thresholds are the sixteen tipping elements of Armstrong McKay et al. 2022, every row
checked yesterday. The measure is simple and a lay reader can hold it: for each path, the
year it first passes each element's central estimate ("likely") and the year it enters
the range ("possible"); how many of each by 2100; the next rung and the gap to it. The
model's own thresholds for people, the 29 °C niche and 31 °C wet-bulb, are read the same
way, as counts along a path.

First the clock. The model is 0.22 °C warm in 2024 (1.68 against HadCRUT5's 1.46) and
passes 1.5 °C in 2018 where the record first does in 2024. Every year below is about six
years early, and the README says so before its first figure.

| | zero by 2050 | falling 3%/yr | held | held, low trigger | trend | growing 1%/yr |
|---|---|---|---|---|---|---|
| warming in 2100 | 2.11 | 2.48 | 3.36 | 3.44 | 3.73 | 3.96 |
| still rising, °C per decade in the 2090s | 0.01 | 0.04 | 0.21 | 0.24 | 0.30 | 0.35 |
| 2.0 °C reached | 2039 | 2038 | 2036 | 2036 | 2036 | 2036 |
| 2.5 °C | — | — | 2058 | 2058 | 2056 | 2054 |
| 3.0 °C | — | — | 2082 | 2081 | 2075 | 2071 |
| 3.5 °C | — | — | — | — | 2092 | 2086 |
| past the central estimate by 2100 | 7 | 7 | 9 | 9 | 10 | 10 |
| inside the possible range by 2100 | 13 | 13 | 14 | 14 | 14 | 14 |
| next rung, gap | Sahel 2.8, 0.69 | Sahel 2.8, 0.33 | Amazon 3.5, 0.14 | Amazon 3.5, 0.06 | the 4.0 four, 0.27 | the 4.0 four, 0.04 |

What that says, in the order the README says it:

- Five elements are past their central estimate in the model's 2024 (Greenland, West
  Antarctica, the low-latitude corals, permafrost abrupt thaw at 1.5; the Barents Sea's
  winter ice at 1.6). The record touched 1.5 for the first time in 2024.
- To 2040 the paths do not separate: all of them pass the Labrador Sea (1.8) in 2029 and
  the mountain glaciers (2.0) by 2040, zero-by-2050 included. The first two decades are in
  the system already.
- Then they part. Zero by 2050 levels at 2.1 and all but stops; held is still climbing a
  fifth of a degree a decade in the 2090s. Held passes the West African monsoon (2.8) in
  2073 and the East Antarctic basins (3.0) in 2083 and ends 0.14 short of the Amazon;
  trend passes the Amazon in 2093, growing in 2087. No path reaches 4.0, where AMOC, the
  boreal forest (both edges) and the deep permafrost sit; growing ends 0.04 below it.
- The possible range is the wider story: thirteen of sixteen entered by 2040 on every
  path, fourteen by 2100 on held and up. Only Arctic winter ice (4.5) and the East
  Antarctic sheet (5.0) stay out of reach.

The knife edges (0.14 and 0.04 °C) are not findings; they are what a central estimate
with a one-and-a-half-degree range looks like when a line is drawn through it. The README
puts the possible years in the same table so no one reads the edge as a wall.

## 4. The one transition the runs found

Along the growing path, sampled at the year each half degree is reached (the UN's
population growing from 8.2 to 10.2 billion beneath):

| warming | year | outside the niche | over 31 °C wet-bulb |
|---|---|---|---|
| 1.7 | 2024 | 147 M (1.8%) | 6.5 M |
| 2.0 | 2036 | 259 M (2.9%) | 11 M |
| 2.5 | 2054 | 552 M (5.6%) | 20 M |
| 3.0 | 2071 | 952 M (9.3%) | 35 M |
| 3.5 | 2086 | 1.38 bn (13.4%) | 204 M |
| 3.9 | 2098 | 1.76 bn (17.3%) | 470 M |

The niche count is smooth and convex: about 110 million for the first half degree,
about 400 million for the last. The wet-bulb count has a step between 3.0 and 3.5 °C,
from 35 to 204 million, and the frames say where: China's cells crossing 31 °C on three
afternoons a year, 20 one-degree cells in the 2070s and 39 in the 2080s, Japan's coast
and Taiwan after (grouped from `wetbulb_crossing_year` and `country_index` with the 50 m
order; with the 110 m order, the first attempt, Denmark and Iceland crossed, which is how
the picker bug surfaced). The humid plains of eastern China move from almost never to
every summer within about fifteen years. That is a transition of the same shape as the
planet's, in the human layer, and the only one on the page that the runs produced rather
than read from a table. It is also the one to check first: against GSOD stations on the
Yangtze and North China plains.

The absence beside it matters as much. The Indus and Ganges plains, which Raymond,
Matthews and Horton 2020 put first for humid heat, barely cross in any run (India 5
cells, Pakistan 8, all after 2070, on the growing path). Two known faults point the same
way: the reanalysis reads humid extremes 2 to 4 °C mild against the stations, and
India's warming is scaled from a record that has run cool (slope 0.82). The China step is
probably real and probably not the first; the README and the limits page both say so.

## 5. What a crossing does in the runs, stated once more

Thirteen of sixteen elements are flags: a sentence with a citation and nothing else
changes. Three release carbon (permafrost abrupt thaw per degree, Amazon 30 GtC,
permafrost collapse 125 GtC), and that comes to 5 ppm and 0.02 °C by 2100 with emissions
held, 69 GtC and 0.07 °C with every element tipping from the low end of its range. The
ladder is a measuring stick held against the runs; apart from the carbon, the runs do not
feel the rungs. The README's last section says so in those words, because a lay reader
who sees red dots on a chart will otherwise assume the chart is the mechanism.

## 6. Open

In the todo under "Communicating a scenario": more embeds from the declared views, a
shared cursor on the README, the drafting facade writing a first README, materials
listed on the page, docs folding into areas, and the China step checked against
stations. Push and deploy are Anselm's call; the live site is at 4a5c255.
