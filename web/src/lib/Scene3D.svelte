<script>
  // The ground in three dimensions: the terrain frame as a surface, buildings as boxes from
  // the building-height frame, and the fire's arrival draped over both at the cursor. Drawn
  // with three.js; the camera orbits by drag, zooms by wheel. Everything comes from the run's
  // own frames; nothing is a picture of a map.
  import * as THREE from 'three'
  import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
  import { FIRE } from './ramp.js'
  import { isoTime } from './fmt.js'

  let { run, base, cursor = null, terrain = 'terrain_elev_m', buildings = 'building_height', fire = 'fire_arrival_h', arrival = 'building_arrival_h', fate = 'building_fate', context = null } = $props()

  // the bodies, when the run has them: each building's own arrival hour (the last frame holds them
  // all) and the inspectors' word; the ignition instant is read off the first fire frame
  const af = $derived(run.fields?.[arrival])
  const faf = $derived(run.fields?.[fate])
  let bodyHours = $state(null)   // Float32Array on the 1-arcsec grid, hours since ignition
  let bodyFate = $state(null)
  let ignitionT = $state(null)
  const cursorH = $derived(cursor !== null && ignitionT !== null ? (cursor - ignitionT) / 3600 : null)

  let el = $state()
  let width = $state(640)
  let status = $state('loading the ground…')
  let hours = $state(null)       // the fire frame at the cursor, as Float32Array
  let fireMax = $state(1)
  const height = $derived(Math.round(width * 0.62))

  const tf = $derived(run.fields?.[terrain])
  const bf = $derived(run.fields?.[buildings])
  const ff = $derived(run.fields?.[fire])

  async function frame(name, i) {
    const r = await fetch(`${base}/fields/${encodeURIComponent(name)}/${String(i).padStart(5, '0')}.f32`)
    if (!r.ok) throw new Error(`${name} ${i}: ${r.status}`)
    const buf = await r.arrayBuffer(); const dv = new DataView(buf); const n = buf.byteLength >> 2
    const out = new Float32Array(n); for (let k = 0; k < n; k++) out[k] = dv.getFloat32(k * 4, true)
    return out
  }

  // the fire frame at or before the cursor
  let fireIdx = $state(-1)
  $effect(() => {
    if (!ff || cursor === null) return
    const times = ff.times ?? []
    let i = -1
    while (i + 1 < times.length && times[i + 1] <= cursor) i++
    fireIdx = i
  })
  $effect(() => {
    if (!ff) return
    const i = fireIdx
    if (i < 0) { hours = null; return }
    let live = true
    frame(fire, i).then((f) => { if (live) { hours = f; fireMax = Math.max(1, ff.max ?? 1) } }).catch(() => {})
    return () => { live = false }
  })

  $effect(() => {
    if (!af || !faf || !ff) return
    let live = true
    const last = (af.times?.length ?? 1) - 1
    Promise.all([frame(arrival, last), frame(fate, 0), frame(fire, 0)]).then(([a, f, f0]) => {
      if (!live) return
      bodyHours = a; bodyFate = f
      // the first fire frame's latest arrival is about an hour after ignition
      let mx = 0; for (let k = 0; k < f0.length; k++) if (Number.isFinite(f0[k]) && f0[k] > mx) mx = f0[k]
      ignitionT = (ff.times?.[0] ?? 0) - mx * 3600
    }).catch(() => {})
    return () => { live = false }
  })

  // the scene, built once from the terrain and building frames
  let scene, camera, renderer, controls, terrainMesh, terrainColors, boxMesh, boxColors, dims
  const fireRgb = FIRE.map((h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16) / 255))
  const fireColor = (t) => { const x = Math.min(1, Math.max(0, t)) * (fireRgb.length - 1); const a = Math.floor(x), b = Math.min(a + 1, fireRgb.length - 1), f = x - a; return fireRgb[a].map((c, k) => c + (fireRgb[b][k] - c) * f) }

  $effect(() => {
    if (!el || !tf) return
    let disposed = false
    ;(async () => {
      try {
        const g = tf.grid
        const elev = await frame(terrain, 0)
        const bh = bf ? await frame(buildings, 0) : null
        if (disposed) return
        const dark = matchMedia('(prefers-color-scheme: dark)').matches
        const { nx, ny } = g
        // metres per cell, so the vertical is in the same units as the horizontal
        const latMid = g.lat_n - (ny * g.dlat) / 2
        const mPerDegLat = 111_320, mPerDegLon = 111_320 * Math.cos((latMid * Math.PI) / 180)
        const dx = g.dlon * mPerDegLon, dy = g.dlat * mPerDegLat
        dims = { nx, ny, dx, dy, w: nx * dx, h: ny * dy }
        scene = new THREE.Scene()
        scene.background = new THREE.Color(dark ? 0x0a0a0a : 0xfafafa)
        camera = new THREE.PerspectiveCamera(45, width / height, 10, 200_000)
        renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false })
        renderer.setPixelRatio(Math.min(2, window.devicePixelRatio || 1))
        renderer.setSize(width, height)
        el.replaceChildren(renderer.domElement)
        controls = new OrbitControls(camera, renderer.domElement)
        controls.enableDamping = true
        // the surface: one vertex per cell, z from the terrain frame (NaN as the lowest value)
        const geom = new THREE.PlaneGeometry(dims.w, dims.h, nx - 1, ny - 1)
        const pos = geom.attributes.position
        let zmin = Infinity, zmax = -Infinity
        for (let k = 0; k < elev.length; k++) { const z = elev[k]; if (Number.isFinite(z)) { if (z < zmin) zmin = z; if (z > zmax) zmax = z } }
        for (let r = 0; r < ny; r++) for (let c = 0; c < nx; c++) { const k = r * nx + c; const z = elev[k]; pos.setZ(k, Number.isFinite(z) ? z - zmin : 0) }
        geom.computeVertexNormals()
        terrainColors = new Float32Array(nx * ny * 3)
        geom.setAttribute('color', new THREE.BufferAttribute(terrainColors, 3))
        const mat = new THREE.MeshLambertMaterial({ vertexColors: true, side: THREE.DoubleSide })
        terrainMesh = new THREE.Mesh(geom, mat)
        terrainMesh.rotation.x = -Math.PI / 2   // the plane's y becomes north; z up
        scene.add(terrainMesh)
        // buildings: a box per built cell, its height from the frame (taken as feet)
        if (bh) {
          const cells = []
          for (let r = 0; r < ny; r++) for (let c = 0; c < nx; c++) { const h = bh[r * nx + c]; if (Number.isFinite(h) && h > 0) cells.push([r, c, h * 0.3048]) }
          const box = new THREE.BoxGeometry(dx * 0.7, 1, dy * 0.7)
          box.translate(0, 0.5, 0)
          // no vertexColors here: the box geometry has no colour attribute, and with the flag set
          // every instance drew black whatever its instance colour said (until 2026-11-03)
          const bmat = new THREE.MeshLambertMaterial()
          boxMesh = new THREE.InstancedMesh(box, bmat, cells.length)
          boxColors = new Float32Array(cells.length * 3)
          const m = new THREE.Matrix4()
          cells.forEach(([r, c, h], i) => {
            const z0 = Number.isFinite(elev[r * nx + c]) ? elev[r * nx + c] - zmin : 0
            m.makeScale(1, Math.max(2, h), 1); m.setPosition((c + 0.5) * dx - dims.w / 2, z0, (r + 0.5) * dy - dims.h / 2)
            boxMesh.setMatrixAt(i, m)
          })
          boxMesh.userData.cells = cells
          boxMesh.instanceColor = new THREE.InstancedBufferAttribute(boxColors, 3)
          scene.add(boxMesh)
        }
        scene.add(new THREE.HemisphereLight(0xffffff, 0x404040, dark ? 0.9 : 1.1))
        const sun = new THREE.DirectionalLight(0xffffff, dark ? 0.8 : 1.0); sun.position.set(-dims.w, 3 * (zmax - zmin) + 2000, dims.h); scene.add(sun)
        camera.position.set(0, (zmax - zmin) + dims.h * 0.9, dims.h * 1.1)
        controls.target.set(0, 0, 0)
        paint()
        status = ''
        const loop = () => { if (disposed) return; controls.update(); renderer.render(scene, camera); requestAnimationFrame(loop) }
        loop()
      } catch (e) { status = e.message }
    })()
    return () => { disposed = true; renderer?.dispose() }
  })

  // colours: the ground by its kind of cover is not known here, so by height; the fire by arrival hour
  function paint() {
    if (!terrainColors || !dims) return
    const { nx, ny } = dims
    const ffg = ff?.grid
    const dark = matchMedia('(prefers-color-scheme: dark)').matches
    const ground = dark ? [0.22, 0.24, 0.22] : [0.80, 0.80, 0.76]
    for (let r = 0; r < ny; r++) for (let c = 0; c < nx; c++) {
      const k = r * nx + c
      let col = ground
      if (hours && ffg) {
        // the fire frame is coarser: sample it by position
        const fr = Math.min(ffg.ny - 1, Math.floor((r * ny / ny) * (ffg.ny / ny))), fc = Math.min(ffg.nx - 1, Math.floor(c * (ffg.nx / nx)))
        const h = hours[fr * ffg.nx + fc]
        if (Number.isFinite(h)) col = fireColor(h / fireMax)
      }
      terrainColors[k * 3] = col[0]; terrainColors[k * 3 + 1] = col[1]; terrainColors[k * 3 + 2] = col[2]
    }
    terrainMesh.geometry.attributes.color.needsUpdate = true
    if (boxMesh) {
      const cells = boxMesh.userData.cells
      const bodies = bodyHours && bodyFate && cursorH !== null
      cells.forEach(([r, c], i) => {
        let col = dark ? [0.55, 0.55, 0.6] : [0.45, 0.45, 0.5]
        if (bodies) {
          // the body's own state against the inspectors' word
          const k = r * nx + c
          const lit = Number.isFinite(bodyHours[k]) && bodyHours[k] <= cursorH
          const f = bodyFate[k]
          if (lit && f === 3) col = [0.85, 0.15, 0.1]            // burned, and it was destroyed
          else if (lit) col = [0.95, 0.55, 0.1]                  // burned in the model, not destroyed in the record
          else if (f === 3) col = [0.2, 0.45, 0.9]               // destroyed in the record, not reached by the model
          else if (f === 2) col = dark ? [0.6, 0.55, 0.45] : [0.6, 0.55, 0.45]
        } else if (hours && ffg) {
          const fr = Math.min(ffg.ny - 1, Math.floor(r * (ffg.ny / ny))), fc = Math.min(ffg.nx - 1, Math.floor(c * (ffg.nx / nx)))
          const h = hours[fr * ffg.nx + fc]
          if (Number.isFinite(h)) col = [0.85, 0.15, 0.1]
        }
        boxColors[i * 3] = col[0]; boxColors[i * 3 + 1] = col[1]; boxColors[i * 3 + 2] = col[2]
      })
      boxMesh.instanceColor.needsUpdate = true
    }
  }
  $effect(() => { hours; bodyHours; bodyFate; cursorH; paint() })
  $effect(() => { if (renderer && camera) { renderer.setSize(width, height); camera.aspect = width / height; camera.updateProjectionMatrix() } })
</script>

<figure class="min-w-0">
  <figcaption class="flex flex-wrap items-baseline justify-between gap-x-3 text-xs">
    <span class="mono">{terrain} <span class="muted">+</span> {buildings} <span class="muted">+</span> {fire}</span>
    <span class="mono muted">{cursor !== null ? isoTime(cursor) : ''} · drag to orbit, wheel to zoom</span>
  </figcaption>
  <div class="relative mt-1 min-w-0 select-none" bind:clientWidth={width}>
    <div bind:this={el} style="width:{width}px;height:{height}px" class="rounded-[2px] bg-neutral-100 dark:bg-neutral-900"></div>
    {#if status}<span class="absolute left-2 top-2 text-xs muted">{status}</span>{/if}
  </div>
  <div class="mono mt-1.5 flex flex-wrap items-center gap-x-4 gap-y-1 text-[10px] muted">
    <span><span class="inline-block h-2 w-2 align-middle" style="background:{FIRE[1]}" aria-hidden="true"></span> burned early</span>
    <span><span class="inline-block h-2 w-2 align-middle" style="background:{FIRE[5]}" aria-hidden="true"></span> burned late</span>
    {#if bodyHours && bodyFate}
      <span><span class="inline-block h-2 w-2 align-middle bg-neutral-500" aria-hidden="true"></span> building, unburned, not destroyed</span>
      <span><span class="inline-block h-2 w-2 align-middle" style="background:#d9261a" aria-hidden="true"></span> burned, and the inspectors found it destroyed</span>
      <span><span class="inline-block h-2 w-2 align-middle" style="background:#f28c1a" aria-hidden="true"></span> burned in the model, not destroyed in the record</span>
      <span><span class="inline-block h-2 w-2 align-middle" style="background:#3373e6" aria-hidden="true"></span> destroyed in the record, not reached by the model</span>
    {:else}
      <span><span class="inline-block h-2 w-2 align-middle bg-neutral-500" aria-hidden="true"></span> building, unburned</span>
      <span><span class="inline-block h-2 w-2 align-middle" style="background:#d9261a" aria-hidden="true"></span> building in a burned cell</span>
    {/if}
    <span>vertical in metres, as the ground is; building heights as the county states them, taken as feet</span>
  </div>
</figure>
