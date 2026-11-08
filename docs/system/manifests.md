<!-- order: 6 -->
<!-- section: The machine -->
# Manifests

A manifest is the whole arrangement of a simulation in one TOML document. It is the
higher-level grammar: the kinds are the vocabulary, the manifest is the sentence. In the
way a game engine's blueprints wire prefabricated behaviours without writing the
behaviours, a manifest wires modules without writing modules.

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

## As a standard

[RFC 0001](/docs/rfc/0001-manifest) proposes the manifest, in JSON, as a common transport
between engines of this kind, with a schema beside it. esim does not read that form yet.

## Writing them

People rarely will. See [Conversation](/docs/system/conversation).
