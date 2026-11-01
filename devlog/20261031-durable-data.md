# 2026-10-31 — A durable data repository

Anselm, same day: "how about data? How do we have a durable repository - for example a
durable collection of base layers or even known future events or chaotic events?" Agent's
assessment. His June 2026 note already wanted to "accumulate geo intelligence ... a public
resource later"; orbital-sim did a first version (an orbital-data GitHub repo holding an
18 MB GEBCO raster, fetched on first run and checked by sha256).

## Four kinds of data, four homes

1. Base layers. Elevation, land cover, soils, hydrography, boundaries, parcels, roads,
   reanalysis weather. Large, read-mostly, external. Keep them in cloud-native formats that
   can be read by range request without downloading the whole file: Cloud Optimized GeoTIFF
   for rasters, Zarr for multidimensional arrays (weather, ocean), GeoParquet or FlatGeobuf
   for vectors. Bytes live in object storage, not in git. Cloudflare R2 is the obvious home
   given the existing Cloudflare account (S3-compatible, no egress charge; the agent is
   confident of both). Each box keeps a local cache. A STAC catalog (the geospatial
   asset-catalog standard) describes every asset: extent, time, resolution, source URL,
   retrieval date, licence, sha256.
2. Event tables: the calendar and the hazards. Small, curated, the most valuable data we
   will own, and the one thing no one else publishes in this form. These belong in git as
   plain text, one row per event, in a repo of their own so they outlive any engine.
   Columns from the brief: event, consequence, module addressed, source, confidence band,
   date or rate. Add one: known_since, the date this fact entered the record. A hindcast from
   1990 filters to rows known by 1990 and gets honest 1990 knowledge for free. Hazard rows
   also name the state they depend on (conflict rate reads food price) so a module can
   modulate them.
3. Observations: the scoreboards. Time series with provenance (USGS loads, cruise areas,
   MODIS indices, prices, landings). Append-only Parquet in object storage, catalogued like
   base layers, with each series carrying source, units, method and retrieval date.
4. Derived data. Terrain derivatives, downsampled rasters, routed networks, synthetic
   populations. A cache, content-addressed by the hash of (inputs, recipe, code version),
   always rebuildable and therefore deletable. Never a source of truth.

## Rules that make it durable

- Immutable versions, content addressed. An asset is its hash; a new retrieval is a new
  asset. The manifest pins hashes, so a run from 2026 reproduces in 2031 against the same
  bytes.
- Catalog in git, bytes in object storage, cache on every box. Git is small and reviewable;
  object storage is cheap and dumb; the box is replaceable, as house policy already says.
- Provenance on every asset and every row: source, retrieved when, by whom or by which
  agent, under what licence. Licences decide what a paid service may use; some public data
  is non-commercial, and that has to be visible before it is baked into a product.
- Machines propose, humans promote, for the event tables above all. Agents reading news,
  papers and registries draft rows (the ontology-scraper project is already a news-reading
  entity extractor); rows enter the canonical table by review. A draft table and a promoted
  table, in the same repo.
- Public by default. Base-layer catalogs and event tables are a public good and match the
  open-source instinct; the paid thing is running ensembles, not owning the data. Anything
  licensed otherwise is marked and kept out of the public catalog.

## Deliverables to add

- D7 Data catalog and fetcher: STAC-shaped catalog in a data repo, a fetcher that resolves a
  hash to a local cached file by range request where possible, licence and provenance
  checks at manifest load. Done: a manifest that names an elevation asset by hash runs on a
  fresh box with nothing predownloaded.
- D8 Event-table repo: calendar and hazards as text, with schema, validation, known_since,
  and a draft/promoted split. Done: the hindcast harness (V2) filters by known_since.
- V1 Observation store becomes the Parquet-plus-catalog described above.
- Storage: an R2 bucket, created when D7 starts, with the catalog repo naming it.

Sizes are left out on purpose; a basin's base layers are gigabytes, global reanalysis is
far larger and is subsetted per case. Measure when D7 lands.
