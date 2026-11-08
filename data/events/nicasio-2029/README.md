# A fire at Devil's Gulch, 1 October 2029: what the scenario rests on

This folder holds no record, because the event has not happened. It holds the list of what the
scenario borrows and where each piece came from, so the page and the runs can be read against it.
Written 2026-11-08 from canada1.

## The ask

Anselm, 2026-11-08: "Lets add a brand new simulation of 550 Nicasio Valley Rd California - this is
the home of Jim Fourniers Planetwork / Solistice Grove. And lets pretend there is a fire starting
not far away at Devil's Gulch Campsite - hmm, lets say oct 1st - and lets put this in 2029. we
probably need some wind conditions." He also asked for the three-dimensional view with the vertical
stretched at least twice.

## Positions

- **Devil's Gulch campsite**: OpenStreetMap relation 17685328, "Devil's Gulch Campsite",
  `tourism=camp_site`, centre 38.0316, -122.7328 (Overpass, 2026-11-08). The group camps and the
  horse camp of the same gulch lie within a few hundred metres (ways 1290158920, 1290158923,
  1290158924). The campsite is in Samuel P. Taylor State Park, Marin County.
- **550 Nicasio Valley Road**: the US Census Bureau geocoder (`geocoding.geo.census.gov`,
  benchmark Public_AR_Current) matched "550 NICASIO VALLEY RD, NICASIO, CA, 94946" at
  -122.667472, 38.028286, interpolated along TIGER line 192759224, left side. OpenStreetMap carries
  no house number 550 on the road (Overpass, 2026-11-08) and no web search found the address in
  connection with Planetwork or Solstice Grove; the description of the place is Anselm's. The parcel
  boundary and the buildings on it are not checked against the county assessor.

## The wind

The year has no weather record. The run borrows NOAA's High-Resolution Rapid Refresh 3 km analysis
(`noaa-hrrr` in the catalog) for 07Z on 27 October 2019 to 07Z on 30 October 2019, played hour for
hour from 07Z on 1 October 2029, so midnight Pacific daylight time falls on midnight. The days were
chosen because the National Weather Service's Bay Area office (MTR) had a Red Flag Warning in force
for 03Z on 27 October to 18Z on 28 October 2019 covering, among other zones, "Coastal North Bay
Including Point Reyes National Seashore" and "North Bay Mountains", which hold Marin County: the Iowa
Environmental Mesonet's VTEC archive, `mesonet.agron.iastate.edu/json/vtec_events.py?wfo=MTR&year=2019&phenomena=FW&significance=W`,
event 6 (read 2026-11-08). Marin County's own release of 25 October 2019 is cited in search
results as warning of "extremely strong north to northeast winds, with gusts up to 80 mph possible
across the highest peaks"; the county's page has since moved and the text was not read directly.
What the analysis actually puts over the window is in each run's `fire.wind_m_s` and
`fire.wind_from_deg` series, and on the page.

## The ground

Copernicus GLO-30 elevation, ESA WorldCover 2021, OpenStreetMap buildings and major roads through
Overpass: the engine fetches all of it into the cache under the catalog's entries. The buildings
are as OpenStreetMap had them on the day of the first run; no record says which exist in 2029.

## Not here, and why

- No perimeter, hotspot or inspection record: nothing has burned.
- No fuel moisture record for 2029: the engine's dead fuel moisture comes from the borrowed hour's
  temperature and humidity; the live fuel moisture is the Palisades guess for January and is
  declared as such on the run page.
