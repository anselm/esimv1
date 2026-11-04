<script>
  // The ground in three dimensions: the terrain frame as a surface, buildings as boxes from
  // the building-height frame, and the fire's arrival draped over both at the cursor. Drawn
  // with three.js; the camera orbits by drag, zooms by wheel. Everything comes from the run's
  // own frames; nothing is a picture of a map.
  import * as THREE from 'three'
  import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
  import { FIRE } from './ramp.js'
  import { isoTime } from './fmt.js'
  import { seriesAt } from './runs.js'

  let { run, base, cursor = null, terrain = 'terrain_elev_m', buildings = 'building_height', fire = 'fire_arrival_h', arrival = 'building_arrival_h', fate = 'building_fate', perimeter = 'obs_perimeter', context = null } = $props()

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
  const pf = $derived(run.fields?.[perimeter])
  // the vertical is stretched this many times, ground and buildings alike, so the canyons read
  const ZS = 2
  // the wind at the cursor, from the fire module's own series
  const wind = $derived(cursor !== null ? { spd: seriesAt(run, 'fire.wind_m_s', cursor), from: seriesAt(run, 'fire.wind_from_deg', cursor) } : null)
  // local clock time from the window's longitude (whole hours; the solar zone, not the legal one)
  const utcOff = $derived(tf ? Math.round((tf.grid.lon_w + (tf.grid.nx * tf.grid.dlon) / 2) / 15) : 0)
  const localTime = $derived.by(() => {
    if (cursor === null) return ''
    const d = new Date((cursor + utcOff * 3600) * 1000)
    return `${String(d.getUTCHours()).padStart(2, '0')}:${String(d.getUTCMinutes()).padStart(2, '0')} local (UTC${utcOff >= 0 ? '+' : '−'}${Math.abs(utcOff)}, from the longitude)`
  })

  // the run's placed events, read from its log: the record's (obs.record: what people did, where
  // and when) and the fire's own ignition and spot fires. Each is drawn where it happened, pulses
  // for a while after its time at the cursor, and carries its time as a label.
  const EVENT_TOPICS = ['record.event', 'fire.ignition', 'fire.spot']
  let events = $state(null)
  $effect(() => {
    if (!base) return
    let live = true
    fetch(`${base}/log.jsonl`).then((r) => (r.ok ? r.text() : '')).then((text) => {
      if (!live) return
      const out = []
      for (const line of text.split('\n')) {
        if (!line || !EVENT_TOPICS.some((t) => line.includes(`"${t}"`))) continue
        try { const e = JSON.parse(line); if (EVENT_TOPICS.includes(e.topic) && e.at != null) out.push(e) } catch {}
      }
      out.sort((a, b) => a.t - b.t)
      events = out
    }).catch(() => { if (live) events = [] })
    return () => { live = false }
  })

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

  // the mapped perimeter current at the cursor, as the outline of its rasterised cells; keyed on
  // the frame index, as the fire frame is, so a moving cursor does not cancel the fetch
  let perim = $state(null)       // { f: Float32Array, grid }
  let perimIdx = $state(-1)
  $effect(() => {
    if (!pf || cursor === null) { perimIdx = -1; return }
    const times = pf.times ?? []
    let i = -1
    while (i + 1 < times.length && times[i + 1] <= cursor) i++
    perimIdx = i
  })
  $effect(() => {
    if (!pf) return
    const i = perimIdx
    if (i < 0) { perim = null; return }
    let live = true
    frame(perimeter, i).then((f) => { if (live) perim = { f, grid: pf.grid, i } }).catch(() => {})
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
  let scene, camera, renderer, controls, terrainMesh, terrainColors, boxMesh, boxColors, dims, hemi, sun, arrow, outline
  let ready = $state(false)
  let labelEl = $state()
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
        dims = { nx, ny, dx, dy, w: nx * dx, h: ny * dy, g, elev, zmin: 0 }
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
        dims.zmin = zmin
        for (let r = 0; r < ny; r++) for (let c = 0; c < nx; c++) { const k = r * nx + c; const z = elev[k]; pos.setZ(k, Number.isFinite(z) ? (z - zmin) * ZS : 0) }
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
            const z0 = Number.isFinite(elev[r * nx + c]) ? (elev[r * nx + c] - zmin) * ZS : 0
            m.makeScale(1, Math.max(2, h) * ZS, 1); m.setPosition((c + 0.5) * dx - dims.w / 2, z0, (r + 0.5) * dy - dims.h / 2)
            boxMesh.setMatrixAt(i, m)
          })
          boxMesh.userData.cells = cells
          boxMesh.instanceColor = new THREE.InstancedBufferAttribute(boxColors, 3)
          scene.add(boxMesh)
        }
        hemi = new THREE.HemisphereLight(0xffffff, 0x404040, 1.0); scene.add(hemi)
        sun = new THREE.DirectionalLight(0xffffff, 1.0); sun.position.set(-dims.w, 3 * (zmax - zmin) * ZS + 2000, dims.h); scene.add(sun)
        dims.ztop = (zmax - zmin) * ZS
        arrow = new THREE.ArrowHelper(new THREE.Vector3(0, 0, 1), new THREE.Vector3(-dims.w / 2 + 1500, dims.ztop + 900, -dims.h / 2 + 1500), 1, 0x129a9a, 500, 320); arrow.visible = false; scene.add(arrow)
        camera.position.set(0, (zmax - zmin) * ZS + dims.h * 0.9, dims.h * 1.1)
        controls.target.set(0, 0, 0)
        paint()
        sky()
        status = ''
        ready = true
        const loop = (now) => { if (disposed) return; controls.update(); animateMarks(now); renderer.render(scene, camera); requestAnimationFrame(loop) }
        loop(performance.now())
      } catch (e) { status = e.message }
    })()
    return () => { disposed = true; ready = false; renderer?.dispose() }
  })

  // ---- the marks: rings that pulse where an event happened, lines where a line was cut, labels with times
  const COL = { record: 0x129a9a, planned: 0x7bbcbc, spot: 0xf28c1a, ignition: 0xd9261a }
  // how long after its time an event keeps pulsing, in hours of run time, and the ring's radius in metres
  const SHOW = { record: [6, 80], line: [3, 0], spot: [1.5, 40], ignition: [1e9, 120] }
  const LINE_FADE_H = 12       // a line is bright for SHOW.line[0] hours, fades until this, then goes; at the run's end every line stays
  let marks = null          // { ring: InstancedMesh, pts: [...], lines: [...] }
  let active = []           // the points pulsing at the cursor
  let labels = []           // [{ el, pos }]
  let recordSoFar = $state(null)
  const hm = (t) => new Date(t * 1000).toISOString().slice(5, 16).replace('T', ' ') + 'Z'
  function worldAt(lat, lon, lift = 3) {
    const { g, dx, dy, w, h, nx, ny, elev, zmin } = dims
    const cf = (lon - g.lon_w) / g.dlon, rf = (g.lat_n - lat) / g.dlat
    const c = Math.min(nx - 1, Math.max(0, Math.floor(cf))), r = Math.min(ny - 1, Math.max(0, Math.floor(rf)))
    const z = elev[r * nx + c]
    return [cf * dx - w / 2, (Number.isFinite(z) ? (z - zmin) * ZS : 0) + lift, rf * dy - h / 2]
  }
  function worldCell(cell, lift = 3) {
    const { dx, dy, w, h, nx, elev, zmin } = dims
    const c = cell % nx, r = Math.floor(cell / nx)
    const z = elev[cell]
    return [(c + 0.5) * dx - w / 2, (Number.isFinite(z) ? (z - zmin) * ZS : 0) + lift, (r + 0.5) * dy - h / 2]
  }
  function buildMarks() {
    if (marks) { scene.remove(marks.ring); for (const l of marks.lines) scene.remove(l.obj); marks = null }
    if (!events?.length) return
    const pts = [], lines = []
    for (const e of events) {
      const d = e.data ?? {}
      if (e.topic === 'record.event' && d.kind === 'line' && Array.isArray(d.path) && d.path.length > 1) {
        const planned = /planned|proposed/i.test(d.category) || d.status === 'Proposed'
        const geom = new THREE.BufferGeometry().setFromPoints(d.path.map(([la, lo]) => new THREE.Vector3(...worldAt(la, lo, 4))))
        const mat = new THREE.LineBasicMaterial({ color: planned ? COL.planned : COL.record, transparent: true, opacity: planned ? 0.5 : 0.9 })
        const obj = new THREE.Line(geom, mat); obj.visible = false; scene.add(obj)
        lines.push({ obj, t: e.t, mat, label: `${hm(e.t)} ${d.category}${d.length_m ? ' ' + Math.round(d.length_m) + ' m' : ''}${d.note ? ' · ' + d.note : ''}`, pos: worldAt(d.path[0][0], d.path[0][1], 4) })
        continue
      }
      const kind = e.topic === 'fire.ignition' ? 'ignition' : e.topic === 'fire.spot' ? 'spot' : 'record'
      const pos = (kind === 'record' && Number.isFinite(d.lat)) ? worldAt(d.lat, d.lon) : worldCell(e.at)
      const t0 = kind === 'ignition' && d.time ? Math.round(Date.parse(d.time) / 1000) : e.t
      const label = kind === 'ignition' ? `${hm(t0)} ignition` : kind === 'record' ? `${hm(e.t)} ${d.category}${d.label ? ' · ' + d.label : ''}${d.note ? ' · ' + d.note : ''}` : null
      pts.push({ t: t0, kind, pos, label })
    }
    const ringGeom = new THREE.RingGeometry(0.72, 1, 40); ringGeom.rotateX(-Math.PI / 2)
    const ring = new THREE.InstancedMesh(ringGeom, new THREE.MeshBasicMaterial({ side: THREE.DoubleSide, transparent: true, opacity: 0.9, depthWrite: false }), Math.max(1, pts.length))
    const zero = new THREE.Matrix4().makeScale(0, 0, 0)
    const col = new THREE.Color()
    pts.forEach((p, i) => { ring.setMatrixAt(i, zero); ring.setColorAt(i, col.setHex(COL[p.kind])) })
    ring.instanceMatrix.needsUpdate = true; if (ring.instanceColor) ring.instanceColor.needsUpdate = true
    ring.frustumCulled = false
    scene.add(ring)
    marks = { ring, pts, lines }
    updateMarks()
  }
  function updateMarks() {
    if (!marks || cursor === null) { active = []; setLabels([]); return }
    const { pts, lines } = marks
    const act = [], lab = []
    let npts = 0
    for (let i = 0; i < pts.length; i++) {
      const p = pts[i]
      if (p.t > cursor) continue
      if (p.kind === 'record') npts++
      const age = (cursor - p.t) / 3600
      if (age <= SHOW[p.kind][0]) { act.push({ i, p, frac: Math.min(1, age / Math.min(SHOW[p.kind][0], 24)) }); if (p.label) lab.push({ t: p.t, text: p.label, pos: p.pos }) }
    }
    let nlines = 0
    // at the end of the run the whole record stays on the ground (Anselm, 2026-11-04)
    const atEnd = run.end != null && cursor >= run.end - 3600
    for (const l of lines) {
      if (l.t > cursor) { l.obj.visible = false; continue }
      nlines++
      const age = (cursor - l.t) / 3600
      const fresh = age <= SHOW.line[0]
      const faded = age >= LINE_FADE_H
      l.obj.visible = atEnd || !faded
      l.mat.opacity = fresh ? 1 : atEnd && faded ? 0.5 : 0.55 * (1 - (age - SHOW.line[0]) / (LINE_FADE_H - SHOW.line[0])) + 0.1
      if (fresh) lab.push({ t: l.t, text: l.label, pos: l.pos })
    }
    // the latest two dozen labels, so the view stays readable; everything else still pulses
    lab.sort((a, b) => b.t - a.t)
    // the archive holds the same feature more than once (versions, statuses): one label per text
    const seen = new Set()
    setLabels(lab.filter((l) => !seen.has(l.text) && seen.add(l.text)).slice(0, 24))
    active = act
    const zero = new THREE.Matrix4().makeScale(0, 0, 0)
    const shown = new Set(act.map((a) => a.i))
    for (let i = 0; i < pts.length; i++) if (!shown.has(i)) marks.ring.setMatrixAt(i, zero)
    marks.ring.instanceMatrix.needsUpdate = true
    recordSoFar = npts + nlines > 0 ? { points: npts, lines: nlines } : null
  }
  function setLabels(list) {
    if (!labelEl) return
    while (labelEl.children.length > list.length) labelEl.removeChild(labelEl.lastChild)
    while (labelEl.children.length < list.length) { const d = document.createElement('div'); d.className = 'mark-label'; labelEl.appendChild(d) }
    labels = list.map((l, k) => { const el = labelEl.children[k]; el.textContent = l.text; return { el, pos: l.pos } })
  }
  const _m = new THREE.Matrix4(), _v = new THREE.Vector3()
  function animateMarks(now) {
    if (!marks) return
    const breath = 1 + 0.12 * Math.sin((now / 1100) * 2 * Math.PI)
    for (const { i, p, frac } of active) {
      const r = SHOW[p.kind][1] * (1 + 1.5 * frac) * breath
      _m.makeScale(r, 1, r); _m.setPosition(p.pos[0], p.pos[1], p.pos[2])
      marks.ring.setMatrixAt(i, _m)
    }
    if (active.length) marks.ring.instanceMatrix.needsUpdate = true
    const placed = []
    for (const { el, pos } of labels) {
      _v.set(pos[0], pos[1], pos[2]).project(camera)
      if (_v.z > 1 || Math.abs(_v.x) > 1.1 || Math.abs(_v.y) > 1.1) { el.style.display = 'none'; continue }
      el.style.display = ''
      const x = ((_v.x + 1) / 2) * width
      let y = ((1 - _v.y) / 2) * height - 18
      const hw = (el.offsetWidth || 120) / 2
      // step down past any label already placed over the same stretch of screen
      for (let guard = 0; guard < 12; guard++) { const hit = placed.find((q) => Math.abs(q.y - y) < 14 && Math.abs(q.x - x) < q.hw + hw); if (!hit) break; y = hit.y + 14 }
      placed.push({ x, y, hw })
      el.style.transform = `translate(${x}px, ${y}px) translate(-50%, -50%)`
    }
  }
  $effect(() => { if (ready && events) buildMarks() })
  $effect(() => { cursor; labelEl; if (ready) updateMarks() })

  // colours: the ground by its kind of cover is not known here, so by height; the fire by arrival hour
  function paint() {
    if (!terrainColors || !dims) return
    const { nx, ny } = dims
    const ffg = ff?.grid
    const dark = matchMedia('(prefers-color-scheme: dark)').matches
    const ground = dark ? [0.22, 0.24, 0.22] : [0.80, 0.80, 0.76]
    const sea = dark ? [0.10, 0.16, 0.26] : [0.62, 0.72, 0.84]
    for (let r = 0; r < ny; r++) for (let c = 0; c < nx; c++) {
      const k = r * nx + c
      let col = ground
      const z = dims.elev[k]
      if (!Number.isFinite(z) || z <= 0.1) col = sea
      else if (hours && ffg) {
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

  // ---- the sky: the sun's place over the window at the cursor sets the light and the backdrop.
  // Low-precision solar position (the Astronomical Almanac's, recalled; good to a fraction of a degree).
  function sunAt(lat, lon, t) {
    const d = t / 86400 - 10957.5
    const rad = Math.PI / 180
    const L = (280.460 + 0.9856474 * d) % 360, g = (357.528 + 0.9856003 * d) * rad
    const lam = (L + 1.915 * Math.sin(g) + 0.020 * Math.sin(2 * g)) * rad
    const eps = (23.439 - 0.0000004 * d) * rad
    const ra = Math.atan2(Math.cos(eps) * Math.sin(lam), Math.cos(lam)), dec = Math.asin(Math.sin(eps) * Math.sin(lam))
    const gmst = (18.697374558 + 24.06570982441908 * d) % 24
    const H = (gmst * 15 + lon) * rad - ra
    const la = lat * rad
    const east = -Math.cos(dec) * Math.sin(H)
    const north = Math.sin(dec) * Math.cos(la) - Math.cos(dec) * Math.cos(H) * Math.sin(la)
    const up = Math.sin(dec) * Math.sin(la) + Math.cos(dec) * Math.cos(H) * Math.cos(la)
    return { east, north, up, elev: Math.asin(up) / rad }
  }
  const mix = (a, b, t) => a.map((x, i) => x + (b[i] - x) * t)
  const smooth = (x) => { const t = Math.min(1, Math.max(0, x)); return t * t * (3 - 2 * t) }
  let sunElev = $state(null)
  let dayness = 1              // 0 at night, 1 in daylight; the perimeter dots pick their colour by it
  function sky() {
    if (!scene || !dims || cursor === null) return
    const g = dims.g
    const lat = g.lat_n - (dims.ny * g.dlat) / 2, lon = g.lon_w + (dims.nx * g.dlon) / 2
    const sp = sunAt(lat, lon, cursor)
    sunElev = sp.elev
    const dark = matchMedia('(prefers-color-scheme: dark)').matches
    dayness = smooth((sp.elev + 6) / 16)                // 0 below −6°, 1 above 10°
    const dusk = 1 - Math.min(1, Math.abs(sp.elev) / 8)  // warm band around the horizon
    const dayCol = dark ? [0.29, 0.35, 0.42] : [0.78, 0.86, 0.94], nightCol = [0.025, 0.03, 0.055], duskCol = [0.78, 0.52, 0.32]
    let c = mix(nightCol, dayCol, dayness); c = mix(c, duskCol, dusk * 0.3)
    scene.background = new THREE.Color(c[0], c[1], c[2])
    // the sun lights the ground from where it stands; below the horizon only the sky's glow is left
    sun.intensity = 1.2 * Math.max(0, sp.up)
    sun.position.set(sp.east, Math.max(0.05, sp.up), -sp.north).multiplyScalar(50_000)
    hemi.intensity = 0.3 + 0.8 * dayness
    hemi.color.setRGB(c[0] * 0.6 + 0.4, c[1] * 0.6 + 0.4, c[2] * 0.6 + 0.45)
    // the wind arrow: pointing the way the wind blows, long in proportion to its speed
    if (arrow && wind && Number.isFinite(wind.spd) && Number.isFinite(wind.from)) {
      const to = (wind.from + 180) * Math.PI / 180
      arrow.setDirection(new THREE.Vector3(Math.sin(to), 0, -Math.cos(to)).normalize())
      const len = Math.max(400, 250 * wind.spd); arrow.setLength(len, Math.min(500, len * 0.45), Math.min(320, len * 0.3))
      arrow.visible = true
    } else if (arrow) arrow.visible = false
    if (outline) outline.material.color.set(dayness > 0.5 ? 0x111111 : 0xf2f2f2)
  }
  $effect(() => { cursor; wind; if (ready) sky() })

  // ---- the mapped perimeter: the outline of the cells the record says were burned at the cursor
  function drawOutline() {
    if (!scene || !dims) return
    if (outline) { scene.remove(outline); outline.geometry.dispose(); outline = null }
    if (!perim) return
    const { f, grid: pg } = perim
    const pts = []
    const inside = (r, c) => r >= 0 && c >= 0 && r < pg.ny && c < pg.nx && f[r * pg.nx + c] >= 0.5
    for (let r = 0; r < pg.ny; r++) for (let c = 0; c < pg.nx; c++) {
      if (!inside(r, c)) continue
      if (inside(r - 1, c) && inside(r + 1, c) && inside(r, c - 1) && inside(r, c + 1)) continue
      const lat = pg.lat_n - (r + 0.5) * pg.dlat, lon = pg.lon_w + (c + 0.5) * pg.dlon
      const w = worldAt(lat, lon, 30)
      if (w) pts.push(...w)
    }
    if (!pts.length) return
    const geom = new THREE.BufferGeometry()
    geom.setAttribute('position', new THREE.Float32BufferAttribute(pts, 3))
    outline = new THREE.Points(geom, new THREE.PointsMaterial({ color: dayness > 0.5 ? 0x111111 : 0xf2f2f2, size: Math.max(dims.dx, dims.dy) * 2.2, sizeAttenuation: true, depthTest: false }))
    outline.renderOrder = 2
    scene.add(outline)
  }
  $effect(() => { perim; if (ready) drawOutline() })
  $effect(() => { if (renderer && camera) { renderer.setSize(width, height); camera.aspect = width / height; camera.updateProjectionMatrix() } })
</script>

<figure class="min-w-0">
  <figcaption class="flex flex-wrap items-baseline justify-between gap-x-3 text-xs">
    <span class="mono">{terrain} <span class="muted">+</span> {buildings} <span class="muted">+</span> {fire}</span>
    <span class="mono muted">{cursor !== null ? isoTime(cursor) : ''}{localTime ? ` · ${localTime}` : ''}{sunElev !== null ? ` · sun ${sunElev >= 0 ? '+' : '−'}${Math.abs(sunElev).toFixed(0)}°` : ''}{wind && Number.isFinite(wind.spd) ? ` · wind ${wind.spd.toFixed(0)} m/s from ${Math.round(wind.from)}°` : ''} · drag to orbit, wheel to zoom</span>
  </figcaption>
  <div class="relative mt-1 min-w-0 select-none" bind:clientWidth={width}>
    <div bind:this={el} style="width:{width}px;height:{height}px" class="rounded-[2px] bg-neutral-100 dark:bg-neutral-900"></div>
    <div bind:this={labelEl} class="pointer-events-none absolute inset-0 overflow-hidden" aria-hidden="true"></div>
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
    {#if events?.length}
      <span><span class="inline-block h-2 w-2 rounded-full border-2 align-middle" style="border-color:#d9261a" aria-hidden="true"></span> ignition</span>
      <span><span class="inline-block h-2 w-2 rounded-full border-2 align-middle" style="border-color:#f28c1a" aria-hidden="true"></span> spot fire the model threw, pulsing for an hour and a half</span>
      {#if recordSoFar || events.some((e) => e.topic === 'record.event')}
        <span><span class="inline-block h-2 w-2 rounded-full border-2 align-middle" style="border-color:#129a9a" aria-hidden="true"></span> the record: a point (dozer push, drop point, helispot, break), pulsing for six hours after it was made</span>
        <span><span class="inline-block h-0.5 w-3 align-middle" style="background:#129a9a" aria-hidden="true"></span> a completed line (dozer, hand, road), bright for three hours, gone after twelve; at the run's end the whole record stays</span>
        {#if recordSoFar}<span>so far: {recordSoFar.points} points, {recordSoFar.lines} lines</span>{/if}
      {/if}
      <span>labels carry the event's time in UTC; the record's times are when the GIS feature was made, which lags the work</span>
    {/if}
    {#if pf}<span><span class="inline-block h-2 w-2 rounded-full align-middle bg-neutral-900 dark:bg-neutral-100" aria-hidden="true"></span> the mapped perimeter current at this time, as dots along its edge</span>{/if}
    {#if wind && Number.isFinite(wind.spd)}<span><span class="inline-block h-0.5 w-3 align-middle" style="background:#129a9a" aria-hidden="true"></span> the wind at the ignition point: the arrow at the north-west corner points the way it blows, 250 m per m/s</span>{/if}
    <span>the sky and the light follow the sun over the window at the cursor; the vertical is stretched twice, ground and buildings alike; building heights as the county states them, taken as feet</span>
  </div>
</figure>

<style>
  :global(.mark-label) {
    position: absolute; left: 0; top: 0; white-space: nowrap; pointer-events: none;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 10px; line-height: 1.3;
    padding: 1px 4px; border-radius: 2px; color: #0f6b6b; background: rgba(250, 250, 250, 0.82);
    max-width: 260px; overflow: hidden; text-overflow: ellipsis;
  }
  @media (prefers-color-scheme: dark) { :global(.mark-label) { color: #7fd6d6; background: rgba(10, 10, 10, 0.78); } }
</style>
