# 2026-11-01 — Research agenda: patches, water, and the events to regress against

Anselm, in four messages while the country layers were being built. Agent: Claude
(Fable 5.1), earth-simulator-1 room on canada1. `todo.md` at the root is the short list;
this entry is the thinking behind it.

## What he said

- "Can we model or predict AMOC or MENA changes? This will have radical influences."
- There is paleoclimate history of AMOC collapses triggering hyper-arid phases in North
  Africa; and the account of why the Anasazi left the Grand Canyon region, drought.
  "Many of our simulations will probably revolve around water."
- The Amazon depends on the AMOC, so any layer about those hyper-organisms should hang
  off a rainfall model. A planetary model can consist of many patches, "not just 7 but
  like 700", the Amazon basin itself an approximation of sub-regions; a planetary sim as
  many patches put together.
- Layer ensembles, or nested manifests, so manifests can be shorter.

## AMOC and MENA, honestly

We cannot simulate the AMOC. It needs an ocean general circulation model, and the
project's rule is to integrate what we can integrate and read the rest as data. Three
things we can do, in order of rigour:

1. **Carry it as a threshold with a citation.** Done today: `data/events/tipping.toml`
   holds the AMOC with the assessed range (recalled values, marked so), and the
   `tipping.elements` module flags the year a run's warming enters the range. That is a
   flag, not a collapse, and the page says so.
2. **Watch the record.** The RAPID array at 26.5 N has measured the overturning since
   2004 and publishes its series; the download page answered today but the file link
   was not followed. A scoreboard module for "is it weakening" is a morning's work once
   the file is fetched and read.
3. **A conceptual model with a real bifurcation.** Stommel's two-box model (1961) has
   the salt-advection feedback that makes the overturning bistable, and later box models
   (Rahmstorf's, Cessi's) are published with parameters. As a module it would be labelled
   conceptual, and its one honest use is to show the shape of a tip: how a slow push
   produces a sudden switch and hysteresis. What it cannot give is the year.

MENA, in the sense of the Gulf and its heat, is where the country layer built today
points first: it is the region whose hottest-month mean passes any marker earliest, and
where the wet-bulb literature (Pal and Eltahir 2016 on the Gulf; recalled) says bodies
fail before thermometers look alarming. The missing piece is humidity; the country
temperature record has none. ERA5 dew point through Open-Meteo at a handful of Gulf
points is the cheap next step, and wet-bulb exposure the metric that replaces today's
guessed hottest-month marker.

## Patches, not layers only

He is right that the stack picture is too flat. The basin work already is a patch: the
Boone River with its own weather, water, crop, animals, farms and rules. A planetary
simulation in this architecture is a mosaic of such patches on one clock, each with its
own records, coupled through shared global state (the energy and carbon ledgers, the
global anomaly) and through boundary fluxes (a river into a sea, rainfall from a
circulation into a basin). The country layer built today is the coarsest possible
patchwork: 177 patches with one number each. The Amazon as a patch would have its
rainfall as an input from a circulation layer that, today, we can only read from data
or from a published scenario, which is exactly the AMOC dependency he named. The honest
version of "the Amazon depends on the AMOC" in this system is: the Amazon patch reads a
rainfall field; a module that owns that field states where it comes from; when it comes
from a tipping assumption, the manifest says so.

Nested manifests landed today for this reason: `include = ["planet-core.toml"]` folds a
bundle of modules into a manifest, so a patch or a core is written once and composed.
"Layer ensembles" and "nested manifests" are the same thing here; a bundle is a file.

## Water

Agreed, and the record supports it: the discharge, nitrate, precipitation and soil work
is the most tested part of the system, and the paleoclimate events he names are water
events. Two research projects for the list, both regressions against history:

- **The end of the African Humid Period** (about 5,500 years ago). Proxies exist in
  public archives (NOAA Paleoclimatology holds dust and lake records). A patch of North
  Africa with a rainfall input stepping down, vegetation feedback, and the question of
  whether the transition is abrupt or gradual, which is the live scientific question
  (deMenocal's abrupt dust record against the gradual pollen records; recalled). The
  AMOC link is through the monsoon; it would enter as the rainfall assumption.
- **The Four Corners megadrought and the Ancestral Puebloan departure** (late 1200s).
  The tree-ring record (the North American Drought Atlas, public) gives summer drought
  year by year; a patch with a water balance, maize, and a population with a stated
  response to harvest. Nothing to invent about the drought; the response is the research.

Both are hindcasts with a known ending, which is what makes them regressions and not
stories.

## What this does to priorities

The far-field list stands, with two insertions: humidity before mortality, and a
rainfall-reading layer before any forest patch. The two paleo projects are the first
entries of a second ladder, the historical one, and the right time to start it is when
the country layer has one real scoreboard behind it.
