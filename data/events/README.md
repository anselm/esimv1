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

## Sources worth mining for rows (not yet mined)

Public lists of future events exist and are the cheap way to fill the calendar: the
Wikipedia lists of future astronomical events and the decade pages (2030s onward), the
UN and IPCC meeting calendars, national election calendars, utility and transmission
plans with in-service dates, dam and power-plant retirement schedules, treaty texts with
their sunset and review clauses, and census and Olympic schedules. Each row taken from
such a list cites the list and the list's own source where it has one.
