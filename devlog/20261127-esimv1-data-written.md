# esimv1-data, written: a builder and a catalogue for the data under the scenarios

27 November 2026. Claude (Fable 5.2), esimv1 room on canada1. Follows `20261127-a-data-service-beside-the-simulator.md`.

Anselm: "Please write this service - then we will incrementally migrate scenarios to use it - hopefully reducing the
size of scenario code and data collections." He asked whether it needs Python and where it would be stood up, and
said of the licence caveats that kept coming up: "absolutely everything I do ever at all is always totally open
source, open data, open participation ... I do support making money but not by building moats around foundation data
or methods." While it was being written he added four things, quoted in the new repository's
`devlog/20261127-origin.md`: rich metadata and durable naming thought through before building; a question about tiles
against whole regions; buildings with good height estimates; and documentation of the purpose.

## What exists

Repository `anselm/esimv1-data`, public, cloned here as `dataservice/` (in `parts.list`; ignored by this
repository's git). Rust, no Python, no GDAL. It does not depend on the core and the core does not read from it yet.

- **Two doors.** Reading is static files in a store (a folder, a web address, an S3-compatible bucket). Building is
  the command `esim-data build`, driven by `products.toml`. Nothing listens, so nobody can make it work by calling it.
- **The catalogue is static STAC 1.1**, which answered nearly every item of his metadata list with an existing field
  (the table is in `dataservice/docs/design.md`). Every file passes `stac-validator`.
- **Whole regions for named places**, not tiles. My first draft cut tiles; his doubt was right for the users this
  has, and the reasons are in the design page. Address: `<plane>/<source>/<place>/<date>/<file>`.
- **Buildings**: four sources in one schema (OpenStreetMap, Microsoft's machine-drawn outlines with height estimates,
  a city's or county's ArcGIS layer, and `massing`, which gives every outline a height and marks where it came from).
  Portland publishes its own footprints with measured heights, storeys and year built, read from the city's service
  that day; they are the base for Portland's massing.
- A plan-view picture is made for every vector product, and `examples/massing.html` draws a product as blocks.

## What was looked at

Portland: 354,413 outlines from the city's inventory, 308,056 with a height the city measured, the rest from storeys,
Microsoft's estimates or a guess. Downtown drawn as blocks shows the towers, the river and the bridges' gaps
(`dataservice/docs/img/portland-massing.png`). Sausalito: 6,592 OpenStreetMap outlines, 5,595 with Microsoft's
estimate. The catalogue page was viewed in light at desktop width and dark at phone width.

## Answers to his questions

- Python: no.
- Where it stands: the standing part is a bucket of files. The builder runs wherever someone holds the bucket's
  write key (canada1 by hand today). To be read by browsers the bucket needs a public web address, which is his to
  choose; runs should record a product's path and checksum, not a full address, so that choice can change later.
- Tiles or regions: regions.

## Not done

- No scenario reads from it yet. The first migration candidates: Portland's 3D view (no building heights today),
  Sausalito's `prepare.py`, the engine's `built.footprints` and `roads.major`.
- Nothing is in a public bucket. `push` was tested once against the private R2 bucket with one small table, under the
  prefix `esimv1-data-test/` (four small objects left there).
- No rasters, no lidar heights, no partial-read vector format. The design page lists these.
- Overpass refused most requests that afternoon, so the OpenStreetMap products for some places were still to build
  when this was written; the fetch resumes from its cache.
