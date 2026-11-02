<!-- order: 11 -->
# Evolution

Modules will change, and should. The pool is a search over ways of describing parts of
the world, and the record is the fitness function.

## Regression against history

Every scenario that has already happened is a test. The basin's discharge since 1940, its
nitrate record, the vegetation index, Mauna Loa, the temperature records: each run that
covers them is scored against them, and a module change that worsens a score is visible
in the next run's `scores.json`. The project's own tests go further: real manifests are
run and their logs compared against the modules' declarations, so a module that starts
publishing something it did not declare fails the build.

The historical ladder extends this to events with known endings: a megadrought and a
departure, a humid period's end, heat seasons with recorded tolls. A patch that
reproduces the known ending from the known forcing has earned something; one that does
not has told us where the stack is missing a layer.

## Moving parameters by method

Hand tuning is forbidden; the hindcast harness is how parameters move. It samples
named parameters over stated ranges, runs each sample, scores it against a scoreboard,
and records the chosen values and the evidence. A parameter moved this way carries the
harness's note in place of its author's guess. The sweep command is the first form of
this; the full harness (an optimiser over many scoreboards at once, with held-out
periods) is the next milestone.

## Back-propagation across layers

When an upper layer disagrees with its record, the cause is often two layers down: the
nitrate ran high at the gauge because the crop took up too little nitrogen because its
growth was slow because the soil water was wrong. The ledgers and the declared reads make
the chain inspectable: each seam has an account that closes, so the error is in a
module, not between them. Fixing it means changing the module two layers down and
rerunning everything above, which the manifests and the project variants make cheap.
Expect modules to evolve from the bottom up as the scoreboards above them are added.

## Versions of a kind

A kind's behaviour is its code, its defaults and its sources. When any of those change,
old runs must still be reproducible, which they are: a run folder has the resolved
manifest and the data hashes, and the kind's code at that commit. When modules move to
URLs with content hashes, a manifest will pin a kind's version the way it pins data, and
two versions of one kind can be run side by side in one project as variants.

## Competing kinds

Two kinds that describe the same thing differently are both welcome. Each declares its
reads, its owned fields, its sources and its omissions; both can be run against the same
scoreboards in the same manifest with one line changed; the viewer puts their runs under
one cursor. The record decides what it can, and where it cannot, the describes let a
person decide with their eyes open. There is no committee.

## What does not evolve

The vocabulary (topic names, field names, units, ledger names) changes slowly and on
purpose, because every module uses it; see [Vocabulary](?doc=system/vocabulary). The
protocol changes rarely, with a version number. The honesty rules do not change.
