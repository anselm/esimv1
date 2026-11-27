# data.earthlight.cloud, a first public address for the data store

27 November 2026. Claude (Fable 5.2), esimv1 room on canada1, session 018TGCFiQHbUYn12HkNti2xk.

Anselm: "I got earthlight.cloud - we can put this at data.earthlight.cloud". Then, from his phone, when the
permission for a public bucket turned out to be hard to grant there: "Let's show what we can - I am headed to a
retreat with industry luminaries and I want to show something".

## What is live

- <https://data.earthlight.cloud/> is the catalogue page; `/catalog.json` is the STAC root; each buildings
  product has a 3D link (`view/massing.html`, written into the store by `esim-data index`).
- 20 products, 52 MB: buildings for Sausalito, Nicasio, Half Moon Bay, Pacific Palisades and Portland, roads
  for the four Californian places, two tables, three upstream pointers.

## How it is served, and why it is a stand-in

Neither token on canada1 may create a bucket (the account token has no storage rights; the storage token is
object read and write on `esimv1` only). So:

- the store was pushed into the private bucket under `data/` (`esim-data push s3:~/.config/cf/r2:data`);
- `data.earthlight.cloud` is a DNS-only CNAME to `esimv1.exe.xyz`, added to the VM with `exe-domain`
  (exe.dev refused it for about a minute until the new record reached its resolver);
- `web/server.mjs` answers that host (`ESIM_DATA_HOST`) by passing on objects under `data/`, read only, with
  `Access-Control-Allow-Origin: *`; `DATA_HOST=data.earthlight.cloud deploy/deploy.sh esimv1` sets it.

Every download passes through the small VM. The addresses are the final ones.

## To finish, when he is at a desk

Add "Workers R2 Storage: Edit" to the account token (or make the bucket by hand). Then: a public bucket
`esimv1-data`, the name attached to it in place of the CNAME, a cross-origin rule, the store pushed there, the
`data/` prefix and `ESIM_DATA_HOST` removed. No address changes.

## Not yet built

Fort McMurray (the OpenStreetMap server answered 502 for many minutes and the build was stopped to publish),
Portland's roads and OpenStreetMap buildings, the LA County footprints and the Palisades massing. The build is
resumable; a new push sends only what is new. Do not index or push while a build runs.
