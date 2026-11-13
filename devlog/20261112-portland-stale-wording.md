# Portland: the stale wording brought up to the page

2026-11-12, night. Claude (Claude Code session 018TGCFhEpYs9hm68xDLd5Xy, room earth-simulator-1 on canada1).

Anselm, after the project README was rewritten: "update stale portland wording overall everywhere",
"don't bother running the simulator on the local machine", and on the sources listed in the last
report: "you re-read them for this pass on the readme? You can add them to the readme if so."

## What changed

The README of the project was current. Everything else that describes the Portland scenario still
carried wording from earlier in the day, and item 9 of
`20261112-portland-readme-rewritten.md` listed most of it. Changed in this pass:

- `projects/anselm/portland-cascadia/project.toml`: the description, the cover caption (the
  picture is seven days after the earthquake, the caption said day 3) and all ten variant titles.
  `burnside-ready` now says in its title that it has no effect on the computed runs.
- `manifests/q1-portland-cascadia.toml`: the header comment, the notes, and the comment on the
  fitted parameters, which gave a 13 percent collapse share where the parameter is 0.25 and fit
  ratios from before the collapses were drawn. The ratios are now those of the runs of 12
  November, read from the run folders.
- `data/events/portland-cascadia/hub.toml` and `record.toml`: the ordinance rows say the vote of
  12 November, 10 to 0 with two absent.
- Module descriptions in `lifelines.rs`, `quake.rs`, `components.rs` and `agents.rs`. The
  lifelines summary described the ranges mode as if it were the model. The ground summary said
  "the best anyone has". The buildings summary said the day-night and wet-dry contrasts "stand",
  which the page shows they do not, and named two fitted scales where six parameters are fitted.
  The agents' summary said their total agrees with the cell model. The components' texts said
  "poles" where the page says transmission structures; the report names (`power.poles_failed`)
  are unchanged.
- Two story sentences in `lifelines.rs`: "The grid is down everywhere" is now said only in ranges
  mode, and the computed mode's "power is back for half the people" is replaced by the day's share
  of people with grid electricity.
- `docs/system/limits.md`, the earthquake section, rewritten to the page's figures.
- The root `README.md`: the Portland bullet and the paragraph in "Where things stand".
- `CLAUDE.md`: a new entry beside the old Portland one saying where the page supersedes it.
- `docs/kinds.json` and `projects/index.json`, regenerated with `esim kinds --json` and
  `esim projects --json`. These list; they run nothing.

## What still carries the old wording

No variant was run and `esim project update` was not run, on his word. So:

- The ten run folders keep their describes and their story sentences as they were written on
  12 November, including "The grid is down everywhere" and "Day 1: power is back for half the
  people". The run page shows those until the variants are next run.
- The project's frozen `manifest.toml` and `manifest.json` keep the old notes.
- `cover.png` and `card.png` are stills of the old run page.
- The devlog entries of the day are a record and are not edited.

The release build was made twice, only to confirm that the edited strings compile and to
regenerate the two indexes. `cargo test` and `esim regress` were not run, because both run
simulations. No topic, field, ledger or parameter changed, only words.

## The sources

The last report listed sources. Checked against this session's record of fetches:

Read on 12 November 2026: the council's page for Ordinance 2026-312 (twice), the two USGS
publication pages for Professional Paper 1707 (abstracts, not the paper), the USGS publication
page for Goldfinger and others 2012 (abstract, not the paper), the city's page on unreinforced
masonry buildings, the Pacific Northwest Seismic Network's blog post on the M9 project by
Elizabeth Urban (13 December 2017), and the USGS catalogue entry for the 1700 earthquake (title,
magnitude and time only).

Seen only as search results and not opened: the council's news item of 1 October 2026, KPTV's
story of 3 October 2026, KATU's story, and the state's PDF on the history of its seismic codes.
The page takes nothing from these four and they are not added.

In the reading list, the seismic network's post is added as read with nothing taken, and the
Hazus entry now says which curves are recalled from the manual and that the older bridges' curve
is this project's judgement.

## The writing guidance

The harness writing-style skill now holds the revision Anselm brought from ChatGPT that evening
(harness devlog `20261112-writing-guidance-jubilee.md`, third round). The README of this project
was written under the version before it and has not been rewritten again.
