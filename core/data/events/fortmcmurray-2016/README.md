# The Fort McMurray (Horse River) fire, May 2016: the record we hold

Copies of the public record, kept here so the scoring and the prose read one stable set of
files. Fetched from canada1 on 2026-11-04. Nothing here is ours; every file names its publisher
and keeps that publisher's figures as published.

## What the publishers say the fire was

From the English Wikipedia article *2016 Fort McMurray wildfire* (read 2026-11-04; an
encyclopaedia, not a primary record, and to be replaced by the provincial review below as the
source of each figure): first reported on 1 May 2016 by a helicopter crew, in a remote area
southwest of Fort McMurray, Alberta; the infobox gives the point 56°39′55″N 111°23′00″W
(56.66528, −111.38333) and the alternate name Horse River Fire; it swept through the city on
3 May, with the heaviest losses in Waterways, Abasand and Beacon Hill; the whole city was under
mandatory evacuation by 18:49 MDT that day, upwards of 88,000 people; 3,244 structures destroyed
by the infobox, about 2,400 homes and buildings in the city by the text; 589,552 ha in all;
declared under control on 4 or 5 July 2016 (the article gives both) and extinguished on
2 August 2017; no direct deaths, two in a collision during the evacuation. Weather: 32.8 °C and
relative humidity as low as 12% on 3 May, 31.9 °C with gusts to 72 km/h on 4 May, after an
unusually hot, dry air mass, a dry winter and a small snowpack. Cause suspected human, not
determined. The time of detection is not in the article; it is recalled here as about 16:00 MDT
and must be checked.

The provincial review, *A Review of the 2016 Horse River Wildfire: Alberta Agriculture and
Forestry Preparedness and Response* (MNP LLP for Alberta Agriculture and Forestry, March 2017;
the fire is MWF-009 there), is on open.alberta.ca (publication page
`2016-horse-river-fort-mcmurray-wildfire-review`; the June 2017 presentation is at
`open.alberta.ca/dataset/85c2194c-deae-41b6-b2c9-b73fe1604051`). The site answered 520 on
2026-11-04 and the review is not copied yet; its chronology is the record of the first days.

## Folders

### `cwfis/` — satellite hotspots

- **Source** Canadian Wildland Fire Information System, Fire M3 hotspots archive,
  `cwfis.cfs.nrcan.gc.ca/downloads/hotspots/archive/2016_hotspots.zip` (26 MB; 457,428 rows for
  North America in 2016). Natural Resources Canada, Canadian Forest Service, Northern Forestry
  Centre.
- **File** `hotspots.csv`: the rows inside 56.3 to 57.2 N, −112.2 to −110.6 E, 25 April to 30 June
  2016 (17,358 rows), all columns as published: `rep_date` (no zone stated in the metadata; taken as
  UTC), `satellite`, `sensor` (VIIRS-I 9,828; MODIS 3,322; AVHRR 2,625; VIIRS-M 1,581; OLI 2),
  `source`, `lat`, `lon`, `frp`, and the agency's fuel type, weather and fire-weather indices at the
  point (`fuel`, `temp`, `rh`, `ws`, `wd`, `ffmc` … `hfi`), whose units the metadata does not
  describe. The first detections are at 00:23 on 2 May (UTC) near 56.67, −111.45.
- **Licence** Open Government Licence - Canada, with an end-user agreement on the download page
  (`cfs.nrcan.gc.ca/common/cwfis/End_User_Agreement_gen_EN.html`, not yet read). Attribute Natural
  Resources Canada.
- **Use** `obs.hotspots` reads it: each detection marks the ground within its sensor's half-pixel as
  seen burning by then, and the engine's burned cells are scored against that every six hours.

## Not copied, and why

- The National Burned Area Composite (NBAC) for 2016, the final perimeter: the archive offers the
  whole 1972 to 2025 shapefile (`downloads/nbac/NBAC_1972to2025_20260513_shp.zip`); not fetched.
- Buildings: OpenStreetMap through Overpass, fetched by the engine into the cache (`buildings =
  "osm"`), heights from tags where present and two storeys otherwise; no record of which burned.
- The weather: ERA5 at one point through Open-Meteo, fetched by the engine. On 3 May the reanalysis
  reads 30.7 °C, 19% humidity and 4 m/s with gusts to 8 at the point, where the article has 32.8 °C,
  12% and, on the 4th, gusts to 72 km/h: the reanalysis reads this day mild, as it read the Palisades.
