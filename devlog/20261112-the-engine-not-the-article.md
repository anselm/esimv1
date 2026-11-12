# The engine, not the article

2026-11-12, night. Anselm, on the Portland page's "a scenario, not a forecast" and its lifelines
drawn from published ranges: "I want to brute force simulate near term outcomes - I do not want
just a report. I want something similar to the palisades example - the goal of this engine is to
build out capability to simulate the future and playtest outcomes." Then: "we need to redo this or
run a real engine in parallel. This should be very clearly in our design doc - we don't want to just
write 'articles' we want to exercise simulations - even if wrong."

## The principle

A page on this site exists to exercise a simulation, not to arrange published numbers into prose.
The brief of 2026-10-31 asks for modules of science on one clock with ledgers that close; this
entry adds the test a module must pass to count as one: it computes its outcome from the ground,
the record and the clock, cell by cell or component by component, and it can be wrong in a way a
record can catch. A number copied from a report, or a day drawn from a published range, is an input
or a check, never the output. Where the engine cannot yet compute a thing, the page says so and the
thing is the next rung, not the page's content.

In practice:

- **Compute first, arrange only as scaffolding.** A lifeline is components with fragilities on the
  ground and crews on the clock, not a ramp to a day from a table. The table is what the computed
  curve is checked against.
- **Wrong is allowed, arranged is not.** The Palisades engine was eight times too slow on its first
  day and the page said so; that is a simulation. A clock drawn to land inside the Resilience Plan's
  range is an article with a slider.
- **Run the real engine beside the arranged one** while both exist, as variants of one project, so
  the reader sees the computed curve against the range it was supposed to fall in.
- **Playtest through the engine.** A cut, a retrofit, a bridge, a crew rate: each a parameter the
  modules read, run over seeds as an ensemble, compared under one cursor. Not a paragraph.
- **Claims tables are inputs, not models.** The aftermath's rates from other disasters are declared
  for what they are and are replaced by computed queues (hospitals with beds, people who leave) as
  those are built.

## What this changes tonight

The Portland project's lifelines module was the arranged kind: twelve crossings with days drawn
log-uniformly in "weeks to years", power and water as straight ramps to a day from the Plan. It is
being redone as computed modules on DOGAMI's own impact bundle: `power.poles` (every transmission
structure in the window with the state's chance of failure at it, corridors chained from the poles,
crews repairing a stated number a day, the people-weighted share computed), `roads.segments` (the
emergency route segments with the state's ground-deformation class at each, closure days by class,
reachability of each hospital and the hub along open segments and open crossings). The old ramps
stay as the `ranges` variant, so the computed curves are read against the Plan's one to three
months. Fire following on the cell engine, the spill as a flow, and people as queues are the rungs
after, in `20261112-the-clock-as-a-film.md`.

The 2100 project is to be read against the same test: `policy.capacity` and `score.regions` move
bands by declared rules, which is the arranged kind, and the devlog of 2026-11-09 on effects as
claims is the way through.

## The first computed run (later that night)

Built: `hazard.components` (`crates/modules/src/components.rs`), tanks, crossings, poles and route
segments as components on the ground at their cells, from `tanks.csv` (the county study's Appendix B,
parsed from the PDF; 228 tanks carry the study's coordinates, the rest sit on OpenStreetMap tank
footprints by property), `poles.csv` and `routes.toml` (DOGAMI's bundle, cut with GDAL, the route
graph with the crossings as links), and `fragility.toml` (recalled Hazus curves, declared). The
lifelines module has `inputs = "computed"`: the spill is the tanks' sum, the crossings are the
components' states, the routes are reachability from the hub's segment, the power is the poles'
people-weighted share and each hospital's own cell; `ranges` is the old behaviour as a variant.
Preconditions are a parameter: `force = ["tank:KMW128:4"]` sets a component's state regardless of
the draw, which is Anselm's "oil tank as an independent agent coupled to the ground".

What the computed base run says, against its checks:

| | computed | the check |
|---|---|---|
| tanks failed, release | 233 of 587, 61 M gal (26 M to the river) | the study: 95 to 194 M gal |
| crossings standing on day 1 | 8 of 12 | the county: none expected usable at once |
| poles failed, power back | 586; half at once, all by day 13 | the Plan: 1 to 3 months |
| routes open on day 1 | 22 percent of length | DOGAMI: 74 percent of segments at risk |
| deaths after the shaking | 2 | the ranges run: 66 |

Every row is the engine wrong in a way the record catches, which is the principle above working.
The tank curve under-fails where the study's rule says every old tank on deforming ground goes;
the bridge curves are generic highway bridges, not Portland's, and ODOT's assessments are the
input; a pole is the whole grid here, with no substation, line or distribution to fail, so the
crews win in two weeks. The arranged run lands in the ranges because it was drawn from them.

Anselm, while this ran: simulate people as super-persons, one agent for ten, 63,900 in reach? How
do other scientists simulate large populations, trees, crowds? What do Cities: Skylines' people do?
Answered in the session; the short form for the record: the super-individual is a named method
(Scheffer and others, 1995, recalled) and the engine's daily tick makes a hundred thousand agents
cheap; the data for each is already here (DOGAMI's occupants by cell day and night, emPOWER's
device-dependent share, the census by tract for ages); the rungs are home and work cells, a state
machine driven by the services at the cell, then trips on the route graph. Synthetic populations
from census marginals (the RTI and the MATSim practice of a 10 percent sample scaled up) and
individual-based forest models (gap models, cohorts) are the precedents; the Skylines agents are
a fixed cap of citizens with home, work and leisure and path-finding on the road graph, which is a
small version of the same thing. The goal stated tonight: make the point that sims are possible,
for funding from the extended peer group; the compute later is significant.

## One more rev (later still)

Anselm: "Let's push hard on these features one more rev. Have the docs reflect status. Real bridges
would be nice. Hmm - unsure if we should try use real people - hard to estimate computational cost -
we don't have a super person generator yet. After this I will return to studying revenue funding and
sustainability."

- **Real bridges.** The FHWA National Bridge Inventory's Oregon file (delimited, 8,305 structures)
  gives every Willamette crossing but the railway bridge its year, type, length, condition and
  traffic; those are `nbi_*` rows in `bridges.toml`. The county's 2014 *Seismic Vulnerabilities and
  Retrofit Report* on its four downtown bridges gives the levels (0.20 g for the 500-year Cascadia,
  0.28 g for the 1,000-year) and says the un-retrofitted bridges' columns, footings and bearing
  seats are vulnerable at both, with liquefaction under the approaches unmitigated. A `seismic_class`
  per crossing picks a curve in `fragility.toml`: vulnerable (a judgement anchored on the report's
  two levels; the state and railway bridges share it by age) or modern (Sellwood, Tilikum). Result:
  eleven of twelve close on the first day, three back by day thirty, five by day 120. The county's
  "weeks to years" is now computed from a class rather than drawn from the phrase. What is still
  not read: ODOT's own assessments of the Fremont, Marquam, Ross Island and St. Johns bridges.
- **The super-person generator.** `people.agents` draws `scale`-sized agents from the occupant
  rasters by the seed (home by residents, work by daytime occupants, a device-dependent share from
  emPOWER), reads the buildings' per-cell death and displaced rates (now world fields written on the
  toll) and the components' power service at the home cell, and reports the cost. At ten people an
  agent: 93,520 agents, totals within two percent of the cell model, twenty million agent-days a
  second on one core, a run of 2.6 seconds. Every person separately is `scale = 1`, about ten times
  the memory and the same order of time; the cost that is hard to estimate is the behaviour, not
  the bookkeeping, and that is the next rung: a decision rule per agent and trips on the route graph.
- **Docs.** The root README (forty-six kinds, the earthquake in the module list, the status), the
  modules and layers pages, the limits page's earthquake section rewritten as the status of record,
  the project README, the record folder's README, CLAUDE.md.

## The collapses drawn (later)

Anselm: "Let's start the sim before the fault. Indicate date of quake clearly. Deaths seem very very
low? Include deaths at quake itself - can we model using collapsed buildings? Use super sized people
now."

- The run already starts a week before; the clock now says on every moment whether it is before,
  during or after the earthquake and prints its date and time beside the relative label.
- On the number: 765 in the window was DOGAMI's own figure for this ground, fitted (810 in
  Portland's nine districts; 1,473 across the three counties by day; the Resilience Plan's 1,250 to
  more than 10,000 is statewide with the coast). The runs are fitted to it, so the page does not make
  it larger; what it now does is draw it.
- `quake.buildings` draws at every cell how many of its buildings collapse (a binomial at the cell's
  collapse share, by the seed; `binom` in quake.rs, exact under 64 buildings, normal above), writes
  `buildings_collapsed` as a frame and the realised collapse share, the collapse death rate and the
  standing-building death rate as world fields, and reports the collapsed count and the dead in
  them. Base run: 1,047 collapses, 974 of 1,000 dead in them; seeds 2 and 3: 998 and 1,030
  collapses, 564 and 602 dead. The expectation is unchanged; the variance is the finding.
- `people.agents` puts each agent in a collapsed or a standing building by the cell's realised share
  and kills it at that building's rate; it publishes `agents.toll`, and `people.aftermath` lets the
  agents' dead carry `deaths.total` when an agents module is present. At ten people an agent the
  agents' count is lumpier than the cells' (670 against 1,000 in the base run), which is the price
  of the scale and goes away at `scale = 1`.
- Noted for the fit: the collapse casualty rate is 0.05 recalled, tripled by the fitted scale to
  0.15; the tripling was fitted on the expected model and should be refitted on the drawn one,
  which is a sweep for another day.

## The date, said (later)

Anselm: "Can we very clearly show the date of the event and say the date clearly? Show a vertical
red line in the timeline. Move the event date to march 2027 also." Done: the scenario is now Friday
26 March 2027 at 2 PM Pacific daylight time (21:00 UTC; the run starts a week before and ends a year
after; the wet-night variant at 2 AM PDT), on soils still saturated from the winter rains. The ground
module gained `tz_offset_h` and `tz_name` for its words and puts the local time in the rupture
crossing's data; the clock says "Before the earthquake of Friday 26 March 2027 at 14:00 PDT" on
every moment, and a red line marks the instant on the timeline and on every chart. The 1700
anniversary framing in the manifest's notes went with the date; the README's deep history keeps
the 1700 event as what the fault did last time.
