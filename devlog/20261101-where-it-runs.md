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

## Addendum, same morning: which cloud, concretely

Anselm: "right now canada1 is my own internal dev machine. You're suggesting we can run a
public service on a rented machine in the cloud - maybe on Google cloud run or exe.dev if
they have beefier machine options? What about cloudflare edge workers or something? How do
people normally run some compute for situations like this in the cloud?"

Checked, not recalled: the exe.dev plan (`ssh exe.dev billing plan`, 2026-11-01) is
2 vCPUs and 8 GB shared across all his VMs, 100 GB disk, 250 GB transfer a month; the
hormuz VM reports 2 cores and 7 GB on an EPYC 9554P host. `new --standalone` exists
"with its own reserved capacity" but needs a different plan and team approval, and the
sizes are not published in the help text. So exe.dev as it stands is where the facade and
the wiki live, not the workers.

How this class of service is normally run, from the agent's knowledge (prices and limits
are as remembered up to mid-2026 and must be checked before buying):

- **Edge workers (Cloudflare Workers, Lambda@Edge):** no. JavaScript or wasm only, about
  128 MB, CPU time capped in seconds to a few minutes. Right for the API shell, the auth
  hop and serving results; wrong for a 15 s to 8 min native job. Cloudflare Containers
  (2025) can run a container at the edge but the instance sizes were small when last seen.
  Cloudflare's real gift here is **R2**, object storage with no egress charge, which is the
  right home for the two immutable stores since run frames are the heavy bytes a public
  viewer pulls.
- **Serverless containers that run to completion (Cloud Run Jobs, AWS Fargate/Batch, Fly
  Machines, Modal):** this is the normal answer for bursty compute of our shape. Hand it a
  container with `esim` in it, one task per branch or per ensemble, up to the hour range,
  tasks fan out in parallel, scale to zero when idle, pay per vCPU-second. Cloud Run Jobs
  as remembered: up to 8 vCPU and 32 GB per task, 24 h max, parallel tasks in the
  thousands, roughly $0.000024 per vCPU-second; a basin branch-year at 15 CPU-s is well
  under a tenth of a cent, a hundred-member ten-year ensemble under a dollar. Modal is the
  same idea with on-demand GPUs per job, which is how a GPU field module should be paid
  for later: per job, never a standing card.
- **Rented bare metal (Hetzner AX-class, OVH):** 32 to 64 EPYC cores and NVMe for a flat
  monthly fee, cheapest per sustained CPU-hour by a wide margin, but someone owns the box.
  This is where to move once the serverless bill is steady and larger than a box.
- **Kubernetes (what Pangeo, large modelling shops do):** correct at scale, needless here.
- **HPC queues (Slurm):** how national weather centres run; not a SaaS shape.

Recommendation, in stages, with no design change between them:

1. **Now:** canada1 is the only worker. Publish finished runs to R2 and make the viewer a
   static site (Cloudflare Pages or exe.dev) reading from it. First users can already use
   this; the box's uptime is the limit, which Anselm tolerates.
2. **First public service:** a small always-on API (one exe.dev VM or a Cloud Run service)
   holding the index and queue; Cloud Run Jobs or Fly Machines as the workers, each warmed
   from the shared R2 cache; R2 for cache and runs. Cost tracks usage to zero.
3. **Sustained:** one Hetzner-class box of workers when the monthly serverless bill
   exceeds it; keep the serverless pool for bursts.
4. **GPU:** per-job on Modal or RunPod when a field module measures cell-bound.

The code asks are unchanged: stores behind a trait, chunked frames, a job runner whose
worker is a container with `esim` in it.
