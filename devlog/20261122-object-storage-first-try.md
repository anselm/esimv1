# Object storage: a first try

22 November 2026. Claude (Fable 5.2, the esimv1 room on canada1), on Anselm's word: "i
would like to exercise cloudflare object storage".

## Where it stopped

Nothing was stored. The two Cloudflare credentials on canada1 cannot reach object storage
(R2):

- `~/.config/cf/token` is the DNS token. It verifies as active and can list zones. Asked
  to list R2 buckets on the account, Cloudflare answers "Authentication error" (code 10000).
  The harness's notes say it is DNS-only on purpose.
- The wrangler sign-in in the harness's notes is scoped to Pages (`pages:write`,
  `account:read`, `zone:read`), and wrangler is not installed in this project.

I did not try to widen either. What is needed is Anselm's to give, one of:

1. In the Cloudflare dashboard: R2, then "Manage API tokens", a token with **Object Read
   and Write** limited to one bucket, saved on canada1 as its own file beside the DNS token
   (it comes as an access key id and a secret, for the S3 interface).
2. Or a wider API token that can also create buckets (Account, Workers R2 Storage, Edit).

R2 has to be switched on for the account first, which the dashboard asks a payment method
for even inside the free allowance. I have not checked today's prices or allowance.

## What the exercise will be, once there is a credential

The useful first test is the one the split has just made necessary: finished runs reaching
the public site without an rsync from the machine that made them.

1. A bucket, public for reading, holding `projects/` as the site serves it: project
   folders with their runs, less `snapshots.json`. On canada1 that is about 6.5 GB of the
   21 GB of project runs (14.4 GB is snapshots, which a reader never needs).
2. The site's server, given the bucket's address, sends a request for a file under
   `/projects/` there when it does not hold the file itself.
3. Open a run page on a copy of the site that holds no runs, and look.

What I expect it to turn up, from reading the code today:

- **The log filter.** Two components ask for `log.jsonl?topic=<name>` and rely on the
  server to send only the lines with that topic (a Portland log is tens of megabytes). A
  bucket cannot filter. Either the engine writes a small file per topic it knows a view
  wants, or the viewer filters after downloading the whole log. The first is the better
  answer and belongs in the run folder's format.
- **Cross-origin reads.** The viewer fetches with `fetch`, so the bucket must allow the
  site's origin. One setting on the bucket.
- **Compression.** The server gzips `.f32` frames and JSON as it sends them, five to ten
  times smaller. A bucket sends what it was given, so the files should be compressed
  before upload and stored with their content encoding.
- **Names that change meaning.** A project's run folder is replaced when a variant is
  rerun, and the site caches run files for a year. With a bucket and Cloudflare's cache in
  front, a rerun would need its address to change or the cache to be told.

Second, later: the engine's fetched cache (4.2 GB) as a mirror, so a second machine or a
cloud session can run a scenario when a publisher's server is slow or refuses it. The
slow USGS file server met in the Sausalito work today is the case for it.

Nothing in this entry was built.

## Later the same day: it works, with the bucket kept private

Anselm made the bucket (`esimv1`) and a token. The credentials are in `~/.config/cf/r2`
on canada1 (mode 600). They passed through a chat as a screenshot, so they should be
rotated. The Cloudflare endpoint refused TLS for the first minutes after R2 was switched
on and then answered.

What the token can do decided the design. It reads and writes objects in the one bucket.
It cannot list buckets, turn on a public address or set a cross-origin rule. So the
bucket stays private and the site's server is the reader:

- `deploy/store.py put <area/name> [run ...]` uploads a project folder as the site serves
  it, leaving out `snapshots.json` and the record. Text and raw-float files are gzipped
  first and stored with their content encoding.
- `web/lib/store.mjs` signs each read itself (no SDK). With `ESIM_STORE` naming the
  credentials file, the server reads from the bucket any file under `/projects/` that the
  machine does not hold, and passes it on as stored. `ESIM_STORE_FIRST=1` asks the bucket
  before the disk, which is how it was tested on a machine that has both.

![A Palisades run page drawn from the bucket](img-20261122-run-from-the-bucket.png)

Measured:

| run | on disk, less snapshots | stored | upload |
|---|---|---|---|
| Half Moon Bay, `as-built` | 90.1 MB, 145 files | 1.0 MB | 2 s |
| Palisades, `palisades-cells/vector-5` | 61.9 MB, 242 files | 9.6 MB | 3 s |

A copy of the site on port 5183, asking the bucket first, served `run.json` and a field
frame byte for byte the same as the disk, answered a log filtered to one topic (156 bytes
on the wire where the whole log is megabytes), fell back to the disk for a run that was
not uploaded, and drew the Palisades run page with its 3D view and no page errors.

Of the four things I expected to have to fix, three went away because the server stayed
in the path: the log filter still runs on the server, there is no cross-origin read, and
compression is done once at upload. The fourth stands: a rerun replaces a run under the
same name, so stored files are marked to be cached for a minute only.

What this does not show yet:

- The public site. The deployed copy has not been touched. To use the bucket there, the
  VM needs the credentials file and `ESIM_STORE`, and the deploy would upload runs in
  place of the rsync. That is a deploy and waits for Anselm's word.
- Size and cost at full scale. Only two runs are in the bucket (387 objects, 10.6 MB).
  The two ratios above differ by ten times, so I will not guess the whole from them.
  Prices and the free allowance were not checked.
- Reads straight from the bucket by a browser, which would take load off the VM. That
  needs a public address and a cross-origin rule, which this token cannot set.
- The engine's fetched cache as a mirror. Not started.
