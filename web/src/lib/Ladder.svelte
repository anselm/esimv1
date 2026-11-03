<script>
  // The ladder: the tipping elements' thresholds as rungs on a warming axis, and each
  // variant's warming path climbing past them. Rungs come from the run's own describe
  // (the table the run was checked against), paths from the runs' series, crossing years
  // from the runs' events; nothing is computed here that a run did not report.
  import { seriesAt, yearStart, yearOf } from './runs.js'

  let { runs = [], elements = [], from = 1990, to = 2100, maxC = 4.5, minC = 0.5 } = $props()

  let width = $state(640)
  const H = 400
  const padL = 34, padT = 10, padB = 22
  const narrow = $derived(width < 480)
  const padR = $derived(narrow ? 80 : 84)

  const sx = (y) => padL + ((y - from) / (to - from)) * (width - padL - padR)
  const sy = (c) => padT + (1 - (c - minC) / (maxC - minC)) * (H - padT - padB)

  const paths = $derived(runs.filter((o) => o.variant !== 'hindcast' && o.run.series?.['energy.gmst_anom_c']))
  // one point a year, from the start of the chart to the run's end
  const yearly = (o) => {
    const out = []
    for (let y = from; y <= to; y++) { const v = seriesAt(o.run, 'energy.gmst_anom_c', yearStart(y)); if (v !== null && yearStart(y) <= o.run.end) out.push([y, v]) }
    return out
  }
  const lines = $derived(paths.map((o) => ({ o, pts: yearly(o) })))
  const toPath = (pts) => pts.map((p, i) => `${i ? 'L' : 'M'}${sx(p[0]).toFixed(1)},${sy(p[1]).toFixed(1)}`).join(' ')

  // the observed record, from whichever run carries it: one dot a year
  const obs = $derived.by(() => {
    const o = runs.find((r) => r.run.series?.['obs.gmst_hadcrut5_c'])
    if (!o) return []
    const out = []
    for (let y = from; y <= to; y++) { const v = seriesAt(o.run, 'obs.gmst_hadcrut5_c', yearStart(y)); if (v !== null && yearOf(o.run.series['obs.gmst_hadcrut5_c'].points.at(-1)[0]) >= y) out.push([y, v]) }
    return out
  })

  // end labels, pushed apart so they can be read
  const ends = $derived.by(() => {
    const items = lines.filter((l) => l.pts.length).map((l) => ({ name: l.o.variant, y: sy(l.pts.at(-1)[1]), x: sx(l.pts.at(-1)[0]) })).sort((a, b) => a.y - b.y)
    for (let i = 1; i < items.length; i++) if (items[i].y - items[i - 1].y < 11) items[i].y = items[i - 1].y + 11
    for (let i = items.length - 2; i >= 0; i--) if (items[i + 1].y - items[i].y < 11) items[i].y = items[i + 1].y - 11
    return items
  })

  const shown = $derived(elements.filter((e) => e.central_c <= maxC).slice().sort((a, b) => a.central_c - b.central_c || a.low_c - b.low_c))
  const beyond = $derived(elements.filter((e) => e.central_c > maxC).slice().sort((a, b) => a.central_c - b.central_c))
  // rung labels: elements sharing a central value are written on one line
  const rungs = $derived.by(() => {
    const m = new Map()
    for (const e of shown) { if (!m.has(e.central_c)) m.set(e.central_c, []); m.get(e.central_c).push(e) }
    // on a phone, the first element's name and a count stand for the rung
    const out = [...m.entries()].map(([c, es]) => ({ c, es, label: narrow && es.length > 1 ? `${short(es[0])} +${es.length - 1}` : es.map((e) => short(e)).join(' · '), right: false }))
    for (let i = 1; i < out.length; i++) if (sy(out[i - 1].c) - sy(out[i].c) < 13 && !out[i - 1].right) out[i].right = true
    return out
  })
  // "about 10,000 years (1,000 to 15,000)" -> "~10,000 y"; "more than 10,000 years (…)" -> ">10,000 y"
  const brief = (t) => t.replace(/\s*\(.*\)\s*$/, '').replace(/^about\s+/, '~').replace(/^more than\s+/, '>').replace(/\s*years?$/, ' y')
  function short(e) {
    return e.title.replace(' collapse', '').replace(' die-off', '').replace(' dieback', ' dieback').replace('Atlantic meridional overturning circulation', 'AMOC').replace('Extra-polar mountain glacier loss', 'Mountain glaciers').replace('Low-latitude coral reef', 'Coral reefs').replace('Boreal permafrost abrupt thaw', 'Permafrost abrupt thaw').replace('Boreal permafrost', 'Permafrost').replace('Labrador-Irminger Sea convection', 'Labrador Sea convection').replace('Sahel and West African monsoon shift', 'West African monsoon').replace('East Antarctic subglacial basins', 'East Antarctic basins').replace('Boreal forest southern dieback', 'Boreal forest dieback').replace('Boreal forest northern expansion', 'Boreal forest expansion').replace('Barents Sea winter ice loss', 'Barents Sea ice').replace('Arctic winter sea ice', 'Arctic winter ice')
  }

  // crossing years per element and variant, from the runs' own events
  const years = $derived.by(() => {
    const m = new Map()  // `${id}|${variant}|${level}` -> year
    for (const o of runs) for (const c of o.audit?.crossings ?? []) if (c.topic === 'tipping.crossing') m.set(`${c.data.id}|${o.variant}|${c.data.level}`, yearOf(c.t))
    return m
  })
  const yr = (e, v, level) => years.get(`${e.id}|${v}|${level}`) ?? null
  const dots = $derived.by(() => {
    const out = []
    for (const l of lines) for (const e of shown) { const y = yr(e, l.o.variant, 'central'); if (y !== null && y >= from) { const v = seriesAt(l.o.run, 'energy.gmst_anom_c', yearStart(y)); if (v !== null) out.push({ x: sx(y), y: sy(v) }) } }
    return out
  })
  // the variants in order of where they end, coolest first
  const endC = (o) => seriesAt(o.run, 'energy.gmst_anom_c', o.run.end) ?? 0
  const cols = $derived(runs.filter((o) => o.variant !== 'hindcast').slice().sort((a, b) => endC(a) - endC(b)))
  const ticks = [1, 1.5, 2, 2.5, 3, 3.5, 4, 4.5].filter((c) => c >= minC && c <= maxC)
  const xt = $derived([from, 2024, 2050, 2075, 2100].filter((y) => y >= from && y <= to))
</script>

<figure class="min-w-0">
  <div class="min-w-0" bind:clientWidth={width}>
    <svg height={H} viewBox="0 0 {width} {H}" class="block w-full" role="img" aria-label="warming paths against the tipping thresholds">
      <style>
        .halo { paint-order: stroke; stroke: var(--color-page); stroke-width: 3px; stroke-linejoin: round; }
      </style>
      <!-- the possible zone of each element, from its low end to its central estimate -->
      {#each shown as e (e.id)}
        <rect x={padL} y={sy(Math.min(e.central_c, maxC))} width={width - padL - padR} height={Math.max(0, sy(Math.max(e.low_c, minC)) - sy(Math.min(e.central_c, maxC)))} class="fill-neutral-500/[0.05]" />
      {/each}
      <!-- axes -->
      {#each ticks as c (c)}
        <line x1={padL} y1={sy(c)} x2={width - padR} y2={sy(c)} class="stroke-neutral-200 dark:stroke-neutral-800" stroke-width="1" stroke-dasharray="2 3" />
        <text x={padL - 4} y={sy(c) + 3} text-anchor="end" class="mono fill-neutral-500 dark:fill-neutral-400" font-size="10">{c.toFixed(1)}°</text>
      {/each}
      {#each xt as y (y)}
        <line x1={sx(y)} y1={H - padB} x2={sx(y)} y2={H - padB + 3} class="stroke-neutral-300 dark:stroke-neutral-700" stroke-width="1" />
        <text x={sx(y)} y={H - 6} text-anchor={y === from ? 'start' : 'middle'} class="mono fill-neutral-500 dark:fill-neutral-400" font-size="10">{y}</text>
      {/each}
      <line x1={padL} y1={H - padB} x2={width - padR} y2={H - padB} class="stroke-neutral-300 dark:stroke-neutral-700" stroke-width="1" />
      <!-- the end of the record -->
      <line x1={sx(2024)} y1={padT} x2={sx(2024)} y2={H - padB} class="stroke-neutral-300 dark:stroke-neutral-700" stroke-width="1" />
      <text x={sx(2024) + 3} y={H - padB - 4} class="mono halo fill-neutral-500 dark:fill-neutral-400" font-size="9">record ends</text>
      <!-- rungs: the central estimates -->
      {#each rungs as r (r.c)}
        <line x1={padL} y1={sy(r.c)} x2={width - padR} y2={sy(r.c)} class="stroke-neutral-400 dark:stroke-neutral-500" stroke-width="1" />
        <text x={r.right ? width - padR - 4 : padL + 4} y={sy(r.c) - 3} text-anchor={r.right ? 'end' : 'start'} class="halo fill-neutral-700 dark:fill-neutral-300" font-size="10">{r.label}</text>
      {/each}
      <!-- the observed record -->
      {#each obs as p (p[0])}
        <circle cx={sx(p[0])} cy={sy(p[1])} r="1.6" class="fill-neutral-900 dark:fill-neutral-100" />
      {/each}
      <!-- the paths -->
      {#each lines as l (l.o.variant)}
        <path d={toPath(l.pts)} fill="none" class="stroke-brand" stroke-width="1.5" stroke-linejoin="round" vector-effect="non-scaling-stroke" />
      {/each}
      {#each dots as d, i (i)}
        <circle cx={d.x} cy={d.y} r="3" class="fill-leak" />
      {/each}
      {#each ends as e (e.name)}
        <text x={e.x + 5} y={e.y + 3} class="mono halo fill-neutral-900 dark:fill-neutral-100" font-size="10">{e.name}</text>
      {/each}
    </svg>
  </div>
  <figcaption class="mono mt-1 flex flex-wrap gap-x-4 gap-y-1 text-[10px] muted">
    <span><span class="inline-block h-0.5 w-4 bg-brand align-middle" aria-hidden="true"></span> warming, one line per assumption</span>
    <span><span class="inline-block h-1.5 w-1.5 rounded-full bg-neutral-900 align-middle dark:bg-neutral-100" aria-hidden="true"></span> the observed record (HadCRUT5)</span>
    <span><span class="inline-block h-px w-4 bg-neutral-400 align-middle" aria-hidden="true"></span> a threshold's central estimate; shaded below it, the range in which crossing is possible</span>
    <span><span class="inline-block h-1.5 w-1.5 rounded-full bg-leak align-middle" aria-hidden="true"></span> a path passing a central estimate</span>
    {#if beyond.length}<span>beyond the chart: {beyond.map((e) => `${short(e)} at ${e.central_c} °C`).join(', ')}</span>{/if}
  </figcaption>

  <!-- the years, one row per element in the ladder's order -->
  <div class="mt-4 overflow-x-auto">
    <table class="mono w-full text-[11px] tabular-nums">
      <thead>
        <tr class="label">
          <th class="py-1 pr-2 text-left font-normal normal-case tracking-normal">element</th>
          <th class="py-1 pr-2 text-right font-normal normal-case tracking-normal">takes</th>
          <th class="py-1 pr-2 text-right font-normal normal-case tracking-normal">likely from</th>
          {#each cols as o (o.variant)}<th class="py-1 pl-2 text-right font-normal normal-case tracking-normal">{o.variant}</th>{/each}
        </tr>
      </thead>
      <tbody>
        {#each elements.slice().sort((a, b) => a.central_c - b.central_c || a.low_c - b.low_c) as e (e.id)}
          <tr class="border-t border-neutral-100 dark:border-neutral-900">
            <td class="py-1 pr-2 whitespace-nowrap" title={e.consequence}>{short(e)}</td>
            <td class="py-1 pr-2 text-right whitespace-nowrap muted">{brief(e.timescale)}</td>
            <td class="py-1 pr-2 text-right whitespace-nowrap">{e.central_c.toFixed(1)}° <span class="muted">({e.low_c.toFixed(1)}–{e.high_c.toFixed(1)})</span></td>
            {#each cols as o (o.variant)}
              {@const c = yr(e, o.variant, 'central')}
              {@const l = yr(e, o.variant, 'low')}
              <td class="py-1 pl-2 text-right whitespace-nowrap">
                {#if c !== null}<span class="text-leak">{c}</span>{:else if l !== null}<span class="muted">({l})</span>{:else}<span class="muted">—</span>{/if}
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
    <p class="mt-1 text-[11px] muted">The year each path first passes an element's central estimate (red); in parentheses, the year it entered the possible range when it got no further; — means neither by the run's end. "Takes" is how long the change takes once it has begun, from the assessment.</p>
  </div>
</figure>
