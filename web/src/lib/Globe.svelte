<script>
  // The world scored, on a globe: every country a glyph at its centroid, the nine bands of
  // its score stacked outward from the surface, the total its height. The table comes from
  // the run's log (region.scored events, the latest at or before the cursor), the outlines
  // from docs/outlines.json. Drag to turn, wheel to zoom, hover a glyph for the bands.
  import * as THREE from 'three'
  import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'

  let { run, base, cursor = null, source = 'region.scored' } = $props()

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

  // glyphs: one instanced mesh per band, an instance per country; rebuilt when the table changes
  $effect(() => {
    if (!three || !table) return
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
    const h = hits[0]; const b = three.glyphs.indexOf(h.object); const iso3 = three.ids[b]?.[h.instanceId]
    const c = iso3 ? table.by_iso3[iso3] : null
    hover = c ? { iso3, name: c.name, total: c.total, s: c.s, x: e.clientX - rect.left, y: e.clientY - rect.top } : null
  }

  const fmt = (v) => (v == null ? '—' : v.toFixed(2))
</script>

<figure class="min-w-0">
  <div class="relative mt-1 min-w-0 select-none" bind:clientWidth={width} bind:this={container}>
    <canvas bind:this={canvas} class="block w-full rounded-[3px]" style="height:{height}px" onmousemove={onMove} onmouseleave={() => { hover = null }}></canvas>
    {#if error}<p class="absolute left-2 top-2 text-sm text-leak">{error}</p>
    {:else if !tables}<p class="absolute left-2 top-2 muted text-sm">loading the scores…</p>
    {:else if !table}<p class="absolute left-2 top-2 muted text-sm">no scores in this run</p>{/if}
    {#if hover}
      <div class="pointer-events-none absolute z-10 rounded-[3px] border border-neutral-200 bg-white/95 px-2 py-1.5 text-xs shadow-sm dark:border-neutral-700 dark:bg-neutral-900/95" style="left:{Math.min(hover.x + 12, width - 170)}px; top:{Math.max(hover.y - 10, 0)}px">
        <div class="font-medium">{hover.name} <span class="mono muted">{hover.iso3}</span></div>
        <div class="mono">total {fmt(hover.total)}</div>
        {#each bands as b, i (b.name)}
          <div class="mono flex items-center gap-1.5"><span class="inline-block h-2 w-2 rounded-[1px]" style="background:{BAND_COLOURS[i]}"></span><span class="w-16">{b.name}</span><span>{fmt(hover.s?.[i])}</span></div>
        {/each}
      </div>
    {/if}
  </div>
  <div class="mono mt-1.5 flex flex-wrap items-center gap-x-3 gap-y-1 text-[10px] muted">
    {#if table}<span>{table.year} · {Object.keys(table.by_iso3 ?? {}).length} countries</span>{/if}
    {#each bands as b, i (b.name)}
      <span class="inline-flex items-center gap-1" title={b.what}><span class="inline-block h-2 w-2 rounded-[1px]" style="background:{BAND_COLOURS[i]}" aria-hidden="true"></span>{b.name}</span>
    {/each}
    <span>· each band 0 to 1, a rank among the countries that report it; the glyph's height is the sum · drag to turn, wheel to zoom</span>
  </div>
</figure>
