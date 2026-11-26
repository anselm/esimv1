# Every run page is written in HTML

26 November 2026. Claude (Fable 5.2), esimv1 room.

Anselm: "every single scenario should use the html style layout pattern that was first employed by
the portland scenario."

Done: a `run.html` beside the manifest of each of the twelve projects that had none, in Portland's
shape: one paragraph saying what the run is, the picture (the 3D scene where the run has an elevation
frame, a map or nothing otherwise), the timeline, a row of counters, then tabs. Every page ends with
"How far to trust it" where there is something to say and "Everything recorded". The words are mine,
drawn from each project's README and this repository's notes; he has not read them.

The files were written by a script kept in the session scratchpad (`pages.py`); they are ordinary HTML
and are to be edited by hand from here.

Changes in the viewer (its own repository):

- `nb-lede`, `nb-counters`, `nb-note` are styled for every page, so a page carries no style block.
- `optional` on a map, chart or counter draws nothing when a run lacks it. Earth 2100 uses it for the
  wild-card series, which only some variants record.
- A page with no `?t=` opened at 1970 when the run spans it (`Number(null)` is 0). Earth 2100 now opens
  at 2100.
- Duplicate keys in the module panels' loops raised an error on Earth 2100's "Everything recorded" tab.
  The keys now carry the index. Chart ticks likewise.

Checked: every map, series and module named in the thirteen pages exists in every run of its project
(a script over all run folders), apart from the two marked optional. One run page of each project was
drawn on the local server; I looked at Fort McMurray, Palisades (Rothermel), Earth 2100, Half Moon Bay
and the Boone fire, in light or dark. The other seven were drawn without a page error but I did not
look at each picture.

Known gaps:

- Three projects' runs predate `describe.json` (two-buckets, planet-co2-1850-2025, boone-fire-september),
  so their last tab links to the full data page in place of panels. Rerunning them would fix it.
- A chart tag draws one series, so model and measurement sit side by side, not on one chart.
- There is no globe tag; Earth 2100's regions tab shows the regions module's own panel.
- Project pages are unchanged: cover, runs, README. Whether they get the same treatment is part of the
  layout discussion Anselm opened the same day.
- Nothing is deployed, and the part repositories are not tagged.
