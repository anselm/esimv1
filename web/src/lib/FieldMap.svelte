<script>
  // One field snapshot drawn to a canvas: nearest-neighbour blocks, a colour
  // scale that is either the run-wide min/max (default) or the current frame's
  // own, a time slider when there is more than one frame, and a hover/tap
  // readout of lat, lon and value.
  import { isoDate, num } from './fmt.js'
  import { LUT, RAMP_CSS } from './ramp.js'

  let { name, field, base } = $props()

  const grid = $derived(field.grid)
  const times = $derived(field.times ?? [])
  let frame = $state(null)      // Float32Array of the current frame
  // colour stretch: 'run' uses run.json's run-wide min/max, 'frame' the loaded frame's own
  let stretch = $state('run')
  const frameRange = $derived.by(() => {
    if (!frame) return null
    let lo = Infinity, hi = -Infinity
    for (let k = 0; k < frame.length; k++) {
      const v = frame[k]
      if (Number.isNaN(v)) continue
      if (v < lo) lo = v
      if (v > hi) hi = v
    }
    return lo <= hi ? [lo, hi] : null
  })
  const lo = $derived(stretch === 'frame' && frameRange ? frameRange[0] : field.min)
  const hi = $derived(stretch === 'frame' && frameRange ? frameRange[1] : field.max)

  // Aspect in degrees, corrected by cos(latitude at the grid's centre) so a
  // degree of longitude is drawn shorter than a degree of latitude.
  const latMid = $derived(grid.lat_n - (grid.ny * grid.dlat) / 2)
  const aspect = $derived(
    (grid.ny * grid.dlat) / (grid.nx * grid.dlon * Math.cos((latMid * Math.PI) / 180)),
  )

  let idx = $state(0)
  let width = $state(320)
  const height = $derived(Math.round(width * aspect))

  let canvas = $state()
  let error = $state(null)
  let hover = $state(null)      // { x, y, lat, lon, v }

  const cache = new Map()       // frame index -> Float32Array (this field only)

  async function fetchFrame(i) {
    if (cache.has(i)) return cache.get(i)
    const r = await fetch(`${base}/fields/${encodeURIComponent(name)}/${String(i).padStart(5, '0')}.f32`, { cache: 'no-store' })
    if (!r.ok) throw new Error(`frame ${i}: ${r.status}`)
    const buf = await r.arrayBuffer()
    // raw little-endian float32, nx*ny values, row-major from the north edge
    const dv = new DataView(buf)
    const n = buf.byteLength >> 2
    const out = new Float32Array(n)
    for (let k = 0; k < n; k++) out[k] = dv.getFloat32(k * 4, true)
    cache.set(i, out)
    return out
  }

  // Load on demand when the selected frame changes.
  $effect(() => {
    const i = idx
    let live = true
    error = null
    fetchFrame(i).then((f) => { if (live) frame = f }, (e) => { if (live) error = e.message })
    return () => { live = false }
  })

  // Paint: one offscreen pixel per cell, then scaled up without smoothing.
  $effect(() => {
    if (!canvas || !frame) return
    const { nx, ny } = grid
    const dpr = window.devicePixelRatio || 1
    const W = Math.max(1, Math.round(width * dpr)), H = Math.max(1, Math.round(height * dpr))
    canvas.width = W
    canvas.height = H

    const off = document.createElement('canvas')
    off.width = nx
    off.height = ny
    const img = off.getContext('2d').createImageData(nx, ny)
    const px = img.data
    const span = hi - lo || 1
    for (let k = 0; k < nx * ny; k++) {
      const v = frame[k]
      const o = k * 4
      if (Number.isNaN(v)) { px[o + 3] = 0; continue }
      const t = Math.min(255, Math.max(0, Math.round(((v - lo) / span) * 255)))
      const c = LUT[t]
      px[o] = c[0]; px[o + 1] = c[1]; px[o + 2] = c[2]; px[o + 3] = 255
    }
    off.getContext('2d').putImageData(img, 0, 0)

    const ctx = canvas.getContext('2d')
    ctx.clearRect(0, 0, W, H)
    ctx.imageSmoothingEnabled = false
    ctx.drawImage(off, 0, 0, W, H)
  })

  function cellAt(e) {
    const r = canvas.getBoundingClientRect()
    const x = e.clientX - r.left, y = e.clientY - r.top
    const col = Math.floor((x / r.width) * grid.nx)
    const row = Math.floor((y / r.height) * grid.ny)
    if (col < 0 || row < 0 || col >= grid.nx || row >= grid.ny) return null
    const v = frame ? frame[row * grid.nx + col] : NaN
    return {
      x, y,
      lat: grid.lat_n - (row + 0.5) * grid.dlat,
      lon: grid.lon_w + (col + 0.5) * grid.dlon,
      v: Number.isNaN(v) ? null : v,
    }
  }
  const onmove = (e) => { hover = cellAt(e) }
  const onleave = (e) => { if (e.pointerType !== 'touch') hover = null }

  const arcsec = $derived(grid.dlon * 3600)
</script>

<figure class="min-w-0">
  <figcaption class="flex flex-wrap items-baseline justify-between gap-x-3 text-xs">
    <span class="mono">{name}</span>
    <span class="mono muted">{field.unit} · {grid.nx}×{grid.ny} · {num(+arcsec.toFixed(2))}″</span>
  </figcaption>

  <div class="relative mt-1 min-w-0 select-none" bind:clientWidth={width}>
    <canvas
      bind:this={canvas}
      style="width:{width}px;height:{height}px"
      class="block rounded-[2px] bg-neutral-100 dark:bg-neutral-900"
      role="img"
      aria-label="{name} map, {grid.nx} by {grid.ny} cells"
      onpointermove={onmove}
      onpointerdown={onmove}
      onpointerleave={onleave}
    ></canvas>
    {#if !frame && !error}
      <span class="absolute left-2 top-2 text-xs muted">loading…</span>
    {/if}
    {#if error}
      <span class="absolute left-2 top-2 text-xs text-leak">{error}</span>
    {/if}
    {#if hover}
      <div
        class="mono pointer-events-none absolute z-10 whitespace-nowrap rounded-[3px] bg-white/90 px-1.5 py-1 text-[11px] shadow-sm ring-1 ring-neutral-200 dark:bg-neutral-900/90 dark:ring-neutral-700"
        style="left:{Math.min(hover.x + 10, width - 150)}px; top:{Math.max(hover.y - 40, 0)}px"
      >
        <div>{hover.lat.toFixed(4)}, {hover.lon.toFixed(4)}</div>
        <div>{hover.v === null ? 'no data' : `${num(+hover.v.toPrecision(6))} ${field.unit}`}</div>
      </div>
    {/if}
  </div>

  <!-- legend: labels follow the chosen stretch -->
  <div class="mono mt-1.5 flex items-center gap-2 text-[10px] muted">
    <span>{num(+lo.toPrecision(5))}</span>
    <span class="h-2 flex-1 rounded-[2px]" style="background:{RAMP_CSS}" aria-hidden="true"></span>
    <span>{num(+hi.toPrecision(5))}</span>
    <span>{field.unit}</span>
    <span class="ml-1 inline-flex select-none" role="radiogroup" aria-label="colour stretch" title="colour scale: run-wide or this frame's min/max">
      {#each ['run', 'frame'] as m (m)}
        <button
          type="button" role="radio" aria-checked={stretch === m}
          class="px-1 leading-4 rounded-[2px] {stretch === m ? 'text-brand ring-1 ring-brand/40' : 'hover:text-neutral-900 dark:hover:text-neutral-100'}"
          onclick={() => { stretch = m }}
        >{m}</button>
      {/each}
    </span>
  </div>

  <!-- time -->
  <div class="mono mt-1.5 flex items-center gap-3 text-xs">
    <span class="muted">{times.length ? isoDate(times[idx]) : '—'}</span>
    {#if times.length > 1}
      <input
        type="range" min="0" max={times.length - 1} step="1" bind:value={idx}
        class="h-1 flex-1 accent-brand"
        aria-label="frame"
      />
      <span class="muted">{idx + 1}/{times.length}</span>
    {/if}
  </div>
</figure>
