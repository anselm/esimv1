<!-- order: 10 -->
<!-- section: Projects and the record -->
# Scoring and tearing

Two different questions. Scoring asks how well a model series matches a record. Tearing
asks whether the system in the run is holding together. Both are measured by code that
is not the model's.

## Scores

A manifest names pairs of a model series and a scoreboard. At every matching time the
pair is compared and five numbers are kept: Nash-Sutcliffe efficiency, Kling-Gupta
efficiency, Pearson correlation, percent bias, root mean square error. A high correlation
with a large bias (the planet's carbon, r 0.995 and 7% high) is the common case and
means the shape is right and a sink is missing; the numbers are shown together for that
reason.

## The auditor

An independent crate reads a finished run folder and never the live run, so no module
can arrange its ordering to look good. It holds no theory of any system. It measures:

- **Spend-down.** Every ledger stock's balance at every close is in the run. A stock that
  falls across the run is being spent; one that rises is accumulating. The verdict is per
  account, with the change over the run and over its last quarter.
- **Diversity.** Shannon evenness and effective number over the population stocks (a
  ledger in individuals). With one population it says so.
- **Shocks and recovery.** Shock events from the log (`fire.burn` and anything under
  `hazard.` by default; a manifest's `[audit] shock_topics` changes it) grouped into
  episodes. For every stock, the drop after the shock and the time to return within 5%
  of its pre-shock mean, compared with the interval to the next shock.
- **Crossings.** Every `*.crossing` event a module published: a country leaving the
  human climate niche, the warming entering a tipping element's range, each with its
  source and confidence. Listed, counted, not judged.
- **Tears.** Extinctions (a population at zero that stays there), failed ledger closes,
  shocks not recovered before the next or a year on.
- **Silence.** What it could not measure in this run and why: no second population, no
  shock topics, no log. It says so instead of printing a zero.

The verdict is one of three words. **Holds**: no shocks, or none that moved anything.
**Stretching**: shocks that were recovered from before the next one. **Tearing**: a tear.
The definition of tearing is recovery slower than recurrence, which needs no model of
the system, only its record; it is the one measured meaning the word has here.

## What the auditor refuses to do

Produce a stress index without sources. Assign a country a collapse date. Call a season's
drying a shock response without saying that a one-year run cannot tell seasonality from
trend (it says exactly that in its limitations). Extrapolate.

## Thresholds with provenance

Where the literature gives a threshold, the system carries it as a table row with a
source and a confidence, and a module compares the run against it and publishes a
crossing. `data/events/tipping.toml` holds the climate tipping elements this way; today
every row is marked recalled and unchecked, and the page shows that word. A country's
human-niche threshold is a parameter with a paper named in its note. The hottest-month
marker next to it has no paper and its note says "guess". The reader can tell which is
which, which is the whole point.

## Precedents instead of indices

For heat the records hold events people remember and their tolls. A country's exposure
in a run can be shown against the exposure of those years rather than against an
invented index. "Your country's ordinary summer in 2060 is the summer of 2003" is
checkable; a riot date is not. This is on the list, not built.

## Stress boards

The hormuz simulator's country board (one row per country, a mark per sourced threshold
crossed at the scrubbed time, ranked by count) is the right reading view over the
crossing events, and the breaking-points row (flow, function, sourced threshold,
consequence) is the right shape for every new threshold layer. Both are planned here.
