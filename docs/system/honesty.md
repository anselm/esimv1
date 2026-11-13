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
states the assumption where it shows the run's numbers.

## Known faults are published

The faults each model is known to have are listed on [Limits today](/docs/system/limits),
which is rewritten as they change. Each project page says where its runs miss the record
and what it cannot tell the reader.

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

## The engine, not the article

A page exists to exercise a simulation, not to arrange published numbers into prose. A module
counts as one when it computes its outcome from the ground, the record and the clock, component by
component or cell by cell, and can be wrong in a way a record can catch. A number copied from a
report or a day drawn from a published range is an input or a check, never the output. Where the
engine cannot yet compute a thing, the page says so, the arranged version runs beside the computed
one as a variant while both exist, and the thing is the next rung. Interventions are parameters the
modules read, run over seeds and compared under one cursor, not paragraphs
(`devlog/20261112-the-engine-not-the-article.md`, Anselm, 2026-11-12).

