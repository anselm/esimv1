# A data service beside the simulator: Anselm's proposal and a first reading

27 November 2026. Claude (Fable 5.2), esimv1 room on canada1. Follows `20261127-the-raw-data-so-far.md`.

## What he proposed

Anselm: "esimv1 and esimv1-core will grow, there will be thousands and thousands of scenarios, most of which are self
similar. i see an opportunity for a consolidator of lower level data products. we could call this an esimv1-data
service." Its features, in his words, shortened:

- "it can be stood up and run by anybody anywhere - so we aren't centralizing"
- "a moderately rich api surface ... it can do all of the kinds of data carve-outs, generations of data that we want"
- "cached durable data products for any situation where we are doing slight refinements - carve-outs, some light llm
  analysis and so on"
- "data products it generates can have direct reasonably named urls or paths ... effectively long lived caches that
  are rebuilt on demand - and where the burden of rebuilding is in this service - not in our code"
- "it would store these artifacts as cloudflare data objects"; "a catalog or directory of the data products it had
  built"; "public, no secret data"
- "somehow prevent anybody asking it to do arbitrary work"

And the question: "a variation of this idea is that the esimv1-core itself takes on this role? or do we separate the
'simulator' ... from the lower level data products that are more 'timeless'?"

## The reading given (nothing decided, nothing built)

Separate it. The seam exists: the kernel knows nothing of data, and the `esim_registry` crate (catalogue, fetcher,
hash list) is this service in embryo. A separate tool may depend on GDAL and call a language model; the engine may not.

Four adjustments:

1. Reading and building are different doors. Reading is static files from the bucket, with no program in the path.
   Building is a command run by whoever holds the bucket's write key; a request from anyone else is a line added to a
   products list in git, built when merged. No anonymous caller can cause work.
2. Two kinds of product. A derivation is a pure function of pinned upstream bytes and can be rebuilt (a cut of
   Copernicus). A snapshot is a dated answer from a moving source (OpenStreetMap, an API, a language model) and cannot:
   for it the stored object is the archive, the date is in the path, and it is never rebuilt in place.
3. A run names a product by path and hash and looks through a list of stores (local cache, own bucket, the public
   one). That is what makes anybody's copy as good as ours.
4. Sharing between similar scenarios happens only on a fixed tiling. A cut to each scenario's own window is unique
   to it. Shared planes are tiles; a place record is the exception that is unique by nature.

Runs stay the engine's to build but can be listed in the same index, and a run's frame can be published as a product.

Open: licences for public redistribution per source (some products may be recipe-only); whether the first form is
the registry crate lifted out or a new tool beside it; cost and size are unmeasured.

First step suggested: buildings as the first plane, since four scenarios use four sources.
