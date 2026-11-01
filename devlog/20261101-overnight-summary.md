# 2026-11-01 — Overnight summary for the morning

Anselm, at about 20:30 on 31 October: "do what you can and test, I will sleep and check
in am." This is what happened between then and about 05:00 on 1 November (San Francisco
time), by Claude (Fable 5.1), session
https://claude.ai/code/session_018TGCFhEpYs9hm68xDLd5Xy, on canada1. Fifty commits in
the repo, all local; nothing pushed. Each milestone has its own dated entry with the
numbers and the honest limits; this is the index.

## The ladder, as it stands

| rung | state | the one number |
|---|---|---|
| M0 kernel | done, tested | 7 kernel tests; replay bitwise |
| M1 Sun | done | clear-sky peak 948 W/m2 on the solstice; ERA5 all-sky 935 |
| M2 Heat | done | two rates, one energy ledger, worst leak 648 J on 1e17 |
| M3 Water | done, scored | basin delineated from the DEM to 2,186 km2 vs USGS 2,186; NSE 0.47 vs the 1940-to-present gauge |
| M4 Weather | done, scored | ERA5 drives the basin; clear-sky vs all-sky r 0.885; winter clearness above 1 exposed the placeholder sun model |
| F1 Forecast | done | 31 GEFS branches, a ten-day band for the Boone River, to be verified on or after 11 November |
| M5 Green | done, scored | crop canopy vs MODIS NDVI r 0.84, a month late; carbon ledger closes |
| M6 Eaters | done | deer and coyote cohorts, population ledger exact, spatial events in the kernel |
| M7 Tearing | done | fire from the hazard table, off/on/forced, 14.5 km2 burned in the forced what-if |
| M8 Zoom | done for populations | 654 animals became individuals and back, count conserved to 1e-11; nested grids not done |
| M9 Takers | done | 243 farm blocks choose, plant, sell; money ledger closes; yields a third of real |
| M10 Rules | done | one calendar row removed 38% of cropland, cut the flood peak 23%, doubled the deer |
| M11 Nitrogen | done, scored | fertilizer to gauge, six ledgers close 1,826 times; nitrate twice too high with the right seasons |
| M12 | begun | a 64-sample sweep found no parameter set better than the defaults: the water module's ceiling is structural |

Fifteen integration tests pass (`cargo test --release`, about 30 s with the data cached);
they skip rather than download when the cache is cold.

## What is true and worth knowing

- Every seam in the brief's list has a ledger across it, and every ledger closes every
  day in every run: energy, water, carbon, animals, money, nitrogen. The ledgers caught
  three real defects overnight (two f32 accounting leaks, one fire that never spread),
  none of which a chart would have shown.
- Every run is deterministic and replays bitwise from any snapshot, including runs with
  cohorts switching representation and three modules negotiating through events.
- Everything stands on real data fetched anonymously and cached once with its hash:
  Copernicus DEM, GHCN-Daily, USGS NWIS (discharge and nitrate), ERA5 and GEFS through
  Open-Meteo, MODIS through Planetary Computer. No synthetic terrain, no invented rain.
- The scores are modest and honest: discharge NSE 0.47, NDVI r 0.84, nitrate twice too
  high. Each devlog entry says why, and the sweep says the water module needs structure,
  not tuning. That is what the ladder was for.
- The viewer shows runs, ledgers, maps, scores, paired sim/obs charts and ensemble bands.
  `cd web && npm run dev`.

## What is yours to decide

- Whether the weak crop and water structures get fixed next (frozen ground, tile drains,
  a second soil layer, a real clear-sky model, a calibrated crop) or the ladder continues
  outward (Gulf delivery and the July cruise; the fire-to-water chain on a western basin;
  the parcel case; Hormuz).
- The product questions the brief deferred are now concrete: the forecast exists, the
  runs folder is a result store in miniature, and the English-to-manifest facade has a
  manifest language worth targeting. None of the three is started.
- Open-Meteo's terms are non-commercial. A paid service needs their commercial tier or
  another source for ERA5 and GEFS.

## How to look

```sh
cd ~/projects/2026/earth-simulator-1
git log --oneline | head -60
cat devlog/20261031-fitness.md
cd web && npm run dev            # then open the f1-boone-forecast ensemble and the m3 run
```
