<script>
  // Inline-SVG line chart. Width follows the container; height is fixed.
  import { isoDate, num } from './fmt.js'

  let { name, unit, points = [], t0, t1 } = $props()

  let width = $state(320)
  const H = 120
  const padL = 4, padR = 4, padT = 6, padB = 18

  const xs = $derived(points.map((p) => p[0]))
  const ys = $derived(points.map((p) => p[1]))
  const tMin = $derived(t0 ?? Math.min(...xs))
  const tMax = $derived(t1 ?? Math.max(...xs))
  const yMin = $derived(ys.length ? Math.min(...ys) : 0)
  const yMax = $derived(ys.length ? Math.max(...ys) : 1)
  const ySpan = $derived(yMax - yMin || 1)

  const sx = (t) => padL + ((t - tMin) / ((tMax - tMin) || 1)) * (width - padL - padR)
  const sy = (v) => padT + (1 - (v - yMin) / ySpan) * (H - padT - padB)

  const path = $derived(
    points.map((p, i) => `${i ? 'L' : 'M'}${sx(p[0]).toFixed(1)},${sy(p[1]).toFixed(1)}`).join(' '),
  )

  // a few evenly spaced time ticks, labelled with ISO dates
  const ticks = $derived.by(() => {
    const n = width < 360 ? 2 : width < 600 ? 3 : 4
    const out = []
    for (let i = 0; i <= n; i++) out.push(tMin + ((tMax - tMin) * i) / n)
    return out
  })
</script>

<figure class="min-w-0">
  <figcaption class="flex items-baseline justify-between text-xs">
    <span class="mono">{name}</span>
    <span class="mono muted">{unit} · {points.length} pts</span>
  </figcaption>
  <div class="mt-1 flex items-start gap-2">
    <div class="mono flex h-[120px] w-16 shrink-0 flex-col justify-between py-1 text-right text-[10px] muted" style="padding-bottom: {padB}px">
      <span>{num(yMax)}</span>
      <span>{num(yMin)}</span>
    </div>
    <div class="min-w-0 flex-1" bind:clientWidth={width}>
      <svg height={H} viewBox="0 0 {width} {H}" class="block w-full" role="img" aria-label="{name} over time">
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
        {#if points.length}
          <path d={path} fill="none" class="stroke-brand" stroke-width="1.5" stroke-linejoin="round" vector-effect="non-scaling-stroke" />
        {/if}
      </svg>
    </div>
  </div>
</figure>
