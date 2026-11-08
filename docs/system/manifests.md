<!-- order: 6 -->
<!-- section: The machine -->
# Manifests

A manifest is the whole arrangement of a simulation in one document. It is the
higher-level grammar: the kinds are the vocabulary, the manifest is the sentence. In the
way a game engine's blueprints wire prefabricated behaviours without writing the
behaviours, a manifest wires modules without writing modules. It comes in three
notations, TOML, JSON and JavaScript, which resolve to one thing (see [Three
notations](#three-notations) below); the grammar is shown in TOML first because that is
what the manifests in the repository are written in.

## The grammar

```toml
include = ["planet-core.toml"]     # optional: other manifests, folded in first

[run]
name = "p4-planet-people"
start = "1850-01-01"
end = "2100-01-01"
seed = 1
close_every = "30d"                # ledger closing cadence
snapshot_every = "60d"             # optional
notes = "What this run is and what it leaves out, in a sentence."

[place]                            # optional: stated once, inherited by every module
lat_s = 42.3
lat_n = 43.2
lon_w = -94.2
lon_e = -93.5
res_arcsec = 30

[[module]]
name = "water"                     # the instance name other modules address
kind = "water.basin"               # the kind, from `esim kinds`
[module.params]
soil_capacity_mm = 150             # only parameters the kind lists, in its units

[audit]
scores = [["hydro.q_sim_daily_m3_s", "hydro.q_obs_daily_m3_s"]]
shock_topics = ["fire.burn", "hazard."]   # optional; these are the defaults
```

## What a manifest fully states

Everything a run depends on except the records, and the records are pinned by hash in
the lock file. Given the resolved manifest text and the seed, a run is reproducible:
same events, same ledgers, same digest. This is why a run folder keeps the resolved
manifest and why nothing may be configured anywhere else: no environment variables that
change results, no hidden defaults that differ between machines. Defaults live in the
kinds and are printed by their describes.

## Includes

`include` names other manifests, relative to this file. Each included file contributes
its `[place]`, `[audit]` and `[[module]]` entries first, in order; the including file's
own come after. The including file's `[run]` stands. A module that restates an included
module's name replaces it, which is how a short manifest changes one module's parameters
on top of a bundle. Keys of `[place]` and `[audit]` merge, the includer winning. The
kernel resolves includes before checking, and a project folder receives the resolved
text, so a project never depends on a file outside itself.

## Place

Spatial kinds share one window and one resolution or they cannot read each other's
fields. `[place]` states them once; the parser copies each key into every module that
does not set it itself. A manifest for a new place sets its gauge and station
parameters in the modules that need them.

## Parameters and variants

A parameter is a named number, string or flag a kind lists, with a default, a unit and a
provenance note. A manifest overrides only what it means to override. A project may
declare variants, named sets of overrides applied on top of its manifest at run time
(`run.end`, `emissions.after_record`, any `module.param`); the run folder records the
variant and the resolved text. A sweep or a hindcast moves parameters by method over
ranges; its runs record the values it chose.

## Order is layout

The order of `[[module]]` entries is the order of sections on the run page. A bundle's
modules come first unless the including manifest restates them; a module that exists to
summarise (the story) is listed first so it is read first. See the views table in
[Modules](/docs/system/modules).

## Checking

`esim check manifest.toml` derives the wiring from the modules' describes and reports
errors, warnings and information; `esim run` refuses errors. Information lines are worth
reading: "the water module would read rules.no_crop but nothing writes it" tells a person
or a model what the manifest could gain.

## Three notations

Since 2026-11-07 a manifest file may be written three ways, told apart by its extension,
and `esim run`, `check`, `describe` and `project` take any of them.

- **TOML**, `.toml`: the form above, with `include`. Every manifest in `manifests/` is one.
- **JSON**, `.json`: the entity form of [RFC 0001](/docs/rfc/0001-manifest). A manifest
  is a list of entities, each an `id` and a bag of components; an entity with a `run` is
  the run, with a `kind` a module instance, with a `place`, an `audit` or a `variant` that
  thing. A later entity with an earlier id replaces it component by component. A component
  nothing here reads is carried and the check reports it. This is the *object form*: plain
  data, no variables, no loops but the `range` on a variant, which unrolls to one variant
  per member.
- **JavaScript**, `.js`: the *source form*. A module whose exports are the entities: the
  export name is the id, an exported array is spliced in, `export default` may be the whole
  list, and an export named `run`, `place` or `audit` that carries no component is that
  component. The program runs on QuickJS with nothing but the language: `Date` is gone,
  `Math.random` throws, there is no network, no process and no file system, and `import`
  reaches only files beside the entry, by relative path. Importing a `.toml` manifest gives
  its entity list with includes folded, so a program can start from any manifest in the
  repository and restate what it wants; importing `.json` gives the value. Functions do not
  survive: exports are read through JSON.stringify, so a component is data or it is dropped.
  Export order is layout order; variables, loops and comments are the point.

Whatever wrote it, the engine resolves a manifest to the same entity list, and the
manifest's id is the sha256 of that list's canonical text, so `esim manifest
manifests/p1-planet-co2-1850-2025.toml --id` and the same for the `.js` beside it print one
id. `esim manifest <file>` prints the entity form of any manifest, `--toml` the TOML,
`--js` a JavaScript module to start editing from. A run folder and a project folder keep
`manifest.toml` (the resolved TOML the pages read) and `manifest.json` (the entity form
with a `from` block naming the file it came from, its hash and every file folded in); a
project made from a JavaScript manifest also keeps the program and its imports under
`source/`. `manifests/j3-planet-wild-members.js`, the manifest of
[Eight draws of the deck](/anselm/planet-wild-members), is the worked example: it imports
the Earth 2100 manifest, restates two entities and declares an ensemble as a range.

## Births

The kernel is an event bus and the manifest is played onto it: each module entry is a
`manifest.entity` event at the start, and the kernel builds the module when the event
comes by. A module may publish the same topic later, `{ id, kind, params }`, and a module
is born mid-run, initialised, logged, and delivered to whoever subscribes to births; a
snapshot carries it so a replay rebuilds the same list. A birth the kernel refuses, an id
already taken, a kind nobody has, a factory that fails, is logged as `manifest.refused`
and the run goes on. The wiring check is static and covers the entities present at the
start; a later birth is checked for its kind and id, not for what it reads and writes.
No first-party module publishes a birth yet.

## As a standard

[RFC 0001](/docs/rfc/0001-manifest) proposes the entity form, in JSON, as a common
transport between engines of this kind, with a schema beside it, and the JavaScript form
as the source form above it. esim reads and writes both since 2026-11-07.

## Writing them

People rarely will. See [Conversation](/docs/system/conversation).
