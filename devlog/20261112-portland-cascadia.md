# Portland and the Cascadia earthquake: the year after

2026-11-12. Anselm: "Separately do some research and spin up a new scenario for this:
https://www.reddit.com/r/Portland/comments/1xug22l/a_big_one_nightmare_scenario_drives_portland_to/ .
What's the best way to model this? What do we want to show to people?" The thread is on the
Oregonian's story of that morning, *A Big One nightmare scenario drives Portland to cut fuel
storage along the Willamette*, the day of the council's second reading of Ordinance 2026-312.
Reddit refused every fetch route (the site, old.reddit, mirrors, a reader proxy); the RSS feed
of the thread answered, with 28 comments. The Oregonian refused too and the Wayback Machine had
no copy. Session: the esim room (Claude Fable 5.1), this folder; scratch under the session's
scratchpad `cascadia/`.

## The research

What I found, in the order it mattered:

- **DOGAMI O-18-02** (2018), the three-county M9.0 impact analysis, is the record. Its two GIS
  bundles (3.2 GB ground motion and ground failure; 70 MB impact analysis with 691,912 footprints,
  876 neighbourhood units with day and night occupants, 47 jurisdictions, 238 emergency route
  segments with a deformation class, 18,098 power structures with a deformation chance, and the
  Hazus loss tables per unit and per jurisdiction, wet and dry) are public downloads. Portland on a
  wet afternoon: 896 dead, 3,654 to hospital, 30,627 needing shelter. The three counties: 1,473 dead
  wet day, 942 dry day, 380 wet night, 169 dry night; displaced 85,211 wet, 16,852 dry.
- **The USGS median ensemble ShakeMap** (cszm9ensemble_se, Wirth et al. 2021) at 0.02 degree gives
  MMI 7.2, PGA 0.24 g at downtown; DOGAMI's site motion over the window is 0.15 to 0.32 g. The USGS
  ground-failure product (500 m) reads 0.002 at the fuel hub where DOGAMI maps 29 percent, so
  DOGAMI's layers are the ones to use.
- **The Oregon Resilience Plan** (2013) restoration table for the valley: power 1 to 3 months,
  police and fire 2 to 4, water and sewer 1 month to 1 year, highways partial 6 to 12, hospitals 18
  months. Fatalities 1,250 to more than 10,000 statewide; more than 30 billion dollars. Retail
  stations hold three to five days of fuel.
- **The CEI Hub Seismic Risk Analysis** (2022): 415 active tanks, 91 percent pre-1993 or unknown,
  350.6 million gallons of capacity (332.7 million in its Table 2), 233.5 million of contents at
  the 67 percent fill it assumes (until the evening this entry said 298.7 million, which is the
  report's capacity of the tanks holding flammable materials; see the end), a
  release of 94.6 to 193.7 million gallons, 57 percent to the ground and 43 percent toward the
  river, by three age-class rules (pre-1993: 50 to 100 percent; 1993 to 2004 and after: up to 10).
- **Ordinance 2026-312**: at least 20 percent of in-service capacity by 1 December 2036 at eleven
  terminals, aviation-only tanks exempt; passed to second reading 11 to 0 on 5 November; second
  reading 12 November at 2:15 pm. The result was not recorded when read.
- **Multnomah County** on its bridges: none of downtown's expected to be usable at once; weeks to
  years before any; the Burnside replacement adopted September 2025, 1.6 to 1.8 billion, 740 million
  in hand, construction date not set.
- Hospitals from OpenStreetMap (13 in the window on the Oregon side). FEMA's Hazus technical manual
  could not be downloaded (bot protection on fema.gov; no Wayback copy), so the fragility and
  casualty numbers are recalled and the fit to DOGAMI's run stands in.

## What to show people

Not the shaking map: the state has one, and a still of it is what every article runs. The thing
nobody draws is the time after, and that is what this engine does: a cursor from the minute of
the rupture to a year later, with the lifelines coming back one by one and a ledger that says
where the fuel went. And the thread's argument as variants: the storage cut, the retrofit, the
earthquake-ready bridge. The page's job is to make the fuel hub debate visible in time.

## The build

Three kinds, in `crates/modules/src/quake.rs` and `lifelines.rs`:

- `quake.ground`: the DOGAMI window rasters (cut with GDAL to `data/events/portland-cascadia/`,
  16 MB; the registry's own GeoTIFF reader reads them; the folder's README says how they were made
  and the catalog carries the five sources) as world fields, frames (PGA, MMI by Wald 1999 from
  PGV, liquefaction chance, deformation), and the `quake.rupture` event at the instant.
- `quake.buildings`: lognormal fragility in PGA, ground failure as complete damage, Hazus-style
  casualty rates, occupants and residents per cell from DOGAMI's units spread by footprint area,
  scored per unit and for Portland's nine districts against DOGAMI's loss tables (`fit.*`).
- `lifelines.recovery`: bridges, hospitals, power, water, the hub, the `fuel` ledger, the story.

The sweep, all against DOGAMI's wet afternoon for Portland (ratios ours over DOGAMI's; r is the
correlation of extensive-or-complete counts across the 876 units):

| fragility_scale | gf_median_cm | gf complete share | complete median g | damage | extensive | complete | deaths | displaced | r |
|---|---|---|---|---|---|---|---|---|---|
| 1.0 | 152 | 0.2 | 1.5 | 0.47 | | | 0.04 | 0.77 | 0.91 |
| 1.4 | 152 | 0.2 | 1.5 | 1.04 | | | 0.11 | 1.62 | 0.88 |
| 1.4 | 25 | 1.0 | 1.5 | 1.28 | | | 0.47 | 2.02 | 0.88 |
| 1.2 | 25 | 1.0 | 1.1 | 0.97 | 1.00 | 0.95 | 1.08* | 0.78 | 0.90 |
| 1.3 | 40 | 1.0 | 1.1 | 1.04 | 1.23 | 0.93 | 1.04* | 0.83 | 0.89 |

(* with collapse 0.13 and casualty scale 2, before collapse was confined to shaking damage.)

Two things the sweep taught. DOGAMI's wet case has more complete than extensive buildings
(9,976 to 5,593 in Portland) where the dry case has fewer (2,883 to 5,961): the liquefaction layer
carries buildings straight to complete, so `gf_complete_share` had to be 1.0, not Hazus's 0.2 for
lateral spread as recalled. And once collapse (and its casualty rates) is confined to shaking
damage, as Hazus does, the wet and dry cases give nearly the same deaths here (765 and 746 in the
window) where DOGAMI's differ by half; a second sweep over the collapse share and the casualty
scale settled on 0.25 and 3.0, which gives deaths 0.69 by day, 1.74 by night, 0.98 dry, and
injuries 1.37. The night excess is the occupants: ours sit in every building by floor area, DOGAMI's
sleep in homes. Both gaps are on the page.

The fuel ledger closed only once the stocks were re-declared at every close (the kernel compares
declared stock against flow-derived balance; a stock declared once at the start leaks by every
transfer after). Resupply is capped at what the tanks held, so the hub cannot fill past itself.

## Results (seed 1, saturated soils, 2 PM)

Within minutes 150 million gallons leave the tanks, 64 million toward the river (the three seeds:
134 to 171), and 84 million of the 233 stay. Twelve crossings closed; generators dry on day 3; fuel from the pipeline on day 14;
power for half on day 16; trucks on day 20; the Hawthorne back on day 25; power for all on day 32;
water for half on day 150 and for all on day 300; nine crossings open after a year. Cut-20 puts
52 million in the river; retrofit 13 million; Burnside-ready changes the hospitals' refuelling and
nothing else. 765 dead in the window (555 in Portland against DOGAMI's 810 for the nine
districts), 5,933 to hospital, 42,925 displaced; 19,769 buildings extensively damaged or worse.

## Next

- A 3D scene: the inventory has no heights; OSM buildings in central Portland (16,328 ways in a
  0.04 by 0.08 degree box answered Overpass) could carry the view, DOGAMI the counts.
- Structure types: DOGAMI's report says its database carries occupancy class and building type per
  footprint; the public bundle has only a category. The three-county database may be requestable.
- The water network and the power network as networks: the gdb has the poles with their
  deformation chances and the routes with their classes, both unused beyond a count.
- Resupply and deliveries as the real question: the fuel ledger is where the thread's argument
  lives, and every number on its demand side is a guess.
- The USGS grid as a second ground (a reader for grid.xml), so a reader can see the two models'
  disagreement on the map.
- Fire following earthquake, and oil on the river as a flow toward the Columbia.

## The article, read (evening)

Anselm pasted the Oregonian's text. Gosia Wozniacka, 12 November 2026, 7:00 am. What it adds to
the council document: the cut is of each terminal's tank capacity, not of the fuel held, and the
story quotes the county's 2022 report for the difference (333 million gallons of capacity, 234
million held, two thirds full); utility storage (NW Natural's LNG tank) and sustainable aviation
fuel are exempt, and jet-fuel storage may grow by 5 percent of a terminal's existing storage; the
city has barred new fossil-fuel storage at the hub since 2016; the state's energy department
modelled a 70 percent fall in fuel use by 2050 and says the transition may be slower than the city
assumes; the hub handles more than 90 percent of Oregon's petroleum, biodiesel, LNG and other
fuels along six miles of U.S. 30 from the Fremont Bridge to the south tip of Sauvie Island; the
Olympic Pipeline leak of 2025 shut the hub's supply for more than a week and the governor declared
a fuel emergency; DEQ's fuel tank seismic stability programme is the state's named tool; the city
administrator must recommend an alternative by June 2027 and the ordinance starts collecting fuel
inventories by type. Novick: "unless demand goes down, the state's not going to let us implement
the drawdown." Mandell: "a stopgap ... an important marker."

The 234 million caught a misread. `hub.toml` had `contents_gal = 298.7 million`, taken from the
report's sentence "the total capacity of tanks with flammable materials is 298.7 million gallons";
the report's contents figure is Table 3, 233.5 million, the 67 percent fill of the 332.7 million
of active capacity in Table 2. Corrected; the release range is in gallons, so the spill is the
same (150 million in the base run, 64 toward the river; 134 to 171 across the seeds; cut-20 52 to
the river, retrofit 10) and the hub now ends the first afternoon with 84 million instead of 149.
The release fractions the module derives are now 41 to 83 percent of the contents, which is
closer to the report's own rule for pre-1993 tanks (50 to 100 percent) than the 32 to 65 it had.
All eight runs remade. The README's *Why now* now carries the article's substance and the point
that the model's cut-20 variant, which takes a fifth off the fuel held, is a stronger reading of
the ordinance than the capacity cut that passed.
