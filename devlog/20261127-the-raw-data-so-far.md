# The raw data so far, sorted by how it is served

27 November 2026. Claude (Fable 5.2), esimv1 room on canada1.

Anselm, on the scenarios each having their own 3D fragment of the world: "How can this simulation foundation make
core data services unified? Typically this is a role for a GIS server - to provide basic data planes." After a first
answer (named planes, one registry function, a prepared bundle per place, one scene component; files in a bucket
instead of a running server): "if we wanted to offer canonical datasets - we can offer them on cloudflare - perhaps
with a public index. Your named plane concept is a good one. Interestingly data sources could be anywhere - even git.
Maybe what would help this conversation is to simply enumerate the kinds of raw data or fuel we are using so far. I
feel like we have to be concrete."

Nothing is decided or built. This is the inventory, read from `core/data/catalog.toml` (46 entries), the cache
(4.2 GB), `core/data/osm`, the record folders and Sausalito's `view/prepare.py`. Licences were not reread for it.

## 1. Global grids a third party already serves as range-readable files

Copernicus DEM (1 arcsec, AWS, one-degree tiles of 20 to 45 MB, ten held); ESA WorldCover 2021 (10 m, three-degree
tiles of 60 to 100 MB, four held); MODIS vegetation index (500 m, Planetary Computer, needs a token per run); NOAA
HRRR wind (3 km hourly, GRIB2 by byte range, United States only); NOAA VIIRS fire detections (NetCDF swaths).
These need only an index entry. WorldPop 2020 (1 km) is one 830 MB file downloaded whole.

## 2. Grids and series that exist only as answers from an API

Open-Meteo: ERA5 hourly weather and wind at a point, ERA5-Ocean waves, the GEFS ensemble. NASA POWER (MERRA-2)
boxes: about 7,550 requests reduced to our own tables. World Bank: CRU country temperature and the development
indicators. USGS river flow. There is no file to point at; the durable copy is whatever we keep.

## 3. Vector features from live query services, whose answers change

OpenStreetMap through Overpass (roads, place names, buildings, bridges, shoreline): nine reduced answers, 9.9 MB,
kept in git. ArcGIS feature services: NIFC fire perimeters, CAL FIRE damage inspections, LA County building
footprints with heights. CWFIS hotspot archive (a zip per year).

## 4. Small global tables, one file each

CO2 emissions (OWID), Mauna Loa, Law Dome, HadCRUT5, GISTEMP, methane, nitrous oxide, AR6 forcings, UN population,
Natural Earth outlines and capitals, GHCN and GSOD station weather. All under 20 MB.

## 5. Place records cut by hand from an archive, kept in git

Portland (17 MB in its repository): DOGAMI's ground motion and building inventory cut from a 3.2 GB geodatabase,
tanks parsed from a county PDF, federal bridge inventory rows, poles, routes, hospitals. Palisades (17 MB): incident
points and lines, satellite hotspots, status reports. Fort McMurray (3.5 MB): hotspots. Half Moon Bay: a table.

## 6. Tables we wrote ourselves

Fuel models, tipping elements, fragility curves, the two future-event lists, calendar, hazards. Mostly recalled.

## 7. Sausalito, outside the engine's registry

Its own pipeline: USGS 3DEP one-metre elevation, NOAA's 2025 coastal lidar (canopy), its own Overpass answer,
Microsoft's building height estimates, the Golden Gate transit feed. 6.5 GB of downloads become 7.7 MB for the view.

## What the list shows

- Elevation has two sources and one obvious rule (finer where it exists). The easiest plane.
- Buildings have four unrelated sources (LA County, OpenStreetMap, DOGAMI, OpenStreetMap with Microsoft heights),
  each with its own fields. The most fragmented plane and the one every 3D view needs.
- Weather has three (HRRR, ERA5 at a point, POWER boxes) at very different grains.
- Group 1 needs an index and no hosting. Groups 2, 3 and 5 are where a canonical copy of our own would earn its
  keep, because the upstream is an API, a moving answer or a large archive. Groups 4 and 6 fit in git.
- Git already serves as a source: the Overpass answers and the record folders.
