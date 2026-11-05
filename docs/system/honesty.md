<!-- order: 3 -->
<!-- section: Start here -->
# Honesty

The rules that make the rest worth doing. They are enforced by code where code can, and
by convention where it cannot; the conventions are the ones that matter most.

## Real records or nothing

A module reads only what the registry fetched, and the registry fetches only real
observations and real base layers: terrain, gauges, stations, reanalyses, ice cores,
satellites, censuses. Nothing is synthetic, including in tests. A formula check that
needs no landscape is labelled as one. Where a record is missing the module says so and
goes without; it does not make one up.

## Integrate, do not speculate

A module advances state by a stated rule from a stated starting point. Where the rule is
from a paper, the paper is named. Where a number is a guess, the parameter's note says
"guess". Where the author recalled a number from memory and has not checked it against
the source, the note says "recalled", and the thing that reads it carries that word
through to the page. Checked is a separate word, earned by reading the source.

## Nothing is tuned by hand

Parameters are named, defaulted and sourced. Moving one to make a curve fit is not
allowed; the hindcast harness moves parameters by method, against scoreboards, and
records that it did. A result that looks right for the wrong reason is reported as such.

## Assumptions are not forecasts

A run past the end of a record rests on an assumption, and the manifest names it:
emissions held at the last rate, a straight line to zero in a year. The project page
says "an assumption, never a forecast", in those words, and the run's numbers are read
with that in front of them.

## Say what is wrong first

Every milestone entry and every project description leads with what the model gets
wrong before what it gets right. Today the planet runs half a degree warm in 2024 and
keeps too much carbon airborne; that sentence comes before any table of crossings.

## Accounts must balance

Every conserved quantity a module moves goes through a ledger, in the precision the
module publishes it, and the kernel closes the ledger on a cadence. A leak larger than
the tolerance fails the run, not a warning. The ledgers have caught precision bugs that
no chart would have shown.

## Replayable

The same manifest and seed produce the same log digest, on any machine, from a snapshot
or from the start. Randomness comes from a seeded stream per module. A run folder holds
the resolved manifest and the hashes of every file it read, so it can be rerun years
later or by someone else.

## Measure, do not narrate

The auditor measures what every run with the right ingredients exposes and lists what it
could not measure. It does not produce a stress index with no sources behind it, and it
does not give a country a collapse date. The words "stretching" and "tearing" have
measured definitions here: recovery faster than recurrence, or not.

## Provenance travels with the number

A parameter's note, a catalog entry's licence and verification date, a tipping row's
source and confidence, a project's "by" line: each is kept next to what it describes and
shown wherever the number is shown.
