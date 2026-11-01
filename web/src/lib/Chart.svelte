<script>
  // Inline-SVG line chart. Width follows the container; height is fixed.
  // With `obs` set it draws two lines on one scale: sim (brand) and obs
  // (neutral text colour), with a two-item legend.
  import { isoDate, num } from './fmt.js'

  let { name, unit, points = [], t0, t1, obs = null, obsName = null } = $props()

  let width = $state(320)
  const H = 120
  const padL = 4, padR = 4, padT = 6, padB = 18

  const paired = $derived(Array.isArray(obs))
  const all = $derived(paired ? [...points, ...obs] : points)
  const xs = $derived(all.map((p) => p[0]))
  const ys = $derived(all.map((p) => p[1]))
  const tMin = $derived(t0 ?? Math.min(...xs))
  const tMax = $derived(t1 ?? Math.max(...xs))
  const yMin = $derived(ys.length ? Math.min(...ys) : 0)
  const yMax = $derived(ys.length ? Math.max(...ys) : 1)
  const ySpan = $derived(yMax - yMin || 1)

  const sx = (t) => padL + ((t - tMin) / ((tMax - tMin) || 1)) * (width - padL - padR)
  const sy = (v) => padT + (1 - (v - yMin) / ySpan) * (H - padT - padB)

  const toPath = (pts) =>
    pts.map((p, i) => `${i ? 'L' : 'M'}${sx(p[0]).toFixed(1)},${sy(p[1]).toFixed(1)}`).join(' ')
  const path = $derived(toPath(points))
  const obsPath = $derived(paired ? toPath(obs) : '')

  // a few evenly spaced time ticks, labelled with ISO dates
  const ticks = $derived.by(() => {
    const n = width < 360 ? 2 : width < 600 ? 3 : 4
    const out = []
    for (let i = 0; i <= n; i++) out.push(tMin + ((tMax - tMin) * i) / n)
    return out
  })
</script>

<figure class="min-w-0">
  <figcaption class="flex items-baseline justify-between gap-x-3 text-xs">
    {#if paired}
      <span class="mono min-w-0 truncate">{name} <span class="muted">vs</span> {obsName}</span>
      <span class="mono muted shrink-0">{unit} · {points.length}+{obs.length} pts</span>
    {:else}
      <span class="mono min-w-0 truncate">{name}</span>
      <span class="mono muted shrink-0">{unit} · {points.length} pts</span>
    {/if}
  </figcaption>
  <div class="mt-1 flex items-start gap-2">
    <div class="mono flex h-[120px] w-16 shrink-0 flex-col justify-between py-1 text-right text-[10px] muted" style="padding-bottom: {padB}px">
      <span>{num(yMax)}</span>
      <span>{num(yMin)}</span>
    </div>
    <div class="min-w-0 flex-1" bind:clientWidth={width}>
      <svg height={H} viewBox="0 0 {width} {H}" class="block w-full" role="img" aria-label="{paired ? `${name} and ${obsName}` : name} over time">
        <!-- frame -->
        <line x1={padL} y1={H - padB} x2={width - padR} y2={H - padB} class="stroke-neutral-300 dark:stroke-neutral-700" stroke-width="1" />
        <line x1={padL} y1={padT} x2={width - padR} y2={padT} class="stroke-neutral-200 dark:stroke-neutral-800" stroke-width="1" stroke-dasharray="2 3" />
        {#each ticks as t, i (t)}
          <line x1={sx(t)} y1={H - padB} x2={sx(t)} y2={H - padB + 3} class="stroke-neutral-300 dark:stroke-neutral-700" stroke-width="1" />
          <text
            x={sx(t)} y={H - 4}
            text-anchor={i === 0 ? 'start' : i === ticks.length - 1 ? 'end' : 'middle'}
            class="mono fill-neutral-500 dark:fill-neutral-400" font-size="10"
          >{isoDate(t)}</text>
        {/each}
        {#if paired && obs.length}
          <path d={obsPath} fill="none" class="stroke-neutral-900 dark:stroke-neutral-100" stroke-width="1.5" stroke-linejoin="round" vector-effect="non-scaling-stroke" />
        {/if}
        {#if points.length}
          <path d={path} fill="none" class="stroke-brand" stroke-width="1.5" stroke-linejoin="round" vector-effect="non-scaling-stroke" />
        {/if}
      </svg>
    </div>
  </div>
  {#if paired}
    <!-- legend: identity is never colour alone, the names sit beside the swatches -->
    <div class="mono mt-1 flex flex-wrap items-center gap-x-4 gap-y-1 pl-[72px] text-[10px]">
      <span class="inline-flex items-center gap-1.5"><span class="inline-block h-0.5 w-4 bg-brand" aria-hidden="true"></span>{name} <span class="muted">sim</span></span>
      <span class="inline-flex items-center gap-1.5"><span class="inline-block h-0.5 w-4 bg-neutral-900 dark:bg-neutral-100" aria-hidden="true"></span>{obsName} <span class="muted">obs</span></span>
    </div>
  {/if}
</figure>
