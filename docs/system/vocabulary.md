<!-- order: 14 -->
# Vocabulary

The protocol between kernel and module is small and will rarely change. What carries
meaning is the vocabulary the modules use with each other: topic names, field names and
their units, ledger names and accounts, cell addressing. Two modules from different hands
interoperate only if they mean the same thing by `wx.t2m_c`. This page is the start of
governing that.

## Conventions in force

- **Names are `layer.quantity_unit`** where the unit is not implied: `wx.precip_mm`,
  `water.soil_mm`, `veg.biomass_gc_m2`, `energy.gmst_anom_c`, `people.outside_niche`.
  The prefix is the owning kind's family, not the instance name.
- **Units are SI or the field's customary unit, written in the name or the describe**, and
  the check refuses a read in one unit of a field written in another.
- **Topics name what happened, in the past tense or as a noun**: `fire.burn`,
  `harvest.done`, `predation.request`, `carbon.emitted`, `energy.state`,
  `people.crossing`. Their data is JSON and the describe's note says its shape.
- **Ledgers are named by the conserved quantity** (`water`, `carbon`, `energy`, `animals`,
  `nitrogen`) in one unit each; accounts within them are nouns (`soil`, `channel`,
  `atmosphere`, `herbivores_n`), and boundary accounts (`sky`, `sea`, `space`,
  `fossil_reserve`) are named as what lies beyond the system.
- **Crossing events share one shape**: a name, a year, a kind or level, a threshold, the
  source and the confidence. New threshold layers keep it.
- **Series reported for scoring come in pairs** whose names differ by `sim` and `obs`, or
  the observation lives under an `obs.` module.

## The current enumeration

`esim kinds --json` is the live list of every topic, field, ledger, frame and series any
kind uses today. It is the seed of the vocabulary and the thing to read before naming
anything new.

## Changing a name

Changing a message in the protocol is a change to the kernel. Changing a name in the
vocabulary is a change to every module that uses it, so it is done rarely, in one commit,
with the test that compares declarations against logs as the guard. Definitions of
shared names, with their units and meanings, belong eventually on a public wiki's
ontology shelf, where other engines can adopt or map to them; this page is the local
record until then.

## What is not standardised

Parameter names inside a kind are the kind's own. Event data beyond the crossing shape is
loosely typed on purpose; a topic's describe note is its documentation.
