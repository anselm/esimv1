# 2026-10-31 — Events, space, late binding, GPU, cells

Anselm, same day: orbital "had an event driven messaging backbone - there was a timer tick,
but many things could just respond to other events as they wished... there was really a need
for spatially related events; for example classical predator prey relationships are spatial."
Then: "rust allows late binding huh, nice - i forgot. also, we can easily talk to gpu via
rust." And: "some features - irradiance and say volumetric weather if we are able to do that -
are cellular - we can handle this?" Agent's answers.

## The backbone stays event-driven

The brief's scheduler is a discrete-event kernel, not a tick loop. A tick is just an event a
module schedules for itself; most modules react to events from others and schedule more.
That is orbital's bus with three things added: every event carries a virtual time and is
delivered in (time, priority, sequence) order, so the run is deterministic; the conservative
rule stops a module reading state from a neighbour that has not caught up; and every event is
logged, so the run replays. Publish and subscribe survive. What goes away is "react whenever
you like" in wall-clock terms, because that is where nondeterminism came from.

## Spatial events

Events carry an optional location (a cell, a point, or a region). Subscriptions can be by
topic, by cell set, or by neighbourhood. The kernel keeps a spatial index (a grid of cells at
the world's base resolution, with finer indices inside zoomed regions) so delivery to "every
subscriber within r of here" is a bucket lookup, not a scan. This is orbital's spatial hash
promoted into the kernel and made deterministic.

Predator and prey as the worked example, at three scales, same mechanism:
- Continental: both are density fields; the interaction is reaction-diffusion (Lotka-Volterra
  with movement) on the grid. No events between individuals at all.
- Basin: both are cohorts attached to cells (Madingley style). A predator cohort's feeding
  event is addressed to its cell and neighbours; prey cohorts there respond. Spatial events
  by cell.
- One valley: individuals with positions, neighbourhood queries, encounter events. Spatial
  events by radius.
The scale switch conserves the count, which is seam test 6. The need Anselm names is real
and is the same need at every scale; the kernel meets it once with the index.

## Late binding in Rust, honestly

Rust gives dynamic dispatch (trait objects) and a registry keyed by name, so a manifest can
wire any compiled-in module to any socket at run time without the kernel knowing it at
compile time. That is late binding of the arrangement, and it is most of what orbital's
bus was used for. It does not give JS's drop-a-file-in-and-it-runs. Adding a module the
binary has never seen needs one of: recompile (seconds to a minute, fine for us), a dynamic
library (fragile ABI, avoid), or the wasm loader (the deferred option). So: wiring is late
bound through the manifest; code is early bound, by design.

## GPU

Yes. wgpu gives compute shaders on Vulkan, Metal and DX12 from Rust with one API, and it
runs on canada1's Radeon through the Mesa Vulkan driver (the agent expects this to work and
has not tested it here). A field module that needs it keeps its arrays on the GPU and
exposes the same trait. Only a few fields will ever need it; start on the CPU and move a
module when a profile says so.

## Cellular things

Irradiance, weather, soil moisture, water, ocean: these are the brief's fields, and a field
is a typed array over a grid or a volume with a stencil update and a timestep set by
stability. Cellular automata and finite-difference PDEs are the same shape to the kernel. A
field module owns its arrays, advances one step per call, and publishes cell-addressed
events when a threshold is crossed. Irradiance is cheap: a function of time, latitude,
slope and aspect per cell, with an atmosphere factor. Terrain-driven layers (slope, aspect,
flow direction, accumulation) are derived once from the elevation reference data.

Volumetric weather is the honest exception. A real numerical weather core is the work of
institutions. The right move for us is to take weather as a boundary condition from
reanalysis and forecast products (ERA5 for the past, open forecast ensembles for the near
future) and run only what is local and cheap: downscaling to terrain, a simple boundary
layer, orographic rain. That gives ensembles for free (the forecast products are already
ensembles) and keeps the scoreboard honest. If a case ever needs a 3D dynamical core, it is
a module like any other, and it is the one that will want the GPU.
