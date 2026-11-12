# Portland and the Cascadia earthquake: the record

What a run of the Portland Cascadia scenario reads, and where each piece came from. There is no
observation of this earthquake; everything here is a published estimate, and the files say whose.
Assembled 2026-11-12 by the esim session (Claude Fable 5.1) at Anselm's ask, from the r/Portland
thread of that morning on the Oregonian's story "A Big One nightmare scenario drives Portland to
cut fuel storage along the Willamette" (Gosia Wozniacka; the site refused this machine, Anselm pasted
the text that evening; the thread's comments came through Reddit's RSS feed). The story caught a
misread in `hub.toml`: the report's 298.7 million gallons is the capacity of the tanks holding
flammable materials, and its contents figure is 233.5 million, the 67 percent fill of 332.7 million of
active capacity, which is what the story quotes. Corrected that evening and every run remade.

## The window

Latitude 45.43 to 45.66, longitude -122.85 to -122.47, one arc-second (1,368 by 828 cells). The
state line is inside it; everything north of the Columbia is empty in the rasters.

## Ground motion and ground failure (DOGAMI)

Oregon Department of Geology and Mineral Industries, Open-File Report O-18-02, *Earthquake
regional impact analysis for Clackamas, Multnomah, and Washington counties, Oregon* (Bauer,
Burns and Madin, March 2018), https://pubs.oregon.gov/dogami/ofr/p-O-18-02.htm. The GIS
bundles (3.2 GB ground motion and ground failure; 70 MB impact analysis) were downloaded on
2026-11-12 and the window cut from them with GDAL 3.8 (`gdalwarp`, average resampling to one
arc-second in WGS84). GDAL is a preparation tool here, not a dependency: the run reads the cut
files with the registry's own GeoTIFF reader.

- `CSZ_M9p0_pga_site.tif`: peak ground acceleration at the soil surface, as a fraction of g (the
  metadata says percent; the values are 0.15 to 0.32 and the report's maps agree with fractions).
- `CSZ_M9p0_pgv_site.tif`: peak ground velocity, cm/s.
- `CSZ_M9p0_Prob_liquefaction_wet.tif`, `CSZ_M9p0_PGD_liquefaction_wet.tif`: probability (percent)
  and amount (cm) of lateral spread from liquefaction, saturated soils. DOGAMI did not publish a
  dry-season liquefaction layer: "liquefaction is much less likely" dry.
- `CSZ_M9p0_Prob_landslide_{wet,dry}.tif`, `CSZ_M9p0_PGD_landslide_{wet,dry}.tif`: earthquake-induced
  landslide probability (percent) and deformation (cm).

The bedrock motions behind these are synthetic (Arthur Frankel, USGS, written communication 2012
and 2016, at 0.01 degree); DOGAMI's site response and Hazus ground-failure methods give the rest.

## Buildings and people (DOGAMI)

From the same report's impact-analysis bundle (`RDPO_Earthquake_Impact_Analysis_Phase1.gdb`):

- `buildings_count.tif`, `building_area_m2.tif`: the 691,912 footprints of the three counties
  (297,120 inside the window, category "Building" only), counted and summed per cell at their
  centroids. The footprints carry no height.
- `people_resident.tif`, `people_day.tif`, `people_night.tif`: each neighbourhood unit's permanent
  residents, daytime and nighttime occupants (the report's own figures, which Hazus used) spread
  over the unit's buildings in proportion to footprint area. The spreading is ours.
- `nuid.tif`, `units.csv`: the 876 neighbourhood units and their totals.
- `jurisdiction.tif`, `jurisdictions.csv`: the 47 jurisdictions; Portland is nine district
  coalitions (indexes 25 to 33).
- `loss_unit_{wet,dry}.csv`, `loss_jurisdiction_{wet,dry}.csv`: DOGAMI's Hazus results for the
  M9.0 scenario per unit and per jurisdiction: building loss, displaced population, casualties by
  level (1 minor, 2 hospital, 3 life-threatening, 4 deaths) for day ("2 PM") and night ("2 AM"),
  and buildings by damage state. This is the record a run is scored against.
- `routes_{wet,dry}_class.tif`: the 238 emergency transportation route segments by expected ground
  deformation class (0 under 50 cm, 1 50 to 100, 2 100 to 200, 3 over 200).
- `transmission.csv`: the 6,146 power poles and towers in the window with their deformation (cm)
  and probability (percent) in each scenario.

## The USGS ensemble

USGS scenario `cszm9ensemble_se`, *M9.0 Cascadia (geometric mean)*, the median of thirty rupture
scenarios (Wirth and others 2021), https://earthquake.usgs.gov/scenarios/catalog/cszm9/. The
25 MB `grid.xml` (0.02 degree) was read on 2026-11-12; at the point nearest downtown Portland it
gives MMI 7.2, PGA 0.24 g, PGV 31 cm/s. Its ground-failure product (Zhu 2017 liquefaction, Jessee
2018 landslide, 500 m) reads 0.002 at the fuel hub, far below DOGAMI's local mapping; the run
uses DOGAMI. Not copied here; the catalog holds the URL.

## The fuel hub

*CEI Hub Seismic Risk Analysis*, ECONorthwest, Salus Resilience and Enduring Econometrics for the
Multnomah County Office of Sustainability and the Portland Bureau of Emergency Management,
released 8 February 2022, https://www.multco.us/info/cei-hub-seismic-risk-analysis (full report
PDF read 2026-11-12). `hub.toml` carries its numbers. Portland Ordinance 2026-312 (Code Chapter
17.112), https://www.portland.gov/council/documents/ordinance/title-17-bulk-fuel-terminal-capacity-reduction-regulations,
read the same day.

## Restoration times

*The Oregon Resilience Plan*, Oregon Seismic Safety Policy Advisory Commission, February 2013,
https://www.oregon.gov/oem/Documents/Oregon_Resilience_Plan_Final.pdf (read 2026-11-12).
`restoration.toml` carries the executive summary's table.

## Bridges and hospitals

`bridges.toml`: Wikipedia's list of Willamette crossings for positions and years; Multnomah
County's Earthquake Ready Burnside Bridge pages for what survives. `hospitals.toml`:
OpenStreetMap through Overpass, 2026-11-12.

## What is not here

Water and sewer pipe damage (the Portland Water Bureau's 2019 estimate of 1,500 to 3,000 breaks
reached us only through a search summary), the tsunami (not Portland's), aftershocks, fire
following the earthquake, the Columbia River bridges, and any Hazus fragility or casualty table:
FEMA's technical manual was not downloadable from this machine, so the module's fragility and
casualty rates are recalled and scaled to DOGAMI's totals, which the module says.

## The components (2026-11-12, night)

- `tanks.csv`: the county study's Appendix B per-tank table, parsed from the report's PDF text: area,
  property, tank id, the study's coordinates where given (228 of 621), contents, capacity, expected
  fill at 67 percent, year, type, tank group, flammability, hazard, the study's zone (on land,
  potentially in water, material in water), tank failure or not, and the study's own percent and
  volume lost, minimum and maximum. Unlocated tanks are placed on the nearest free OpenStreetMap
  storage-tank footprint (Overpass, 352 along the hub) to the property's located tanks or named
  area, else at the area's centre; the `placed` column says which. Commas inside fields became
  semicolons.
- `poles.csv`: DOGAMI O-18-02's Electrical_Transmission_Structures inside the window, cut with
  ogr2ogr to EPSG:4326: the pole id, the state's deformation (cm) and chance of failure (percent),
  wet and dry; a `corridor` column from chaining poles within 450 m (one component holds 6,088 of
  6,146, so it is not used).
- `routes.geojson` and `routes.toml`: the Emergency_Transportation_Routes inside the window (137
  of 238 segment polygons, simplified), the class per segment (the state's potential ground
  deformation: 0 under 50 cm, 1 50 to 100, 2 100 to 200, 3 over 200, wet and dry), and in the TOML
  the graph: touching polygons as neighbours, each crossing linking its nearest segment on each
  bank, each hospital and the hub given its nearest segment.
- `fragility.toml`: the curves and repair rules the components module runs, every one recalled from
  Hazus and unchecked; the file says so row by row.
- `aftermath.toml`: the claims the aftermath module applies, with sources (Kishore et al. 2018; HHS
  emPOWER's county layer, read 2026-11-12; the 2020 county populations recalled).

