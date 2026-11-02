<!-- order: 13 -->
# Time

## The clock

Virtual time is a count of seconds, UTC, as a signed 64-bit integer, so 1850 is as valid
as 2100 and the proleptic Gregorian calendar is used throughout. Field physics runs at
minutes to hours; sub-second behaviour is not this kernel's business. Modules choose
their own step (`dt`) and schedule their own ticks; nothing forces a global timestep,
which is how a river at fifteen minutes, a crop at a day and a planet at a month share one
run.

## Ordering at an instant

Events are delivered in order of (time, priority, sequence). Lower priority runs first at
the same instant. A module that reads at an instant what another writes at that instant
takes the higher priority number: weather before sun, sun before heat, water before
crop, crop before animals, animals before fire, fire before nitrogen. The describe
declares a kind's priority and the check warns when a required tick read would see last
tick's value. Calendar-aware kinds (the planet's monthly and yearly ticks) schedule to
the first of the next month or year rather than by a fixed number of seconds.

## Cadences

Ledgers close on the manifest's cadence (a day for a basin, thirty days for a planet); a
close compares every declared stock with its flows. Snapshots, when asked for, are taken
on their own cadence and allow a run to be resumed exactly. Map frames are written on a
cadence a module chooses, independent of its step, to keep run folders small.

## The cursor

In the viewer, one time cursor spans a run: every map shows the frame at or before the
cursor and every chart marks it with the value under it. The cursor is the reader's
clock; it makes the maps move and lets two things that happened in the same year be seen
together. Audit tables and crossing lists are indexed by the same times.

## Records in time

Records arrive at their own resolution: hourly weather, daily gauges, sixteen-day
vegetation composites, annual temperature means, five-yearly census. A module reads a
record at the time it needs and the reader says how it was aligned (a daily value stands
for its day at the observation hour; an annual mean is reported at mid-year). Scores
match model and record at identical instants only.

## Before and after the record

A run can start before a record begins and end after it ends. Before, a module says what
it does (the planet's population holds its first value flat and calls it a placeholder);
after, the manifest's assumption applies and is named. The boundary is in the describe
and in the run's notes, never implicit.
