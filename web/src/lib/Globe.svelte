<script>
  // The world scored, on a globe: every country a glyph at its centroid, the nine bands of
  // its score stacked outward from the surface, the total its height. The table comes from
  // the run's log (region.scored events, the latest at or before the cursor), the outlines
  // from docs/outlines.json. Drag to turn, wheel to zoom, hover a glyph for the bands.
  import * as THREE from 'three'
  import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
  import { LineSegments2 } from 'three/examples/jsm/lines/LineSegments2.js'
  import { LineSegmentsGeometry } from 'three/examples/jsm/lines/LineSegmentsGeometry.js'
  import { LineMaterial } from 'three/examples/jsm/lines/LineMaterial.js'

  let { run, base, cursor = null, source = 'region.scored' } = $props()

  // two glyphs for the same data: 'bars', the nine bands stacked into a pole, or 'icons', a
  // block of nine emoji in fixed positions, each scaled by its band (area carries the amount,
  // position names the measure, as the icon-block grammar on Anselm's Living Region page has
  // it; the artwork here is the device's emoji font, not fixed yet)
  let glyph = $state('outline')
  // the bands in three groups for the outline glyph (Anselm, 2026-11-05): human factors,
  // nature and its resilience, and plain economic output; a group's health is the mean of
  // the bands it has, 0 to 1; each group one pastel, drawn as a ring of the country's own
  // outline, brightness and thickness by the score, the rings stacked outward like a rainbow
  const GROUPS = [
    { name: 'human', bands: [0, 2, 3, 4], light: '#e8809a', dark: '#f4a3b8' },     // living, fairness, voice, air
    { name: 'nature', bands: [5, 6, 7, 8], light: '#5fb37a', dark: '#8fd4a3' },    // forest, protected, water, footprint
    { name: 'economy', bands: [1], light: '#6c9fe0', dark: '#9cc2f0' },             // means
  ]
  const groupScore = (c, g) => { const v = g.bands.map((i) => c.s?.[i]).filter((x) => x != null); return v.length ? v.reduce((a, b) => a + b, 0) / v.length : null }
  let countryRings = $state(null)   // { ISO3: [[ [lon, lat], ... ], ...] }
  const ICONS = ['🫀', '💰', '⚖️', '🗳️', '🌬️', '🌳', '🏞️', '💧', '👣']
  const ICON_SPRITE = 0.14
  // the totem: a thin cylinder standing on the centroid, the nine emoji stacked up it in fixed
  // slots, each scaled by its band, the strip printed twice around so it reads from any side;
  // thin where the countries crowd, fanning out as the poles rise (Anselm, 2026-11-05)
  const TOTEM_R = 0.014, TOTEM_H = 0.26

  // one colour per band, in the module's band order
  const BAND_COLOURS = ['#2a9d8f', '#e9c46a', '#f4a261', '#264653', '#8ecae6', '#588157', '#a7c957', '#219ebc', '#9a8c98']
  const R = 1, BAND_H = 0.03, GLYPH_W = 0.022

  let container = $state(null)
  let width = $state(640)
  const height = $derived(Math.round(width * 0.62))
  let tables = $state(null)      // [{ t, year, bands, by_iso3 }] sorted by t
  let outlines = $state(null)    // [[ [lon, lat], ... ], ...]
  let hover = $state(null)       // { iso3, name, total, s, x, y }
  let error = $state(null)

  $effect(() => {
    if (!base) return
    let live = true
    fetch(`${base}/log.jsonl`).then((r) => (r.ok ? r.text() : '')).then((text) => {
      if (!live) return
      const out = []
      for (const line of text.split('\n')) {
        if (!line.includes(`"${source}"`)) continue
        try { const e = JSON.parse(line); if (e.topic === source) out.push({ t: e.t, ...e.data }) } catch {}
      }
      out.sort((a, b) => a.t - b.t)
      tables = out
    }).catch((e) => { if (live) error = e.message })
    fetch('/docs/outlines.json').then((r) => (r.ok ? r.json() : null)).then((o) => { if (live) outlines = o }).catch(() => {})
    fetch('/docs/country-outlines.json').then((r) => (r.ok ? r.json() : null)).then((o) => { if (live) countryRings = o }).catch(() => {})
    return () => { live = false }
  })

  // the table at or before the cursor (the last one when there is no cursor)
  const table = $derived.by(() => {
    if (!tables?.length) return null
    if (cursor === null) return tables[tables.length - 1]
    let i = -1
    for (let k = 0; k < tables.length; k++) if (tables[k].t <= cursor) i = k
    return i >= 0 ? tables[i] : tables[0]
  })
  const bands = $derived(table?.bands ?? [])

  function xyz(lat, lon, r) {
    const phi = ((90 - lat) * Math.PI) / 180, theta = ((lon + 180) * Math.PI) / 180
    return new THREE.Vector3(-r * Math.sin(phi) * Math.cos(theta), r * Math.cos(phi), r * Math.sin(phi) * Math.sin(theta))
  }

  // the geography: an equirectangular canvas with every country's rings filled as land,
  // painted once from the same rings the outline glyph uses, and wrapped onto the sphere
  function landTexture(rings, dark) {
    const W = 2048, H = 1024
    const cv = document.createElement('canvas'); cv.width = W; cv.height = H
    const ctx = cv.getContext('2d')
    ctx.fillStyle = dark ? '#161a1f' : '#dde6ef'; ctx.fillRect(0, 0, W, H)          // sea
    ctx.fillStyle = dark ? '#2b2f33' : '#f4f1ea'                                      // land
    ctx.strokeStyle = dark ? '#4a4f55' : '#aab2bb'; ctx.lineWidth = 1
    for (const list of Object.values(rings)) for (const ring of list) {
      ctx.beginPath()
      ring.forEach(([lon, lat], i) => { const x = ((lon + 180) / 360) * W, y = ((90 - lat) / 180) * H; i ? ctx.lineTo(x, y) : ctx.moveTo(x, y) })
      ctx.closePath(); ctx.fill(); ctx.stroke()
    }
    const tex = new THREE.CanvasTexture(cv); tex.colorSpace = THREE.SRGBColorSpace
    // three's sphere puts u = 0 on the -x axis and xyz() puts lon -180 there too, so no offset
    return tex
  }

  // distress: a country tilts into red when its total falls under a floor, or falls far under
  // its own total at the run's start; the whole country is filled with a red whose opacity
  // is that distress, painted as an overlay sphere from the same rings
  const RED_FLOOR = 0.35, RED_DROP = 0.15
  const first = $derived(tables?.[0] ?? null)
  // the red is the worst pressure, never a mean: the share of the country's people past the
  // heat thresholds this year (the run's own stress), the weakest of its three groups under
  // the floor, or a fall under its own start (Anselm: a rich person with a heart attack is not
  // averaged well; some factors gate)
  function distress(c, c0) {
    if (c?.total == null) return 0
    const worst = Math.min(...GROUPS.map((g) => groupScore(c, g)).filter((v) => v != null).concat([1]))
    const under = Math.max(0, (RED_FLOOR - worst) / RED_FLOOR)
    const drop = c0?.total != null ? Math.max(0, (c0.total - c.total) / RED_DROP) : 0
    const stress = c.stress ?? 0
    return Math.min(1, Math.max(under, drop, stress))
  }
  function distressTexture(rings, t, t0) {
    const W = 2048, H = 1024
    const cv = document.createElement('canvas'); cv.width = W; cv.height = H
    const ctx = cv.getContext('2d')
    let n = 0
    for (const [iso3, c] of Object.entries(t.by_iso3 ?? {})) {
      const d = distress(c, t0?.by_iso3?.[iso3])
      if (d <= 0.02 || !rings[iso3]) continue
      n++
      ctx.fillStyle = `rgba(220, 38, 38, ${(0.15 + 0.6 * d).toFixed(3)})`
      for (const ring of rings[iso3]) {
        ctx.beginPath()
        ring.forEach(([lon, lat], i) => { const x = ((lon + 180) / 360) * W, y = ((90 - lat) / 180) * H; i ? ctx.lineTo(x, y) : ctx.moveTo(x, y) })
        ctx.closePath(); ctx.fill()
      }
    }
    const tex = new THREE.CanvasTexture(cv); tex.colorSpace = THREE.SRGBColorSpace
    return { tex, n }
  }
  let inRed = $state(0)

  let canvas = $state()
  let three = null   // { renderer, scene, camera, controls, globe, glyphs: InstancedMesh[], ids: [[iso3...] per band], raycaster }

  // build the scene once
  $effect(() => {
    if (!canvas) return
    const dark = matchMedia('(prefers-color-scheme: dark)').matches
    const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true })
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2))
    const scene = new THREE.Scene()
    const camera = new THREE.PerspectiveCamera(38, 1, 0.05, 50)
    camera.position.set(0, 0.9, 3.0)
    scene.add(new THREE.AmbientLight(0xffffff, dark ? 0.9 : 1.1))
    const sun = new THREE.DirectionalLight(0xffffff, dark ? 0.9 : 0.8); sun.position.set(-3, 2, 4); scene.add(sun)
    const globe = new THREE.Group()
    globe.add(new THREE.Mesh(new THREE.SphereGeometry(R, 96, 96), new THREE.MeshStandardMaterial({ color: dark ? 0x1c1c1c : 0xe9edf2, roughness: 1, metalness: 0 })))
    const red = new THREE.Mesh(new THREE.SphereGeometry(R + 0.0015, 96, 96), new THREE.MeshBasicMaterial({ transparent: true, opacity: 1, depthWrite: false, visible: false }))
    red.name = 'red'
    globe.add(red)
    scene.add(globe)
    const controls = new OrbitControls(camera, canvas)
    controls.enableDamping = true; controls.enablePan = false; controls.minDistance = 1.4; controls.maxDistance = 6
    const raycaster = new THREE.Raycaster()
    three = { renderer, scene, camera, controls, globe, glyphs: [], ids: [], raycaster, dark, lines: null }
    let raf
    const tick = () => { raf = requestAnimationFrame(tick); controls.update(); renderer.render(scene, camera) }
    tick()
    return () => { cancelAnimationFrame(raf); controls.dispose(); renderer.dispose(); three = null }
  })

  // size
  $effect(() => {
    if (!three) return
    three.renderer.setSize(width, height, false)
    three.camera.aspect = width / height
    three.camera.updateProjectionMatrix()
  })

  // land on the sphere once the rings are here
  $effect(() => {
    if (!three || !countryRings) return
    const tex = landTexture(countryRings, three.dark)
    const sphere = three.globe.children.find((o) => o.geometry?.type === 'SphereGeometry')
    if (sphere) { sphere.material.map = tex; sphere.material.color.set(0xffffff); sphere.material.needsUpdate = true }
    return () => { tex.dispose(); if (sphere) { sphere.material.map = null; sphere.material.needsUpdate = true } }
  })

  // the red overlay follows the table at the cursor
  $effect(() => {
    if (!three || !countryRings || !table) return
    const { tex, n } = distressTexture(countryRings, table, first)
    inRed = n
    const red = three.globe.getObjectByName('red')
    if (red) { red.material.map = tex; red.material.visible = n > 0; red.material.needsUpdate = true }
    return () => { tex.dispose(); if (red) { red.material.map = null; red.material.visible = false } }
  })

  // outlines as line segments just above the surface
  $effect(() => {
    if (!three || !outlines) return
    const pts = []
    for (const ring of outlines) {
      for (let i = 0; i + 1 < ring.length; i++) {
        const a = xyz(ring[i][1], ring[i][0], R + 0.001), b = xyz(ring[i + 1][1], ring[i + 1][0], R + 0.001)
        pts.push(a.x, a.y, a.z, b.x, b.y, b.z)
      }
    }
    const g = new THREE.BufferGeometry(); g.setAttribute('position', new THREE.Float32BufferAttribute(pts, 3))
    const lines = new THREE.LineSegments(g, new THREE.LineBasicMaterial({ color: three.dark ? 0x5a5a5a : 0x9aa3ad, transparent: true, opacity: 0.9 }))
    three.globe.add(lines); three.lines = lines
    return () => { three?.globe.remove(lines); g.dispose() }
  })

  // a block of emoji on a canvas: three by three, fixed slots, each icon scaled by its band
  function iconCanvas(c) {
    const S = 96, cell = S / 3
    const cv = document.createElement('canvas'); cv.width = S; cv.height = S
    const ctx = cv.getContext('2d')
    ctx.textAlign = 'center'; ctx.textBaseline = 'middle'
    for (let i = 0; i < 9; i++) {
      const v = c.s?.[i]
      const col = i % 3, row = Math.floor(i / 3)
      const cx = col * cell + cell / 2, cy = row * cell + cell / 2
      if (v == null) { ctx.fillStyle = 'rgba(128,128,128,0.35)'; ctx.font = `${cell * 0.5}px sans-serif`; ctx.fillText('·', cx, cy); continue }
      const size = cell * (0.35 + 0.65 * Math.sqrt(Math.max(0, v)))   // area carries the amount
      ctx.font = `${size}px "Noto Color Emoji", "Apple Color Emoji", "Segoe UI Emoji", sans-serif`
      ctx.fillText(ICONS[i], cx, cy + size * 0.04)
    }
    return cv
  }

  // a vertical strip of emoji for the totem's skin: slot 0 at the bottom, each icon scaled by
  // its band; a missing band leaves its slot empty
  function totemCanvas(c, dark) {
    const W = 64, S = 64 * 9
    const cv = document.createElement('canvas'); cv.width = W; cv.height = S
    const ctx = cv.getContext('2d')
    ctx.fillStyle = dark ? '#2a2a2a' : '#f3f4f6'; ctx.fillRect(0, 0, W, S)
    ctx.textAlign = 'center'; ctx.textBaseline = 'middle'
    for (let i = 0; i < 9; i++) {
      const v = c.s?.[i]
      const cy = S - (i + 0.5) * W
      if (v == null) continue
      const size = W * (0.3 + 0.7 * Math.sqrt(Math.max(0, v)))
      ctx.font = `${size}px "Noto Color Emoji", "Apple Color Emoji", "Segoe UI Emoji", sans-serif`
      ctx.fillText(ICONS[i], W / 2, cy + size * 0.04)
    }
    return cv
  }

  // a ring pushed inward by d degrees along its own normal, so three rings sit side by side
  // on the ground instead of on top of one another (a stack at three radii projects onto one
  // line when seen from above, and the outermost hides the rest)
  function inset(ring, d) {
    const n = ring.length
    let area = 0
    for (let i = 0; i < n; i++) { const [x0, y0] = ring[i], [x1, y1] = ring[(i + 1) % n]; area += x0 * y1 - x1 * y0 }
    const sign = area > 0 ? 1 : -1          // which side is inside depends on the ring's winding
    const out = []
    for (let i = 0; i < n; i++) {
      const [px, py] = ring[(i - 1 + n) % n], [nx, ny] = ring[(i + 1) % n]
      const cosl = Math.max(0.2, Math.cos((ring[i][1] * Math.PI) / 180))
      let tx = (nx - px) * cosl, ty = ny - py
      const len = Math.hypot(tx, ty) || 1
      tx /= len; ty /= len
      out.push([ring[i][0] + (sign * -ty * d) / cosl, ring[i][1] + sign * tx * d])
    }
    return out
  }

  // outline glyphs: each country's own outline drawn once per group, the three rings side by
  // side from the border inward (economy on the border, then nature, then human), each the
  // group's pastel at a brightness set by its score and a width that grows a little with it
  $effect(() => {
    if (!three || !table || !countryRings || glyph !== 'outline') return
    const objs = [], ids = []
    const res = new THREE.Vector2(width * (window.devicePixelRatio || 1), height * (window.devicePixelRatio || 1))
    const r = R + 0.003
    for (const [iso3, c] of Object.entries(table.by_iso3 ?? {})) {
      const rings = countryRings[iso3]
      if (!rings || c.total == null) continue
      const scored = GROUPS.map((g, gi) => ({ gi, v: groupScore(c, g) })).filter((x) => x.v != null).reverse()
      scored.forEach(({ gi, v }, layer) => {
        const pos = []
        for (const ring0 of rings) {
          const ring = layer ? inset(ring0, 0.28 * layer) : ring0
          for (let i = 0; i + 1 < ring.length; i++) {
            const a = xyz(ring[i][1], ring[i][0], r), b = xyz(ring[i + 1][1], ring[i + 1][0], r)
            pos.push(a.x, a.y, a.z, b.x, b.y, b.z)
          }
        }
        const geo = new LineSegmentsGeometry(); geo.setPositions(pos)
        const colour = new THREE.Color(three.dark ? GROUPS[gi].dark : GROUPS[gi].light)
        const mat = new LineMaterial({ color: colour, linewidth: 2 + 1.5 * v, transparent: true, opacity: 0.3 + 0.7 * v, resolution: res, depthTest: true })
        const m = new LineSegments2(geo, mat); m.computeLineDistances()
        three.globe.add(m); objs.push(m); ids.push(iso3)
      })
    }
    three.glyphs = objs; three.ids = [ids]
    return () => { for (const m of objs) { three?.globe.remove(m); m.geometry.dispose(); m.material.dispose() } }
  })

  // totem glyphs: one textured cylinder per country, standing on its centroid along the normal
  $effect(() => {
    if (!three || !table || glyph !== 'totem') return
    const entries = Object.entries(table.by_iso3 ?? {})
    const up = new THREE.Vector3(0, 1, 0)
    const meshes = [], ids = []
    for (const [iso3, c] of entries) {
      if (c.total == null) continue   // a territory with no band at all has no pole
      const tex = new THREE.CanvasTexture(totemCanvas(c, three.dark)); tex.colorSpace = THREE.SRGBColorSpace
      tex.wrapS = THREE.RepeatWrapping; tex.repeat.set(2, 1)
      const h = TOTEM_H * (0.6 + 0.8 * (c.total ?? 0.5))
      const geo = new THREE.CylinderGeometry(TOTEM_R, TOTEM_R, h, 14, 1, false)
      const m = new THREE.Mesh(geo, new THREE.MeshBasicMaterial({ map: tex }))
      const normal = xyz(c.lat, c.lon, 1).normalize()
      m.quaternion.setFromUnitVectors(up, normal)
      m.position.copy(normal).multiplyScalar(R + h / 2)
      three.globe.add(m); meshes.push(m); ids.push(iso3)
    }
    three.glyphs = meshes; three.ids = [ids]
    return () => { for (const m of meshes) { three?.globe.remove(m); m.geometry.dispose(); m.material.map.dispose(); m.material.dispose() } }
  })

  // icon glyphs: one sprite per country, a billboard at its centroid
  $effect(() => {
    if (!three || !table || glyph !== 'icons') return
    const entries = Object.entries(table.by_iso3 ?? {})
    const sprites = [], ids = []
    for (const [iso3, c] of entries) {
      const tex = new THREE.CanvasTexture(iconCanvas(c)); tex.colorSpace = THREE.SRGBColorSpace
      const sp = new THREE.Sprite(new THREE.SpriteMaterial({ map: tex, transparent: true, depthTest: true }))
      const total = c.total ?? 0.5
      const size = ICON_SPRITE * (0.7 + 0.6 * total)   // the whole block grows a little with the total
      sp.scale.set(size, size, 1)
      sp.position.copy(xyz(c.lat, c.lon, R + 0.02 + size / 2))
      three.globe.add(sp); sprites.push(sp); ids.push(iso3)
    }
    three.glyphs = sprites; three.ids = [ids]
    return () => { for (const sp of sprites) { three?.globe.remove(sp); sp.material.map.dispose(); sp.material.dispose() } }
  })

  // bar glyphs: one instanced mesh per band, an instance per country; rebuilt when the table changes
  $effect(() => {
    if (!three || !table || glyph !== 'bars') return
    const entries = Object.entries(table.by_iso3 ?? {})
    const n = entries.length, nb = bands.length
    const box = new THREE.BoxGeometry(GLYPH_W, 1, GLYPH_W)
    const up = new THREE.Vector3(0, 1, 0)
    const meshes = [], ids = []
    for (let b = 0; b < nb; b++) {
      const mat = new THREE.MeshStandardMaterial({ color: new THREE.Color(BAND_COLOURS[b % BAND_COLOURS.length]), roughness: 0.7 })
      const m = new THREE.InstancedMesh(box, mat, n)
      const list = []
      let k = 0
      const mtx = new THREE.Matrix4(), q = new THREE.Quaternion(), pos = new THREE.Vector3(), scl = new THREE.Vector3()
      for (const [iso3, c] of entries) {
        // the band stands on the bands below it; a missing band takes no room
        let below = 0
        for (let j = 0; j < b; j++) below += (c.s?.[j] ?? 0) * BAND_H
        const h = (c.s?.[b] ?? 0) * BAND_H
        if (h <= 0) continue
        const normal = xyz(c.lat, c.lon, 1).normalize()
        pos.copy(normal).multiplyScalar(R + below + h / 2)
        q.setFromUnitVectors(up, normal)
        scl.set(1, h, 1)
        mtx.compose(pos, q, scl)
        m.setMatrixAt(k, mtx); list.push(iso3); k++
      }
      m.count = k
      m.instanceMatrix.needsUpdate = true
      three.globe.add(m); meshes.push(m); ids.push(list)
    }
    three.glyphs = meshes; three.ids = ids
    return () => { for (const m of meshes) { three?.globe.remove(m); m.material.dispose() } box.dispose() }
  })

  function onMove(e) {
    if (!three || !table) return
    const rect = canvas.getBoundingClientRect()
    const p = new THREE.Vector2(((e.clientX - rect.left) / rect.width) * 2 - 1, -((e.clientY - rect.top) / rect.height) * 2 + 1)
    three.raycaster.setFromCamera(p, three.camera)
    const hits = three.raycaster.intersectObjects(three.glyphs, false)
    if (!hits.length) { hover = null; return }
    const h = hits[0]; const b = three.glyphs.indexOf(h.object); const iso3 = glyph === 'bars' ? three.ids[b]?.[h.instanceId] : three.ids[0]?.[b]
    const c = iso3 ? table.by_iso3[iso3] : null
    hover = c ? { iso3, name: c.name, total: c.total, s: c.s, groups: GROUPS.map((g) => groupScore(c, g)), red: distress(c, first?.by_iso3?.[iso3]), stress: c.stress ?? 0, x: e.clientX - rect.left, y: e.clientY - rect.top } : null
  }

  const fmt = (v) => (v == null ? '—' : v.toFixed(2))
</script>

<figure class="min-w-0">
  <div class="relative mt-1 min-w-0 select-none" bind:clientWidth={width} bind:this={container}>
    <canvas bind:this={canvas} class="block w-full rounded-[3px]" style="height:{height}px; touch-action:none; overscroll-behavior:contain" onmousemove={onMove} onmouseleave={() => { hover = null }} onwheel={(e) => e.preventDefault()}></canvas>
    {#if error}<p class="absolute left-2 top-2 text-sm text-leak">{error}</p>
    {:else if !tables}<p class="absolute left-2 top-2 muted text-sm">loading the scores…</p>
    {:else if !table}<p class="absolute left-2 top-2 muted text-sm">no scores in this run</p>{/if}
    {#if hover}
      <div class="pointer-events-none absolute z-10 rounded-[3px] border border-neutral-200 bg-white/95 px-2 py-1.5 text-xs shadow-sm dark:border-neutral-700 dark:bg-neutral-900/95" style="left:{Math.min(hover.x + 12, width - 170)}px; top:{Math.max(hover.y - 10, 0)}px">
        <div class="font-medium">{hover.name} <span class="mono muted">{hover.iso3}</span></div>
        <div class="mono">total {fmt(hover.total)}{#if hover.red > 0.02} <span class="text-leak">· in the red {Math.round(hover.red * 100)}%</span>{/if}</div>
        {#if hover.stress > 0.005}<div class="mono text-leak">heat stress {Math.round(hover.stress * 100)}% of its people</div>{/if}
        {#each GROUPS as g, i (g.name)}
          <div class="mono flex items-center gap-1.5"><span class="inline-block h-2 w-2 rounded-full" style="background:{g.light}"></span><span class="w-16">{g.name}</span><span>{fmt(hover.groups?.[i])}</span></div>
        {/each}
        {#each bands as b, i (b.name)}
          <div class="mono flex items-center gap-1.5"><span class="inline-block h-2 w-2 rounded-[1px]" style="background:{BAND_COLOURS[i]}"></span><span class="w-16">{b.name}</span><span>{fmt(hover.s?.[i])}</span></div>
        {/each}
      </div>
    {/if}
  </div>
  <div class="mono mt-1.5 flex flex-wrap items-center gap-x-3 gap-y-1 text-[10px] muted">
    {#if table}<span>{table.year} · {Object.keys(table.by_iso3 ?? {}).length} countries</span>{/if}
    <span class="inline-flex items-center gap-1" title="a country is filled red by the worst of its pressures, never a mean: the share of its people past the heat thresholds this year, its weakest group under {RED_FLOOR}, or a fall of more than {RED_DROP} under its own start"><span class="inline-block h-2 w-3 rounded-[1px]" style="background:rgba(220,38,38,0.55)" aria-hidden="true"></span>in the red · {inRed}</span>
    {#if glyph === 'outline'}
      {#each GROUPS as g (g.name)}
        <span class="inline-flex items-center gap-1" title={g.bands.map((i) => bands[i]?.name).join(', ')}><span class="inline-block h-2 w-3 rounded-[1px]" style="background:{g.light}" aria-hidden="true"></span>{g.name}</span>
      {/each}
    {/if}
    {#each (glyph === 'outline' ? [] : bands) as b, i (b.name)}
      <span class="inline-flex items-center gap-1" title={b.what}>{#if glyph !== 'bars'}<span aria-hidden="true">{ICONS[i]}</span>{:else}<span class="inline-block h-2 w-2 rounded-[1px]" style="background:{BAND_COLOURS[i]}" aria-hidden="true"></span>{/if}{b.name}</span>
    {/each}
    <span class="inline-flex select-none" role="radiogroup" aria-label="glyph">
      {#each ['outline', 'totem', 'icons', 'bars'] as m (m)}
        <button type="button" role="radio" aria-checked={glyph === m} class="px-1 leading-4 rounded-[2px] {glyph === m ? 'text-brand ring-1 ring-brand/40' : 'hover:text-neutral-900 dark:hover:text-neutral-100'}" onclick={() => { glyph = m }}>{m}</button>
      {/each}
    </span>
    <span>· {glyph === 'outline' ? 'each country\'s outline drawn three times, side by side from the border inward: economy, nature, human; brighter the higher the group\'s score' : glyph === 'totem' ? 'nine icons up a pole in fixed slots, living at the foot and footprint at the top, each sized by its band, the pole by the total' : glyph === 'icons' ? 'nine icons in fixed slots, each sized by its band, the block by the total' : 'the nine bands stacked, the height the sum'} · each band 0 to 1, a rank among the countries that report it · drag to turn, wheel to zoom</span>
  </div>
</figure>
