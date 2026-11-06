# 2026-11-06 — Substack draft, second: a world you can take apart

A machine draft for Anselm's pass; nothing here publishes under his byline until he has rewritten it in his own hand. Asked for on 2026-11-06: "we made a substack post, i want to revisit that post - lets write it up again just dump it here to the console as fuel for me to tidy up and publish". It follows the 2026-11-03 draft (goals) and reports the week since: every figure is from the runs and devlog entries in this repository as of this morning. Heckler: 1,432 words, 14 paragraphs, burstiness 0.58, mean sentence 17.4 words, 0 tics per paragraph, fingerprint cos 0.826 against his corpus (drifts); no paragraph-level tells.

---

# A world you can take apart

What should I plant this year? Where will the fire go tonight? What does my coast look like in 2100? I keep coming back to the shape these questions share. Here is what I know about a place, here is what I am thinking of doing, what happens next?

Three weeks ago I started building a tool for that question, with Claude, in the open. Not a forecast, which is somebody else's answer handed down, but a thing you can ask yourself about your own ground at your own scale. A field, a watershed, a planet. I wrote about the goals when it was a few days old. This is what it looks like a week later, and what it taught me about the question itself.

## Two fires

The Palisades fire of January 2025 is the scenario I keep testing against, because the record is so good. The incident's own mapping teams drew the perimeter as the fire grew, inspectors visited twelve thousand structures afterwards, and the county's GIS holds every dozer line with a timestamp. We run the fire again on its own ground. Thirty-metre terrain, land cover as fuel, every building as a body with a height, the hour's wind from NOAA's three-kilometre analysis, lit where and when the record says, with nobody fighting it.

The first day is now right. At the first infrared perimeter, twenty-seven hours in, the model has burned 16,500 acres against 15,800 mapped, and the shapes overlap by four fifths: the same lobe along the coast to Malibu, the same climb toward Topanga. That took four rewrites of the engine, and the devlog has each one, including the morning I found that one column in every three of the elevation model was missing and the slope code had been reading each gap as a cliff. Every fire run before that afternoon was burning on terrain that did not exist.

The second day is wrong, and wrong in an instructive way. The real fire grew by a few thousand acres while the wind eased and crews held it. The model fills the mountains north to Calabasas. No constant in the engine reaches that, and I tried them all. What the model lacks is the wind as it actually blew over the ridges, and the suppression the record does not place in time: the first completed line in the GIS is dated two days in, after the overrun. So the gap on day two is not a bug. It is the two things we do not yet know how to represent, drawn on the page.

Then I asked the same engine to burn Fort McMurray, in boreal Alberta in May 2016, to see how much of it was the Palisades and how much was fire. Three things had to change. Buildings came from OpenStreetMap because no county serves footprints there. The wind came from a coarser reanalysis because the three-kilometre analysis stops at the border. And the record became satellite hotspots, because nobody mapped that fire hour by hour. The pace carried over with one constant moved, and that constant is standing in for a fuel model a spruce stand and a chaparral slope should not share. The shape did not carry: the reanalysis read the day the fire ran into the city as a light breeze, and the model's fire went the other way, into the forest. The model is honest about this on its own page. It drapes the ground the satellites saw burning over the ground it burned, and you can see the two disagree.

## The planet, and who is in the red

The other scenario is Earth 2100. One model of the planet, with people on it, run under six assumptions about emissions after 2024 and read against the sixteen tipping thresholds a 2022 assessment set out. It ends the century at 3.4 degrees with emissions held, and it counts the people living where the heat has left the range humans have always lived in. By 2100 that is a tenth or more of the people in forty-nine countries: the Gulf, the Sahel, the Caribbean islands, Samoa. Bangladesh is not on that list, and the page says why. The reanalysis under the model reads humid heat mild, and that is a fault to fix before any conclusion is drawn about a delta.

Two things went in this week that I want to say more about, because they are less about climate than about communication.

The first is a globe. Every country is drawn by its own outline three times, one ring for its people, one for its nature, one for its economy, each scored by where it stands among the countries that report the same numbers, and each ring brighter the higher the score. A country whose people are past the heat thresholds, or whose weakest ring has fallen under a floor, fills with red. Drag it, scrub the century, watch the red spread. I tried three other glyphs before this one: stacked bars, a block of icons, emoji up a pole. The block was the most legible for one country and the worst for a hundred, because small islands pile up. The rings read at a glance and fail gracefully. What I learned is that the design decision is not the glyph. It is the choice of nouns, and whether the score is a mean. A country with a strong economy and a heat emergency is not doing medium well, the way a rich person having a heart attack is not in average health. Some factors gate. The red reads the worst pressure, never the mean.

The second is a deck of wild cards. The world will not go to 2100 without an eruption, a pandemic, a crash, a war. We had one of each in the last forty years. So there is a module that draws from a deck of twenty-two such things each year by rough odds, repeats allowed, and applies each as a pulse on the climate, a scaling of emissions, or a fraction of everyone. Every number in that deck is a placeholder and says so on every row. The result surprised me by being small. A century of cards takes two to five percent off the population and a tenth of a degree off the warming, because no card in the deck bends the emissions path for good. With emissions held, the slow things decide the century and the shocks are weather on top. The card to add is the one that changes the path.

## What a page is for

Somewhere in the week I realised the readme on each scenario had become the product. I kept asking for more of it: the deep history of the ground before the fire, the policy that changed after it, the court case, the insurance, the permits. Six hundred years of charcoal in offshore mud. A reader coming to a fire or a century cold needs the before and the after as much as the event, and the simulation is the middle third of a story.

So each page now carries two models. The machine model, which is the modules and their wiring, drawn from what the code declares. And a human model, the actors and the relationships a person at the fire would name, wind, fuel, slope, homes, crews, drawn in their own words, resting on the frameworks the field already teaches. A solid arrow is something the runs model. A dashed one is something the world has and the runs lack. The gap between the two pictures is the honest state of the project, and I would rather draw it than hide it.

## How it is built, still

Less a product than a collection of parts. Each part is a module that declares what it reads, what it owns and what it publishes, and the kernel refuses a scenario whose wiring does not check. Where something is conserved, a ledger has to close at every step. Every number on a page points back to public data fetched anonymously. The interventions in the fire became a module this week for exactly that reason: so one engine could burn another fire with another record, or none. I do not trust a model I cannot take apart, and a week of taking this one apart has been the most useful week it has had.

The code and the scenarios are at github.com/anselm/esimv1. The pages are at esimv1.exe.xyz. If a scenario is wrong about somewhere you know, that is the thing I most want to hear.
