<!-- order: 1 -->
<!-- section: Building a twin -->
<!-- author: Claude -->
<!-- ordered-by: Anselm -->
<!-- written: 2026-11-21 -->
<!-- revised: 2026-11-21 -->
<!-- revision: 1 -->
# How to write your own digital twin

This guide is for a person about to build a simulation of a real place or system, and for
a language model helping them. It is written to be loaded whole into a model's context:
the plain Markdown file is at `/docs/guides/writing-a-digital-twin.md`, and each section
can be read on its own.

It is about how, not what or why. "Digital twin" is used loosely here for any running
model of a real place or system that is tied to real data: a watershed, a fire, a city
after an earthquake, a fishery, the planet's carbon. The reasons to build one are many and
are covered better elsewhere (see Further reading, at the end). Briefly: a twin
lets people ask "what happens if" about something too large, slow or dangerous to try. The
questions can be local (will this fire reach this house, where does the sand go if the
breakwater is removed), regional (how long is a city without fuel after its bridges close)
or global (how many people live outside a livable climate in 2100 under each emissions
path). A twin does not answer such a question. It lets people explore possible answers and
see what each one rests on.

The guide is not a manual for esimv1, the simulator this documentation belongs to. It
describes practices and problems that hold for any engine. Where esimv1 made a choice, the
text says so and gives the reason, so a reader can choose differently.

## A simulator, not a replay

The most important distinction in this guide is between showing what happened and
computing what would happen.

A recording of the past, however well drawn, is a visualization. Satellite frames of a
fire played in order, a flood animated from gauge readings, a dashboard of last year's
traffic: each is useful, and none is a simulator. Nothing in it can answer "what if the
wind had turned" or "what if the levee were a metre higher", because nothing in it
computes.

A simulator starts from a state of the world and a set of rules, and computes forward to
outcomes that have not been observed. It can be given a state that never existed and a
choice nobody made, and it will produce a result. That result can be wrong, and a record
can show that it is wrong. Both properties are the point.

Tests for which one you have:

- Change an input. If the output cannot change, it is a replay.
- Ask where a number on the screen came from. If it was copied from a report, it is an
  input or a check. If it was computed from the ground, the rules and the clock, it is an
  output.
- Ask whether it could be wrong in a way a record could catch. A replay cannot.

The past still matters to a simulator in two ways: as the starting state, and as the
record a run is checked against. Draw the record beside the model. Do not let the record
stand in for the model.

## Use what exists

Few twins need to be written from nothing. There are open engines for most physical
processes (hydrology, fire, ocean, atmosphere, traffic, crops), open platforms for hosting
and display, and far more open data than any one project will use: global elevation, land
cover, decades of weather reanalysis, satellite archives, census grids and building
footprints, most of it free and much of it without a key.

Look before building. The [Digital Twins Wiki](https://digitaltwins.wiki) is collecting
as much of this as it can; when read for this page it listed 112 engines and 100 data
sources, with platforms, ontologies and notable twins beside them. Borrowing a tested
hydrology code and spending your effort on the question, the coupling and the telling is
usually the better trade. When you borrow, check the licence, record the version, and run
the borrowed part against a record yourself.

## Start from one question

Write down the question the twin exists to explore, who will act on what it shows, and
what they would do differently. Almost every later choice follows from that: the area, the
resolution, the time span, which parts of the world are modelled and which are left out.

A twin with no question grows without limit, because the world has no edge. A twin with a
question can be small. "Does the fire reach this address, and when" needs terrain, fuel,
wind and one fire model. It does not need hydrology.

Then build the smallest version that runs end to end: one place, one scenario, one output.
Extend it where its users push.

## Decide how good and how fast

The real world is very hard to model. A simulation that tries to be faithful in every
respect can cost a great deal to build and a great deal to run: weather centres spend
supercomputers on one forecast. Most projects cannot do that and do not need to.

The work is to find a representation simple enough to compute and still good enough for
your specific question. This is what programmers are usually asked to do: build a model
that is good enough. "Good enough" has to be stated to mean anything. Say what error is
tolerable for the decision at hand (the right valley, the right day, the right order of
magnitude), then stop refining when a check against the record meets it. A coarse model
that runs a thousand times and shows its spread is often worth more than a fine one that
runs once.

Decide early how fast the twin has to run, because it changes the design.

- **Faster than the event, for operations.** A person steering a live situation, such as
  a fire, a flood or a grid under strain, needs an answer in seconds or minutes, updated
  as conditions change, so that an intervention can be tried in the model before it is
  tried in the world. This calls for simple physics, coarse grids, precomputed tables or a
  learned stand-in for the slow parts, and a link to live data.
- **Slower, for planning.** A team weighing options over weeks can wait hours for a run.
  This allows finer physics, many runs with different seeds and parameters, and a careful
  comparison of scenarios.

The same model rarely serves both. Naming which one you are building is part of naming
the question.

## The core: how to arrange a simulation

Most simulation engines, whatever their subject, have the same few parts. Keeping them
separate is the single most useful structural decision.

### Shared data

Some resources are used by many parts of the model and belong to none of them:

- base layers such as elevation, land cover, coastlines, roads and building footprints;
- large observational datasets such as weather reanalyses, satellite products, river
  gauges and census tables;
- adaptors that talk to outside services such as Google Earth Engine, a weather API or an
  open-data bucket.

Put these behind one layer that fetches, caches and describes them. Practices that repay
the effort:

- **Record where each file came from**, its licence, and the day it was fetched. Pin it by
  hash so a run can say exactly which bytes it read.
- **Fetch before the clock starts.** A run that downloads in the middle is slow and cannot
  be repeated when the service changes.
- **Agree on coordinates and units once.** Reprojection and unit errors are the most
  common silent faults. A grid laid half a pixel off its source once gave our fire model
  cliffs where there were none.
- **Check a reader against a second tool** the first time it is used.
- **Use real records.** A synthetic stand-in that looks plausible will be mistaken for
  data later, including by you.

### Modules

A module holds the behaviour of one part of the system over time. In practice it is a
science library with a clock: an algorithm that does the heavy work for irradiance,
hydrology, fire spread, a crop, a population of deer, a power grid, or a single animal.

A good module:

- does one thing and owns one kind of state;
- names its inputs and outputs, with units;
- names every parameter, with a default and a source, or the word "guess";
- has an area and a resolution it applies to, which may differ from its neighbours';
- can be run alone against a record of the thing it models.

The Grammar of Simulation, an atlas of the mechanics in 179 simulation games, uses nearly
the same test for a part: it "owns one kind of state and one update rule", and it can be
named, tuned and removed. The test is useful outside games. If a part cannot be removed
without breaking the others, it is not yet a module.

### Choosing how to represent things

The same subject can be modelled in different ways, and the choice matters more than the
code.

- **As a field.** A quantity that has a value everywhere: water depth, soil moisture,
  vegetation density, people per square kilometre. Fields are cheap, conserve quantities
  easily, and suit things that are numerous and alike. Flora and fauna can be treated this
  way, as densities that grow, move and are eaten, much as hydrology treats water.
- **As individual agents.** Each animal, household, vehicle or firm is a separate entity
  with its own state and rules. This suits cases where differences between individuals,
  encounters between them (predator and prey), or their decisions drive the outcome.
  Agents are costly in large numbers, and their results vary from run to run.
- **As a network.** Roads, rivers, pipes, power lines and trade links, where what matters
  is what is connected to what.
- **As an aggregate.** One number for the whole: a planet's carbon stock, a city's fuel.

The atlas puts the trade plainly: "Aggregates are cheap and faceless, agents are expensive
and legible." A practical middle is the super-agent, one entity standing for a hundred or
a thousand like individuals. Most real twins mix forms, and the hard part is the seam
between them: a field of grass eaten by individual deer has to lose exactly what the deer
ate.

State then has to be stored so that it can be found by place. Common encodings:

- regular grids of cells, in latitude and longitude or a projection;
- hierarchical cells such as quadtrees, H3 hexagons or S2 cells, when resolution varies;
- spatial hashes or trees (a grid of buckets, an R-tree, a k-d tree) holding individual
  agents;
- graphs for networks.

Spatial query is central whichever is chosen. "What is near this point", "what is inside
this polygon" and "what lies upstream" are asked constantly, by modules of each other and
by the display. Decide early how those questions are answered and make the answer fast.

### Time and messages

Parts of a model need to tell each other things. Two mechanisms cover most needs.

- **Shared state.** One module writes a field (the burned area) and others read it.
- **Messages.** One module announces an event (a bridge has closed) and any module that
  cares hears it. A tick of the clock is itself a useful message: "an hour has passed".

Processes run at different rates. Fire moves in minutes, a river in hours, a forest in
years. Let each module choose its own step, and be explicit about the order in which
modules act within one instant, because order changes results.

esimv1 relies heavily on late binding through a message bus. A module does not call
another module. It publishes to a named topic and subscribes to others, and the wiring is
settled when a scenario is loaded. The fire engine does not know that a suppression module
exists; it only hears that certain ground is now held. The cost is that the flow of cause
is harder to read in the code, so the engine records every event in a log and checks each
module's stated topics against what it really sent. Direct calls between modules are
simpler and are a reasonable choice for a small model.

Keep the kernel small. A clock, a way to pass messages, and a few services such as
randomness and accounting are enough. Knowledge of any subject or place belongs in a
module.

### A manifest for each scenario

A scenario should be a document, not a state of the code: which modules, with which
parameters, over which place and time span, from which random seed. This document is
often called a manifest. With one:

- a run can be repeated exactly, by anyone;
- two scenarios can be compared by comparing two files;
- an intervention is a changed parameter, so "what if" is a second manifest;
- a language model can draft a scenario without touching code.

Keep the person's own description of what they wanted beside the manifest. When the two
disagree, the description is the claim and the manifest is the attempt.

### Three habits that catch errors

- **Balance the books.** Where something is conserved (water, carbon, fuel, people), pass
  every movement through an account that must close. This catches errors no chart shows.
- **Make runs replayable.** Draw randomness from a seeded stream. Keep the manifest, the
  data hashes and the event log with the results.
- **Run more than once.** Any result that depends on chance or on an uncertain parameter
  should be shown as a spread over many runs, not as one line.

## Check it against what happened

A model that has never been compared with a record is an illustration. Run it on a past
event and compare.

- Keep the measuring code separate from the model, so the model cannot flatter itself.
- Compare more than one thing. A fire model can match the final burned area and be wrong
  about the shape, or about the hour.
- When fitting parameters to a record, hold part of the record back and test on it.
- Do not move a parameter by hand until a chart looks right. Move it by a stated method
  and keep the record of having done so.
- Treat a failed fit as a finding. When no setting of our fire model matched both the
  first night and the later days, the lesson was that the model lacked a way for a fire to
  stop. When firefighting was added, the fire stopped at the right size but far too
  suddenly, and a series the fit had never seen, the reported containment, was what showed
  it.
- Say which level a claim is about: the world, the scenario, the model, or one run.
- Publish the known faults next to the results.

A run past the end of the record rests on assumptions. Name them where the numbers are
shown, and do not call the run a forecast.

## Keep it tied to the world

A check against one past event is a start. A twin that is meant to stay in use has to be
kept in step with the world and with its own earlier results.

- **Live sensors.** Gauges, weather stations, satellites, cameras and meters can feed the
  model as it runs. This link is what some fields mean by the word "twin". Treat each feed
  as data like any other: record its source, expect gaps and faults, and decide what the
  model does when a sensor goes quiet.
- **Data assimilation.** A running model drifts from the world. Assimilation corrects its
  state toward new observations as they arrive, weighing how much to trust the model
  against how much to trust the measurement. Weather forecasting depends on it. Simple
  forms, such as restarting from the latest observed state or nudging toward it, go a long
  way. Keep a version of the run that was not corrected, so the model's own error stays
  visible.
- **Forecast, then check.** Issue a dated forecast, keep it unchanged, and compare it with
  what happened. Repeated over months this is the strongest evidence a twin can offer.
- **Regression tests.** Keep a small set of reference runs with their expected results,
  and rerun them on every change to the code or the data. A change that moves a result
  should do so on purpose, and the reason should be written down. Without this, a fix in
  one module quietly breaks another, and a new version of a dataset changes answers that
  nobody rechecks.

## Show it

A running simulation that nobody can see persuades nobody. Visualization is how the work
reaches people who do not read code or tables, and most of the people a twin is for are
in that group.

Three-dimensional graphics are especially effective. Terrain with the model's state draped
over it, moving through time, lets a person recognise their own valley and see the fire
come over the ridge. To people who have not worked with 3D it can look like a rare skill.
It is not hard: a height field, a camera, and points or meshes coloured by the model's
values go a long way. esimv1 uses three.js, which is easy to work with on the web and runs
in any browser. Bevy, a Rust engine, is also good, especially when the simulation itself is
native code. Game engines and map libraries such as MapLibre, deck.gl and Cesium are other
routes.

Guidelines:

- Show time. A scrubber or a play button turns a map into a story.
- Draw the record beside the model wherever one exists, in a visibly different style.
- Use charts for quantities over time, and tables for exact values. A chart of the
  model against the record is often the most informative picture there is.
- Make each colour and mark mean one thing, and say what it means next to the picture.
- Let layers be switched on and off. A complete picture is rarely a readable one.
- Do not let the picture claim more than the model knows. Smooth rendering makes a coarse
  guess look like a measurement.
- Tell it as a story with a before, an event and an after. The drama should come from the
  results.

## Intentions, outcomes and bottom lines

Visualization is still not the purpose. A simulation meant to communicate has to do three
further things, and most simulations do them poorly.

**Let people state their intentions clearly.** A person should be able to say what they
want to try (cut the fuel stored by the river by a fifth, retrofit the old bridges, reach
zero emissions by 2050) in words close to their own, and see that it was understood. A
panel of forty sliders does not do this. Named interventions that map to stated parameter
changes do.

**Show results clearly.** State the outcome of each choice in a sentence, with a number
and its spread, beside the outcome of not making it. Put the comparison first and the
detail behind it.

**Measure outcomes on several bottom lines.** Money alone hides most of what matters. Keep
separate accounts for lives, health, homes, livelihoods, ecosystems, public trust, and
whatever else the people affected name. Report them side by side. Do not fold them into
one index, because the weights in an index are a political choice presented as arithmetic.
Say who bears each cost: an average can improve while one neighbourhood is ruined.

Simulations are often weak at showing impact and risk. They give a central line where the
decision depends on the bad tail. Show the spread, say how likely the bad cases are as far
as the model can tell, and say what they would cost.

They are also weak at showing the difference between a system stretching and a system
tearing. Stretching is gradual strain: a little less food each year. Tearing is a change
of kind: a population going extinct, a food riot, a government falling. A model built
from smooth equations will show stretching for ever, because nothing in it can break.
Things to do about that:

- Give the model thresholds, with a source for each, and report each crossing as an event.
- Watch for recovery that is slower than the interval between shocks. That is a
  definition of tearing that needs no theory of the system, only its record.
- Say plainly which kinds of break the model cannot represent. A model with no politics
  cannot show a government falling, and its silence on the matter is not reassurance.
- Compare with precedents people remember ("an ordinary summer in 2060 is the summer of
  2003") in preference to an invented stress score.

## What surrounds the simulation

Data, ownership and the people affected surround the core, the pictures and the story.
These questions are best asked before building, and again before publishing. Write the
answers down and keep them with the project.

**Purpose and change**

- Who asked for this, and who pays for it?
- What decision or understanding should change because of it, by whom, and how would you
  know that it had? This is the theory of change.
- What happens if it goes well? What happens if it goes badly, or is believed when it is
  wrong?
- If it succeeds and has influence, what does that crowd out? A persuasive model can
  silence other voices, other ways of knowing a place, and options that were never encoded
  in it.
- Who could misuse it, and for what?

**People**

- Who is affected by the decision, and have you talked to them? They hold data and raise
  questions the builders miss.
- Do the people represented in the model have any say in how they are represented? Can
  they correct it, contest it, or leave it?
- Are people modelled as having agency, or only as things that happen to be in the way?
- Is anyone being spoken for by the model, including by a language model playing a
  person? Such stand-ins give too narrow a range of views and should not replace asking.

**Data**

- Where does each dataset come from, who owns it, and under what licence?
- Is it trustworthy? Who collected it, how, how old is it, and what does it leave out?
- Does it contain personal information, or let individuals be found by combining layers?
  Decide what not to collect, aggregate the rest, and say what you did.
- Does it belong to a community with its own rules about use, such as Indigenous data?
- Could publishing a location cause harm (a rare species, a vulnerable household, a
  critical facility)?

**Openness and upkeep**

- Should it be public? All of it, or only the results?
- Can someone else rerun it and check it?
- Who owns it once it is built, who keeps the data current, and who pays?
- What happens to it when the decision has been made? Either give it an owner or retire
  it visibly. An abandoned twin that still looks live is worse than none.

## An order of work

1. Name the question, the user and the decision. Confirm it needs a simulator and not
   a replay. Say what "good enough" means and how fast it must run.
2. Talk to the people affected.
3. Answer the questions in the section above, in writing.
4. Find the data and the existing tools, and record every source.
5. Choose the representation for each part: field, agents, network or aggregate.
6. Build the smallest version that runs end to end.
7. Check it against a past event. Publish where it misses.
8. Add the display: map or 3D view, time, record beside model.
9. Add interventions as named parameter changes, and outcomes on several bottom lines.
10. Put it in front of its users and watch what they misread.
11. Decide who keeps it alive. If it stays live, add the sensor feeds, the dated
    forecasts and the regression tests.

Keep a dated log of decisions throughout. A year later the order of thinking is as
valuable as the code.

## For a language model reading this as context

When helping someone build a twin:

- Ask for the question, the user and the decision before proposing any structure.
- Check that what is being built computes forward from a state by rules. If it only
  replays recorded data, say that it is a visualization and ask whether that is the aim.
- Look for an existing engine, dataset or tool before writing one, and name it.
- Propose the smallest model that could address the question, and name what it leaves out.
- Ask what error is tolerable for the decision, and whether the twin must run fast enough
  for live operations or only for planning.
- For each part, state the representation you chose and the one you rejected, with the
  reason.
- Mark each number as sourced, recalled from memory, or a guess. Do not present a recalled
  value as checked.
- Do not invent data. If a record is needed and not in hand, say so and say where it
  might be found.
- Propose a check against a past event together with the model, in the same reply.
- When a result is shown, give its spread, its assumptions, and what the model cannot
  represent.
- Raise the questions under "What surrounds the simulation" if the person has not. Do not
  answer them on the person's behalf.
- Never describe a run that goes past the record as a forecast or prediction.

## Contributing

Corrections and additions to this guide are welcome, and so is input to the projects
around it. The wiki takes contributions through its
[repository](https://github.com/anselm/digitaltwin-wiki): a missing engine, a data
source, a twin worth noting, a correction to a draft entry. esimv1 is developed at
[anselm/esimv1](https://github.com/anselm/esimv1), and a module, a record to score
against, or a report of where a run misses is useful there.

## What was left out

This guide does not cover the methods of data assimilation in any depth, learned models
trained on data in place of written rules, the engineering of performance and parallel
computing, or the budget for running a service. Each deserves its own page.

## Further reading

- The [Digital Twins Wiki](https://digitaltwins.wiki) is an atlas of the field: engines,
  platforms, data sources, notable twins, terminology and articles. It describes itself as
  pre-alpha, and most entries are drafts not yet checked against their sources. Its nine
  [guides](https://digitaltwins.wiki/guides/) follow much the same order of work as this
  page, from naming the question to keeping the model alive.
- [The Grammar of Simulation](https://digitaltwins.wiki/articles/the-grammar-of-simulation/)
  is the wiki's account of an atlas by Yohei Nakajima, Pedro Lucca and Shrey Kothari
  (version 0.8, 4 November 2026) that breaks 179 simulation games into 199 mechanics and
  reads each game through six layers: substrate, dynamics, coupling, control, legibility,
  and pacing and stakes. The last three are about how intent enters a model and how the
  model explains itself, which is the subject of the section on intentions above. Its
  mechanics are tuned for play and are not calibrated to real places.
- On the wiki, [Inside an engine](https://digitaltwins.wiki/articles/inside-an-engine/)
  and [The parts of a digital twin](https://digitaltwins.wiki/articles/parts-of-a-twin/)
  cover the ground of the first half of this page.
- In this documentation: [Honesty](/docs/system/honesty) for the rules on sources and
  guesses, [Scoring and tearing](/docs/system/scoring-and-tearing) for the measured meaning
  of stretching and tearing, [Telling a scenario](/docs/system/telling) for the story, and
  [Limits today](/docs/system/limits) for an example of published faults.

## What this page has read

- The wiki's page on The Grammar of Simulation, read on 21 November 2026. The two
  quotations are as that page gives them. The paper itself was not read.
- The wiki's front page, its list of guides and its list of articles, read the same day
  for their titles and summaries. The guides and the two articles named above were not
  read in full.
- Everything else is drawn from building esimv1 and from general practice in simulation.
  It cites no source and should be read as experienced opinion.
