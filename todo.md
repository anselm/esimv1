# todo

Research projects and open work, one line each. The thinking behind each line is in the
dated `devlog/` entry it names; status lives there too, not here. Asked for by Anselm on
2026-11-01.

## Research projects (hindcasts against events with known endings)

- The end of the African Humid Period: an AMOC-linked monsoon step and the Sahara's
  drying, abrupt or gradual. `devlog/20261101-research-agenda.md`
- The Four Corners megadrought and the Ancestral Puebloan departure, from the tree-ring
  drought atlas. `devlog/20261101-research-agenda.md`
- The AMOC as a record (RAPID array) and as a conceptual box model with a real
  bifurcation, labelled conceptual. `devlog/20261101-research-agenda.md`
- The 2003, 2010, 2015 and 2021 heat events as precedents for country exposure.
  `devlog/20261101-the-2100-scenario.md`

## Making the planet page read as it should (Anselm, 2026-11-02; `devlog/20261102-why-it-reads-mild.md`)

- Records to date on the first screen: the hindcast's record years as a sequence with
  sources, before any assumption; the present tense is the most convincing thing we have.
- The deep-time axis: EPICA Dome C and Vostok CO2 from NOAA paleoclimatology as a context
  module, so the present is drawn as the spike it is.
- Paleo analogues: for a year's CO2, the last time the air was there and what the sea
  stood at then; numbers to be checked (Dutton et al. 2015, Pliocene literature) first.
- Extremes, not means: per country per year, how often the previous record month falls,
  and when the 2024 record becomes an ordinary year.
- A few named places in the bottom line, chosen by their crossings, each with its own
  sentence, instead of planet-sized totals alone.
- Coincidence: the count of stated limits a country has crossed this year, mapped zero to
  five; the stress board on the platform list is this.
- Cascade: consequence modules that subscribe to crossings and move a stock someone reads,
  in this order: permafrost and Amazon crossings into the carbon ledger; ice-sheet
  crossings into sea level; AMOC into regional temperature (its sign differs; keep it for
  that reason); country temperature into harvest; harvest and heat into migration and
  migration into who lives under which heat. Every parameter recalled until checked, and
  the feedbacks with the other sign (CO2 fertilisation, aerosols) included so the
  selection is not the frightening ones.

## Layers to build

- Humidity, then wet-bulb exposure, replacing the guessed hottest-month marker; the Gulf
  first. `devlog/20261101-research-agenda.md`
- Aerosol forcing from a published assessment series, cited. `devlog/20261101-the-2100-scenario.md`
- A box carbon cycle with real stocks in place of the impulse response, then the hindcast
  harness against Mauna Loa. `devlog/20261101-p1-planet.md`
- Ice and sea level beyond thermal expansion; a satellite altimetry scoreboard.
- A rainfall-reading layer, so that forest patches (the Amazon) can hang off it.
- Soil life and soil carbon as a layer (Anselm, 2026-11-01, after the New Scientist piece
  on the decline of Earth's microbiome). Honest form: soil organic carbon stocks per
  country or cell from SoilGrids (one snapshot, anonymous) spent down under cultivation
  and heat by published rates, with the harvest and the carbon ledger coupled; there is
  no global time series of microbial biomass to score against, so it would be a stock
  with a stated response, labelled so, until a record exists.
- Heat mortality with published exposure-response curves and their ranges, last.
- Check every `confidence = "recalled"` row in `data/events/tipping.toml` against the paper.

## Accounts and the namespace

- Real accounts through attest.monster (passkey identity; the handle is the area; a user
  may create only under their handle). About a day on the server; do it together with
  the public deployment, after aerosols and the carbon sinks. `devlog/20261101-docs-and-accounts.md`
- Crumple the namespace: handles at the top level (`/anselm/planet`, `/anselm/notes`), not
  under `/projects/`; `docs`, `system`, `runs`, `new`, `kinds`, `api` become reserved
  top-level names. An area then holds projects and pages alike. Path-based URLs in the
  viewer first, folders second. Anselm, 2026-11-01: flattening is cognitively simpler
  even with conceptual collisions.
- The swear-word filter from his other project, on area and project names only, when
  names become public.

## Platform

- A 1 km population layer (WorldPop) in place of people spread evenly over a country.
- A country stress board at the cursor (count of crossed markers per country, ranked), after
  the hormuz room's board; the breaking-points row shape for every new threshold layer.
  `devlog/20261101-people-layers.md`
- An overview world map per project; coastlines over world fields; a pin for "your place".
- A second conversational turn on a drafted manifest.
- Auditing ensembles.
- The wire binding and a stdio module host. `docs/protocol-v0.md`
- `scenarios/` as a curated ladder. `devlog/20261101-roots.md`
