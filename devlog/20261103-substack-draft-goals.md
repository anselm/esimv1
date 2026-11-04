# 2026-11-03 — Substack draft: what this is for

A machine draft for Anselm's pass; nothing here publishes under his byline until he has
rewritten it in his own hand. Asked for on 2026-11-03: "a brief substack article on goals
here". Every figure below comes from the runs and records in this repository on that day;
nothing is recalled from outside it. Heckler's report is at the bottom.

---

## Asking the future questions

What should I plant this year? Where will the fire go tonight? What does my coast look
like in 2100? Three questions, three sizes, and the same shape: here is what I know about
a place, here is what I am thinking of doing, what happens next?

I have wanted a tool for that question for a long time. Not a forecast, which is somebody
else's answer handed down, but a thing you can ask yourself, about your own ground, at
your own scale: a field, a watershed, a planet. I have started building it with Claude,
in the open, and I want to say what it is for before it is anything much.

### Two scenarios

There are two scenarios in it today, and we will keep working on both.

The first is the Palisades fire of January 2025. CAL FIRE's record says it started at 10:30
in the morning on the 7th, burned 23,448 acres, destroyed 6,845 buildings and killed twelve
people. We run it again on its own ground: the real terrain at 30 metres, the real land
cover as fuel, the hour's wind from a weather reanalysis, lit where and when the record
says, with nobody fighting it. Then we hold the result against the perimeters the fire's
own mapping teams drew as it grew. The honest news is that the model is slow: at 27 hours
it has about an eighth of what they mapped, because a 25 km reanalysis reads a Santa Ana
night as a stiff breeze, and it reaches the fire's final size only on the fourth day, when
the real fire was being held. A second engine of our own, on NOAA's 3 km wind with its
gusts, matches the first day's acreage and then burns twice the record because nothing in
it stops. Neither is a tool a fire chief could use. The gap is the work. (An earlier
version of this paragraph said the model burned four times the record. It did, on ground
with a hole in it every third column that the slope code read as a cliff; the devlog has
the account.)

The second is Earth 2100. One model of the planet, six assumptions about what we do with
emissions after 2024, each run to the end of the century and read against the sixteen
tipping thresholds a 2022 assessment set out: the ice sheets, the Amazon, the monsoon, the
overturning of the Atlantic. The model runs about six years ahead of the record, and the
page says so before it says anything else. Every path passes the mountain glaciers' line
by 2040, including the one where emissions reach zero in 2050. No path reaches four
degrees. Whether that comforts you depends on where you live.

### Slow questions and fast ones

A planting question is slow. You have all winter, the answer is a distribution, and being
a little wrong costs a little. A wildfire question is fast. Someone is standing on a ridge
with a radio and needs to know, in the next twenty minutes, whether the line will hold
at the road or at the creek. The same model can serve both only if it is quick enough
and honest enough: quick, so it can be rerun as the wind changes; honest, so the person
on the ridge knows what it does not know. A fire day at 30 metres runs in three seconds
on my desk. The honesty is harder and takes longer, and it is most of what we do.

Between the field and the planet there is a third scale, and almost nobody models it for
the people who live in it. The bioregion. A valley, a river and its towns, a coast and the
weather it makes. Questions there are picky in the way local questions always are, because
the people asking can see out the window and will tell you when the creek is on the wrong
side of the road. I think that is where a tool like this earns its keep. We have not
started it.

### How it is built

Less a product than a collection of parts. Each part is a module: a crop, a river, a carbon
cycle, a fire, a country's people. Each one declares what it reads, what it owns and what
it publishes, and the kernel refuses to run a scenario whose wiring does not check. They
share one clock. Where something is conserved, water, carbon, animals, money, a ledger has
to close at every step or the run is marked as leaking. Every number on a page points back
to the data it came from, and every piece of data is public and fetched anonymously, so
anyone can run the same scenario and get the same digest.

I built it this way because I do not trust a model I cannot take apart. A product gives
you an answer. A collection of modules gives you an argument you can inspect, and a place
to put the module that is wrong when you find it.

### What I want from you

Questions. The ones you would ask about your own place, at whatever scale it is. If a
scenario here is wrong about somewhere you know, that is the most useful thing you can
tell me. The code and the scenarios are at github.com/anselm/esimv1, and the pages are at
esimv1.exe.xyz.

---

Heckler (2026-11-03, before this paragraph's rewrite): 816 words, 10 paragraphs, burstiness
0.6, mean sentence 16.9 words, 0 tics per paragraph, fingerprint cos 0.799 against his
corpus (drifts); one flag, metronome rhythm in the bioregion paragraph, since rewritten.
