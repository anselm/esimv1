# 2026-11-03 — A neighbour: ageospatial.com

Anselm: "Here is a fun website that has some similar goals - worth having a devlog entry on
and maybe having todos around it for later : https://www.ageospatial.com/". Read once on
2026-11-03 from canada1 (the front page only; everything below is what the page says, quoted
where it can be, and nothing about the company beyond it).

## What the page says

Headline: "Physical intelligence for every decision." The product is "Our GeoAI Operating
System", which "powers critical decisions in the physical world." It "fuse[s] satellite
imagery, maps and geospatial data into one view, from space to property", takes in "aerial,
satellite and drone imagery, hazard maps, permits, listings, news, spreadsheets, weather and
wildfire alerts", and produces one record per property in which "each fact carries its
source and date", delivered "as spreadsheets, reports, GIS, dashboards, models or an API".
The worked example is a property (the Golden Gate Bridge and Fort Point): structural facts,
land and building details, exposures (flood, wildfire, earthquake, landslide), and a
timeline of condition ratings and retrofit work. Its four questions, as section titles:
"What should I know about this infrastructure?", "What defines this land and property?",
"What exposures surround these assets?", "What has changed over time?"

Three sectors with one headline each: insurance ("every property decision"), finance ("every
credit decision"), government ("every public decision"). "Trusted by" names AXA, Die
Mobiliar, swisstopo, the Swiss Federal Department of Foreign Affairs, the Republic and Canton
of Geneva, the Canton of Bern, the Canton of Valais, Geoneon, People in Need and 4 Earth
Intelligence; "Backed by" names Y Combinator and The Residency. Absent from the page:
pricing, open source or licensing, founders, documentation, an API reference, and any
methodology for the scores shown.

## Where it meets this project, and where it does not

The shared ground is the record per place with every fact sourced and dated. That is this
project's honesty rule applied to a property instead of a run: our run folders carry every
number's source; their property record carries every fact's. Their four questions are the
ones a person asks about a place, and three of them (what is here, what is exposed, what has
changed) are questions a scenario here should be able to answer for any building in its
window: the Palisades work already holds the footprint, the height, the inspection outcome
and the hour the model's fire arrived, per structure.

The difference is the verb. Theirs is retrieval and fusion: assemble what is known. Ours is
simulation: run what is known forward and hold it against the record. Their wildfire
"exposure" is a hazard map; ours is a fire that arrives at a particular hour, or does not,
under a particular wind. The two are complementary, and the honest reading is that a tool
like theirs is where our structure facts should come from, and a tool like ours is what
their exposures should be tested against. They have paying sectors (insurance, credit,
government); we have a planet page and two fires. Nothing on their page says how a score is
made, which is the one thing every page here says first.

## Todos (in todo.md under "Neighbours")

- A record per structure: for every building in a fire window, one card with the sourced
  facts (footprint, height, inspection outcome, the hour the model's fire arrived in each
  variant, the land cover and slope under it), each fact with its source and date; the
  facade answers "what should I know about this building?" from it.
- Exposure as a run output: per building, the arrival hour's distribution across an
  ensemble, not a hazard class.
- "What has changed": the before and after imagery (Sentinel-2 scenes already found) beside
  the inspection, per structure.
- Read their public material again before any public comparison; this entry is one front
  page on one day.
