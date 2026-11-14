<!-- order: 2 -->
<!-- section: Telling it -->
# Pages written in HTML

A project can lay out its own pages. The page is ordinary HTML, written by hand or with a
language model, and the site's views are placed in it by tag. This is how a scenario gets a page
made for its audience: tabs, columns, commentary beside a map, whatever the story needs.

There are two files, both beside the project's manifest:

- `page.html` is the project's page. It takes the place of the automatic 3D view and the README.
- `run.html` is the page for every run of the project. It takes the place of the automatic list
  of panels. A link at the foot of the page, and `?raw` on the address, still show everything the
  run recorded as the engine lays it out.

A project with neither file gets the automatic pages, and a `README.md` is still rendered as
before.

## The tags

Each tag draws one view. A tag shows the page's own run. On a project page, or to show another
run beside it, add `run="wet-night"` with a variant's name or a run's id. Every tag of the same
run shares one moment in time, so one timeline moves them all.

| Tag | What it draws |
|---|---|
| `<esim-scene>` | The place in three dimensions, with the model's parts by layer. |
| `<esim-timeline>` | The bar that moves the moment. Add `play` to start it playing. |
| `<esim-map field="deaths">` | One map at the moment shown. |
| `<esim-chart series="power.share" title="...">` | One series over the run. |
| `<esim-counter series="deaths.total" label="...">` | One number at the moment shown. `at="end"` fixes it at the end of the run. |
| `<esim-story count="6" sources>` | What has happened so far, newest first. `sources` adds each sentence's source. |
| `<esim-module name="tanks">` | One module's own panel. |
| `<esim-modules only="tanks,bridges">` | Every module's panel, or the named ones. |
| `<esim-wiring>` | How the modules are connected. |
| `<esim-readme>` | The project's `README.md`. |
| `<esim-run-link>text</esim-run-link>` | A link to the run's full data page. |

The names a map or a series goes by are listed on a run's full data page and on the
[kinds page](/kinds).

## Tabs and columns

`<esim-tabs>` turns each `<section title="...">` inside it into a tab. A `<div class="nb-cols">`
lays its children out in columns that fold on a narrow screen. Anything else is up to the page:
it may carry its own `<style>` and its own `<script>`.

```html
<esim-scene></esim-scene>
<esim-timeline></esim-timeline>
<esim-counter series="deaths.total" label="dead, all causes"></esim-counter>

<esim-tabs>
  <section title="Fuel">
    <p>The terminals are both the spill and the reserve.</p>
    <esim-map field="tank_release_gal"></esim-map>
    <esim-chart series="fuel.hub_gal" title="fuel left at the terminals"></esim-chart>
  </section>
  <section title="Everything recorded">
    <esim-modules></esim-modules>
  </section>
</esim-tabs>
```

`projects/anselm/portland-cascadia/run.html` is a full example.

## What to know

- One `run.html` serves every variant of a project, so its words should say what to look at and
  leave the numbers to the counters, which read them from the run.
- The HTML is trusted. It comes from the repository, where changes are reviewed, and its scripts
  run. When people can upload pages from the site this will need a sandbox.
- Run folders are rebuilt whenever a scenario is rerun, which is why the page lives with the
  project and not inside a run.
