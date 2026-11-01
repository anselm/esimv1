# 2026-10-31 — Runtime choice and the use cases to validate against

Anselm, same day, after the brief: "we probably need to not worry too much about how i did
things before ... javascript is easier true - but since we can write rust just as easily - is
rust a good candidate or wasm ... i worry that if we are promising to try tackle this 'for
real' for real world use cases - we need to be more performant." And: "it has to do one thing
that other products cannot do - which is be able to do near term whole system predictions ...
not just model a fire, but model impacts on wildlife and so on - like unexpected consequences.
we need a list of practical real world uses to validate the work against."

Written by the agent (Claude, this room). Recommendations, not decisions.

## Runtime

Recommendation: Rust for the kernel and the heavy modules. Wasm later as a module boundary,
not as the runtime. JavaScript stays for the facade, the viewer and manifest tooling.

Why Rust and not JS:
- The brief's expensive parts are fields (PDE-like, small timesteps) and ensembles (a
  thousand branches over decades). Single-thread typed-array JS is within a small factor of
  native for the arithmetic, but it has no cheap threads, no SIMD worth the name, no GPU
  without a browser, and a garbage collector in the inner loop. Those are the things that
  turn into a rewrite later.
- Determinism under parallelism is an open question in the brief. Rust gives data-race
  freedom at compile time, which is the half of that problem a language can solve.
- The reference models to port are C (SWMM), C++ (LPJ-GUESS), C# (Madingley), Fortran
  (DSSAT) and Rust already (Whitebox). Rust ports those more naturally than JS does and can
  link the originals for proof-by-matching.
- Agents write Rust about as fast as JS now, and the compiler catches a class of agent
  mistakes that JS lets through to runtime. The scarce resource is validation, so a language
  that moves errors earlier is worth the slower compile.

Why not wasm as the core: no threads without the browser's cooperation, no GPU, a 4 GB
memory ceiling on wasm32, and a call cost at the boundary that punishes fine-grained events.
It is the right answer to the deferred question of how a module gets into the process (any
language, sandboxed, same bits on server and in browser). Keep the module interface wasm-
compatible from day one (plain memory buffers, no shared pointers) and the option stays open.

What is lost: the late-binding sloppiness he liked. Compensation: the late binding moves
from code to data. The manifest is the loose, fast, editable layer; it wires modules,
parameters and sockets at run time with no recompile. Fast-and-sloppy happens in manifests
and parameter sweeps, the compiled kernel stays strict. That is also what the conversational
facade needs: a document an LLM can edit.

The one serious alternative is Julia (scientific, fast, multiple dispatch suits the five
representations). Ruled out for a product: deployment weight, small ecosystem for the
server-and-browser story, and the two-language problem comes back the moment a facade exists.

## What this product does that others do not

Every existing tool models one domain to high fidelity: fire (FARSITE, WRF-SFIRE), floods
(HEC-RAS), nutrients (SWAT, SPARROW), traffic, crops, catastrophe losses (the reinsurance
models). The gap is the consequence chain across domains, near term (one to thirty years),
with the mechanism visible. The customer question is never "how big is the fire"; it is "what
does the fire do to my water supply, my elk herd, my tax base, my insurance in the three years
after." Nobody sells that answer today because nobody has the coupler.

## Use cases, as validation targets

Each has a named kind of user, a question they already ask on a working day, the chain of
domains crossed, and a scoreboard that reality keeps without us. Willingness to pay is the
agent's guess throughout and is marked as such.

1. Nutrient and hypoxia (the spine). User: a state water-quality agency, a Mississippi basin
   nutrient task force staffer, a fertilizer co-op. Question: if these counties shift to
   cover crops or split nitrogen application, what happens to spring nitrate loads, July
   hypoxia, and Gulf shrimp? Chain: rules and markets, crop populations, soil and runoff
   fields, river network, ocean biology, fishery prices. Scoreboard: USGS nitrate loads, the
   NOAA/LUMCON July hypoxia cruise since 1985, shrimp landings. Pays: agencies by grant or
   contract (guess).
2. Fire to water (his own example of unexpected consequences). User: a municipal water
   utility or county emergency manager in the western US. Question: a fire in this watershed
   this summer; what are the odds of debris flows and reservoir turbidity events in the next
   three monsoon seasons, what does treatment cost, what happens to the trout and elk? Chain:
   fire spread field, vegetation populations, soil and hydrology, river network, built
   infrastructure, wildlife cohorts, utility budget. Scoreboard: MTBS burn perimeters, USGS
   gauge turbidity and sediment, utility treatment records. Precedent the agent is confident
   of: Denver Water's post-fire sediment costs after the 1996 and 2002 fires. Pays: utilities
   do spend on this already (guess at the amount).
3. A parcel over twenty years (the consumer case). User: a landowner, a small farm, a land
   trust. Question: what happens to this forty acres under these choices, with the county's
   zoning, the water right, the market and the climate already in the pipeline? Chain: all
   five representations at parcel scale, nested in a basin. Scoreboard: weak until a hindcast
   exists; MODIS greenness and county yield data per parcel area. Pays: little per user, but
   this is the facade's native use and the thing that makes the product legible. The
   portfolio already has a candidate parcel (dadsfarm).
4. River basin shortage. User: an irrigation district or city water planner on the Colorado
   or another over-allocated river. Question: under a shortage declaration, who is curtailed,
   what gets fallowed, what happens to local jobs and to dust from a shrinking terminal lake?
   Chain: rules (prior appropriation), network, fields, crops, agents. Scoreboard: Reclamation
   reservoir elevations, state acreage reports, air monitors. Builds on the colorado project.
   Pays: districts and cities (guess).
5. Marine heat to coastal economies. User: a dive-tourism association, a fisheries council, a
   coastal town. Question: given this season's heat forecast, what are the odds of bleaching
   or a kelp collapse, and what does that do to bookings and landings over three years? Chain:
   ocean field, reef or kelp populations, fauna cohorts, firms. Scoreboard: Coral Reef Watch
   degree heating weeks, landings data. Close to into-the.blue. Pays: unknown.
6. Compound hazard losses. User: an insurer or reinsurer, a mortgage portfolio. Question:
   where do fire, flood after fire, heat and grid failure compound in the next five years,
   and what does that do to loss ratios and to who can get insured? Chain: hazards with
   state-dependent rates, fields, built landscape, rules, markets. Scoreboard: claims and
   non-renewal data, which are mostly private. Pays: the most of anyone (confident), hardest
   to get the scoreboard (confident).
7. Commodity chokepoint to hunger. The Hormuz chain, generalised: any shock to a chokepoint
   or breadbasket, through fertilizer and fuel to yields, stocks, export bans and consumption
   in import-dependent countries. User: a food-security analyst, an importer, a development
   bank. Scoreboard: FAO price index and stocks, the 2008, 2011 and 2022 crises as hindcasts.
   Pays: institutions (guess).
8. Grid and heat. User: a utility planner or county health department. Question: this heat
   wave with this hydro year; outage odds, pumping and cooling failures, excess deaths by
   neighbourhood. Chain: weather field, network, built landscape, agents. Scoreboard: EIA
   outage reports, mortality records. Pays: utilities and public health (guess).

## Which three to build against first

The agent's recommendation: 1, 2 and 3. Together they cover all five representations and the
nesting problem, two of them have scoreboards decades long, and the third is the one a person
can hold in their hand. 6 is where the money is but its scoreboard is private; revisit once 1
and 2 close their ledgers.

## Things Anselm needs to decide

- Rust kernel, or stay in JS and accept a later rewrite.
- Open source with a paid service on top (his stated instinct), which argues for the kernel
  and modules being public and the data assimilation, hosting and ensembles being the product.
- Whether 1, 2, 3 are the right three, and who the first real user is for each. A name of a
  real person who would run it beats any amount of this list.

## Addendum, same day: Wasm 3.0

Anselm pointed at https://webassembly.org/news/2025-09-17-wasm-3.0/ (2025-09-17). Read by the
agent on 2026-10-31. It changes the wasm half of the recommendation above, not the Rust half.

Useful to this project, from the announcement:
- Deterministic profile. The spec now defines a deterministic default for everything that was
  implementation-defined, NaN payloads and relaxed SIMD included, and names "replayable
  systems" as a target. Replay of a branch across machines becomes a property of the module
  sandbox rather than of module authors' discipline. This is the strongest argument for wasm
  as the module boundary.
- memory64. The 4 GB ceiling the agent cited is gone; the page says 16 GB on the web, and the
  standalone-engine limit is not stated there (unknown).
- Multiple memories per module. A module can own a private memory and import a shared one,
  which maps onto the brief's split of module-private state from shared world state, and the
  spec adds copying between memories.
- Relaxed SIMD. Faster field arithmetic, with the deterministic profile pinning its behaviour
  where replay matters more than speed; a per-module choice.
- GC and typed references. Irrelevant to Rust modules, but a module written in a GC language
  (a port of Madingley from C#, say) can target wasm without shipping its own collector.

Not in the announcement: threads and the component model are not mentioned at all. The agent
believes shared-memory atomics ship separately in browsers and in Wasmtime, from memory and
unverified here. Still absent: GPU access, and the per-call cost at the boundary.

Revised shape: a native Rust kernel (scheduler, event log, ledgers, ensemble fan-out across
threads, any GPU fields) hosting modules compiled to wasm and run under the deterministic
profile in Wasmtime. Modules advance one dt per call, not one event per call, so the boundary
cost is amortised. Status: "Wasm 3.0 is already shipping in most major web browsers" and
Wasmtime support is "on track to completion", per the page; check the feature status page it
links before relying on memory64 or the deterministic profile in a given engine version.
