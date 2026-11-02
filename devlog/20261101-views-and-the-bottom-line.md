# 2026-11-01 — Modules own the page; wet-bulb at the capitals; the bottom line

Anselm, looking at the planet run page: which code makes which part of it; could the
page be composed from the manifest, each module asking for its own widgets, so that the
special cases go away; the cursor should not start at the end, because the tip-over is
the point; the countries list could be a map; a total score across bottom lines; and,
most sharply, that the page "has zero convincing value, it just shows some slight colour
changes and some graphs", so anyone seeing it would carry on polluting. Agent: Claude
(Fable 5.1), earth-simulator-1 room on canada1.

## What made which part, before

The maps were already each module's frames and the charts each module's reported
series. The section order, the audit block, the crossings table and the counters were
code in the viewer, and the crossings were a list because nobody had declared them a map.
The audit is a separate crate that reads the finished folder; it is not a module and
should stay independent.

## Views

A module's describe now carries `views`: `counter` (a report as one number at the
cursor), `map` (a frame), `chart` (a report, optionally with an observed series beside
it), `crossings` (a topic's crossing events so far), `story` (the sentence each crossing
event carries, newest first, with its source) and `text`. Every run writes
`describe.json` beside `run.json`, and the run page is composed from it: the kernel's
header and cursor, then each module's views in the manifest's order, then the auditor,
scores, ledgers, and the raw events and manifest behind disclosures. A module that
declares nothing gets a map per frame and a chart per report. There is no module-specific
code left in the run page.

Manifest order is therefore page order. Because a top-level `include` puts the bundle's
modules first, a `[[module]]` entry that is only `include = "planet-core.toml"` now
splices the bundle in at its own position, so `p4-planet-people.toml` lists the story
module first, then the core, then the country layers.

The cursor starts at the beginning and plays, so a reader sees the crossings arrive.
The two heat layers write a second map each, the year every country crossed, which
fills in as the cursor passes.

## Wet-bulb at the capitals

`people.wetbulb`: ten observed years of daily maximum temperature and mean dew point at
the weather station nearest each capital (NOAA GSOD, 1,700 small files from the AWS
bucket; Open-Meteo rate-limited the hourly version after twenty capitals), an afternoon
wet-bulb by Stull's formula, shifted by the country's warming at constant relative
humidity, counted against 31 °C (the chamber-study limit for healthy adults at rest,
Vecellio et al. 2022, recalled) and 35 °C (Sherwood and Huber 2010, recalled). A capital
crosses when it passes a limit on more than three afternoons a year. 155 of 177 countries
have a usable station; the thirteen left out are named in the run log (Kinshasa, Quito,
Kabul, Sanaa among them).

## The bottom line, told as it happens

Every crossing event now carries a sentence written by the module that knows the source,
and a `story.bottomline` module, first in the manifest, shows the headline counters and
those sentences at the cursor, with a paragraph naming what the run does not yet measure.
Under emissions held at the 2024 rate the sentences read, in order:

- 2024: Muscat (Oman) passes 31 °C wet-bulb on more than 3 afternoons a year, the limit
  past which a healthy adult at rest can no longer shed heat. 5 million people live in
  Oman.
- 2029: Kuwait City. 5 million.
- 2037: the warming passes 2 °C, the central estimate at which extra-polar mountain
  glacier loss becomes likely: loss of seasonal water storage for the rivers below them.
- 2048: New Delhi (India) passes 31 °C wet-bulb on more than 3 afternoons a year. 1.7
  billion people live in India.
- 2072: Taipei. 2074: the Sahel monsoon's central estimate. 2083: Djibouti. 2084: the
  East Antarctic subglacial basins' central estimate.

Under emissions growing one percent a year the list goes on to Qatar passing 35 °C in
2083, Tokyo in 2084 (82 million), Dodoma in 2096 (253 million) and Manila in 2100. Under
emissions to zero by 2050 it stops at Kuwait City in 2029.

| 2099 | warming | outside the niche | capital past 31 °C wet-bulb | tipping past central |
|---|---|---|---|---|
| zero by 2050 | +2.09 | 394 M | 55 M | 7 |
| held at 2024 | +3.33 | 741 M | 1.58 bn | 9 |
| growing 1%/yr | +3.92 | 942 M | 1.92 bn | 10 |

## On "zero convincing value"

He is right about the page as it was, and partly right about the page as it is. What the
run can honestly show is who lives where a stated physical limit is passed and when, and
which assessed thresholds the warming crosses; it now says those things in sentences
with sources, first, before any map. What it cannot yet show is what he is asking for
underneath: deaths, harvests, forests, species, water, and people moving. Those are
layers, each with a record and a published response, and they are the list in the story
module's last paragraph and in `todo.md`. The population here is the UN's path and does
not respond to anything; that is the single most misleading thing on the page, and the
text says so where the number is. The first layer to add is heat mortality with the
multi-country exposure-response curves and their ranges, because it is the one the
sentences already point at.

A note on the sentences: they are the moment the system starts to say things to people,
and the house rules apply hardest here. Each is generated from an event that carries its
threshold, its source and its confidence; the viewer prints them and adds nothing. A
sentence that outruns its source is a bug in the module that wrote it.

## Not done

- Heat mortality; harvests; forest cover; biodiversity; water; migration; a population
  that responds. The honest "total score" waits on those.
- Checking the recalled thresholds against their papers.
- A country stress board at the cursor.
- The thirteen capitals without a station within 80 km.
