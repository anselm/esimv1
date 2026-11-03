# 2026-11-03 — The fire you can watch, the ground it stands on, and an engine of our own

Anselm, over the evening and from an airport: six points on the fire ("when i run the fire,
the top image shows basically an 'explosion' ... am i seeing what actually occurred?"; "i
need like county overlay political boundaries and or roads"; "how about actual buildings -
can we show the actual structures burned"; "the relationships seem minimalistic?"), then
"Our fire model feels like some kind of weird third party tool that doesn't consider real
factors? - we may have to write our own. Can we bake our own? we have real elevation data,
real tree cover, real buildings, breaks, roads, volumetric weather", "we could even speculate
about firefighters", "the point here is to try to make a model that corresponds to all the
facts we have available - it then becomes a predictive resource", "I can even see
visualizing this in 3d", "let's spend the tokens here - let's try harder to have a simulation
and also a vivid story", and, boarding: "1) evaluate a custom or different fire engine model
2) exercise a 3d view. I land in 2 hours."

Agent: Claude (Fable 5.1), the earth-simulator-1 room on canada1, session
`0bad2b71-90e5-4e88-978e-bd10ad8801f7`; a forked agent found the structure and road doors.
Commits 8b364c0 (the fire you can watch), 7587dda (context and structures) and the engine
commit after this entry; none pushed; the live site is at 4edc83c.

## Answers to the six points

1. The timeline shows the whole run; charts now draw the unplayed part faint rather than
   hiding it. The run is a finished artefact and its future is known; the eye follows the
   cursor. In an operator mode nothing past now would exist, and that mode is not built.
2. The cover was red because I drew it with a warm ramp; the run page used the one blue ramp
   every field got. Fields now choose: arrival hours warm, land cover categorical with a
   legend, the rest blue.
3. The explosion was both. Frames were six hours apart, so a fire that filled the window in
   twelve hours showed as two steps; frames are hourly now, on a 3-arcsecond frame grid. And
   the model did fill the window in about twelve hours: a free-burning fire with nothing to
   stop it does that, which is the finding, not a display fault.
4. Roads, place names and city limits from OpenStreetMap, fetched once per window through the
   Overpass mirror (overpass-api.de refused), reduced and clipped in the cache, carried in the
   fire module's describe (read from the cache, never fetched at describe time) and drawn over
   every map of the run.
5. CAL FIRE's damage inspections: 12,081 structures as points with the damage recorded, three
   anonymous services; the public view has damage and type, the data-share view the whole
   form, and LA County's footprints carry the same damage joined to 13,292 polygons with
   heights. `obs.dins` bins the points and, each hour, counts them against the fire: the
   Rothermel reanalysis run reaches 6,789 of 6,845 destroyed structures and overruns 4,232
   undamaged ones (recall 0.99, precision 0.57), because it burns everything.
6. The wiring was thin because the first cut was spread plus score, and the wind lived inside
   the fire module. The engine below reads more of the ground; a weather module publishing
   wind fields, a structures module and a suppression module are the shape the wiring should
   take, and the todo says so.

## The engine

`fire.cells`: not Rothermel. A heat budget on one-arc-second cells. Each cell: elevation,
slope and aspect (Copernicus), land cover (WorldCover), the share under buildings and their
mean height (LA County footprints, sub-sampled 3 by 3 per cell), a road mask (OSM highways
buffered by class: motorway 30 m to tertiary 8 m), and the hour's wind, gust, temperature
and humidity from NOAA's HRRR 3 km analysis, bilinear from the block over the window. A
burning cell radiates intensity I into every unburned cell within `reach_m`, as
I·exp(−d/λ) with λ longer downwind (1 + wind_gain·U·cos) and uphill (1 + slope_gain·tan s);
the receiving cell's heat decays with `cool_min` and it ignites when the heat passes
`ignite_scale` × the fuel kind's threshold × (1 + moisture_gain·M), M the dead fuel
moisture by Simard from the hour's humidity. Grass, shrub, tree and built cells have their
own threshold, intensity and burn time, built ones scaled by the built fraction. Burning
cells in wind over 3 m/s throw embers with a rate ∝ (U/10)² and an exponential flight, which
light dry fuel with a stated chance. Thirty-second substeps in an hourly weather step; a
four-day run is about 25 s once the data is cached.

The HRRR door: the open bucket, hourly files of 150 MB with a 9 KB index; the registry
fetches single records by byte range (`Fetcher::fetch_range_reduced`, the raw record's hash
in the lock) and decodes them with its own GRIB2 reader (`grib.rs`: grid template 3.30
Lambert conformal, data templates 5.0 simple and 5.3 complex with second-order spatial
differencing). At the ignition point, 18Z on 7 January: 9.9 m/s from 36° with a 20.6 m/s gust
(bilinear) against GDAL's nearest-pixel 11.3 m/s from 38°, gust 21.4; the reanalysis had 6.9
and 13. The first run fetched 103 hours in nine minutes.

Every constant is a guess and the describe says so for each. One number, `ignite_scale`, is
swept across the project's variants so the engine can be read against the record; nothing
else is adjusted.

| variant | 4 h | 12 h | 27 h | 48 h | 96 h | destroyed reached | undamaged overrun | spots |
|---|---|---|---|---|---|---|---|---|
| scale-2 (reach 90 m) | 30,069 | 78,621 | 78,820 | 78,820 | 78,820 | 6,596 (0.96) | 4,070 | 212 |
| scale-4 | 10,056 | 63,176 | 76,743 | 76,743 | 76,743 | 6,216 (0.91) | 3,945 | 514 |
| scale-6 | 3,825 | 34,096 | 69,187 | 69,504 | 69,504 | 3,751 (0.55) | 3,349 | 866 |
| scale-8 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| near-2 (reach 45 m) | 3,065 | 20,870 | 65,315 | 67,201 | 67,201 | 4,929 (0.72) | 3,428 | 1,028 |
| near-3 | 804 | 10,170 | 37,788 | 57,977 | 58,934 | 1,712 (0.25) | 2,110 | 951 |
| near-4 | 392 | 5,506 | 22,431 | 44,889 | 50,340 | 835 (0.12) | 1,251 | 697 |
| the record | — | 2,920 (a triangle) | 15,824 | 15,824 | 21,317 | 6,845 | 0 | — |

Two findings about the engine. A cliff: with heat felt three cells deep the front
accelerates (every cell behind it adds heat) and the fire either fills the window in hours
or, past a threshold, cannot light the next cell; nothing in between walks at the record's
pace. With heat felt only by the next cell, the early hours come within a factor of two of
the record and the fire then keeps growing, because nothing stops it. And the houses: the
plausible-pace runs reach a tenth of the destroyed structures, because a built cell has a
high threshold and the fire walks around the neighbourhoods; the real fire went into Pacific
Palisades on the first evening and burned it house to house. The engine has no ember into
an eave and no house as the next house's fuel, and that is the next module, with the Eaton
fire of the same day kept back as the test.

## The 3D view

A declared `scene` view (`Scene3D.svelte`, three.js 0.170 bundled): the terrain frame as a
surface in metres, every built cell a box of its stated height (the county's unit is not
stated; the values read as feet), the fire's arrival frame draped over both at the cursor,
buildings in burned cells red. Orbit by drag. It needs `terrain_elev_m` and
`building_height` frames, which the engine publishes once. Headless Chrome had no WebGL
until the screenshot helper got the swiftshader flags. The engine project's cover is the
scene itself.

## Found on the way

- A snapshot of the fire module was 335 MB of JSON (the per-direction progress array);
  snapshots every ten days now, and a four-day run folder is 28 MB.
- `pgrep -f` with a pattern from my own command line matched my own shell and killed it.
- A derived that referenced the awaited `run` before it existed blanked the run page twice.
- The engine's `--set` on a plain `esim run` is a range for ensembles, not an override; the
  project's variants are the way.

## Open

Push and deploy are his call. The book he pointed at (*Fire Weather*, Fort McMurray 2016) is
in the todo as the second fire. The front page still features only Earth 2100 and the first
Palisades project; the engine project is reachable from "all projects".

## Later the same evening: the houses, first pass

Anselm, landed: "I just want more granular control. Your comments about eaves is good - the
fire behaved in non linear ways - skipping some buildings and burning others - due to micro
conditions - slight variations in slope, building composition and so on. We can grind on this
over time iteratively to get closer to the historical regression." And: is the "engine
project" a fire model or a 3D engine? It is neither in particular: `fire.cells` is a module
like any other, and the project was only a second scenario folder that runs it. Renamed
`anselm/palisades-cells`, "The Palisades fire, cell by cell", so the folder says what it is.

Three knobs added to `fire.cells`, each a guess and each declared: `ember_built_p`, the
chance an ember landing on a built cell lights it, times the cell's built fraction;
`built_to_built_gain`, how much further heat reaches from a burning built cell into the next
built cell (house to house); `threshold_jitter`, a fixed log-normal factor on each cell's
ignition threshold from a hash of its index, so a run is reproducible and a house can be
easier or harder to light than its twin: the micro-conditions nobody mapped. Two variants on
top of near-4:

| variant | 27 h acres | 96 h acres | destroyed reached | undamaged overrun | precision |
|---|---|---|---|---|---|
| near-4 | 22,431 | 50,340 | 835 (0.12) | 1,251 | 0.36 |
| houses (embers 0.5, house to house 2.5) | 23,093 | 50,842 | 2,211 (0.32) | 2,242 | 0.45 |
| houses-jitter (sigma 0.6) | 22,844 | 50,577 | 1,642 (0.24) | 1,508 | 0.47 |

The houses move it the right way without changing the acreage: recall from a tenth to a
third, precision up. Jitter buys precision at recall's expense, as a scatter should. Still a
long way from 6,845. What the engine lacks for the next pass: the real first-night wind in
the village (HRRR is 3 km; the canyons funnel), structures as individual bodies rather than
a cell's built fraction, and something that stops the fire on days two to four, which in the
record was crews.
