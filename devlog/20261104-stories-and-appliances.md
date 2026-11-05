# Stories and appliances

Anselm, 2026-11-04, after reviewing the actors figure on the Palisades page ("this is good.
lets do the same for the 2100 scenario") and as an aside, in his words:

> although the tool we are exercising (by revising and improving modules over time) is
> intended to be entirely pragmatic - similar to say ESRI or ARCGIS - really just a software
> appliance - I am coming to appreciate that a lot of what is needed for novices encountering
> work like this in general is that they need a grounding context, a story. This was a similar
> motif that I arrived at when I wrote this essay on computational journalism:
> https://anselm.substack.com/p/computational-journalism and the topic of "presentations"
> became a shelf in my otherwise fairly sober/technical review of digital twin technologies
> (at digitaltwins.wiki). I want to note that what we are seeing here - with me asking you to
> elaborate more and more on the readme per project is that it reflects a real need or feeling
> i have that i need to help novices understand what the heck is going on.
>
> later this probably means that this project will probably use full blown html with our
> computational widgets embedded in that page - ie; we're almost certainly going to move to
> more of a story like capability... as scientists, researchers, lay-people, try to not only
> provide simulations but also tell a story along with that simulation.
>
> the 'human diagrams' are really just one more way to storytell - and we may never fully
> reconcile technical models with human models.

The essay (2026-09-06) puts it as "we need stories AND models, we need a visceral emotional
connection AND some grounding", and "People are not driven by facts."

What this means for the shape of the thing, as read by the session that evening:

- The appliance stays an appliance. Modules, ledgers, describes, the regress ladder, the
  audit: none of that is for novices and none of it changes.
- The project README is already the story layer, and it has been growing one figure
  vocabulary at a time (ladder, table, actors). The direction he names is the inverse: the
  story is the page, HTML with the figures embedded as widgets, rather than Markdown with
  fenced blocks. The widgets exist (Ladder, Chart, BandChart, Actors, Scene3D, the tables);
  what would change is the container and who writes it. The renderer passes no raw HTML by
  design, so a story page would be its own thing beside the README, authored and served
  differently, with the same rule that a figure's numbers come from the runs.
- The human diagram and the module wiring are two pictures of one system and may never
  meet. The devlog entry of the same day, *The deep history of the Palisades ground, and the
  actors the diagram does not show*, has the principle: the human model's standard is
  agreement, the delta is examined, and closing it is a choice each time.

The 2100 page got its actors figure the same evening: six actors around the warming, on the
IPCC's 2021 summary (A.1; D.1's 0.45 °C per 1,000 GtCO₂, likely 0.27 to 0.63; the sinks' 56
percent over six decades, all from the SPM as quoted by secondary pages, since ipcc.ch
answered 403 twice), the Global Carbon Budget 2024's 2023 figures (11.1 GtC emitted, 5.9 to
the air, 2.9 ocean, 2.3 land, from the paper's abstract as indexed), the Kaya identity and
Armstrong McKay 2022. Twelve arrows, eight solid; the biggest dashed one is people to
emissions, since the emissions after 2024 are a named assumption and not derived from anyone.

## Deep histories (Anselm, the same evening)

> in general i should mention also that i am always interested in _deep_ histories - this is
> why i asked you to ground the pallisades fire in a larger context - it is even arguably worth
> talking about how that fire changed future policy for example - in many ways. basically we're
> bringing a lens to a topic, trying to understand it _deeply_ - how was it primed for that
> situation? not just the situation. what were the changes or impacts on land use law and
> policy - how is it taking to rebuild due to zoning hassles and insurance delays - we're
> interested in the totality of a situation.

So a scenario page has three parts and the simulation is the middle one: how the ground was
primed (the deep history, now on the Palisades page), the event (the runs and the record), and
what it changed afterwards (policy, law, insurance, rebuilding), and the last is still missing
from every project. For the Palisades the questions to source are: what the fire changed in
state and city law and in the fire code; what the insurers did and what the state's plan of
last resort carried; how many permits have issued and how many houses stand a year on; and
what the after-action reviews concluded. Nothing on that is written yet; the sources are to be
fetched and read, not recalled.

## Later: the aftermath, the practice page, the sections

Anselm: "go ahead and source that and add a post event outcomes to the readme there - i'm ok
with just casual research on this"; "we may want to have some global commentary on the site as
a whole - say in the /docs - around best practices - talking about these topics above; deep
time, human diagrams, story telling"; "can you break the /docs into sections also - right now
it is one long run of individual essays".

The first Palisades README now has *After the fire* (the case and its retrial opening the
same day, the county's and the city's reviews, the governor's orders and the permit counts,
Zone 0 approved and then withdrawn from review, the insurance moratorium, assessment and rates,
the rebuild tracker's funnel), sourced from news and official releases read that evening and
said to be casual; and the sections now run priming, event, aftermath, actors, runs.
`docs/system/telling.md` is the practice page. The docs index groups pages by a `section`
comment; the server writes it into the index and the list draws the headings.

## A second kind of human diagram: the cascade map (Anselm, late evening)

Anselm pointed at a reference, a claude.ai artifact titled *Climate Cascade Map*, "an example of
how somebody tried to paint core and knock-on effects; this isn't a functional relationships
diagram in terms of inputs/outputs - but rather indicates impacts radiating from core issues".
Read this evening: a square poster, a dark disc on a red field, population growth at the
centre, about forty nodes placed by hand in rough rings outward (climate change, methane,
permafrost, fertiliser, farming, deforestation, fossil fuel near the core; ocean acidification,
coral, fisheries, food prices, riots, sea level, ecomigration, water wars further out;
awareness, alternative energy, efficient vehicles, LEED buildings, rethinking urban growth at
the bottom edge), and about seventy curved links in three colours: red for climate
consequences, white for demand and economic chains, cyan for responses and feedbacks. Hovering a
node lights its neighbourhood. No arrowheads; direction is read outward from the centre.

What it is: a map of consequence, not of mechanism. Distance from the centre is "how many
steps removed", colour is "what kind of chain", and the picture's claim is breadth, that one
cause touches everything. The actors figure claims the opposite thing, that a few named
relationships decide the outcome, and the wiring claims only what the code does. Three
pictures, three questions: what matters (actors), what follows (cascade), what runs (wiring).

What it would take here, not built: a `cascade` README block with a centre, nodes in rings
(`ring 1`, `ring 2`, ...), and links typed by kind, drawn radially in the house style (no red
field, no uppercase mono poster; the brand teal for one chain kind at most, the rest in greys
and dashes). The honesty rule carries over unchanged: a link the runs model is solid, one the
world has and the runs lack is dashed, and for the 2100 project nearly every ring past the
first would be dashed, which is the true picture of a planet model with no economy in it. The
hover-to-light-a-neighbourhood is worth keeping; it is how a reader asks "and then what?" of a
dense picture. Not started.
