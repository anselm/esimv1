# The Palisades fire, 7 to 31 January 2025: the record we hold

Copies of the public record of the fire, kept here so the scoring, the structure model and the
prose read one stable set of files. Fetched from canada1 on 2026-11-03 unless stated; the
devlog entries `20261103-six-links.md` and `20261103-beyond-rothermel.md` say why each was
sought. Nothing here is ours; every file names its publisher below and keeps that publisher's
figures as published, disagreements included. The catalog (`data/catalog.toml`) carries the
live sources the modules fetch; this folder is the reading copy.

## What the publishers say the fire was

| | CAL FIRE incident page (read 2026-11-03, page last updated 2026-04-09) | UL FSRI timeline report, Table 1.1 | City of Malibu archive |
|---|---|---|---|
| started | 7 January 2025, 10:30 local, southeast of Palisades Drive | 7 January 2025, 10:30 | |
| acres | 23,448 | 23,707 | 23,707 |
| contained | 31 January 2025, 10:48 | | 100% |
| structures destroyed | 6,845 | | 6,831 (Palisades and Malibu; 720 in the city) |
| structures damaged | 975 | | 973 |
| civilian deaths | 12 | 12 | |
| cause | "Arson" on the page as of April 2026; "Under Investigation" in the January 2025 reports | | |
| jurisdiction | Los Angeles City Fire Department; "Not a CAL FIRE Incident" | LAFD, LACoFD | |

Our perimeter reader gives 23,448 acres for the WFIGS perimeter of record, matching CAL FIRE's page.

## Folders

### `calfire/` — CAL FIRE's status reports

- **Source** `https://www.fire.ca.gov/incidents/2025/1/7/palisades-fire/updates`, 344 report
  pages from 11:06 on 7 January to 15:46 on 10 February 2025 (local time), one page each. The
  site answers 403 to a browser user agent and 200 to a plain one.
- **Files** `updates.csv`: one row per report with the as-of time, URL, size in acres,
  containment, engines, water tenders, helicopters, dozers, hand crews, total personnel,
  structures threatened, destroyed and damaged, civilian fatalities, as the page states them
  (blank where the report has no such field). `updates.json`: the same rows with the page's text.
- **Licence** State of California public web content; no licence text on the pages. Attribute CAL FIRE.
- **Use** The reported-acreage series for the first days and the daily resource counts from the
  afternoon of 8 January (149 engines, 4 dozers, 7 helicopters, 836 personnel at 13:20 on the 8th;
  476 engines, 48 dozers, 44 helicopters, 113 crews, 4,471 personnel at 08:08 on the 17th).

### `ul-fsri/` — UL Research Institutes' timeline dataset and report

- **Source** Fire Safety Research Institute, UL Research Institutes, for the Office of the Governor
  of California. The dataset is the public Google Sheet linked from
  `https://fsri.org/research-update/southern-california-fires-incident-analysis-report-released`
  (`s.ul.org/fsri-califonia-data-p2`), read as CSV through the sheet's export, one file per tab.
  The report text is `pdftotext -layout` of the Phase One PDF, *Southern California Fires
  Timeline Report*, DOI 10.60752/102376.29905157, published 2025-11-20 (351 pages; the PDF itself
  is 34 MB and not kept here). The Phase Two report, *Southern California Fires Incident Analysis
  Report*, DOI 10.60752/102376.31982232, published 2026-10-02, 82 MB, is not kept here either.
- **Files** `palisades.csv` (4,785 events, 2 to 9 January, 3,777 on the 7th), `eaton.csv`,
  `lachman.csv` (the 1 January fire whose holdover is the suspected origin), `weather.csv` (NWS
  products, soundings, SCE and CWOP station reports), `title-and-caveats.csv` (the sheet's own
  front page, with the authors), `timeline-report-phase-one.txt`.
- **Columns** Source Description, Event Date, Event Time (local), Incident Name, Event
  Location/Address, Event Units / Personnel, Event Description.
- **Licence** The reports are CC BY-SA 4.0 on figshare. The sheet carries "© 2025 Underwriters
  Laboratories Inc." and no licence of its own; to be checked with UL before anything derived
  from it is published. Attribute UL Research Institutes, Fire Safety Research Institute.
- **Caveats, theirs** Radio was transcribed by machine and is incomplete; the sheet "should not be
  considered a complete record of events"; names are redacted to asterisks.
- **Use** The report's section 4 holds maps of fire activity for every hour of the first eight
  hours and then for 18:30 to 22:29, 22:30 to 06:29 and 06:30 to 12:29 on the 8th (Figures 4.3.7 to
  4.3.73), the hourly perimeters the incident never mapped; they are figures in a PDF and have to
  be georeferenced to be scored against. The text gives the first hour as the report found it: the
  fire "backing into Temescal Canyon" at 10:33, "just below the old burn scar and pushing directly
  towards Palisades Drive" at 10:49, a quarter mile south along Temescal Ridge Trail by 10:56,
  across Palisades Drive by 11:25, structures burning on Lachman Lane from 11:27, "over a mile
  south towards the ocean" within the first hour. Weather at 10:30 as the report states it: Malibu
  Hills RAWS 47 mph sustained, gusts to 64; SCE Topanga Beach 13 mph sustained, gusts to 34,
  shielded by terrain.

### `nifc/` — the incident's own GIS lines and points

Read by the `obs.record` module (both Palisades manifests), which publishes each feature into the
run at its `CreateDate` and the 3D view draws them; see `devlog/20261103-the-record-on-the-map.md`.

- **Source** NIFC Operational Data Archive 2025, `services3.arcgis.com/T4QMspbfLg3qTGWY/.../Operational_Data_Archive_2025/FeatureServer`,
  layers 4 (Event_Line) and 2 (Event_Point), `IncidentName = 'Palisades'`, paged at 2,000, GeoJSON
  in EPSG:4326, all fields. The perimeters (layer 6) are fetched live by `obs.perimeters` and are
  not copied here.
- **Files** `palisades-event-lines.geojson` (2,334 features), `palisades-event-points.geojson` (825).
- **Content** Lines by category and total length: completed dozer line 554 km, completed hand line
  130 km, road as line 94 km, road repair 69 km, planned dozer line 53 km, fire edge collected in
  the field 7 km. Status: 1,531 approved, 693 proposed, 109 archived. Points: 313 dozer pushes, 208
  stream crossings, 74 cut fences, 43 drop points, 34 division breaks, 30 helispots, 24 branch
  breaks, 18 dip sites, 4 safety zones, 4 incident command posts, 2 spot fires, 2 mobile retardant
  bases, among others.
- **Dates** `LineDateTime` and `PointDateTime` are empty throughout. `CreateDate` runs from
  2025-01-09 19:40Z (lines) and 2025-01-08 20:34Z (points) to early February, so the archive says
  what was built and roughly when it was drawn, not when it held. The first two days have no lines.
- **Licence** US federal interagency data on a public service; no licence text on the service.
  Attribute NIFC and the incident's GIS specialists.

## Not copied, and why

- NOAA NGS emergency response imagery (Maxar 9 to 20 January, NOAA aerial 28 January, tiles to
  zoom 21): catalog asset `noaa-ngs-eri-2025`; tiles are fetched into the cache when a view needs them.
- CAL FIRE DINS inspections and LA County building footprints: catalog assets, fetched by the
  modules, 12,081 and 13,292 features.
- Los Angeles County's after-action reviews (McChrystal Group, 2025 and 2026) at
  `lacounty.gov/aar`: PDFs not yet read; prose, not data.
- The policy institute timeline Anselm sent (`americafirstpolicy.com`) refuses every fetch from here.
- OroraTech's hotspot story: no download offered; the public VIIRS and MODIS detections behind it
  are the thing to fetch when a hotspot scoreboard is built.
