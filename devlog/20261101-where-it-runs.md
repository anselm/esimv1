# 2026-11-01 — Where a service like this runs, and how it scales to many users

Anselm, the morning after the overnight ladder: "What's a good place to run a tool like
this - do we want gpu of some kind? If this is a saas service presumably it will have some
kind of data store which might be distributed, maybe some kind of compute instance per user
or something? I'm trying to imagine how a service like this scales to multiple users."
Agent's assessment, resting on measured costs from the overnight runs, not estimates.
Companion to `20261031-large-runs-vs-web.md` (runs are jobs, the app is a client of results).

## What a run actually costs today (canada1, release build, one thread per branch)

| run | simulated span | wall | events | on disk |
|---|---|---|---|---|
| m10 all modules, Boone basin | 1 year | 15 s | 96k | 45 MB |
| m11 nitrogen hindcast | 5 years | 74 s | 486k | 167 MB |
| f1 forecast ensemble, 31 members | 22 months each | 118 s total, 32 threads | 4.6M | 85 MB |
| registry cache for the basin (DEM, GHCN, USGS, ERA5, GEFS, MODIS) | | | | 527 MB |

Disk is almost entirely field frames (daily f32 rasters). CPU is one core per branch; the
kernel is a serial discrete-event loop by design and does not use the GPU at all.

## No GPU, for now

The work is a scheduler delivering events to modules. That is serial and branchy, the
opposite of GPU-shaped. What parallelises is (a) branches of an ensemble, which already
run one per thread, and (b) per-cell field maths inside a module: fire spread, routing,
irradiance, ET. At the current grid size (b) is a few milliseconds a day and not worth
the trip to the GPU.

A GPU starts to pay when a field module runs on millions of cells per step: the fire CA
on a 30 m grid for a western basin, routing a whole state at 30 m, planetary irradiance,
IPCC-scale gridded fields. Before that, rayon over cells inside the field modules gives
most of a 10 to 20 times on a many-core box with no new dependency. The brief's note that
Rust reaches the GPU through wgpu stands; it is a later worker class, not the first box.
canada1's 96 GB iGPU is a fine place to develop that when the time comes.

## Not an instance per user

The users never run code. A project is a manifest and a stack of diffs (what-ifs), which
is data, and the modules are compiled into the one `esim` binary. So there is nothing to
isolate per user and no reason to pay for an idle machine each. The right shape is a
queue and a pool of identical stateless workers that pull jobs. A run is seconds to
minutes, bursty, and embarrassingly parallel across branches, which is exactly what a
shared pool is good at. Fairness is a scheduler policy (branch-years per user per hour),
not a VM boundary.

If user-authored modules ever arrive, the isolation unit is a wasm sandbox inside the
worker (the brief's optional loader), still not a VM per user.

## Three stores, two of them immutable

1. **Reference cache** (`data/cache/`): content-addressed, sha256-locked, shared by every
   user and every run, written once. This is the easiest thing in the world to distribute:
   an object store (R2 or S3) with a CDN in front, keyed by hash. "Never fetch base data
   twice" becomes "never fetch it twice for the whole fleet", which also fixes the
   Open-Meteo terms problem: the service pulls ERA5 and GEFS once from a licensed source
   into the shared cache and no worker talks to Open-Meteo.
2. **Run store** (`runs/`): finished runs are immutable datasets (log, snapshots, frames,
   ensemble quantiles). Same answer: object store, immutable blobs, served straight to the
   viewer. Frames need chunked and compressed storage before this gets big (167 MB for
   five basin-years will not survive a state-sized grid); zarr-style chunking or COG
   output is the obvious move, and it is what the viewer wants anyway.
3. **Index**: users, projects, manifests, job states, run ids. Small and mutable. SQLite
   on one box is honest for a long time; Postgres when there are two boxes. `runs/index.json`
   is this today.

Snapshots make forking a what-if a seek, not a rerun, so the run store is also the
snapshot store (D3 fork-from-snapshot).

## Where, concretely

- **Now:** canada1. Thirty-two threads is a hundred-member ensemble over ten basin-years
  in about eight minutes. That covers development and the first users.
- **First service:** one dedicated many-core box with NVMe (Hetzner-class rentals of 32 to
  64 cores are cheap compared with cloud VMs for sustained CPU) running the workers and
  the index, plus R2 for the two immutable stores. exe.dev stays the right home for the
  small web facade and the wiki, not for compute.
- **Growth:** more identical workers pulling from the queue, warmed from the shared cache
  on first job. A GPU worker class when the field modules need it. Nothing in the design
  changes between these stages; only the number of workers and the store URLs.
- **Cost model:** billing by branch-year is still the right unit because it is the unit the
  machine pays in. Today a basin branch-year is 15 CPU-seconds.

## What this asks of the code, in order

1. Chunked, compressed frame output and a viewer that reads ranges (also fixes disk now).
2. Stores behind a trait: filesystem today, object store tomorrow, same hashes.
3. A job runner: queue, worker pool, progress events, result store (V5 in the deliverables).
4. rayon inside the field modules when a grid gets large enough to notice.
5. wgpu for a field module only when a measured step is cell-bound at millions of cells.

Because manifests, the cache and runs are all plain files keyed by hash, the same `esim`
runs on a laptop with no service at all. The SaaS is a shared cache, a queue and a viewer
around that; it is a convenience, not a dependency, which fits the open-source intent.
