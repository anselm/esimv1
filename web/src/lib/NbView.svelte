<script>
  // One `esim-` tag of a notebook page, drawn. The tag's name says what to draw and its
  // attributes say which part of the run; the run and the moment come from the shared context.
  //
  //   <esim-scene>                      the place in three dimensions, by layer
  //   <esim-timeline play>              the bar that moves the moment (play: start playing)
  //   <esim-map field="deaths">         one map frame at the moment
  //   <esim-chart series="power.share"> one series over the run
  //   <esim-counter series="deaths.total" label="dead, all causes" [at="end"]>
  //   <esim-story [count="6"]>          what has happened so far, newest first, with sources
  //   <esim-module name="tanks">        one module's own panel as the default page draws it
  //   <esim-modules [only="a,b"]>       every module's panel, or the named ones
  //   <esim-wiring>                     how the modules are connected
  //   <esim-readme>                     the project's README.md
  //   <esim-run-link>text</esim-run-link>  a link to this run's full data page
  import Layers3D from './Layers3D.svelte'
  import Timeline from './Timeline.svelte'
  import FieldMap from './FieldMap.svelte'
  import Chart from './Chart.svelte'
  import ModuleViews from './ModuleViews.svelte'
  import Wiring from './Wiring.svelte'
  import Readme from './Readme.svelte'
  import { seriesAt } from './runs.js'
  import { num, isoTime } from './fmt.js'

  let { tag, attrs = {}, ctx, path, text = '' } = $props()
  const run = $derived(ctx.run)
  const context = $derived((run?.describe?.modules ?? []).map((m) => m.tables?.context).find(Boolean) ?? null)
  const at = $derived(attrs.at === 'end' ? run?.end : attrs.at === 'start' ? run?.start : attrs.at ? Date.parse(attrs.at) / 1000 : ctx.cursor)
  function show(name) {
    const x = seriesAt(run, name, at); if (x === null || x === undefined) return '—'
    const a = Math.abs(x)
    if (a >= 1e9) return `${(x / 1e9).toFixed(2)} bn`
    if (a >= 1e6) return `${(x / 1e6).toFixed(1)} M`
    if (a >= 100 || Number.isInteger(x)) return num(Math.round(x))
    if (a <= 1 && (run.series?.[name]?.unit ?? '') === '1') return `${(x * 100).toFixed(0)}%`
    return x.toFixed(a < 10 ? 2 : 1)
  }
  let readme = $state(null)
  $effect(() => { if (tag === 'esim-readme') fetch(`/projects/${path}/README.md`).then((r) => (r.ok ? r.text() : '')).then((t) => { readme = t }) })
  const sentences = $derived(ctx.crossings.filter((c) => c.t <= (ctx.cursor ?? Infinity) && c.data?.sentence).slice().reverse().slice(0, Number(attrs.count ?? 6)))
  const only = $derived(attrs.only ? attrs.only.split(',').map((s) => s.trim()) : attrs.name ? [attrs.name] : null)
</script>

{#if ctx.error}
  <p class="text-sm text-leak">{tag}: {ctx.error}</p>
{:else if !run}
  <p class="muted text-sm">loading…</p>
{:else if tag === 'esim-scene'}
  {#if run.fields?.terrain_elev_m}<Layers3D {run} base={ctx.base} cursor={ctx.cursor} event={ctx.event} />{:else}<p class="muted text-sm">this run has no elevation frame, so there is no scene to draw</p>{/if}
{:else if tag === 'esim-timeline'}
  {#if ctx.cursor !== null}<Timeline t0={run.start} t1={run.end} bind:cursor={ctx.cursor} autoplay={'play' in attrs} moments={ctx.moments} event={ctx.event} />{/if}
{:else if tag === 'esim-map'}
  {#if run.fields?.[attrs.field]}<FieldMap name={attrs.field} field={run.fields[attrs.field]} base={ctx.base} cursor={ctx.cursor} {context} />{:else}<p class="text-sm text-leak">no map frame named {attrs.field} in this run</p>{/if}
{:else if tag === 'esim-chart'}
  {#if run.series?.[attrs.series]}<Chart name={attrs.title ?? attrs.series} unit={run.series[attrs.series].unit} points={run.series[attrs.series].points} t0={run.start} t1={run.end} cursor={ctx.cursor} event={ctx.event} />{:else}<p class="text-sm text-leak">no series named {attrs.series} in this run</p>{/if}
{:else if tag === 'esim-counter'}
  <span class="inline-block align-top">
    <span class="mono block text-2xl">{show(attrs.series)}{#if attrs.unit}<span class="ml-1 text-sm muted">{attrs.unit}</span>{/if}</span>
    <span class="block text-xs muted">{attrs.label ?? attrs.series}</span>
  </span>
{:else if tag === 'esim-story'}
  <ol class="space-y-2">
    {#each sentences as c, i (c.t + ':' + (c.data.id ?? i))}
      <li class={i === 0 ? '' : 'muted text-sm'}>{c.data.sentence}{#if c.data.source && 'sources' in attrs}<span class="mono block text-[11px] muted">{c.data.source}{c.data.confidence ? ` · ${c.data.confidence}` : ''}</span>{/if}</li>
    {/each}
    {#if !sentences.length}<li class="muted text-sm">nothing has happened yet at {ctx.cursor === null ? '' : isoTime(ctx.cursor)}</li>{/if}
  </ol>
{:else if tag === 'esim-modules' || tag === 'esim-module'}
  {#each (run.describe?.modules ?? []).filter((d) => !only || only.includes(d.name)) as d (d.name)}
    <ModuleViews describe={d} {run} base={ctx.base} cursor={ctx.cursor} crossings={ctx.crossings} {context} event={ctx.event} />
  {/each}
{:else if tag === 'esim-wiring'}
  {#if run.describe?.modules?.length}<Wiring modules={run.describe.modules} info={run.describe.info ?? []} />{/if}
{:else if tag === 'esim-readme'}
  {#if readme}{#key readme}<Readme {path} md={readme} />{/key}{/if}
{:else if tag === 'esim-run-link'}
  <a href={`/${path}/${encodeURIComponent(ctx.id)}?raw`} class="text-brand hover:underline">{text || 'everything this run recorded'}</a>
{:else}
  <p class="text-sm text-leak">unknown tag {tag}</p>
{/if}
