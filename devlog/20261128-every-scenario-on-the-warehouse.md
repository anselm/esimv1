# Every scenario on the warehouse: what switched, what moved, what did not

27 to 28 November 2026 (the evening of the 27th in California; products are dated in UTC). Claude (Fable 5.2),
esimv1 room on canada1, session 018TGCFiQHbUYn12HkNti2xk.

Anselm, after the store went public at data.earthlight.cloud: "The terrestrial data warehouse is excellent ... I'm
very happy being able to separate data from higher level products and to surface this to real users (and to
funders! It helps clarify the story)". Then: "Now that we have the data warehouse every single project should
switch to it and we should commit and push - report on how that goes. There is also a Bay Area model that has data
that is outside of esimv1 that should also be migrated. I am going out for a few hours go as far as you can and
report back."

## What switched

- **The engine (esimv1-core 0.4.0)** asks OpenStreetMap and county services for nothing. A manifest names
  warehouse products in a `data` table handed to modules beside `place`. `registry::warehouse` reads them from a
  list of stores and checks each against the checksum its listing states. The engine's Overpass and ArcGIS
  building code and its kept answers (`data/osm`, 9.9 MB) are deleted: about 300 lines out, 270 in.
- **Seven place scenarios** name their products: both Palisades projects, Fort McMurray, both Devil's Gulch
  projects, Half Moon Bay, Portland.
- **Fifteen small global files** (emissions, the CO2, methane and temperature records, forcings, population,
  country temperatures, station list, Natural Earth outlines and capitals) are read from the store through the
  engine's catalogue. They were deposited from the engine's cache, so they are the bytes the runs already used;
  all fifteen were checked against the hash list after the push. This covers Earth 2100 and the two smaller
  planet projects for those files.
- **The Sausalito model** (`anselm/esimv1-sausalito-model`) reads its buildings, roads, piers and creeks from the
  store. Its own devlog has the counts.
- **The Bay Area model's window** is a new place in the store, `san-francisco-bay`, built from the model's own
  sources: San Francisco's footprints with lidar heights (177,151) and OpenStreetMap for the rest (316,592
  buildings in the massing, 162,744 on the city's measurement), with roads, names, shore and water.

The store went from 28 products to 61 (172 MB).

## What it did to the runs

Every run's log digest and final series were recorded before the switch and compared after. Of 83 runs:

| Project | Runs | After the switch |
|---|---|---|
| Portland (Cascadia) | 10 | identical |
| Palisades, Rothermel | 4 | identical |
| Devil's Gulch, Rothermel | 4 | identical |
| Half Moon Bay | 6 | identical |
| Earth 2100, the two planet projects, the Boone projects, two buckets | 31 | not rerun; inputs are the same bytes (the regression ladder shows no drift) |
| Palisades, cell fire | 20 | all moved |
| Fort McMurray | 4 | all moved |
| Devil's Gulch, cell fire | 4 | all moved, slightly |

The identical ones take only map labels from the products, or (Half Moon Bay) a shoreline that has not been
edited. The three that moved read roads and buildings as ground:

- **Roads are a newer map.** The engine's old road answers were fetched between 3 and 14 November, most of them
  from the kumi.systems Overpass mirror. On 28 November that mirror was serving the map of 31 May 2026. Whether
  it was already behind when the old answers were fetched is not known. In the Palisades window the old answer
  has 159 motorway and 242 trunk pieces and the new has 137 and 173, with more primary; a road's class sets the
  width of the break it makes. Fort McMurray gained 54 secondary and tertiary pieces.
- **Building heights are estimates now, not one number.** Where the map states no height the engine used to
  assume two storeys (6.4 m) for every building: 21,373 of Fort McMurray's 21,380. The massing product gives
  Microsoft's estimate where there is one and a guess from neighbours or kind otherwise (median 4.3 m there).
- **Outlines are rounded to about a metre** in the products, which moves the share of a 30 m cell under a roof
  slightly.

How far they moved:

- Palisades cell fire: burned area by a few percent. `vector-5` has 16,220 acres at the first infrared
  perimeter where it had 16,525 (mapped 15,824), overlap still 0.79; the fought run ends at 19,500 where it had
  19,000. Small counters (animals caught) moved by much more. The page's findings stand and its numbers were
  reread. The two `esim fit` results on the page were computed on the old inputs and not searched again.
- Fort McMurray: the three faster settings moved a good deal (thresholds times five ends at 141,000 acres where
  it had 118,000; times three at 120,000 where it had 184,000), the times-eight setting hardly at all (98,506
  against 98,425). The page's account, pace right at times eight and shape wrong, reads the same.
- Devil's Gulch: under 0.1 percent in burned area; the house is reached at the same hours.

This is a finding about the engine as much as about the data: a cell fire's result depends on the road map's date
and on a building-height rule nobody had examined.

## Faults found on the way

- The data tool dropped county rows with an empty identifier: 191 of the 13,292 Palisades footprints. Found
  because the engine's old copy had more. Fixed before the product was pushed.
- A script of mine emptied one manifest by opening it for writing before reading it. Restored from git at once.
- OpenStreetMap's main query server failed most requests all evening. The French mirror answered in under a
  second with the day's map and is now asked first.

## Not done

- **Nothing is deployed.** esimv1.exe.xyz still serves the runs made before the switch, and the Sausalito site
  its data of 24 November. Both change at the next deploy, on Anselm's say.
- **Records are still in the scenario repositories**: Portland's 17 MB (ground motion, inventory, tanks, bridges),
  the Palisades' 17 MB, Fort McMurray's hotspots. They are the bulk of what a scenario carries, and moving them
  needs the engine to fetch a record folder by listing, which is not written.
- **Answers from an API are still fetched by the engine**: weather (ERA5, HRRR, POWER), river flow, station
  records, World Bank indicators, perimeters and damage inspections. So the Boone projects did not change.
- **Rasters**: elevation and land cover are still read from their publishers by range. The store only lists them.
- **The Bay Area model itself is unchanged.** Its repository says prepared assets stay in git, and it publishes
  from `main`. Its vegetation, street trees, streams and bathymetry have no warehouse product.
- **Portland's 3D view still shows no buildings.** The massing product exists; the scene does not read it yet.
- **The LA County terms** are still unread, so those footprints stay listed-only and a machine without the
  builder's store must rebuild them to run the Palisades cell fire.
- The store is still served through the esimv1 VM from the private bucket.
