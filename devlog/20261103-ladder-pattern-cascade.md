# The ladder, the pattern, the first cascade, and three papers read

2026-11-03, overnight. Anselm, after the cells runs were pushed and deployed: "Let's tackle
these all now. I can check back in a few hours." The four things were the ones I had
recommended in that order: a regression ladder before anything is coupled, a gridded
warming pattern in place of one slope per country, the first cascade (permafrost and
Amazon carbon into the carbon ledger), and reading the three papers the layers lean on.
Done in this session (Claude Fable 5.1, session 018TGCFhEpYs9hm68xDLd5Xy) on canada1;
three reading agents did the papers. Runs are in the planet-2100 project folder; the
regression runs are in `runs/regress/` (not in git).

## 1. The ladder: `esim regress`

`regress/ladder.toml` names rungs: a manifest, optional parameter overrides, extra
scoreboards, and series whose end value is watched. `esim regress` runs each rung into
`runs/regress/`, reads its scores and watched values, and compares them with
`regress/baseline.json` within the ladder's tolerances (NSE and KGE 0.005 absolute, PBIAS
half a point, RMSE one percent, watched values half a percent). Drift fails the command.
`esim regress --accept` writes a new baseline with the commit and the date; that is a
decision made by hand and this file is where it is explained. Three rungs to start:

| rung | manifest | what it pins |
|---|---|---|
| planet-co2 | p1-planet-co2-1850-2025 | CO2 against Mauna Loa and Law Dome; warming against HadCRUT5 and GISTEMP; the 2025 CO2, warming and airborne fraction |
| planet-people-hindcast | p4-planet-people to 2025 | the same scores with aerosols in; people outside the niche, countries outside it, people and person-days over 31 °C wet-bulb, tipping elements passed |
| boone-water | m3-water-boone | the gauge year, NSE 0.469 and PBIAS +31% as they stand |

A thing learned: an override's value is TOML text, so a date must carry its quotes in
the ladder (`'run.end="2025-01-01"'`); bare, it parses as a TOML date and the manifest
will not re-parse. The first baseline was accepted at 2a8b276, before the pattern and
the cascade, so that both would be measured against it below.

## 2. Three papers, read rather than recalled

Three reading agents fetched the papers and reported with quotes; nothing below is from
memory. Where the agents could not read a figure they said so, and so do I.

**Armstrong McKay et al. 2022, Table 1** (the uOttawa course copy of the Science PDF,
cross-checked against the first author's own reproduction of the table). Every
threshold in `data/events/tipping.toml` agreed with the paper, all fourteen rows, low,
central and high. One timescale was wrong: the East Antarctic ice sheet has no central
estimate, only "more than 10,000 years". Two elements were missing: boreal permafrost
collapse (4.0 °C, range 3 to 6; 50 years; 125 to 250 GtC, low confidence) and the boreal
forest's northern expansion (4.0 °C, range 1.5 to 7.2). The sea-level metres in the
consequence column were not from this paper (its Table 1 has none; the text names 3.5 m
as Greenland's irreversible-loss equivalent and about 50 m of potential for East
Antarctica), so those rows now say what the paper says. Every row is `confidence =
"checked"` with the date. The carbon numbers, quoted in the table's comments: permafrost
abrupt thaw adds "~50% emissions beyond gradual thaw (~10 to 25 GtC per °C)", 10 GtC per
°C by 2100 and 25 by 2300, the per-degree strength "declines with further degrees of
warming (by ~21% per °C)"; Amazon partial dieback of 40% releases "~30 GtC" (total 75);
permafrost collapse "~125 to 250 GtC". The paper's reading of its own thresholds:
crossing is "possible" beyond the minimum and "likely" beyond the central estimate.

**Lenton et al. 2023** (the PIK open-access copy). Hot exposure is mean annual temperature
at or above 29 °C, chosen because only 0.3% of the 1980 population (12 million) lived
there in the 1960 to 1990 climate; climate and people at about 10 km. Today (2010 people,
the 2000 to 2020 climate, +1.0 °C): 62 ± 26 million. At 1.5 °C about 0.3 billion; at 2.7
°C in 2090 under SSP2, 2.1 ± 0.8 billion (23%); at 3.6 °C, 3.3 billion; at 4.4 °C, 4.2
billion; the slope 13.8% of people per degree. India more than 600 million at 2.7 °C,
Nigeria more than 300 million, Indonesia about 100 million. Their local warming comes
from eight CMIP6 models downscaled by WorldClim, in which the land warms about 1.5 times
the global mean and the mean temperature a person lives in rises 1.5 °C per degree.

**Raymond, Matthews and Horton 2020** (PMC copy), with Vecellio et al. 2022 and Sherwood
and Huber 2010. Station data (HadISD, 1979 to 2017, 1- to 6-hourly), wet-bulb by
Davies-Jones. Two stations have reported daily maxima above 35 °C; exceedances of 31 and
33 °C have more than doubled since 1979; the extremes last one to two hours. The
sentence this project needed: "the spatiotemporal averaging inherent in reanalysis
products causes ERA-Interim to be unable to represent the short durations and small
areas of critical heat stress", with Persian Gulf differences "consistently in the range
of −2° to −4°C". Vecellio: a critical wet-bulb of 30.55 ± 0.98 °C across the warm-humid
conditions, 24 young adults at about 10 W; our 31 °C is that rounded. Sherwood and Huber:
35 °C from skin at 35 °C, and "with global-mean warming of about 7 °C" for the first
regions.

What changed in the code from the reads: the tipping table as above; the three limits'
describe lines now cite the measured figures and say "checked"; the wet-bulb describe
states the reanalysis's 2 to 4 °C shortfall on the Gulf coast as the paper gives it.

## 3. The warming pattern: built, measured, and not applied

The idea was to replace one fitted slope per country with a slope per cell, from data we
already reach: POWER's monthly API gives MERRA-2's annual means per reanalysis point for
1981 to 2025 in one request per 10-degree box (343 requests, five minutes, now part of
`esim warm cells`). `cells::warming_pattern` regresses those on the global record, both
smoothed over eleven years as the country fit is, and `climate.countries` with `pattern =
"cells"` publishes the global anomaly for the cell layers to multiply by their own slope.

Fitted point by point the field ran from −5 to +7 degrees per degree: forty-five years at
one point is not enough. Fitted on the mean series of each 5-degree block of points (about
eighty points) and interpolated, the field is smoother (median 1.05, tenth to ninetieth
percentile 0.35 to 2.14) and still wrong where it matters: Delhi −0.49, Dhaka −0.33,
Khartoum 0.72, Doha 1.80, Phoenix 1.88. Checked outside the code, from the raw POWER answer
for the India box: MERRA-2's decadal means over the box run 26.26, 26.34, 26.51, 26.10 °C
for the 1980s to the 2010s while the global anomaly runs 0.53, 0.67, 0.91, 1.12. The
reanalysis cools South Asia by 0.4 °C between the 2000s and the 2010s. The station-based
country record (CRU) does not, and neither does anyone's thermometer; it is the
reanalysis's own problem, the kind that comes from changes in what it assimilates, and it
makes its trend unusable as a pattern over the most populous region on Earth.

So: the code stays, the map is drawn on the run page for the record (`climate.countries`
keeps `pattern = "cells"` so `climate_slope_cells` is written), and the cell layers keep
`pattern = "country"`. The limits page says so. The next route, noted in the todo, is a
model-ensemble pattern: WorldClim serves CMIP6 models' annual mean temperature for 2081 to
2100 at ten arc-minutes as anonymous GeoTIFFs beside its 1970 to 2000 baseline, so
(future − baseline) per model, divided by that model's global warming, is a pattern with
the land-sea contrast and the aerosol fade in it, which is what Lenton et al. used. An
afternoon's work, with the stated catch that the model's global warming between those
periods has to come from somewhere other than the land-only tiles.

## 4. The first cascade: `carbon.tipped`

A tipping element past its threshold releases the carbon its row of the tipping table
states, on the row's timescale, into the carbon ledger (a transfer from a `permafrost` or
`amazon` account to `atmosphere`, published as `carbon.emitted` so the pools module treats
it as it treats fossil carbon). The amounts, thresholds and timescales are the
assessment's, read from the same table the tipping module reads; the module only moves
the stated mass over the stated time, as a first-order approach. The per-degree rule for
abrupt thaw (25 GtC per degree, declining 21% per degree) is integrated from
pre-industrial, so at today's warming it is 22 GtC. `trigger = "central"` starts a
release at the assessment's central estimate ("likely"); `trigger = "low"` at the bottom
of the range ("possible"), the sensitivity case, which the project has as the variant
`held-low`. The story lists the start of a release as a sentence; the wiring graph now has
a loop: tipping → tipped → carbon ledger → pools → energy → tipping.

What it does, held variant, by 2099: permafrost abrupt thaw starts in 2019 (the run
passes 1.5 °C), releases 15.8 GtC by 2099 (the fossil record is about 10 GtC a year), and
that is +5 ppm and +0.024 °C: 689.6 ppm and +3.36 °C against 684.2 and +3.34. The Amazon's
central estimate (3.5 °C) is not reached in the held run, only in trend (2093) and
growing. At the low end of the ranges (`held-low`): abrupt thaw from 2003, the Amazon from
2037 (2 °C), permafrost collapse from 2082 (3 °C), 69 GtC by 2099, 709.7 ppm, +3.44 °C,
1.37 billion outside the niche against 1.32. The cascade exists, is auditable (the ledger
closes, worst leak 1e-10 GtC), and is small this century at the assessment's numbers.
That is a finding worth having on the page: the stacking that the alarmed framing assumes
is, for the carbon elements, a tenth of a degree by 2100 even at the low end. What stacks
more is what is still missing: the albedo terms (sea ice, boreal forest) that this energy
balance has no place for, methane as itself, and the slow ice.

The ladder saw what changed: `esim regress` after all of this reported one drift on the
hindcast rung, `tipping.elements_past_low` from 9 to 10, the boreal forest's northern
expansion added to the table from the paper (low end 1.5 °C). The cascade's 1 GtC by 2025
moved the 2025 CO2 by 0.4 ppm, inside the half-percent tolerance, and no score moved past
0.005. The new baseline was accepted on purpose with this paragraph as the reason.

## 5. Numbers, every variant, 2099 (hindcast: 2024)

| variant | warming | CO2 ppm | tipped GtC | outside niche | over 31 °C wet-bulb (3 d/yr) | countries outside niche |
|---|---|---|---|---|---|---|
| hindcast (2024) | +1.71 | 453 | 1 | 147 M (1.8%) | 6.5 M | 4 |
| zero by 2050 | +2.11 | 452 | 13 | 528 M (5.2%) | 22 M | 9 |
| falling 3%/yr | +2.48 | 508 | 14 | 724 M (7.1%) | 31 M | 15 |
| held | +3.36 | 690 | 16 | 1.32 bn (13.0%) | 124 M | 22 |
| held, low trigger | +3.44 | 710 | 69 | 1.37 bn (13.4%) | 152 M | 22 |
| trend | +3.72 | 787 | 18 | 1.60 bn (15.7%) | 278 M | 29 |
| growing 1%/yr | +3.96 | 854 | 20 | 1.78 bn (17.5%) | 529 M | 31 |

Against Lenton et al. 2023, now read: they have 62 ± 26 million at +1.0 °C and 2.1 ± 0.8
billion at +2.7 °C; we have 147 million at +1.7 °C and 1.32 billion at +3.4 °C. Our
present is higher (a warmer model year, and a 2001 to 2025 climatology against their
1960 to 1990 one), our future lower, and the reason is now clear: their local warming is
the models' pattern, in which the land warms 1.5 times the global mean and India more
than that; ours is the observed country slope, India's 0.82, which carries the aerosol
haze of the record forward. The pattern is the lever, which is why section 3 matters.

Commits follow this entry. Nothing is pushed or deployed; both are Anselm's call.
