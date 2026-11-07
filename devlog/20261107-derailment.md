# Knock-on effects in the 2100 scenario: derailment

Anselm, 2026-11-07: "Thinking again about the 2100 scenario - I feel we are still failing to
really consider knock on effects. What are some ways we can improve this scenario? Here is an
article discussing how some impacts reduce optionality", with James Dyke's piece in The
Conversation of 2026-11-05, "The worse climate change gets, the harder it becomes to stop it"
(Global Systems Institute, Exeter; read that morning through the fetcher, the coauthors not
named on the page).

The article's argument, as read: warming has effectively doubled its rate in a decade;
tipping points may cascade (an Atlantic shutdown starving the Amazon of water vapour, the
Amazon's dieback moving the carbon cycle, both speeding the ice); and, its own contribution,
"derailment risk": the consequences of climate change erode the capacity to address its
causes. Its mechanisms: a sovereign debt doom loop (extreme weather damages vulnerable
economies, debt service crowds out mitigation and adaptation finance); conflict, which
undermines the cooperation mitigation needs, stronger where inequality and polarisation are
high; political backlash (Vox after the 2024 Valencia floods); and the asymmetry that
"collective action can be easier to destroy than build". Its prescription: map the feedback
loops, build resilience against disruption, make governance responsive in unstable periods.

## What the model has, and the loop it lacks

Every knock-on the article names is a feedback from impacts to capacity, and the model has no
such loop. Emissions after 2024 are an assumption chosen by the variant and nothing in the
run can move them. The sinks feed back (Millar), three tipping elements feed carbon back,
and that is the whole set of feedbacks; the heat exposure, the wild cards and the region
scores are all downstream and nothing reads them upstream. The actors figure on the page
draws this as the dashed arrow from people to emissions, and the article is a paper about
that one arrow.

## Five ways in, in the order I would build them

1. **Derailment as a module, through a port that exists.** `carbon.emissions` already takes
   `emissions.scale { name, factor }` from the wild cards. A `policy.capacity` module computes
   a world capacity to cut, 0 to 1, each year from what the run already knows: the share of
   people under heat stress (score.regions), the damage of the year's wild cards, the voice
   band (institutions) and the means band weighted by population, and the asymmetry the
   article names (capacity falls fast on a bad year and recovers slowly). It publishes a
   factor that slows the planned decline: with capacity c and a planned path that cuts by
   r a year, the effective cut is c·r, so `zero_by:2050` becomes zero by later or never. One
   parameter set, declared as placeholders, and a variant `zero-by-2050-derailed` beside the
   clean one. The page then shows the gap between the plan and what the model's own impacts
   let happen, which is the article's claim in a chart.
2. **Cascading tipping.** `tipping.elements` checks each element against the warming alone.
   Add a coupling table to `data/events/tipping.toml`: when the AMOC passes its central
   estimate, the Amazon's thresholds drop by a stated amount; when Greenland passes, the
   AMOC's drop; West Antarctic and Greenland raise each other's chance. The interaction
   signs and magnitudes are in Wunderling et al. 2021 (Earth System Dynamics), recalled and
   to be read before any number is typed. Then a run can tip a chain, and the ladder shows
   the chain.
3. **A place for the pressure.** Debt, conflict and backlash are national, not global. The
   region scores already hold per-country stress and bands; give the capacity module a
   per-country capacity, let the wild cards take a region (the field exists, unused), and
   let conflict's annual chance rise with a region's stress and fall with its voice band,
   which is the article's conflict mechanism with its inequality modifier. The globe's red
   then has a cause beyond heat.
4. **The people's response.** Deaths and movers from the heat and the sea
   (devlog 2026-11-05), so that the population contracts where the stress is and the
   capacity module has a loss to read rather than an exposure count.
5. **Optionality as a number on the page.** The article's word is options. For each year,
   the remaining budget to a warming level under the current path, and the cut rate now
   needed to hold it, both of which the run can compute from its own series; drawn beside
   the capacity, the reader sees the gap open: what must be done rising while what can be
   done falls. That is the picture the piece is asking for and no module is needed, only a
   view.

Everything above is a placeholder in its numbers and honest in its shape; the first and the
fifth are a day, the second an afternoon once the paper is read, the third and fourth are
the larger work already planned. Not built this morning.
