# Object storage: a first try that stopped at the credential

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
