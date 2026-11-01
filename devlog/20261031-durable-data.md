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

## Addendum, same day: where users' own simulations live

Anselm: "Where would we store ... sims belonging to users?" The brief puts accounts and
access control out of scope, but the shape of the storage is decided now, because it falls
out of determinism.

A user's simulation is a manifest plus a seed. Everything else is derivable. That splits
what to keep into three tiers:
- The manifest and its history. Tiny text, versioned, kept forever. This is the thing the
  user owns. Manifests form a lineage: a user's sim is usually a baseline plus a diff, and
  the diff is what they wrote. Store the chain, not copies.
- Summaries. Distributions, ledgers, scores, the narrated prose. Small, kept with the
  manifest version that produced them.
- Full results. Per-branch event logs and snapshots. Large, content-addressed by the hash of
  (manifest, seed, kernel and module versions), stored as a cache that can expire. Two users
  running the same manifest share one result. A rerun recreates any expired result bitwise,
  which is what the determinism was for.

Users also bring private data: a parcel boundary, their own observations, a local plan. Those
enter the same catalog as any asset, with an owner and visibility, and the manifest pins
them by hash like everything else. Private assets never enter the public catalog.

Where this lives physically: the same object storage with per-owner prefixes for results and
private assets, and a document store for manifests with ownership, visibility, forking and
history. The portfolio already has the pieces: orbital-filespace is a namespaced document
store with permissions and a post verb, and attest is the identity layer by passkey. The
decision whether to use them is deferred with the rest of accounts; what is decided now is
that a sim is a forkable document with a lineage, results are a content-addressed cache, and
private assets are catalog entries with an owner. Nothing in the kernel needs to know who a
user is.

## Addendum, same day: digitaltwins.wiki or a data server

Anselm: "We could maybe store common data on digitaltwins.wiki ... Or have a data server I
guess hmm." Agent's view after reading the wiki's manifest and README
(~/projects/2026/twin-compendium, repo anselm/digitaltwin-wiki, one exe.dev VM, the repo is
the site, every entry a markdown file, machines draft and people promote by pull request).

Both, split by size and by what each is good at.

- The wiki is the right home for the catalog and the event tables, as text. A base layer's
  wiki page is its catalog entry: what it is, source, licence, resolution, how to fetch, the
  hash of the current version. The calendar and hazard tables are exactly the wiki's model:
  public, plain text, sourced, reviewed by pull request, machine-readable exports already
  promised to practitioners. The wiki's own devlog already has a shocks entry (2026-10-25)
  that is the start of the hazards table. Putting the tables there makes them a civic good
  with an editorial process, which is better than a private repo nobody reviews.
- The wiki is the wrong home for bytes. It is one exe.dev VM with the single-VM durability
  the portfolio has already learned about, and a 461 MB repo today; gigabytes of rasters do
  not belong in git or on that disk. Bytes go to object storage (R2), and the wiki page
  points at them by hash.
- A data server is not needed at first. Cloud-native formats are read by range request
  straight from object storage, so the "server" is a bucket plus a fetcher library. Build a
  real server only if a need appears that a bucket cannot meet: tile serving for the viewer,
  authenticated private assets, or a query interface over observations. Those are later and
  each is a small service in front of the same bucket.

So: catalog and tables in the wiki repo, bytes in R2, fetcher in the kernel, no server until
proven necessary. This adds a dependency from this project onto the wiki's schema, which is
fine because both are his and the wiki wanted machine-readable exports anyway. D7 and D8
above are revised to target the wiki repo for the text half.
