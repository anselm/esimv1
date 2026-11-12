<script>
  // One time cursor for the whole run page: maps show the frame at or before it, charts
  // mark it. Play walks the run's moments: every frame time, every crossing's time and a
  // few hundred even steps, given by the page; it dwells longer on a moment less than an
  // hour from the next, so four minutes of shaking play as a scene and a year as a sweep.
  // With an event time the label reads relative to it: "6 days before", "2 min in", "day 25".
  import { isoTime, relTime } from './fmt.js'

  let { t0, t1, cursor = $bindable(t1), steps = 400, autoplay = false, moments = [], event = null } = $props()
  let playing = $state(autoplay)
  const span = $derived(Math.max(1, t1 - t0))
  const step = $derived(Math.max(1, Math.round(span / steps)))
  const walk = $derived.by(() => {
    const set = new Set(moments.filter((t) => t >= t0 && t <= t1))
    for (let i = 0; i <= steps; i++) set.add(Math.min(t1, t0 + i * step))
    return [...set].sort((a, b) => a - b)
  })
  const next = (t) => { let lo = 0, hi = walk.length - 1; while (lo < hi) { const m = (lo + hi) >> 1; if (walk[m] <= t) lo = m + 1; else hi = m }; return walk[lo] > t ? walk[lo] : null }

  $effect(() => {
    if (!playing) return
    let id
    const tick = () => {
      const n = next(cursor)
      if (n === null) { playing = false; return }
      cursor = n
      const after = next(n)
      // a scene: moments less than an hour apart get a third of a second each
      id = setTimeout(tick, after !== null && after - n < 3600 ? 350 : 60)
    }
    id = setTimeout(tick, 60)
    return () => clearTimeout(id)
  })
  const pct = $derived(((cursor - t0) / span) * 100)
  function key(e) {
    if (e.key === 'ArrowRight') { const n = next(cursor); if (n !== null) { cursor = n; e.preventDefault() } }
    else if (e.key === 'ArrowLeft') { let i = walk.findIndex((t) => t >= cursor); if (i < 0) i = walk.length; if (i > 0) { cursor = walk[i - 1]; e.preventDefault() } }
  }
</script>

<div class="sticky top-0 z-[5] -mx-4 mb-6 bg-white/90 px-4 py-2 backdrop-blur dark:bg-neutral-950/90 sm:-mx-6 sm:px-6">
  <div class="flex items-center gap-3 mono text-xs">
    <button class="w-6 rounded-[3px] ring-1 ring-neutral-300 hover:text-brand dark:ring-neutral-700" onclick={() => { if (cursor >= t1 && !playing) cursor = t0; playing = !playing }} aria-label={playing ? 'pause' : 'play'}>{playing ? '❚❚' : '▶'}</button>
    <input type="range" min={t0} max={t1} step={1} bind:value={cursor} onkeydown={key} class="h-1 flex-1 accent-brand" aria-label="time" title="arrow keys step moment by moment" />
    <span class="w-36 text-right tabular-nums">{isoTime(cursor)}</span>
    {#if event !== null}
      <span class="hidden w-28 text-right sm:inline" title="the event: {isoTime(event)}">{relTime(cursor, event)}</span>
    {:else}
      <span class="hidden w-10 text-right muted sm:inline">{pct.toFixed(0)}%</span>
    {/if}
  </div>
</div>
