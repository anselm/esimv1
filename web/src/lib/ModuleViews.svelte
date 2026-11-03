<script>
  // One module's section of a run page, composed from the views its describe declared:
  // counters in a row, maps in a grid, crossings beside their map, charts in pairs, text
  // as prose. A module that declared no views gets a map per frame and a chart per report.
  import { isoDate, num } from './fmt.js'
  import FieldMap from './FieldMap.svelte'
  import Chart from './Chart.svelte'

  let { describe, run, base, cursor, crossings = [] } = $props()

  const views = $derived.by(() => {
    if (describe.views?.length) return describe.views
    const out = []
    for (const f of describe.frames ?? []) if (run.fields?.[f.name]) out.push({ kind: 'map', title: f.name, source: f.name, note: '' })
    for (const r of describe.reports ?? []) if (run.series?.[r.name]) out.push({ kind: 'chart', title: r.name, source: r.name, note: '' })
    return out
  })
  const of = (kind) => views.filter((v) => v.kind === kind && (kind === 'text' || kind === 'crossings' || (kind === 'map' ? run.fields?.[v.source] : run.series?.[v.source])))
  const counters = $derived(of('counter'))
  const maps = $derived(of('map'))
  const charts = $derived(of('chart'))
  const texts = $derived(of('text'))
  const crossViews = $derived(of('crossings'))
  const stories = $derived(views.filter((v) => v.kind === 'story'))
  // every crossing so far with a sentence, newest first
  const sentences = $derived(crossings.filter((c) => c.t <= cursor && c.data?.sentence).slice().reverse())

  function at(name) {
    const pts = run.series?.[name]?.points
    if (!pts?.length || cursor === null) return null
    let lo = 0, hi = pts.length - 1
    if (pts[0][0] > cursor) return null
    while (lo < hi) { const mid = (lo + hi + 1) >> 1; if (pts[mid][0] <= cursor) lo = mid; else hi = mid - 1 }
    return pts[lo][1]
  }
  function show(v) {
    const x = at(v.source)
    if (x === null) return '—'
    const unit = run.series?.[v.source]?.unit ?? ''
    if (unit === 'persons') return x >= 1e9 ? `${(x / 1e9).toFixed(2)} bn` : x >= 1e6 ? `${Math.round(x / 1e6)} M` : num(Math.round(x))
    if (unit === 'degC') return `${x >= 0 ? '+' : ''}${x.toFixed(2)} °C`
    if (unit === 'ppm') return `${Math.round(x)} ppm`
    if (unit === 'm') return `${(x * 100).toFixed(0)} cm`
    if (Number.isInteger(x)) return num(x)
    return num(+x.toPrecision(4)) + (unit ? ` ${unit}` : '')
  }
  const soFar = (topic) => crossings.filter((c) => c.topic === topic && c.t <= cursor)
  const label = (c) => c.topic === 'tipping.crossing' ? `${c.data.level} ${c.data.threshold_c}°` : c.topic === 'carbon.crossing' ? `releases ${Math.round(c.data.gtc_target)} GtC` : c.data.kind === 'niche' ? 'left the niche' : c.data.kind === 'wetbulb_a' ? '31° wet-bulb' : c.data.kind === 'wetbulb_b' ? '35° wet-bulb' : 'hottest month'
  const who = (c) => c.topic === 'tipping.crossing' || c.topic === 'carbon.crossing' ? c.data.title : `${c.data.name}${c.data.capital ? ` (${c.data.capital})` : ''}`
  const persons = (c) => (c.data.persons ? `${Math.round(c.data.persons / 1e6)} M` : '')
  const grid = (n) => (n === 1 ? '' : n === 2 ? 'sm:grid-cols-2' : 'sm:grid-cols-2 lg:grid-cols-3')
</script>

{#if views.length}
  <section class="mb-10">
    <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
      <span class="mono text-sm font-medium">{describe.name}</span>
      <span class="mono text-xs muted">{describe.kind}</span>
    </div>
    {#if describe.summary}<p class="mt-1 max-w-3xl text-sm muted">{describe.summary}</p>{/if}

    {#if maps.length}
      <div class="mt-5 grid gap-x-6 gap-y-6 {grid(maps.length)}">
        {#each maps as v (v.source)}
          <div>
            <div class="mb-1 text-sm">{v.title}{#if v.note} <span class="muted">· {v.note}</span>{/if}</div>
            <FieldMap name={v.source} field={run.fields[v.source]} {base} {cursor} />
          </div>
        {/each}
      </div>
    {/if}

    {#if counters.length}
      <div class="mt-4 grid gap-x-6 gap-y-3 sm:grid-cols-2 lg:grid-cols-4">
        {#each counters as v (v.source)}
          <div>
            <div class="mono text-2xl tabular-nums">{show(v)}</div>
            <div class="text-sm">{v.title}</div>
            {#if v.note}<div class="text-xs muted">{v.note}</div>{/if}
          </div>
        {/each}
      </div>
    {/if}

    {#each stories as v (v.title)}
      <div class="mt-5 max-w-3xl">
        {#if sentences.length === 0}
          <p class="text-sm muted">Nothing crossed yet at this point in the run.</p>
        {:else}
          <ol class="space-y-2">
            {#each sentences.slice(0, 3) as c, i (c.t + c.topic + (c.data.iso3 ?? c.data.id ?? i))}
              <li class="text-[15px] leading-relaxed {i === 0 ? 'font-medium' : ''}">
                {c.data.sentence}
                {#if c.data.source || c.data.confidence}<span class="mono text-xs muted"> {c.data.source ?? ''}{c.data.confidence ? ` · ${c.data.confidence}` : ''}</span>{/if}
              </li>
            {/each}
          </ol>
          {#if sentences.length > 3}
            <details class="mt-2">
              <summary class="cursor-pointer text-sm muted">{v.title} · {sentences.length - 3} earlier{#if v.note} · {v.note}{/if}</summary>
              <ol class="mt-2 space-y-2">
                {#each sentences.slice(3) as c, i (c.t + c.topic + (c.data.iso3 ?? c.data.id ?? i))}
                  <li class="text-sm leading-relaxed">{c.data.sentence}{#if c.data.source || c.data.confidence}<span class="mono text-xs muted"> {c.data.source ?? ''}{c.data.confidence ? ` · ${c.data.confidence}` : ''}</span>{/if}</li>
                {/each}
              </ol>
            </details>
          {/if}
        {/if}
      </div>
    {/each}

    {#if crossViews.length}
      <div class="mt-5 grid gap-x-8 gap-y-4 lg:grid-cols-2">
        {#each crossViews as v (v.title)}
          {@const xs = soFar(v.source)}
          <div>
            <div class="mb-1 text-sm">{v.title} <span class="mono muted">· {xs.length} so far</span>{#if v.note} <span class="muted">· {v.note}</span>{/if}</div>
            {#if xs.length === 0}
              <p class="text-xs muted">none yet at this point in the run</p>
            {:else}
              <table class="mono w-full text-xs">
                <tbody>
                  {#each xs.slice().reverse() as c, i (i)}
                    <tr class="border-t border-neutral-100 dark:border-neutral-900">
                      <td class="whitespace-nowrap pr-3 py-0.5">{isoDate(c.t).slice(0, 4)}</td>
                      <td class="pr-3 py-0.5">{who(c)}</td>
                      <td class="whitespace-nowrap pr-3 py-0.5 {label(c).includes('central') || label(c).includes('niche') || label(c).includes('wet-bulb') ? 'text-leak' : 'muted'}">{label(c)}</td>
                      <td class="whitespace-nowrap py-0.5 text-right">{persons(c)}{c.data.confidence ? c.data.confidence : ''}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    {#each texts as v (v.title)}
      <div class="mt-5 max-w-3xl">
        <div class="mb-1 text-sm">{v.title}</div>
        <p class="text-sm leading-relaxed muted">{v.source}</p>
      </div>
    {/each}

    {#if charts.length}
      <div class="mt-5 grid gap-x-6 gap-y-5 sm:grid-cols-2">
        {#each charts as v (v.source + v.obs)}
          <Chart name={v.title} unit={run.series[v.source]?.unit ?? ''} points={run.series[v.source].points} t0={run.start} t1={run.end}
            obs={v.obs && run.series[v.obs] ? run.series[v.obs].points : null} obsName={v.obs || null} {cursor} />
        {/each}
      </div>
    {/if}
  </section>
{/if}
