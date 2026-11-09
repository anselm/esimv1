# 2026-11-08: Half Moon Bay, the breakwater and the beach

Anselm, after the Devil's Gulch pair: "Let's add a separate sim for the half moon bay dredging
issue. Apparently the army core of engineers built sea walls back in the day and they are eroding
the coastline and filling the harbor with silt. We may need a new fluid dynamics model - this is a
hard one." Built by Claude in the esim room the same afternoon and evening. Project
`anselm/halfmoonbay-shoreline`, manifest `manifests/c1-halfmoonbay-oneline.toml`, the record
`data/events/halfmoonbay/record.toml` with its README, the module `coast.oneline`
(`crates/modules/src/coast.rs`), the readers `crates/registry/src/coast.rs`.

## What the record says

Read first, because nothing here could be made up. The Corps' East Breakwater went in between 1959
and 1961. Griggs (2026, J. Mar. Sci. Eng. 14(16):1493, abstract only; the publisher refused the
fetch) has the bay before it as a smooth hook in equilibrium with waves refracted round Pillar
Point, bluff recession about 8 cm a year, then up to 2 m a year once wave energy was concentrated
at the breakwater's downcoast end, erosion 1.4 km downcoast, a county road and a sewer line lost,
revetments at the highway and at Mirada Road. The Corps' own draft EA (October 2015, citing Lin et
al. 2015, read as text from the Harbor District's copy) has 1.64 ft a year of bluff retreat along
Vallejo Beach 1993 to 2012, seven times a geologically similar stretch down the coast; a net beach
loss of 4,200 cubic yards a year from the breakwater to Mirada Road; sand accumulating in the harbor
"approximately 2,000 CY" (whether per year is not said), partly by "surging through voids in the
breakwater", into a shoal 2,000 ft by 200 to 250 ft; the Corps modelled sealing the breakwater,
notching it, a spur, and a sand transfer, found the transfer feasible and of no federal interest.
The Harbor District dredged up to 100,000 cubic yards of the shoal onto 1,000 ft of Surfers Beach
in 2025. The IEM and NDBC were not needed here; the waves come from the ERA5-Ocean reanalysis.

## What was built

Not a fluid dynamics model. A one-line shoreline model (Pelnard-Considere 1956, the CERC transport
formula, the GENESIS lineage), which is the oldest useful approximation and the right first rung:
it runs 64 years in half a minute and every term is a sentence.

- **The line.** OpenStreetMap's coastline from the breakwater's root (the first node of way
  638337850) south for 8 km, resampled every 50 m; today's shoreline, revetments and all, so a run
  from 1961 measures rates, not positions. The first three transects take the fourth's normal: the
  mapped coast wraps the structure's base.
- **The waves.** ERA5-Ocean hourly through Open-Meteo's marine API, 1961 to 2024, for the
  half-degree cell that holds the bay (37.5 N, 122.5 W; mean significant height 1.65 m in 1993,
  seven hours in ten from between 270 and 300 degrees). One request per year, cached; a run past
  2024 repeats the last year. New catalog entry `open-meteo-era5-ocean-waves`.
- **The shadows.** Pillar Point (base to Sail Rock) and the East Breakwater (root to tip, both from
  OpenStreetMap nodes) as segments. A transect whose wave ray crosses a segment is in its shadow:
  height factor 0.5 at the shadow line falling as exp(-angle/25 degrees) inside (the shape of the
  Penney and Price solution, rounded), waves arriving from the tip; on the lit side rising from 0.5
  with an 8 degree scale. That is the mechanism that makes a hook-shaped bay, and the breakwater
  moves the shadow.
- **Breaking and transport.** Komar and Gaughan's breaker height from the deep-water height and
  period; Snell's law for the breaking angle; the CERC formula with K = 0.39 for significant height
  (SPM 1984, recalled). The one-line update with the line's own angle fed back. Where the beach
  (20 m, a guess) is gone, the bluff (10 m, half sand, guesses) retreats with the line and yields
  its sand.
- **The leak.** A declared share of the mean transport, either way, over the first 300 m passes
  through the breakwater's voids into the harbor: the Corps' mechanism, the one fitted number.
  Before 1959 the same flux goes north along the coast instead.
- **Books.** A ledger `sand` in m3: beach, bluff, harbor, downcoast, upcoast, offshore; closes
  monthly; every run must close it (the first three did not, see below).
- **Also.** Bruun's rule for sea level rise; a dredge every N years from the harbor onto the first
  305 m; a yearly `shore_change_m` frame; zone means (surfers 0 to 700 m, vallejo 700 to 1,400,
  miramar 1,400 to 3,000, south); the record's rows as a describe table; `fit.*` ratios.

## What went wrong, in order

1. The first run blew up: the south zone advanced 110 km. The slope feedback's sign was inverted
   (an equality test on floats chose the wrong rotation), so the equation was anti-diffusive.
2. Fixed, it still blew up: a free far boundary with a net transport toward it fed sand in without
   limit, and the first transect had no transport of its own because the mapped coast at the root
   turns into the harbor, so it starved. Far end closed, then opened as a pass-through; the root's
   normals pinned; the leak taken over 300 m rather than one transect.
3. Still a blow-up, now a checkerboard: odd and even transects running apart by kilometres, the
   bluff feeding one side and the beach bulging on the other. Transport at the transects from a
   centred slope decouples the two lattices. Moved to the faces between transects with the face's
   own slope, the standard staggering. The ledger closed for the first time at this point (the
   beach stock had also double-counted the bluff).
4. One more: the first 400 m is a pocket facing south-east, so diffracted waves from the tip
   arrive at 75 to 88 degrees. Holding the angle at 45 degrees (GENESIS's cure for the high-angle
   instability) made grazing waves carry the maximum transport and drained the pocket; tapering it
   to zero at grazing made the feedback anti-diffusive again. The split that holds: the angle to the
   mapped coast sets a fixed cosine taper, the angle to the moving line, clamped at 45 degrees,
   carries the feedback. Sixty days of real hours then ran smooth.
5. A 50 km cell's deep-water direction is about 22 degrees oblique to the whole bay, which drove
   millions of cubic metres a year of drift through a shoreline the record calls an equilibrium.
   One angle is now fitted at the start so that the net transport along the open stretch beyond
   3 km is zero over the first three years, reported as `coast.dir_bias_deg` (-22.6). Declared.
6. With K at the SPM value the as-built run retreated Surfers Beach 1.2 km in 64 years and put
   half a million cubic metres in the harbor: the pattern right, the pace tens of times too fast,
   as the raw CERC coefficient is on any sheltered coast. The ratio between the as-built and the
   before-1959 runs in the breakwater's zone, about six, does not depend on K, and the record's
   ratio is seven. K is swept and fitted to the Corps' erosion rate; the rest is the test.

## Numbers

The sweep (K = 0.05, 0.02, 0.01, 0.005, breakwater on and off): the loss within 1.4 km scales less
than linearly with K (100,800 m3 a year at 0.05 to 18,900 at 0.005, against the Corps' 3,211), the
breakwater's zone retreats about twice the headland-only line at every K, and the bluff within
1.4 km at K = 0.005 as built gives 67 m in 64 years, about 1 m a year. K = 0.005 and leak = 0.2
went into the manifest as the fitted pair. The six variants (each 30 to 70 s; every ledger closes):

| variant | surfers | vallejo | miramar | south | bluff within 1.4 km | harbor m3 |
|---|---|---|---|---|---|---|
| as-built, 2025 | -115 m | -65 | -18 | +64 | 70 m | 83,190 |
| before-1959, 2025 | -50 | -49 | -16 | +64 | 31 | 0 |
| sealed, 2025 | -107 | -65 | -18 | +64 | 66 | 0 |
| to 2075, held | -188 | -100 | -17 | +101 | 125 | 144,750 |
| to 2075, SLR high | -211 | -119 | -42 | +70 | 144 | 145,641 |
| to 2075, dredged every 5 yr | -176 | -98 | -17 | +101 | 118 | 6,205 (138,814 placed) |

What stands: the pattern (erosion to 1.4 km, Miramar still, sand gone south), the sealed finding
(the Corps' own), the dredge's arithmetic (a quarter of the placed sand still in the zone after
fifty years). What does not: the before-1959 line loses half a metre of bluff a year where the
record has eight centimetres, so the breakwater's effect is a doubling where the record has seven
times; the fitted K is eighty times below the open coast; the southern end imports 19,000 m3 a
year. The README says all of it.

## Next

- A closed southern end at Miramontes Point and a line that reaches it.
- The revetments placed from the record as non-erodible transects.
- A 1956 shoreline (the agencies compare 1956 air photos) so the hindcast measures positions.
- Lin et al. 2015 itself, for the sediment budget and the profiles.
- The second rung: a two-dimensional wave and sediment field (what the Corps' CMS does) on the
  same record, scored against the same table. That is the fluid dynamics model Anselm named; this
  page is the yardstick for it.

## Evening: the line moves, and the story

Anselm, looking at the site: the runs "appear static ... There is an implication they are showing
live state but visually appear to be static?" Checked: the shoreline frame did move with the cursor,
but only its tint, because it painted the change along today's line. Now the frames cover the line's
own box at half an arcsecond (about 15 m), `shore_change_m` paints the strip between today's line
and the line now, and a second frame `shore_line` draws the line now in the brand teal over today's in
grey; the viewer has a `line` ramp for frames named `*_line` and stretches change fields to each
frame's own range by default (Anselm: "stretch the color range yes - within the fit"). The project
covers stay stills (Anselm: "i don't think we need to update the main project pages").

Then: "flesh out a bit of the deep story behind the half moon bay scenario - as per usual the text
becomes important to the reader to contextualize why we are seeing erosion now." Four sections went
in before the model: how a bay came to have a harbor (the district of 1933 and Redwood City, the
fishing fleet's harbor of refuge, the Corps 1959 to 1961, the dogleg and the inner breakwaters, the
pier and berths), what harbors do to sand (Griggs's beach compartments as bank accounts; Santa
Barbara 1928 to 1930 and the erosion wave to Carpinteria at 1.7 km a year, from Griggs 2015, *Lost
Neighborhoods of the California Coast*; Santa Cruz 1965; why Pillar Point is a different case), the
sixty years since (the 1980s road, the revetments, 1993 to 2012, the 2011 meeting and the Section 111
ceiling, the 2016 withdrawal, the district's seven-year path to the 2025 dredge, OneShoreline), and
why now. Sources added to the event folder's README as they were read: the visitor bureau's history
(the 1948 authorization is its claim, unchecked), the KPIX report of 1 September 2026 with Griggs's
and residents' words, Coastside Buzz on the Coastal Commission's approval of 6 September 2023. Version
bumped to 0.1.1; covers made for the seven projects that had none (chart rows and the 3D or globe
views from their run pages, dark mode).
