# Pillar Point Harbor and the Half Moon Bay shoreline: the record we hold

Copies and extracts of the public record the Half Moon Bay sediment project is read against.
Written from canada1 on 2026-11-08. Nothing here is ours; every row of `record.toml` names its
source and says whether we read it ourselves.

## The ask

Anselm, 2026-11-08: "Let's add a separate sim for the half moon bay dredging issue. Apparently the
army core of engineers built sea walls back in the day and they are eroding the coastline and filling
the harbor with silt. We may need a new fluid dynamics model - this is a hard one."

## What the record says, in short

The US Army Corps of Engineers built the East Breakwater of Pillar Point Harbor between 1959 and
1961 (and the West Breakwater from Pillar Point). Before that, by Griggs (2026), the bay had a smooth
hook-shaped shoreline in equilibrium with waves refracted around the point, and the bluffs south of
the harbor site retreated at about 8 cm a year. After it, wave energy was concentrated at the
downcoast end of the breakwater and bluff retreat rose to as much as 2 m a year, destroying a county
road and a wastewater line and threatening Highway 1 and homes; the erosion has reached 1.4 km
downcoast, and revetments now protect the highway (Surfers Beach) and Mirada Road (Miramar). The
Corps' own feasibility study (Lin et al. 2015, as cited in the 2015 draft EA) measured 1.64 ft a
year of bluff retreat along Vallejo Beach from 1993 to 2012, seven times the background rate down
the coast, a net beach erosion of 4,200 cubic yards a year from the breakwater to Mirada Road, and
sand accumulating in the harbor beside the East Breakwater, partly by surging through the rubble
mound's voids, into a shoal 2,000 ft long and 200 to 250 ft wide that blocks the boat launch. The
Corps found a sand-transfer project feasible but of no federal interest; the Harbor District dredged
up to 100,000 cubic yards of that shoal and placed it on 1,000 ft of Surfers Beach in 2025.

## Sources

- **USACE San Francisco District**, *Draft Environmental Assessment and 404(b)(1) Analysis for the
  N. Half Moon Bay (Princeton) Pillar Point Harbor CAP §111 Feasibility Study*, October 2015. Copied
  from the Harbor District's site (`smharbor.com/files/aaf64544e/NHMB+CAP+111+draft+EA+with+404(b)(1)+Analysis+c.pdf`)
  on 2026-11-08 and read as text; the numbers it cites are from Lin, L., Li, H., Zoulas, J., Andes,
  L. and Wu, F., 2015, *North Half Moon Bay shoreline...* (the Corps' coastal study, not copied). The
  PDF is not kept in the repository; the rows quote it.
- **Griggs, G. B., 2026.** *The Impacts of the Construction of the Pillar Point Harbor Breakwaters on
  the Half Moon Bay Shoreline, San Mateo County, California.* Journal of Marine Science and
  Engineering 14(16):1493, CC BY. Only the abstract was readable from canada1 (Crossref and Semantic
  Scholar; the publisher's site refused the fetch); its numbers are in the rows marked so.
- **San Mateo County Harbor District**, *Completed Surfers Beach Sand Replenishment Pilot Project*
  page and its Section 111 request to the Corps (2024 and 2025), read 2026-11-08.
- **OneShoreline**, *Pillar Point Harbor Area Shoreline Management Study* page, read 2026-11-08:
  Princeton shoreline "retreating at a rate over 7 times greater than the rate of erosion before the
  breakwater"; the Corps studied mitigation from 2010 to 2016 and "concluded that such a project does
  not have a federal interest"; the page says "no comprehensive erosion modeling has been performed in
  and around the harbor", which the Corps' own 2015 modelling (Lin et al.) contradicts in part.
- **Half Moon Bay Patch**, 7 June 2011, on the Corps' first public meeting: the Corps' planning chief
  called the breakwater "the largest cause of the erosion, mainly by pulling sand from Surfer's Beach
  into the harbor" (the article's paraphrase).

- **The San Francisco Peninsula** (the county's visitor bureau), *Pillar Point Harbor in Half Moon Bay*,
  23 June 2025, read 2026-11-08: the district formed by county election in 1933 to build a harbor at
  Redwood City, the Depression, the coastside's push for a harbor of refuge for the fishing fleet, the
  Corps' work begun after the war and finished in 1961, the Johnson Pier, 369 berths and the inner
  breakwater in the 1970s and 1980s. A listing on the same site says Congress authorized the project in
  1948; neither is checked against the Corps' or the Congressional record.
- **Wikipedia**, *San Mateo County Harbor District*, read 2026-11-08: the 1931 state act, the 1933
  formation by the Board of Supervisors, inactive until 1948.
- **KPIX (CBS San Francisco)**, Kenny Choi, 1 September 2026, *Lasting effects of Half Moon Bay dredging
  project to combat coastal erosion are uncertain*, read 2026-11-08: Griggs's words ("When they built those
  two breakwaters, all the wave energy that was dissipated in that nice hook-shaped bay"; "Without a
  complete redesign, it's just gonna continue"), the loss of Mirada Road and a wastewater line, a resident's
  thirty years of pressing the county, the riprap, the agencies.
- **Coastside Buzz**, 5 October 2023, on the Harbor District's meeting of 20 September 2023: the Coastal
  Commission's approval of 6 September 2023, eelgrass to be relocated before dredging and a restoration
  area in the west basin, up to 100,000 cubic yards onto about 1,000 feet of Surfers Beach.
- **Griggs, G. B., 2015.** *Lost Neighborhoods of the California Coast.* Journal of Coastal Research
  31(1), doi 10.2112/13a-00007.1, read as text from eScholarship 2026-11-08: the Santa Barbara breakwater
  of 1928 to 1930, the beach widened 250 m behind it, the erosion wave downcoast at about 1.7 km a year
  (citing Wiegel 1965) as far as Carpinteria, 75 m cut at Sand Point in January 1940.
- **Griggs, G. B., 2018.** *California's Coastal Harbors, Beach Compartments and Sand Dredging*, Coastal
  Care, 1 October 2018, read 2026-11-08: the Santa Cruz jetties of 1965 and the dredging since, beach
  compartments as bank accounts, harbors in the middle or downcoast of a strong drift dredged forever.

## Data the model reads

- **Waves**: the ERA5-Ocean wave reanalysis (ECMWF, Copernicus Climate Change Service) through
  Open-Meteo's marine API, hourly significant wave height, mean direction and period from 1961, at the
  half-degree cell centred 37.5 N, 122.5 W, which covers the bay. Free for non-commercial use; attribute
  Copernicus. A 50 km reanalysis does not see the bay's own refraction; the model does that itself.
- **Shoreline and structures**: OpenStreetMap's coastline ways and the outer breakwater polygon
  (way 23624000) through Overpass, as mapped on 2026-11-08: today's shoreline, with the revetments in
  it, not the 1959 shoreline.

## Not here, and why

- Lin et al. 2015, the Corps' coastal study with the modelling, the beach profiles and the sediment
  budget: not found online on 2026-11-08.
- Historical shoreline positions (the 1956 and later aerial photographs the agencies compare): not
  copied; the USGS national shoreline change assessment for California has transect rates but was
  not fetched.
- Dredging records before 2025: the harbor has been dredged for navigation; no volumes were found.
