<!-- order: 1 -->
<!-- section: Standards -->
# RFC 0001: a common transport for simulations

Status: draft 1, 2026-11-07. Author: Anselm Hook, drafted with Claude in the esim session
of that day. Adopted by no engine yet, including this one.

This is an argument, with a schema attached. It proposes that engines which couple
modules of science on one clock should exchange their simulations as a single document,
the manifest, in a shape they all read. The full shape is the JSON Schema at
[`/docs/rfc/manifest.schema.json`](/docs/rfc/manifest.schema.json); this text gives the
reasons, the outline and the one page of rules that make it a transport rather than a
configuration file. esim writes its manifests in TOML, and the shape here is abstracted
from that practice, not copied from it.

## 1. The case

Everyone who builds a whole-system simulator builds the same four things. A clock. A set
of modules, each owning one piece of the world, a river, a carbon cycle, a crop, the
people of a country. Some way for the modules to affect each other. And a way to say what
is in the run and what it assumes. The first three are the engine, and engines will
compete: on speed, on the fidelity of their modules, on their records, on their pictures.
The fourth is a document, and today each engine writes it in a dialect of its own.

The document is the only part worth exchanging. A scenario is a claim: these modules, with
these parameters, over this window, coupled this way, say the planet is 2.1 °C warmer in
2100 with emissions at zero by 2050. Everything that makes the claim arguable is in the
arrangement. Which modules were included and which left out. Which numbers were set and
which were the author's guesses. What the modules were allowed to tell each other. A
reader who has the arrangement can argue with the claim; a reader who has only the chart
cannot.

Today the arrangement is locked inside the engine that ran it. To criticise a scenario you
must install the vendor's software, learn its dialect, and trust that what it shows you is
what it ran. A common transport changes five things:

- **A scenario can be read before it is run.** Someone with no engine at all can open the
  document, see the modules and the couplings, and draw the graph.
- **It can be checked without being run.** Whether every coupling has both ends, whether
  two modules fight over one field, whether a required input is missing, follows from the
  document and the modules' own self-descriptions.
- **It can be run elsewhere.** A second engine with the same kinds, or with its own kinds
  under the same names, runs the same arrangement, and the two results can be put side by
  side.
- **A disagreement becomes a finding.** When two engines run one manifest and differ, the
  difference is about the models, not about the file format. That is the comparison the
  field keeps saying it wants and never gets, because no two groups ever ran one
  arrangement.
- **Guesses travel as guesses.** The transport carries the provenance of every number, so
  a parameter nobody has sourced cannot arrive at the next engine looking like one that
  was measured.

Formats have done this before for other arts. Images, 3D assets and musical performances
each had a decade in which every vendor's files were an island, and each got a transport
that did not try to standardise the tool, only what passed between tools. Simulation of
the whole system is at the start of that decade.

## 2. What a manifest is

One JSON object. Here is a complete one, in the vocabulary of the reference
implementation; another engine's kind names would be its own.

```json
{
  "manifest": "manifest/1",
  "vocabulary": "https://esimv1.exe.xyz/docs/system/vocabulary",
  "run": {
    "name": "planet_people",
    "start": "1850-01-01",
    "end": "2100-01-01",
    "seed": 1,
    "close_every": "30d",
    "notes": "The planet core with people on half-degree cells, 1850 to 2100. Emissions after the record follow the variant's assumption; the default holds them at the last recorded rate. An assumption, never a forecast."
  },
  "modules": [
    { "name": "story", "kind": "story.bottomline" },
    { "name": "emissions", "kind": "carbon.emissions", "params": { "after_record": "hold" } },
    { "name": "pools", "kind": "carbon.pools" },
    { "name": "forcing", "kind": "forcing.ar6" },
    { "name": "energy", "kind": "energy.twolayer" },
    { "name": "people", "kind": "people.countries", "params": { "res_deg": 0.5, "distribution": "worldpop" } },
    { "name": "heat", "kind": "people.heat", "params": { "res_deg": 0.5, "niche_c": 29.0 } },
    { "name": "tipping", "kind": "tipping.elements", "params": { "coupling": 0.0 } }
  ],
  "audit": {
    "scores": [["energy.gmst_anom_c", "obs.hadcrut5_anom_c"]],
    "shock_topics": ["shock.", "people.crossing"]
  },
  "variants": [
    { "name": "held", "title": "Emissions held at the last recorded rate", "set": { "emissions.after_record": "hold" } },
    { "name": "zero-by-2050", "title": "Emissions to zero by 2050", "set": { "emissions.after_record": "zero_by:2050" } },
    { "name": "hindcast", "title": "The record only", "set": { "run.end": "2025-01-01" } }
  ]
}
```

Five blocks, and that is the whole grammar.

**run** is the window, the seed and the cadences. Times are UTC, dates or RFC 3339 with a
`Z`; a run that means a local time has a hidden dependency on where it was written. The
seed is the only source of randomness, and each module's stream derives from the seed and
the module's name, so adding a module changes nobody else's draws.

**place**, absent above, states a window and a resolution once for every spatial module,
so that they share one grid and can read each other's fields. It is copied into each
module's parameters at resolution time, and the copy is kept, so the rule need never be
applied twice.

**modules** are the instances, in order. Each has a name other modules address, a kind,
and only the parameters it means to set; defaults live in the kinds. A kind the engine
does not have may carry a `ref`, a URL and a content hash, to an implementation it can
fetch. The order is the layout of the result and must not affect the result: delivery
order is time, priority and sequence, never position in the file.

**audit** names the pairs of series to score, simulated beside observed, and the topics
that count as shocks. It advises the auditor and touches nothing in the run.

**variants** are named override sets, `run.end` or `instance.parameter`, so that one
document holds a family of assumptions, emissions held, falling, zero by 2050, as a table
of differences. A variant may not add a module or change a kind; a scenario that needs
that is a different manifest.

A document may `include` others, which fold in first; a module that restates an included
module's name replaces it, so a short manifest can change one parameter on top of a
shared bundle.

## 3. What makes it a transport

A configuration file depends on the machine that reads it: its defaults, its library, its
copy of the data. A transport cannot. Three rules close the gap.

**Resolution.** Before a run, the engine folds the includes, copies `place` down, fills
every parameter from the kind's default, applies the variant, and writes the result as the
run's own manifest. That resolved document is complete: the engine must be able to run
from it alone, and a reader must be able to see every number the run used without
knowing any engine's defaults.

**The resolved form carries what it rests on.** Two blocks appear only after resolution.
`kinds` holds the self-description of every kind used: its parameters with units and
provenance, the topics it publishes and subscribes to, the fields it owns and reads, its
ledgers, its outputs, the records it reads. `locks` holds the content hash of every record.
With these a reader anywhere can check the wiring and know which data was read, with no
access to the engine or its cache.

```json
{
  "locks": {
    "owid-co2": { "url": "https://example.org/owid/co2.csv", "hash": "sha256:9b1e…", "bytes": 4120377, "fetched": "2026-11-01" },
    "hadcrut5-annual": { "hash": "sha256:51aa…" }
  }
}
```

(Hashes are shortened in the examples.)

**Canonical text and identity.** For hashing and comparison the resolved manifest is
serialised by the JSON Canonicalization Scheme, RFC 8785, and its id is the SHA-256 of
that text. Two resolved manifests with one id are one manifest, whatever files they came
from and however their keys were ordered. A run is then identified by its manifest id and
the digest of its event log, and two people on two continents can say whether they ran
the same thing.

## 4. Checking without running

Because every kind describes itself, the couplings of a manifest follow from the document
and the descriptions, before anything is fetched or run. A checker reports three grades.
Errors, which make the manifest unrunnable: an unknown kind with no `ref`, a parameter
the kind does not list, two writers of one field, a required read nothing writes, a read in
one unit of a field written in another, a message to a module not present, one ledger in
two units. Warnings: a reader whose priority puts it before its writer at the same
instant, so it sees last tick's value; a guessed parameter the manifest left at its
default. Information: optional couplings not present, subscriptions nobody publishes,
outputs nobody reads. The information grade is the manifest's own account of what it
could gain, and it is the line a reviewer reads first.

## 5. In what sense the same run

The hardest question about any transport is what it promises, and the honest answer is a
ladder. Two engines agree about a manifest at one of five levels, each including those
before it.

1. **Readable.** Both parse it, resolve it the same way and compute the same id. Testable
   with no modules at all.
2. **Checkable.** Both derive the same wiring from the `kinds` block and report the same
   errors and warnings. Also testable without modules.
3. **Runnable.** Both have, or fetch, every kind, and run to the end with every ledger
   closing within tolerance.
4. **Comparable.** Both report the series the audit names and the crossings the kinds
   publish, and a person can put them side by side. The numbers need not agree. The
   shape of the record does.
5. **Reproducible.** Both produce the same event log digest. This needs bit-identical
   module semantics, the same random generator and the same floating point behaviour,
   and is a property of two engines that share modules, not of the standard. No engine
   may claim it on the standard's authority.

Most of the value is at levels one to four. A manifest written in one shop can be read,
drawn and criticised in another before any code moves; when both run it, what differs is
the science. Level five is where competing vendors will never meet, and the standard does
not need them to.

## 6. One rule about honesty

Every parameter in a kind's description carries a provenance: `checked`, read from its
source and compared; `recalled`, from memory and not yet checked; `guess`, sourced by
nobody; `fitted`, set by a stated method against a record. The transport requires it for
every number with a physical meaning. This is the one place the document legislates
conduct rather than shape, because a manifest that lets a guessed number travel as a
checked one defeats the purpose of exchanging manifests at all. A chart does not carry
this; a document can.

## 7. What it leaves out, on purpose

Not the engine: how a kernel orders events, stores fields or closes ledgers is the
vendor's. Not the protocol between a kernel and a module, which is a second document and
lets a module be compiled in, loaded as WebAssembly or run as a process. Not the
vocabulary: whether two engines mean the same thing by `energy.gmst_anom_c` is the real
interoperability problem and belongs in a shared, versioned list a manifest names by URL;
this document only insists that the name be carried. Not the run folder, the log, the
frames or the scores.

An engine that needs more puts it under `ext`, keyed by its own name, and nothing under
`ext` may change a result. Readers ignore what they do not know there and preserve it
through resolution.

## 8. From practice

esim's TOML manifests map onto this one for one: `[run]`, `[place]`, `[[module]]` with its
`[module.params]`, `[audit]`. Three things differ. esim keeps variants in the project
beside the manifest, where this document puts them in it, because a family of assumptions
is the thing worth sending. esim keeps the kind descriptions and the data locks in files
beside a run, which is the same information in a shape a reader elsewhere cannot use. And
esim's include can be spliced at a point in the module order, which this document does not
yet have.

The reference implementation does not yet read this form. That is deliberate: the
argument should stand before the code follows it.

## 9. Security

A manifest can name code and data by URL. An engine verifies the content hash before
using either, and runs a fetched module with no more authority than the module protocol
grants: its parameters, its seed, its events, the fields it reads. Parameters are never
evaluated as expressions. A manifest from a stranger is a document to check, not a program
to run; levels one and two are safe on any input, level three is not.

## 10. Open questions

- Whether `place` should be a general grid declaration rather than a geographic window,
  so that a tank or an abstract lattice is first class.
- Whether a variant may add a module, which would make "with and without this coupling"
  one document at the cost of variants no longer being a table of differences.
- Where the vocabulary lives and how a version of it is named.
- Whether the audit belongs in the manifest or in a document that names one.

## References

- The schema: [`manifest.schema.json`](/docs/rfc/manifest.schema.json).
- [The module protocol, version 0](/docs/protocol-v0), the second document.
- [Manifests](/docs/system/manifests), [Vocabulary](/docs/system/vocabulary),
  [Terminology](/docs/system/terminology): the practice this was abstracted from.
- RFC 3339 (timestamps), RFC 8785 (JSON canonicalization), JSON Schema draft 2020-12.

## Changes

- Draft 1, 2026-11-07: first text.
