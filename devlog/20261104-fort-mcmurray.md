# 2026-11-04 — Fort McMurray: the engine in another country

Anselm's fourth chore of the afternoon: "add a scenario for the 2016 Fort McMurray wildfire - as
best we can. this will be substantial work but it gives us a way to test how portable our firemodel
is. the climate, vegetation and circumstances in Fort McMurray are different and may force us to
generalize our mode. lets only focus on the 3d engine version."

## What was found and fetched

- The fire, from the English Wikipedia article (read; a secondary source, used until the primary
  is in hand): found 1 May 2016 southwest of Fort McMurray, the infobox point 56.66528, −111.38333,
  into the city on 3 May, 589,552 ha, 2,400 buildings in the city, 88,000 evacuated, under control
  in July. The time of detection is not in the article; I recall about 16:00 local and say so where
  it is used. The provincial review (MNP LLP for Alberta Agriculture and Forestry, March 2017,
  wildfire MWF-009) is on open.alberta.ca, which answered 520 all afternoon. A web search gave no
  clock time either.
- The record: CWFIS's Fire M3 hotspot archive, one zip a year, open; 2016 is 26 MB and 457,428
  detections for North America, 17,358 in a box round the fire, copied into
  `data/events/fortmcmurray-2016/cwfis/hotspots.csv` with the publisher's columns (the archive
  also carries the agency's fuel type and fire weather at each point, which could check the wind).
  The first detection is at 00:23 UTC on 2 May, 4 km west of the infobox point. The National Burned
  Area Composite (final perimeters) is on the same server as one shapefile for 1972 to 2025; not
  fetched.
- Buildings: no county service; OpenStreetMap through Overpass, 21,380 ways with a building tag
  in the window. Overpass's kumi mirror answers 504 to anything large; the main server answers.
- Wind: HRRR stops at the border. ERA5 through Open-Meteo at the window's centre: on 3 May it reads
  30.7 °C, 19% humidity, 4 m/s and gusts to 8; the article has 32.8 °C, 12% and, on the 4th, gusts
  to 72 km/h.

## What had to generalise

- `fire.cells` takes `buildings = "osm"` (`registry::wildfire::osm_buildings`): heights from the
  `height` tag, else `building:levels` × 3.2 m, else two storeys, converted to feet so the frame and
  the viewer, which take the county's heights as feet, are unchanged. `wind = "era5"` existed.
- A new scoreboard, `obs.hotspots` (`crates/modules/src/hotspots.rs`): a CSV of detections, each
  marking the ground within its sensor's half-pixel (VIIRS-I 187 m, VIIRS-M 375, MODIS and AVHRR
  500, OLI 15; recalled nadir values) as seen burning by then; every six hours the engine's burned
  cells are scored against the seen area with the perimeter scorer's measures and series names, so a
  README table reads either. The frame `obs_hotspots` drapes on the 3D view as the perimeter did.
- The Overpass context query drops the admin-boundary relation for a window above 0.1 square
  degrees: the Wood Buffalo municipality is 63,000 km² and `out geom` on it never answers.
- The elevation sampler. GLO-30 is 1.5 arc-seconds wide in longitude above 50°N, and `dem_window`
  scattered pixel centres into cells, so at one arc-second one cell in three took no pixel and came
  out NaN: 778,320 of 2,336,041 cells, the holes in the ground again, and the first four runs ran on
  cliffs and were thrown away. Cells no pixel centre reaches now take the nearest pixel at their own
  centre (`terrain-v4`). The regression ladder reports no drift: the Palisades and the Boone have
  one-arc-second pixels and never had an empty cell.

## What the runs say

Acres burned against acres seen from orbit, and the overlap, at 06:00 UTC on the 2nd, 3rd, 4th
and 5th of May and at the end (07 May 00:00):

| variant | 2 May | 3 May | 4 May | 5 May | end |
|---|---|---|---|---|---|
| seen from orbit | 2,852 | 7,897 | 29,160 | 45,868 | 89,468 |
| vector-3 | 10,527 · 0.24 | 87,810 · 0.15 | 184,426 · 0.14 | 184,426 · 0.23 | 184,426 · 0.46 |
| vector-5 (the Palisades' setting) | 4,404 · 0.28 | 27,875 · 0.36 | 66,326 · 0.23 | 109,689 · 0.25 | 118,321 · 0.43 |
| vector-8 | 1,957 · 0.33 | 15,355 · 0.37 | 28,645 · 0.34 | 50,401 · 0.35 | 98,425 · 0.43 |

The engine ports: it runs on a boreal window with other buildings, another wind and another
record, and at thresholds times eight its pace is the record's from the third day. The shape is
not: the overlap never passes two fifths, and the pictures say why. The reanalysis point reads a
light south-easterly on the morning of 3 May; the model's fire spreads as a disc from the ignition
and grows into the forest south and west, and never crosses into Abasand and Beacon Hill, where the
record has the fire and 2,400 buildings lost. The Palisades taught that the reanalysis cannot give
a Santa Ana day and the 3 km analysis can; here there is no 3 km analysis, and the day that
mattered is the day the model misses.

What the second fire says about the engine, then, is that the one fitted constant is doing the
work of a fuel model (a spruce stand and a chaparral slope should not share it, and do not), and
that the wind is the input that decides the shape in both countries. Next for this fire: the
review's chronology, a wind from the day's stations (the hotspot archive carries the agency's
station wind at each point, which is a check we already hold), a boreal fuel kind, the NBAC
perimeter, and the lost buildings as a record if one is public.

Session: Claude Fable 5.1, the esimv1 room on canada1, 2026-11-04 afternoon.
