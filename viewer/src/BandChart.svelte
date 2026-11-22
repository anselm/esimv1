<script>
  // Inline-SVG band chart for one ensemble series: p10–p90 and p25–p75 bands
  // and the p50 line in the brand colour, min and max as thin dotted neutral
  // lines. Optional overlays: an observed line (neutral, solid) and a vertical
  // "forecast" rule at the time forecast members take over from reanalysis.
  // A two-state control sets the x-range to the whole window or its last 30 d.
  import { isoDate, isoTime, num } from './fmt.js'

  let { name, unit, series, obs = null, forecastT = null, initialRange = 'all' } = $props()

  const DAY = 86400
  // the parent picks the starting range; after that the control owns it
  // svelte-ignore state_referenced_locally
  let range = $state(initialRange)   // 'all' | 'last30'
  let width = $state(320)
  let hover = $state(null)           // index into series.times, or null
  const H = 140
  const padL = 4, padR = 4, padT = 14, padB = 18
  const cid = $derived('clip-' + name.replace(/[^a-z0-9]/gi, '-'))

  const times = $derived(series.times)
  const n = $derived(times.length)
  const tStart = $derived(times[0])
  const tEnd = $derived(times[n - 1])
  const tMin = $derived(range === 'last30' ? Math.max(tStart, tEnd - 30 * DAY) : tStart)
  const tMax = $derived(tEnd)
  const long = $derived(tEnd - tStart > 90 * DAY)   // offer the range control at all
  const hourly = $derived(n > 1 && times[1] - times[0] < DAY)

  // index window: first..last point inside the range, and the same widened by
  // one neighbour each side so lines run to the frame (clipped there)
  function window(ts) {
    let a = 0
    while (a < ts.length && ts[a] < tMin) a++
    let b = ts.length - 1
    while (b >= 0 && ts[b] > tMax) b--
    return { a0: a, b0: b, a: Math.max(0, a - 1), b: Math.min(ts.length - 1, b + 1) }
  }
  const win = $derived(window(times))
  const obsWin = $derived(obs ? window(obs.times) : null)

  const yDomain = $derived.by(() => {
    let lo = Infinity, hi = -Infinity
    const take = (v) => { if (v != null && Number.isFinite(v)) { if (v < lo) lo = v; if (v > hi) hi = v } }
    for (let i = win.a0; i <= win.b0; i++) { take(series.min[i]); take(series.max[i]) }
    if (obs) for (let i = obsWin.a0; i <= obsWin.b0; i++) take(obs.p50[i])
    if (!Number.isFinite(lo)) return { lo: 0, hi: 1 }
    return { lo, hi: hi === lo ? lo + 1 : hi }
  })
  const yMin = $derived(yDomain.lo)
  const yMax = $derived(yDomain.hi)

  const sx = (t) => padL + ((t - tMin) / ((tMax - tMin) || 1)) * (width - padL - padR)
  const sy = (v) => padT + (1 - (v - yMin) / (yMax - yMin)) * (H - padT - padB)

  // a polyline that breaks at nulls
  function line(ts, vs, a, b) {
    let d = '', pen = false
    for (let i = a; i <= b; i++) {
      const v = vs[i]
      if (v == null) { pen = false; continue }
      d += `${pen ? 'L' : 'M'}${sx(ts[i]).toFixed(1)},${sy(v).toFixed(1)} `
      pen = true
    }
    return d
  }
  // a closed band between lo and hi, one polygon per run without nulls
  function band(ts, lo, hi, a, b) {
    let d = ''
    let run = []
    const flush = () => {
      if (run.length < 2) { run = []; return }
      d += 'M' + run.map((i) => `${sx(ts[i]).toFixed(1)},${sy(hi[i]).toFixed(1)}`).join('L')
      d += 'L' + run.slice().reverse().map((i) => `${sx(ts[i]).toFixed(1)},${sy(lo[i]).toFixed(1)}`).join('L') + 'Z '
      run = []
    }
    for (let i = a; i <= b; i++) {
      if (lo[i] == null || hi[i] == null) flush()
      else run.push(i)
    }
    flush()
    return d
  }

  const dOuter = $derived(band(times, series.p10, series.p90, win.a, win.b))
  const dInner = $derived(band(times, series.p25, series.p75, win.a, win.b))
  const dP50 = $derived(line(times, series.p50, win.a, win.b))
  const dMin = $derived(line(times, series.min, win.a, win.b))
  const dMax = $derived(line(times, series.max, win.a, win.b))
  const dObs = $derived(obs ? line(obs.times, obs.p50, obsWin.a, obsWin.b) : '')
  const fcVisible = $derived(forecastT != null && forecastT >= tMin && forecastT <= tMax)
  const fcX = $derived(fcVisible ? sx(forecastT) : 0)
  const fcRight = $derived(fcVisible && fcX > width - 60)

  const ticks = $derived.by(() => {
    const k = width < 360 ? 2 : width < 600 ? 3 : 4
    const out = []
    for (let i = 0; i <= k; i++) out.push(tMin + ((tMax - tMin) * i) / k)
    return out
  })

  // crosshair: nearest sample to the pointer's time
  function nearest(ts, t, a, b) {
    let lo = a, hi = b
    while (lo < hi) { const m = (lo + hi) >> 1; if (ts[m] < t) lo = m + 1; else hi = m }
    if (lo > a && t - ts[lo - 1] < ts[lo] - t) lo--
    return lo
  }
  function move(e) {
    const r = e.currentTarget.getBoundingClientRect()
    const t = tMin + ((e.clientX - r.left - padL) / (width - padL - padR)) * (tMax - tMin)
    hover = nearest(times, Math.min(tMax, Math.max(tMin, t)), win.a0, win.b0)
  }
  const hoverObs = $derived.by(() => {
    if (hover == null || !obs) return null
    const t = times[hover]
    const i = nearest(obs.times, t, 0, obs.times.length - 1)
    const step = n > 1 ? times[1] - times[0] : DAY
    return Math.abs(obs.times[i] - t) <= step / 2 ? obs.p50[i] : null
  })
  const fmtT = (t) => (hourly ? isoTime(t) : isoDate(t))
</script>

<figure class="min-w-0">
  <figcaption class="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 text-xs">
    <span class="mono">{name}</span>
    <span class="flex items-baseline gap-3">
      <span class="mono muted">{unit} · {n} pts</span>
      {#if long}
        <span role="group" aria-label="x-range" class="mono inline-flex overflow-hidden rounded-[3px] text-[10px] ring-1 ring-neutral-200 dark:ring-neutral-800">
          <button
            type="button" aria-pressed={range === 'all'} onclick={() => (range = 'all')}
            class="px-1.5 py-0.5 {range === 'all' ? 'bg-neutral-200 text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100' : 'muted hover:text-neutral-900 dark:hover:text-neutral-100'}"
          >all</button>
          <button
            type="button" aria-pressed={range === 'last30'} onclick={() => (range = 'last30')}
            class="px-1.5 py-0.5 {range === 'last30' ? 'bg-neutral-200 text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100' : 'muted hover:text-neutral-900 dark:hover:text-neutral-100'}"
          >last 30 d</button>
        </span>
      {/if}
    </span>
  </figcaption>
  <div class="mt-1 flex items-start gap-2">
    <div class="mono flex w-16 shrink-0 flex-col justify-between text-right text-[10px] muted" style="height: {H}px; padding-top: {padT - 4}px; padding-bottom: {padB}px">
      <span>{num(yMax)}</span>
      <span>{num(yMin)}</span>
    </div>
    <div class="min-w-0 flex-1" bind:clientWidth={width}>
      <svg
        height={H} viewBox="0 0 {width} {H}" class="block w-full touch-none" role="img"
        aria-label="{name}: ensemble spread over time"
        onpointermove={move} onpointerleave={() => (hover = null)}
      >
        <defs>
          <clipPath id={cid}><rect x={padL} y="0" width={Math.max(0, width - padL - padR)} height={H - padB} /></clipPath>
        </defs>
        <!-- frame -->
        <line x1={padL} y1={H - padB} x2={width - padR} y2={H - padB} class="stroke-neutral-300 dark:stroke-neutral-700" stroke-width="1" />
        <line x1={padL} y1={padT} x2={width - padR} y2={padT} class="stroke-neutral-200 dark:stroke-neutral-800" stroke-width="1" />
        {#each ticks as t, i (t)}
          <line x1={sx(t)} y1={H - padB} x2={sx(t)} y2={H - padB + 3} class="stroke-neutral-300 dark:stroke-neutral-700" stroke-width="1" />
          <text
            x={sx(t)} y={H - 4}
            text-anchor={i === 0 ? 'start' : i === ticks.length - 1 ? 'end' : 'middle'}
            class="mono fill-neutral-500 dark:fill-neutral-400" font-size="10"
          >{isoDate(t)}</text>
        {/each}
        <g clip-path="url(#{cid})">
          <path d={dOuter} class="fill-brand" fill-opacity="0.15" stroke="none" />
          <path d={dInner} class="fill-brand" fill-opacity="0.35" stroke="none" />
          <path d={dMin} fill="none" class="stroke-neutral-500 dark:stroke-neutral-400" stroke-width="1" stroke-dasharray="1 3" stroke-linecap="round" />
          <path d={dMax} fill="none" class="stroke-neutral-500 dark:stroke-neutral-400" stroke-width="1" stroke-dasharray="1 3" stroke-linecap="round" />
          {#if obs}
            <path d={dObs} fill="none" class="stroke-neutral-900 dark:stroke-neutral-100" stroke-width="1.5" stroke-linejoin="round" />
          {/if}
          <path d={dP50} fill="none" class="stroke-brand" stroke-width="1.5" stroke-linejoin="round" />
        </g>
        {#if fcVisible}
          <line x1={fcX} y1={padT} x2={fcX} y2={H - padB} class="stroke-neutral-500 dark:stroke-neutral-400" stroke-width="1" stroke-dasharray="3 2" />
          <text x={fcX + (fcRight ? -3 : 3)} y={padT - 4} text-anchor={fcRight ? 'end' : 'start'} class="mono fill-neutral-500 dark:fill-neutral-400" font-size="10">forecast</text>
        {/if}
        {#if hover != null}
          <line x1={sx(times[hover])} y1={padT} x2={sx(times[hover])} y2={H - padB} class="stroke-neutral-400 dark:stroke-neutral-500" stroke-width="1" />
          {#if series.p50[hover] != null}
            <circle cx={sx(times[hover])} cy={sy(series.p50[hover])} r="3.5" class="fill-brand stroke-white dark:stroke-neutral-950" stroke-width="2" />
          {/if}
        {/if}
      </svg>
    </div>
  </div>
  <!-- legend: identity is never colour alone, the names sit beside the swatches -->
  <div class="mono mt-1 flex flex-wrap items-center gap-x-4 gap-y-1 pl-[72px] text-[10px]">
    <span class="inline-flex items-center gap-1.5"><span class="inline-block h-0.5 w-4 bg-brand" aria-hidden="true"></span>median</span>
    <span class="inline-flex items-center gap-1.5"><span class="inline-block h-2.5 w-4 bg-brand/35" aria-hidden="true"></span>p25–p75</span>
    <span class="inline-flex items-center gap-1.5"><span class="inline-block h-2.5 w-4 bg-brand/15" aria-hidden="true"></span>p10–p90</span>
    <span class="inline-flex items-center gap-1.5"><span class="inline-block h-px w-4 border-t border-dotted border-neutral-500 dark:border-neutral-400" aria-hidden="true"></span>min · max</span>
    {#if obs}
      <span class="inline-flex items-center gap-1.5"><span class="inline-block h-0.5 w-4 bg-neutral-900 dark:bg-neutral-100" aria-hidden="true"></span>observed</span>
    {/if}
    {#if forecastT != null}
      <span class="inline-flex items-center gap-1.5"><span class="inline-block h-3 w-px border-l border-dashed border-neutral-500 dark:border-neutral-400" aria-hidden="true"></span>forecast start</span>
    {/if}
  </div>
  <!-- crosshair readout: every value at the hovered time, values first -->
  <div class="mono mt-1 min-h-4 pl-[72px] text-[10px] muted" aria-live="polite">
    {#if hover != null}
      <span class="text-neutral-900 dark:text-neutral-100">{fmtT(times[hover])}</span>
      · median <span class="text-neutral-900 dark:text-neutral-100">{num(series.p50[hover])}</span>
      · p10–p90 {num(series.p10[hover])}–{num(series.p90[hover])}
      · min–max {num(series.min[hover])}–{num(series.max[hover])}
      {#if hoverObs != null}· observed <span class="text-neutral-900 dark:text-neutral-100">{num(hoverObs)}</span>{/if}
    {/if}
  </div>
</figure>
