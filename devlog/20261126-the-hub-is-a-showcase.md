# The hub is a showcase, and Earth 2100's pictures

26 November 2026. Claude (Fable 5.2), esimv1 room. Decisions are Anselm's, in his words where quoted.

## Decided

- **esimv1 presents scenarios; it is not where they are written.** "rather than scenario authors logging
  into esimv1 (in the future) I feel like esimv1 should be more a presentation layer that shows several
  nice projects." A scenario is a repository; the hub lists repositories and shows them.
- **No sign-in.** "I had thought sign-in and handles were a good idea. I now believe this is not a good
  idea. I don't think it is possible to offer a single web based experience right now that lets anybody
  trivially cut/paste remix - this product is not yet a consumer saas product. But it can at least
  showcase good scenarios."
- **Paths stay.** "The path concept is still relevant - that we have say /anselm for example."
- **The small projects stay in this repository.**
- **The hub rebuilds the scenarios it lists** ("hub may as well rebuild scenarios"), so a run on the site
  is one the hub made from the listed version.
- **A scenario's main page is fine as it is** (he looked at `/anselm/palisades-2025`).

## Not done yet, following from the above

- The account code is still in the tree (`web/lib/attest*.mjs`, `web/src/lib/{attest.js,Account.svelte}`,
  the handle rules in `server.mjs` and `NewProject.svelte`, `web/scripts/attest-e2e.mjs`). It is inert
  unless the server is started with `ESIM_PUBLIC=1`, which the deployed copy is not. To delete.
- The deploy does not rebuild runs; it uploads the run folders that are on canada1. A rebuild step
  (for each row of `parts.list`: check out the tag, `esim project run` every variant, upload) is to write.
- `parts.list` has no row for a scenario with its own site (Sausalito) and the front page has no card
  that links out.

## A note for the engine: thin wiring

Anselm, looking at a Palisades (Rothermel) run: "the modules and relationships in the scenario seem
almost degenerate - we will want to argue for richer independent modules wired together". That run is
one fire module, four records it is compared with, and a suppression module switched off. The fire
module reads its own weather, fuel, terrain and buildings, so nothing is wired to anything. The richer
form would have weather, fuel moisture, terrain, buildings and wind as modules of their own that the
fire subscribes to, as Portland and Earth 2100 already do. In `todo.md`.

## Earth 2100's pictures (his list, and what was done)

- "We show a globe - that isn't working well for me ... we may have to go back to a flat map", and the
  coloured borders "I cannot visually discern what is going on". The globe is replaced by a flat map,
  `viewer/src/RegionMap.svelte`: every country filled by one number. `Globe.svelte` is still in the
  viewer, unused, until he says to delete it.
- "why most african states start 'red' on day 1": the globe filled a country red when its weakest group
  of bands was under a floor, and the bands are percentile ranks among today's countries on World Bank
  indicators. A low income rank is not harm and was the same in 1850 as in the record. The new map's
  default is the change since the run's first table, so every country starts grey and reddens as heat
  and shocks move its bands. The rank and the share of people under heat stress are the other two choices.
- The niche map "showing blue for areas of concern": fields that are harm (outside the niche, wet-bulb
  days, heatwaves, deaths, damage and the like) now take the warm ramp (`rampFor` in `viewer/src/ramp.js`).
- "what is the 'country_index' map for": it is a lookup, the position of each cell's country in the
  people module's table, used by the country layers and now by the flat map. It was never meant as a
  picture; frames whose unit is `index` are no longer listed as maps in a module's panel.
- "labels are a bit cryptic as in 'energy.twolayer' - I wonder if modules need an english summary":
  `Describe` has a `title`, set for every kind by `title_of` in `core/crates/modules/src/lib.rs`
  ("Heat in the ocean"), with a test that no kind is without one. The wiring diagram leads with it and
  keeps the kind beneath; a module's panel shows it before the name. A run carries titles once it is
  rerun; Earth 2100's runs were rerun (log digests compared before and after).
- New page tag `<esim-regions>` for the map; both planet pages open with it.
- "we may need to move this to its own repo": it has been `anselm/esimv1-earth2100` since 22 November.

## Limits of what was done

- The flat map is one-degree cells, so small countries and coasts are blocky, and a country smaller
  than a cell may not appear.
- The titles are mine and unreviewed.
- Other projects' runs do not carry titles until rerun; their diagrams show the module's name as before.
- Nothing is deployed.
