# esimv1-viewer

Svelte 5 components that draw an esim run folder: field maps, charts, the timeline, the
two 3D scenes, the globe, ledgers, the audit, the wiring diagram, and the figures a
project README can embed.

A run folder is a set of files (`run.json`, `describe.json`, `log.jsonl`, `fields/`); the
engine writes it and this package reads it. The two share no code. The folder's layout is
described in the esimv1 documentation, *Projects and runs*, and `run.json` carries its
`format` number. This package reads format 1.

## Status

A first cut, made on 22 November 2026 by moving files out of the esimv1 site. It builds
and runs as part of that site and has not been used by any other page yet. It is not on
npm. Until it is used elsewhere, expect its edges to be the site's edges:

- The components are styled with Tailwind 4 utility classes and two theme colours
  (`--color-brand`, `--color-ok`, `--color-leak`). A host page needs Tailwind 4 and must
  point it at this package's `src/`.
- `three` and `svelte` are peer dependencies.
- Maps want coastlines and the README figures want a project folder. Their addresses are
  in `src/config.js`, with defaults that match the esimv1 site:

```js
import { configure, FieldMap, Timeline } from 'esimv1-viewer'
configure({ projects: 'https://example.org/projects', outlines: '/outlines.json' })
```

## Use

Every component that draws a run takes `base`, the address of the run folder, or `run`,
the parsed `run.json`:

```svelte
<script>
  import { FieldMap } from 'esimv1-viewer'
  let { base, run, t } = $props()
</script>
<FieldMap {base} {run} name="fire_arrival_h" {t} />
```

The props of each component are read from its source for now; they are not documented.
