# 2026-11-01 — F1: the first ensemble, a ten-day river forecast with a band

Overnight, same session (Claude, Fable 5.1, canada1). Issued 2026-11-01 about 04:00 UTC
(the evening of 31 October in San Francisco).

## What was built

- Weather source `chain`: ERA5 hourly where it exists (it ends about five days behind
  the present, here 2026-10-27 01Z), then one GEFS ensemble member for the recent days
  and the next ten. The member is chosen by the run seed: seed 1 is the control, seeds 2
  to 31 the perturbed members. Members come from Open-Meteo's ensemble API in one request
  for the whole point grid, keyed by the day of issue so each day's forecast is kept.
- `esim ensemble <manifest> --branches N --threads T`: warms the cache once, then runs N
  branches with seeds base+1..base+N across T threads, keeps each branch's series, and
  writes quantile bands (min, p10, p25, p50, p75, p90, max) per series to `ensemble.json`,
  plus one slim file per branch. Each branch is a complete deterministic run with its own
  digest and ledgers.
- The run seed is now folded into every module's parameters by the CLI, so any module can
  make seed-dependent choices without the kernel knowing.

## The forecast

`manifests/f1-boone-forecast.toml`: spin-up from 2025-01-01 on ERA5, hand-over to the
member on 2026-10-27, forecast to 2026-11-11, at 30 arc-seconds. 31 branches on 8 threads
in 119 s; 31 distinct digests; all ledgers closed in every branch (water worst leak
2.4e-6 m3 over 679 daily closes per branch). Output folder: `runs/f1-boone-forecast-ens31-7904a474`
(85 MB, outside git; the GEFS file it used is in `data/cache/open-meteo-gefs/` keyed
2026-11-01 and listed in the lock).

Boone River near Webster City, daily mean discharge, m3/s, across the 31 branches:

| date | p10 | p50 | p90 | max | USGS |
|---|---|---|---|---|---|
| 2026-10-30 | 34.3 | 35.0 | 36.0 | 37.5 | 12.3 |
| 2026-10-31 | 33.1 | 34.1 | 35.4 | 37.2 | 13.7 |
| 2026-11-03 | 31.9 | 33.9 | 36.2 | 36.6 | |
| 2026-11-04 | 37.3 | 41.5 | 47.7 | 53.8 | |
| 2026-11-05 | 40.4 | 47.3 | 59.3 | 73.6 | |
| 2026-11-06 | 39.8 | 48.1 | 62.1 | 74.1 | |
| 2026-11-08 | 36.6 | 46.4 | 61.2 | 67.9 | |
| 2026-11-10 | 33.3 | 43.0 | 55.5 | 66.1 | |

Ten-day basin rain across members: p10 15 mm, p50 31 mm, p90 78 mm, max 158 mm. The
members agree on an event starting 4 November and disagree on its size, which is what the
band is for.

## Read it honestly

- The level is wrong and the shape may be right. On the last observed days the model
  sits near 35 m3/s against a gauge reading of 12 to 14. That is the autumn wet bias
  documented at M3 and M4 (no frozen ground, constant drainage, no tile-drain dynamics),
  not the ensemble's doing. A forecast from this model should be read as "a rise of about
  10 to 30 m3/s from the current level, most likely starting 4 November", not as absolute
  flows. The hindcast score over 2025 to October 2026 for the control branch is NSE 0.0,
  bias +47%, which says the same thing.
- The spread is the weather's only. Parameter and structural uncertainty are not sampled.
  A later ensemble should perturb soil capacity and drainage as well, and the hindcast
  harness should calibrate them by method.
- The member choice by seed is honest but coarse: 31 branches is the whole GEFS, so more
  branches would only repeat members. Open-Meteo's ensemble API also serves ECMWF and
  ICON ensembles; mixing them is a one-line change in the fetch.

## How to verify this forecast, on or after 2026-11-11

Fetch USGS 05481000 daily discharge for 2026-10-27 to 2026-11-10 and compare with the
bands above. A proper command for that (reading an ensemble folder, refetching the
observation, computing coverage of the p10–p90 band and the CRPS) is the V1 scoring
deliverable; it does not exist yet. Until it does, the table above is the record. The
useful question is not whether the median was right but whether the observed rise, if
any, began on the day the members agreed on and fell inside the band after removing the
known bias.

## What this proves for the product

A manifest, one line changed (`source = "chain"`), and one command turned the hindcast
model into a probabilistic forecast of a real river for the next ten days, with every
branch replayable and every ledger closed. That is the shape of the product: the same
machinery, pointed at tomorrow. The missing pieces are exactly the ones the deliverables
list already names: calibration by method (M12), the verification command (V1), a job
runner and result store for many such forecasts (V5), and the sober viewer's band charts
(P4), which a subagent is adding tonight.
