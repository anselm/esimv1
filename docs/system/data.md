<!-- order: 9 -->
# Data

## The registry

Every byte a module reads comes through the registry: a catalog of sources, a fetcher
with a content cache and a lock file, and readers for each format. The catalog
(`data/catalog.toml`) holds, per asset, its URL template, format, units, source,
licence, the date and machine it was last verified from, and notes about its quirks.
The lock file (`data/catalog.lock.toml`, committed) holds the sha256 of every file ever
fetched, so a run's inputs can be checked byte for byte later. The cache
(`data/cache/`, not committed) holds the bytes.

Sources today, all anonymous: Copernicus elevation tiles; USGS gauges; GHCN-Daily
stations; ERA5 and GEFS through Open-Meteo; MODIS vegetation through the Planetary
Computer; Global Carbon Project emissions through Our World in Data; Mauna Loa, methane
and nitrous oxide from NOAA; Law Dome ice cores; HadCRUT5 and GISTEMP; the IPCC AR6
assessed forcings; Natural Earth outlines and capitals; UN population; CRU country
temperatures through the World Bank portal; NOAA's station history and daily station
summaries (GSOD) for the weather station nearest each capital. Each
catalog entry says what was verified and what licence text still has to be fetched and
stored before any public use; [Sources](?doc=system/sources) is that list as a page.

## Sizes and how they are kept in hand

A basin stage warms about half a gigabyte of cache: four elevation tiles, a year of
hourly weather at a dozen points, vegetation composites, gauge and station records. The
planetary layers add about 25 MB of small files. Four habits keep this workable:

- **Fetch once, hash, reuse.** Nothing is downloaded twice; ensembles warm the cache
  before they spawn threads.
- **Resolve to what the question needs.** The basin runs at 30 arc-seconds; the country
  layer at two degrees. Finer is four times the cost per halving and rarely more
  truthful than the record behind it.
- **Mind the other side's limits.** Open-Meteo weights large hourly requests heavily and
  rate-limited the capitals' ten-year hourly fetch after twenty; the wet-bulb layer uses
  NOAA's daily station summaries instead, 1,700 small files from a bucket with no such
  limit. When a source throttles, change the source or the resolution, not the record.
- **Thin the output, not the state.** Modules write map frames on a stated cadence
  (every five years for a 250-year run) while integrating at their own step. Series are
  kept at full resolution because they are small.

## Scoreboards

A record a module reports as a series, beside the model's series of the same quantity:
the gauge's discharge next to the simulated discharge, the satellite's vegetation index
next to the crop's, Mauna Loa next to the carbon pools. A manifest names the pairs to
score and `esim run` scores them. The viewer draws the pair on one chart.

## Provenance of the data layer itself

A reader converts units on the way in and says so in the catalog; a record's known gaps
are in the notes (the vegetation archive has no composites for part of 2025 over the
basin's tile; the weather archive lags five days); provisional records are marked and
the lock pins the bytes a run used so a later revision does not silently change a result.

## What is not here

A general gridded climate record at useful resolution: the obvious file is HDF5 and the
project avoids heavy dependencies, so the country layer uses a per-country record
instead and says the loss. A population layer finer than a country. Humidity. Ocean
heat content, whose server was down the day it was tried. Each is on the list, with the
URL that answered.

## Where data lives later

The cache and the run folders are immutable blobs with content hashes and belong in
object storage; the catalog and the lock file belong in git. Which data sources exist, at
what licence, for what questions, is the kind of enumeration that belongs on a public
wiki rather than in one project's catalog, and the catalog is written so it can be
lifted there.
