<script module>
  // shared by every map on the page
  const frameCache = new Map()      // `${base}/${name}/${i}` -> Promise<Float32Array>
  let outlinesPromise = null
  function outlinesOnce() {
    if (!outlinesPromise) outlinesPromise = fetch('/docs/outlines.json').then((r) => (r.ok ? r.json() : null)).catch(() => null)
    return outlinesPromise
  }
</script>

<script>
  // One field snapshot drawn to a canvas: nearest-neighbour blocks, a colour
  // scale that is either the run-wide min/max (default) or the current frame's
  // own, a time slider when there is more than one frame, and a hover/tap
  // readout of lat, lon and value.
  import { isoDate, num } from './fmt.js'
  import { LUT, RAMP_CSS, FIRE_LUT, FIRE_CSS, COVER, COVER_NAMES, rampFor } from './ramp.js'

  let { name, field, base, cursor = null, context = null } = $props()
  const ramp = $derived(rampFor(name, field.unit))
  const coverRgb = Object.fromEntries(Object.entries(COVER).map(([k, h]) => [k, [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16))]))

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
  // with a shared cursor, show the last frame at or before it
  $effect(() => {
    if (cursor === null || !times.length) return
    let i = 0
    while (i + 1 < times.length && times[i + 1] <= cursor) i++
    idx = i
  })
  let width = $state(320)
  const height = $derived(Math.round(width * aspect))

  let canvas = $state()
  let error = $state(null)
  let hover = $state(null)      // { x, y, lat, lon, v }

  // frames are cached across every map on the page (the same frame is often shown twice),
  // and once the first frame is up the rest of a field's frames are fetched in the
  // background so playing the timeline does not wait on the network
  const cache = frameCache

  // country outlines, drawn over grids that span the world (lon from -180): the raster is
  // one value per country at two degrees, the outlines say which country is which
  const world = $derived(grid.lon_w <= -179 && grid.nx * grid.dlon >= 359)
  let outlines = $state(null)
  $effect(() => {
    if (!world || outlines) return
    outlinesOnce().then((j) => { outlines = j })
  })

  function fetchFrame(i) {
    const key = `${base}/${name}/${i}`
    if (cache.has(key)) return cache.get(key)
    const p = fetch(`${base}/fields/${encodeURIComponent(name)}/${String(i).padStart(5, '0')}.f32`).then(async (r) => {
      if (!r.ok) throw new Error(`frame ${i}: ${r.status}`)
      const buf = await r.arrayBuffer()
      // raw little-endian float32, nx*ny values, row-major from the north edge
      const dv = new DataView(buf)
      const n = buf.byteLength >> 2
      const out = new Float32Array(n)
      for (let k = 0; k < n; k++) out[k] = dv.getFloat32(k * 4, true)
      return out
    })
    p.catch(() => cache.delete(key))
    cache.set(key, p)
    return p
  }
  // after the first frame, warm the rest two at a time
  let warmed = false
  $effect(() => {
    if (!frame || warmed || times.length < 2) return
    warmed = true
    ;(async () => { for (let i = 0; i < times.length; i += 2) await Promise.all([fetchFrame(i), i + 1 < times.length ? fetchFrame(i + 1) : null].filter(Boolean)).catch(() => {}) })()
  })

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
      let c
      if (ramp === 'cover') { c = coverRgb[Math.round(v)]; if (!c) { px[o + 3] = 0; continue } }
      else { const t = Math.min(255, Math.max(0, Math.round(((v - lo) / span) * 255))); c = ramp === 'fire' ? FIRE_LUT[t] : LUT[t] }
      px[o] = c[0]; px[o + 1] = c[1]; px[o + 2] = c[2]; px[o + 3] = 255
    }
    off.getContext('2d').putImageData(img, 0, 0)

    const ctx = canvas.getContext('2d')
    ctx.clearRect(0, 0, W, H)
    ctx.imageSmoothingEnabled = false
    ctx.drawImage(off, 0, 0, W, H)
    // roads, city limits and place names over a local window
    if (context && !world) {
      const dark = matchMedia('(prefers-color-scheme: dark)').matches
      const sx = (lon) => ((lon - grid.lon_w) / (grid.nx * grid.dlon)) * W
      const sy = (lat) => ((grid.lat_n - lat) / (grid.ny * grid.dlat)) * H
      const line = (pts) => { ctx.beginPath(); pts.forEach(([lon, lat], i) => (i ? ctx.lineTo(sx(lon), sy(lat)) : ctx.moveTo(sx(lon), sy(lat)))); ctx.stroke() }
      ctx.lineCap = 'round'; ctx.lineJoin = 'round'
      ctx.setLineDash([4 * dpr, 3 * dpr]); ctx.strokeStyle = dark ? 'rgba(255,255,255,0.55)' : 'rgba(0,0,0,0.45)'; ctx.lineWidth = 1 * dpr
      for (const b of context.boundaries ?? []) line(b.pts)
      ctx.setLineDash([])
      for (const r of context.roads ?? []) {
        const major = /motorway|trunk|primary/.test(r.class)
        ctx.strokeStyle = dark ? (major ? 'rgba(255,255,255,0.8)' : 'rgba(255,255,255,0.4)') : (major ? 'rgba(0,0,0,0.7)' : 'rgba(0,0,0,0.3)')
        ctx.lineWidth = (major ? 1.4 : 0.7) * dpr
        line(r.pts)
      }
      ctx.font = `${11 * dpr}px ui-monospace, SFMono-Regular, Menlo, monospace`; ctx.textAlign = 'center'; ctx.textBaseline = 'middle'
      ctx.lineWidth = 3 * dpr; ctx.strokeStyle = dark ? 'rgba(0,0,0,0.75)' : 'rgba(255,255,255,0.85)'; ctx.fillStyle = dark ? '#fff' : '#111'
      const drawn = []
      for (const p of (context.places ?? []).filter((p) => /city|town|suburb|neighbourhood|village|quarter/.test(p.kind)).sort((a, b) => ['city', 'town', 'suburb', 'neighbourhood', 'quarter', 'village'].indexOf(a.kind) - ['city', 'town', 'suburb', 'neighbourhood', 'quarter', 'village'].indexOf(b.kind))) {
        const x = sx(p.lon), y = sy(p.lat)
        if (x < 0 || y < 0 || x > W || y > H) continue
        if (drawn.some(([dx, dy]) => Math.abs(dx - x) < 70 * dpr && Math.abs(dy - y) < 14 * dpr)) continue
        ctx.strokeText(p.name, x, y); ctx.fillText(p.name, x, y); drawn.push([x, y])
      }
    }
    if (world && outlines) {
      const dark = matchMedia('(prefers-color-scheme: dark)').matches
      ctx.strokeStyle = dark ? 'rgba(255,255,255,0.45)' : 'rgba(0,0,0,0.35)'
      ctx.lineWidth = Math.max(0.6, 0.6 * dpr)
      const sx = (lon) => ((lon - grid.lon_w) / (grid.nx * grid.dlon)) * W
      const sy = (lat) => ((grid.lat_n - lat) / (grid.ny * grid.dlat)) * H
      for (const ring of outlines) {
        ctx.beginPath()
        ring.forEach(([lon, lat], i) => (i ? ctx.lineTo(sx(lon), sy(lat)) : ctx.moveTo(sx(lon), sy(lat))))
        ctx.closePath()
        ctx.stroke()
      }
    }
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
        <div>{hover.v === null ? 'no data' : ramp === 'cover' ? (COVER_NAMES[Math.round(hover.v)] ?? hover.v) : `${num(+hover.v.toPrecision(6))} ${field.unit}`}</div>
      </div>
    {/if}
  </div>

  <!-- legend: labels follow the chosen stretch; a categorical field lists its classes -->
  {#if ramp === 'cover'}
  <div class="mono mt-1.5 flex flex-wrap items-center gap-x-3 gap-y-0.5 text-[10px] muted">
    {#each Object.entries(COVER_NAMES) as [code, label] (code)}<span class="inline-flex items-center gap-1"><span class="inline-block h-2 w-2 rounded-[1px]" style="background:{COVER[code]}" aria-hidden="true"></span>{label}</span>{/each}
  </div>
  {:else}
  <div class="mono mt-1.5 flex items-center gap-2 text-[10px] muted">
    <span>{num(+lo.toPrecision(5))}</span>
    <span class="h-2 flex-1 rounded-[2px]" style="background:{ramp === 'fire' ? FIRE_CSS : RAMP_CSS}" aria-hidden="true"></span>
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
  {/if}

  <!-- time -->
  <div class="mono mt-1.5 flex items-center gap-3 text-xs">
    <span class="muted">{times.length ? isoDate(times[idx]) : '—'}</span>
    {#if times.length > 1 && cursor === null}
      <input
        type="range" min="0" max={times.length - 1} step="1" bind:value={idx}
        class="h-1 flex-1 accent-brand"
        aria-label="frame"
      />
    {/if}
    {#if times.length > 1}
      <span class="muted ml-auto">{idx + 1}/{times.length}</span>
    {/if}
  </div>
</figure>
