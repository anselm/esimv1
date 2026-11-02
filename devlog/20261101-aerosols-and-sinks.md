# 2026-11-01 — Aerosols and the carbon sinks: the planet stops running warm

Anselm: "lets go down the path you suggest, aerosols etc.", plus the accounts notes for
the todo, his crumpled namespace, and the question whether there is a user-visible list
of modules (there was not; there is now). Agent: Claude (Fable 5.1), earth-simulator-1
room on canada1.

## Two changes to the planet core

**`forcing.ar6`.** The IPCC AR6 WG1 assessed effective radiative forcing time series,
1750 to 2019, from the Chapter 7 authors' repository (Smith et al.; the Zenodo copy
10.5281/zenodo.5705391 says CC BY 4.0, read from the record API). The module carries
every component this model does not compute itself: aerosols (radiation and cloud
interactions together), ozone, the minor well-mixed gases, stratospheric water vapour,
contrails, black carbon on snow, land use, volcanoes and the sun, each published to the
energy module by name. After 2019 the stated assumption is hold; for aerosols, which are
falling, that is the warm side, and the describe says so. An assessment is not an
observation and the catalog calls it one.

**The sink feedback in `carbon.pools`.** The fixed Joos response kept too much carbon
airborne. The Millar et al. 2017 scheme (FaIR v1) scales the three decay timescales each
year by a factor chosen so that the 100-year integrated response equals r0 + rc times
the carbon taken up so far + rt times the warming; the parameters (35 years, 0.019
years per GtC, 4.165 years per kelvin) are recalled and marked so. The ledger is
unchanged; only how fast the excess drains.

A project folder's manifest is frozen at creation, which is right, but it meant the
`anselm/planet` project did not see the new core until `esim project update` existed.
It does now.

## What the hindcast says now

| 1 July | CO2 model | CO2 record | warming model | warming record |
|---|---|---|---|---|
| 1950 | 321 | 311 | +0.27 | +0.13 |
| 1980 | 353 | 339 | +0.36 | +0.55 |
| 2000 | 387 | 370 | +0.87 | +0.69 |
| 2024 | 451 | 425 | +1.69 | +1.54 |

Skill against the records, 175 matched years: CO2 r 0.999, bias +2.8%, Nash-Sutcliffe
0.90 (was 0.42); temperature against HadCRUT5 bias +7%, Nash-Sutcliffe 0.86 (was
negative), root mean square error 0.14 °C. The 1980 dip is the aerosol maximum (−1.44
W/m²), which the model now has and had not before. The remaining 26 ppm of CO2 is the
next thing to chase, by the harness and not by hand.

## The 2100 variants, rerun

| 2099 | warming | CO2 | outside the niche | countries | tipping past central |
|---|---|---|---|---|---|
| held at 2024 | +3.33 | 683 | 741 M | 22 | 9 |
| zero by 2050 | +2.09 | 448 | 394 M | 10 | 7 |

Central tipping estimates are now passed in 2019 for the four 1.5 °C elements, 2022 for
the Barents Sea ice, 2029 for Labrador Sea convection, and under held emissions 2037 for
mountain glaciers, 2074 for the Sahel monsoon and 2084 for the East Antarctic basins.
Those years are a few years early (the model is 0.15 °C warm in 2024), not twenty. The
table's values are still recalled and unchecked, and the page says so on every row.

## Also today

- A kinds page in the viewer (`?kinds`), linked from the home and the docs: every kind's
  summary, parameters with provenance (guesses and recalled values marked), reads, owns,
  topics, ledgers, outputs and data. The answer to "is there a user-visible list of the
  modules" is now yes.
- The accounts design and the crumpled namespace in `todo.md`; the namespace preference
  in shared memory.
- "Patch" redefined in the terminology to Anselm's meaning: anything bound to one place,
  one module or a bundle; a module need not span the planet; a manifest is the grouping,
  and a bundle of many patches is a world-spanning layer.

## Next

Humidity and wet-bulb exposure (the Gulf first), then the hindcast harness against Mauna
Loa for the remaining ppm, then accounts with the deployment.
