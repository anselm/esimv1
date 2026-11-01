<script>
  // One time cursor for the whole run page: maps show the frame at or before it, charts
  // mark it. A play button walks it forward at a steady pace.
  import { isoTime } from './fmt.js'

  let { t0, t1, cursor = $bindable(t1), steps = 400 } = $props()
  let playing = $state(false)
  const span = $derived(Math.max(1, t1 - t0))
  const step = $derived(Math.max(1, Math.round(span / steps)))

  $effect(() => {
    if (!playing) return
    const id = setInterval(() => {
      if (cursor >= t1) { playing = false; return }
      cursor = Math.min(t1, cursor + step)
    }, 60)
    return () => clearInterval(id)
  })
  const pct = $derived(((cursor - t0) / span) * 100)
</script>

<div class="sticky top-0 z-[5] -mx-4 mb-6 bg-white/90 px-4 py-2 backdrop-blur dark:bg-neutral-950/90 sm:-mx-6 sm:px-6">
  <div class="flex items-center gap-3 mono text-xs">
    <button class="w-6 rounded-[3px] ring-1 ring-neutral-300 hover:text-brand dark:ring-neutral-700" onclick={() => { if (cursor >= t1 && !playing) cursor = t0; playing = !playing }} aria-label={playing ? 'pause' : 'play'}>{playing ? '❚❚' : '▶'}</button>
    <input type="range" min={t0} max={t1} step={step} bind:value={cursor} class="h-1 flex-1 accent-brand" aria-label="time" />
    <span class="w-36 text-right tabular-nums">{isoTime(cursor)}</span>
    <span class="hidden w-10 text-right muted sm:inline">{pct.toFixed(0)}%</span>
  </div>
</div>
