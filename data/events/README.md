# Event tables, and the record of events

Beside the two tables below, a folder per studied event holds copies of the public record we
read it against, organised for our use with a README that names every source, its licence and
the day it was fetched: `palisades-2025/` (since 2026-11-03) is the first. The catalog
(`data/catalog.toml`) remains the registry for anything a module fetches; a copy here is for
reading, scoring and reference, so the record cannot drift or vanish under the runs.

# Event tables

Two flavours of one table, as the design brief says: the **calendar** holds known or
committed future events with a date; the **hazards** hold possible events with a rate.
Both are plain TOML, one `[[event]]` per entry, so a pull request can add a row and a
reviewer can read it. These seed files live here until the shared catalog on
digitaltwins.wiki exists; the schema is meant to move with them.

Rules. Never invent a date, a rate or a source. A row with a source we have not checked
says so in `confidence`. A row that is somebody's prediction is `speculative` and names
who predicted it. Rates that are guesses say `guess` in `source`. `known_since` is the
date the fact entered the record, so a hindcast started in 1990 can filter to what was
known then.

## Fields

| field | calendar | hazard | meaning |
|---|---|---|---|
| id | yes | yes | stable slug |
| title | yes | yes | short name |
| category | yes | yes | celestial, demography, law, capital, rhythm, climate, technology, geophysical, space-weather, biological, human, economic |
| date | yes | no | ISO date, or `window = ["earliest","latest"]` when only a span is known |
| rate_per_year | no | yes | base annual probability; `null` when unknown |
| depends_on | no | optional | state names a hazard module reads to modulate the rate |
| consequence | yes | yes | what it does to the world, in words |
| module | optional | optional | module the event is addressed to, when one exists |
| valence | yes | yes | good, bad, mixed |
| confidence | yes | yes | certain, committed, projected, speculative |
| source | yes | yes | who says so; a citation, not a URL we have not opened |
| known_since | yes | yes | ISO date the fact became known |
| setting | no | yes | default in a manifest: off, on, or forced |
| notes | optional | optional | caveats |

`confidence` meanings: *certain* (celestial mechanics, demography already born),
*committed* (law in force, contracts signed, dates fixed by institutions; can still slip),
*projected* (an institution's forecast with a method), *speculative* (an opinion or a
book's table).

## Sources worth mining for rows (found 2026-10-31, not yet mined)

Anselm: "There are encyclopedias of the future out there." There are, and they are the
cheap way to fill the calendar. Found by web search this session; the pages exist, their
contents have not yet been read into rows:

- Wikipedia's hub, https://en.wikipedia.org/wiki/Timelines_of_the_future, which points
  to the "Timeline of the near future" (to 2299), the "3rd millennium" page (to 3000),
  the "Timeline of the far future", the "List of future astronomical events"
  (https://en.wikipedia.org/wiki/List_of_future_astronomical_events) and the "List of
  future calendar events". Wikipedia's own sourcing varies by row; each row we take
  cites the page and the page's own source where it has one.
- futuretimeline.net, a long-running site of dated predictions in a press style; useful
  as a list of claims, each to be marked speculative with that site as the source.
- Institutional calendars, which are better than encyclopedias for committed events: UN
  and IPCC meeting schedules, national election calendars, utility and transmission
  plans with in-service dates, dam and plant retirement schedules, treaty texts with
  sunset and review clauses, census and Olympic schedules, launch manifests.

Mining is a job for an agent that reads a page and drafts rows into a `drafts/` file;
rows move into these tables by review. Machines propose, humans promote.
