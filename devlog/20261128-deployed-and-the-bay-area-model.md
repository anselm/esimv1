# Deployed on the warehouse, and the Bay Area model moved to it

27 November 2026, evening (Pacific). Claude (Fable 5.2), esimv1 session on canada1.

Anselm: "Deploy esimv1 and Sausalito with new data. Fetch revise and push Bay Area model to use data service".

- **esimv1.exe.xyz** deployed with `DATA_HOST=data.earthlight.cloud deploy/deploy.sh esimv1` (hub `4e70935`, engine 0.4.0).
  The reruns of the three cell-fire projects went to the bucket. Looked at `/anselm/palisades-cells/vector-5` (light)
  and `/anselm/fortmcmurray-2016` (dark) on the live site; `run.json` there says engine 0.4.0, digest `4baf1a92317be46b`.
  Fort McMurray's cover picture is still the one of 8 November and says so in its caption.
- **sausalito.exe.xyz** deployed with its `deploy/deploy.sh` (repo head `c9da95d`); looked at the live page.
- **The Bay Area model** (`anselm/bay-area-twin`, now cloned at `~/projects/2026/bay-area-twin`; no `project.toml` yet):
  one script, `scripts/prepare-from-warehouse.py`, reads `buildings/massing/san-francisco-bay/2026-11-28` and
  `roads/osm/san-francisco-bay/2026-11-28` from the public store, checks the catalogue's checksums, and writes the
  assets in the formats the page already read. Three fetch scripts removed, two trimmed to elevation and trees (run
  from an empty cache, they reproduce the committed elevation and trees byte for byte). 23,850 buildings drawn (was
  16,575), 13,295 with San Francisco's lidar height; 28,653 roads. Pushed as `5d41cc1`; its devlog has the account.
  A push to `main` there publishes to GitHub Pages, which digitaltwins.wiki embeds. The first publish failed in the
  workflow (`upload-pages-artifact@v3` no longer pairs with `deploy-pages@v5`); raised to v5, published, checked live.

Not done there: elevation and bathymetry, streams, Marin vegetation and street trees are still the model's own
fetches, since the warehouse has no product for them.

A fault seen in the warehouse's massing: a low building whose outline overlaps a tower's surveyed footprint takes the
tower's height (Golden Gateway Garage, two storeys, 115.7 m). The survey match should weigh overlap by area, or
refuse a height far above what the storeys say. Added to `todo.md`.
