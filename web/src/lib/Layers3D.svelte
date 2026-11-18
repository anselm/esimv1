<script>
  // The place in three dimensions, with the run's modules grouped by layer and switched on and
  // off. The relief is the run's own elevation frame, drawn as contour lines on a dark ground;
  // each module's map frames are draped over it as points of light in its layer's colour, and the
  // things a module places (tanks, bridges, hospitals, a fire's ignition) stand on it as marks in
  // the state they are in at the cursor. It is one view for every scenario: it asks only for an
  // elevation frame and modules that carry a layer, and draws whatever frames and placed things
  // the run has. What changes between one moment and the next is marked by a small effect. Layers can be lifted apart to see each alone. The look follows the Bay
  // Layers study (github.com/anselm/bay-area-twin): luminous, wire-like, the scene carrying the
  // explanation. Nothing here is a picture of a map; every point comes from the run folder.
  import * as THREE from 'three'
  import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
  import { LAYERS, HARM } from './layers.js'
  import { createEffects } from './effects.js'
  import { isoTime } from './fmt.js'

  let { run, base, cursor = null, event = null, terrain = 'terrain_elev_m' } = $props()

  const ZS = 4            // the vertical is stretched this many times so the hills read
  const LIFT_KM = 2.6     // how far apart two layers stand at full separation
  const BG = '#070d15'

  let el = $state(); let wrap = $state()
  let width = $state(960)
  const height = $derived(Math.max(360, Math.round(Math.min(width * 0.6, 720))))
  let status = $state('loading the ground…')
  let sep = $state(0)
  let spin = $state(true)
  let showLabels = $state(true)
  let showFx = $state(true)
  let panel = $state(true)
  let off = $state({})          // keys switched off: a layer id, or module:field, or module:marks
  let seeded = false
  let chosen = $state({})       // module -> the frame drawn for it

  const tf = $derived(run.fields?.[terrain])
  const modules = $derived(run.describe?.modules ?? [])
  const pretty = (s) => s.replace(/^quake_|_frac$|_gal$|_cm$|_g$|_p$/g, '').replace(/_/g, ' ')
  // a module's frames that the run holds, with the title its map view gives when it has one
  function framesOf(m) {
    const titles = new Map((m.views ?? []).filter((v) => v.kind === 'map').map((v) => [v.source, v.title]))
    const names = [...titles.keys(), ...(m.frames ?? []).map((f) => f.name)]
    return [...new Set(names)].filter((n) => n !== terrain && run.fields?.[n]).map((n) => ({ name: n, title: titles.get(n) ?? pretty(n) }))
  }
  const groups = $derived(LAYERS.map((L) => ({ ...L, modules: modules.filter((m) => m.layer === L.id).map((m) => ({ name: m.name, kind: m.kind, frames: framesOf(m), roads: (m.tables?.context?.roads ?? []).length })) })))
  const present = $derived(groups.filter((g) => g.modules.length || g.id === 'ground'))
  $effect(() => {
    if (seeded || !modules.length) return
    seeded = true
    const o = {}
    // a disturbance's standing map (the shaking, everywhere at once) starts off so that what it did reads
    // first; one that moves (a fire's front) starts on. The record starts off: it is there to compare with.
    for (const m of modules) if (m.layer === 'across' && !framesOf(m).some((f) => (run.fields[f.name]?.times?.length ?? 0) > 1)) o[`${m.name}:field`] = true
    o.record = true
    off = o
  })
  const isOn = (k) => !off[k]
  const hasArrival = $derived(wanted.some((w) => w.show && ARRIVAL.test(w.name)))
  const toggle = (k) => { off = { ...off, [k]: !off[k] } }
  // a module's first frame is the one drawn, except for a disturbance, where the frame that moves (a fire's front) comes first
  const frameFor = (m) => chosen[m.name] ?? ((modules.find((x) => x.name === m.name)?.layer === 'across' ? m.frames.find((f) => (run.fields[f.name]?.times?.length ?? 0) > 1) : null) ?? m.frames[0])?.name ?? null

  // ---- data
  async function frame(name, i) {
    const r = await fetch(`${base}/fields/${encodeURIComponent(name)}/${String(i).padStart(5, '0')}.f32`)
    if (!r.ok) throw new Error(`${name} ${i}: ${r.status}`)
    return new Float32Array(await r.arrayBuffer())
  }
  const frameIndex = (f, t) => { const ts = f.times ?? []; let k = 0; for (let i = 0; i < ts.length; i++) if (t === null || ts[i] <= t) k = i; return k }

  // the things modules place, from the log: where each is and the state it is in over time
  // Any module may place things: a component with a state (component.state), a lifeline that is
  // marked, fails and is restored, or an event that happens at a cell at a time (a fire's ignition
  // and its spots, a place staked and reached, an entry in the record).
  const MARK_TOPICS = ['component.state', 'lifeline.marked', 'lifeline.failed', 'lifeline.restored', 'fire.ignition', 'fire.spot', 'place.marked', 'place.reached', 'record.event']
  const AT_KIND = { 'fire.ignition': 'ignition', 'fire.spot': 'spot', 'place.marked': 'place', 'place.reached': 'place', 'record.event': 'record' }
  let marks = $state([])        // { src, kind, name, lat, lon, from, changes: [[t, down, status]] }
  $effect(() => {
    if (!base) return
    let live = true
    fetch(`${base}/log.jsonl`).then((r) => (r.ok ? r.text() : '')).then((text) => {
      if (!live) return
      const by = new Map(); let componentBridges = false
      for (const line of text.split('\n')) {
        if (!line || !MARK_TOPICS.some((t) => line.includes(`"${t}"`))) continue
        let e; try { e = JSON.parse(line) } catch { continue }
        const d = e.data ?? {}
        if (AT_KIND[e.topic]) {
          // an event at a cell of the run's grid: where it is comes from the cell
          const g = tf?.grid; if (!g || e.at == null) continue
          const lat = d.lat ?? g.lat_n - (Math.floor(e.at / g.nx) + 0.5) * g.dlat, lon = d.lon ?? g.lon_w + ((e.at % g.nx) + 0.5) * g.dlon
          const kind = AT_KIND[e.topic], name = d.name ?? d.title ?? kind, key = `${e.src}|${kind}|${kind === 'place' ? name : e.seq}`
          let m = by.get(key)
          if (!m) { m = { src: e.src, kind, name, lat, lon, own: true, from: e.t, changes: [] }; by.set(key, m) }
          if (e.topic === 'place.reached') m.changes.push([e.t, true, 'reached'])
          continue
        }
        if (!d.kind || !d.name) continue
        if (e.topic === 'component.state' && d.kind === 'bridge') componentBridges = true
        const key = `${e.topic === 'component.state' ? e.src : 'L'}|${d.kind}|${d.name}`
        let m = by.get(key)
        if (!m) { m = { src: e.src, kind: d.kind, name: d.name, lat: d.lat, lon: d.lon, own: e.topic === 'component.state', from: -Infinity, changes: [] }; by.set(key, m) }
        if (d.lat != null && m.lat == null) { m.lat = d.lat; m.lon = d.lon }
        if (e.topic === 'component.state') m.changes.push([e.t, !!d.down, d.down ? 'closed' : 'open'])
        else if (e.topic === 'lifeline.failed') m.changes.push([e.t, true, d.status ?? 'failed'])
        else if (e.topic === 'lifeline.restored') m.changes.push([e.t, false, d.status ?? 'open'])
      }
      // a bridge the bridges module reports is drawn once, from that module
      marks = [...by.values()].filter((m) => m.lat != null && !(componentBridges && !m.own && m.kind === 'bridge'))
    }).catch(() => { if (live) marks = [] })
    return () => { live = false }
  })
  const stateAt = (m, t) => { let s = [0, false, 'open']; for (const c of m.changes) if (t === null || c[0] <= t) s = c; return s }
  const PINNED = new Set(['bridge', 'hospital', 'hub', 'ignition', 'place'])
  const shown = (m, t) => t === null || m.from <= t

  // ---- the scene
  let three = null   // { renderer, scene, camera, controls, layers: Map, km, elevAt, raf }
  function build(elev) {
    const g = tf.grid
    const lat0 = g.lat_n - (g.ny * g.dlat) / 2, lon0 = g.lon_w + (g.nx * g.dlon) / 2
    const kx = 111.32 * Math.cos((lat0 * Math.PI) / 180), kz = 110.57
    const km = (lon, lat) => [(lon - lon0) * kx, (lat0 - lat) * kz]
    const elevAt = (lon, lat) => {
      const c = Math.min(g.nx - 1, Math.max(0, Math.floor((lon - g.lon_w) / g.dlon))), r = Math.min(g.ny - 1, Math.max(0, Math.floor((g.lat_n - lat) / g.dlat)))
      const v = elev[r * g.nx + c]; return Number.isFinite(v) ? (v / 1000) * ZS : 0
    }
    const renderer = new THREE.WebGLRenderer({ canvas: el, antialias: true, alpha: false })
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2))
    const scene = new THREE.Scene(); scene.background = new THREE.Color(BG)
    scene.fog = new THREE.FogExp2(BG, 0.009)
    const spanX = g.nx * g.dlon * kx, spanZ = g.ny * g.dlat * kz
    const camera = new THREE.PerspectiveCamera(38, width / height, 0.05, 400)
    camera.position.set(-spanX * 0.5, spanZ * 0.85, spanZ * 1.2)
    const controls = new OrbitControls(camera, el)
    controls.enableDamping = true; controls.target.set(0, 0.4, 0); controls.maxPolarAngle = Math.PI * 0.495
    controls.minDistance = 2; controls.maxDistance = 120; controls.autoRotateSpeed = 0.35

    // the ground: a dark surface that hides what is behind a hill, and contour lines on it
    const st = Math.max(1, Math.ceil(g.nx / 340)), nx = Math.floor((g.nx - 1) / st) + 1, ny = Math.floor((g.ny - 1) / st) + 1
    const H = new Float32Array(nx * ny)
    for (let j = 0; j < ny; j++) for (let i = 0; i < nx; i++) { const v = elev[j * st * g.nx + i * st]; H[j * nx + i] = Number.isFinite(v) ? v : 0 }
    const pos = new Float32Array(nx * ny * 3)
    const X = (i) => (g.lon_w + (i * st + 0.5) * g.dlon - lon0) * kx, Z = (j) => (lat0 - (g.lat_n - (j * st + 0.5) * g.dlat)) * kz
    for (let j = 0; j < ny; j++) for (let i = 0; i < nx; i++) { const k = (j * nx + i) * 3; pos[k] = X(i); pos[k + 1] = (H[j * nx + i] / 1000) * ZS; pos[k + 2] = Z(j) }
    const idx = new Uint32Array((nx - 1) * (ny - 1) * 6); let q = 0
    for (let j = 0; j < ny - 1; j++) for (let i = 0; i < nx - 1; i++) { const a = j * nx + i, b = a + 1, c = a + nx, d = c + 1; idx[q++] = a; idx[q++] = c; idx[q++] = b; idx[q++] = b; idx[q++] = c; idx[q++] = d }
    const geo = new THREE.BufferGeometry(); geo.setAttribute('position', new THREE.BufferAttribute(pos, 3)); geo.setIndex(new THREE.BufferAttribute(idx, 1))
    const ground = new THREE.Mesh(geo, new THREE.MeshBasicMaterial({ color: '#08141b', polygonOffset: true, polygonOffsetFactor: 2, polygonOffsetUnits: 2 }))
    // contours by marching squares: every 20 m, each fifth a little brighter
    const seg = [], segBold = []
    const y = (h) => (h / 1000) * ZS + 0.004
    for (let j = 0; j < ny - 1; j++) for (let i = 0; i < nx - 1; i++) {
      const a = H[j * nx + i], b = H[j * nx + i + 1], c = H[(j + 1) * nx + i + 1], d = H[(j + 1) * nx + i]
      const lo = Math.min(a, b, c, d), hi = Math.max(a, b, c, d)
      for (let L = Math.ceil(lo / 20) * 20; L < hi; L += 20) {
        if (L <= 2) continue
        const p = []
        const edge = (v0, v1, x0, z0, x1, z1) => { if ((v0 < L) !== (v1 < L)) { const t = (L - v0) / (v1 - v0); p.push(x0 + (x1 - x0) * t, y(L), z0 + (z1 - z0) * t) } }
        edge(a, b, X(i), Z(j), X(i + 1), Z(j)); edge(b, c, X(i + 1), Z(j), X(i + 1), Z(j + 1)); edge(c, d, X(i + 1), Z(j + 1), X(i), Z(j + 1)); edge(d, a, X(i), Z(j + 1), X(i), Z(j))
        const out = L % 100 === 0 ? segBold : seg
        for (let k = 0; k + 5 < p.length; k += 6) out.push(p[k], p[k + 1], p[k + 2], p[k + 3], p[k + 4], p[k + 5])
      }
    }
    const lines = (arr, opacity) => { const lg = new THREE.BufferGeometry(); lg.setAttribute('position', new THREE.BufferAttribute(new Float32Array(arr), 3)); return new THREE.LineSegments(lg, new THREE.LineBasicMaterial({ color: LAYERS.find((l) => l.id === 'ground').color, transparent: true, opacity, blending: THREE.AdditiveBlending, depthWrite: false })) }
    const layers = new Map()
    for (const L of LAYERS) { const grp = new THREE.Group(); grp.name = L.id; scene.add(grp); layers.set(L.id, grp) }
    const relief = new THREE.Group(); relief.add(lines(seg, 0.28), lines(segBold, 0.6)); layers.get('ground').add(relief)
    scene.add(ground)
    // the window's edge, so the slab reads as a cut piece of the world
    const ex = spanX / 2, ez = spanZ / 2
    const rim = new THREE.LineLoop(new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(-ex, 0, -ez), new THREE.Vector3(ex, 0, -ez), new THREE.Vector3(ex, 0, ez), new THREE.Vector3(-ex, 0, ez)]), new THREE.LineBasicMaterial({ color: '#35b3a4', transparent: true, opacity: 0.35 }))
    scene.add(rim)
    const fx = createEffects(scene, { capacity: 6000 })
    three = { renderer, scene, camera, controls, layers, km, elevAt, fields: new Map(), markObjs: new Map(), spanX, spanZ, visible: true, fx, relief, shake: 0, last: performance.now() }
    const loop = () => {
      three.raf = requestAnimationFrame(loop)
      if (!three.visible) return
      const now = performance.now(), dt = Math.min(0.1, (now - three.last) / 1000); three.last = now
      controls.autoRotate = spin; controls.update()
      // the ground shaking: the whole scene trembles and the relief flares, both dying away in a few seconds
      if (three.shake > 0) {
        three.shake = Math.max(0, three.shake - dt / 2.6)
        const a = three.shake * three.shake * 0.11
        scene.position.set((Math.random() - 0.5) * a, (Math.random() - 0.5) * a * 0.6, (Math.random() - 0.5) * a)
        relief.children[0].material.opacity = 0.28 + 0.6 * three.shake; relief.children[1].material.opacity = 0.6 + 0.4 * three.shake
        if (three.shake === 0) scene.position.set(0, 0, 0)
      }
      fx.update(dt)
      renderer.render(scene, camera); placeLabels()
    }
    loop()
  }

  // a frame as points of light on the ground, in the layer's colour (or the colour of harm)
  const HARMFUL = /damage|destroyed|collapsed|deaths|displaced|release|without|pgd|liquefaction|fire|burn|arrival|flame|fuel_on/
  const HEIGHTS = /height/
  const ARRIVAL = /arrival/
  const FRONT_H = 2      // ground reached within this many hours of the frame's latest hour is drawn as the front
  function drawField(layerId, key, name, data) {
    const t3 = three; const f = run.fields[name]; const g = f.grid
    const old = t3.fields.get(key); if (old) { old.parent?.remove(old); old.geometry.dispose(); old.material.dispose(); t3.fields.delete(key) }
    if (!data) return
    if (HEIGHTS.test(name)) {
      // a frame of heights (buildings) stands up: one line per cell from the ground to its height
      const k = (/^ft/.test(f.unit ?? '') ? 0.3048 : 1) / 1000 * ZS, base = new THREE.Color(LAYERS.find((l) => l.id === layerId)?.color ?? '#ffffff'), P = [], C = []
      for (let r = 0; r < g.ny; r++) for (let c = 0; c < g.nx; c++) {
        const v = data[r * g.nx + c]; if (!(v > 0)) continue
        const lon = g.lon_w + (c + 0.5) * g.dlon, lat = g.lat_n - (r + 0.5) * g.dlat, [x, z] = t3.km(lon, lat), y0 = t3.elevAt(lon, lat)
        P.push(x, y0, z, x, y0 + Math.max(v * k, 0.012), z); C.push(base.r * 0.25, base.g * 0.25, base.b * 0.25, base.r * 0.8, base.g * 0.8, base.b * 0.8)
      }
      const geo = new THREE.BufferGeometry(); geo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(P), 3)); geo.setAttribute('color', new THREE.BufferAttribute(new Float32Array(C), 3))
      const obj = new THREE.LineSegments(geo, new THREE.LineBasicMaterial({ vertexColors: true, transparent: true, opacity: 0.9, blending: THREE.AdditiveBlending, depthWrite: false }))
      t3.layers.get(layerId).add(obj); t3.fields.set(key, obj); return
    }
    if (ARRIVAL.test(name) && /^h/.test(f.unit ?? '')) {
      // a frame of arrival hours (a fire's): what matters is which ground has burned and where the
      // front is now, so the cells are drawn by how long ago the thing arrived. The latest hour in
      // the frame is the present: cells reached in the last hour or two are the front, bright and
      // yellow-white; behind it the ground cools through orange to a steady dark red. Ground not
      // reached has no point at all, so burned and unburned cannot be confused.
      let now = 0; for (let k = 0; k < data.length; k++) if (data[k] > now) now = data[k]
      const P = [], C = [], PF = [], CF = []
      for (let r = 0; r < g.ny; r++) for (let c = 0; c < g.nx; c++) {
        const v = data[r * g.nx + c]; if (!(v > 0)) continue
        const age = now - v, lon = g.lon_w + (c + 0.5) * g.dlon, lat = g.lat_n - (r + 0.5) * g.dlat
        const [x, z] = t3.km(lon, lat), y = t3.elevAt(lon, lat) + 0.03
        if (age <= FRONT_H) { PF.push(x, y + 0.01, z); const k = 1 - 0.4 * (age / FRONT_H); CF.push(1.0 * k, 0.72 * k, 0.22 * k) }
        else { const k = Math.exp(-(age - FRONT_H) / 5); P.push(x, y, z); C.push(0.34 + 0.5 * k, 0.05 + 0.2 * k, 0.04) }
      }
      const cell = g.dlat * 110.57, grp = new THREE.Group()
      const mk = (P, C, size, blending) => { const geo = new THREE.BufferGeometry(); geo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(P), 3)); geo.setAttribute('color', new THREE.BufferAttribute(new Float32Array(C), 3)); return new THREE.Points(geo, new THREE.PointsMaterial({ size, vertexColors: true, transparent: true, opacity: 0.85, blending, depthWrite: false, sizeAttenuation: true })) }
      // the burned ground is laid opaque enough to cover the contours under it; the front glows over it
      grp.add(mk(P, C, cell * 4.2, THREE.NormalBlending)); grp.add(mk(PF, CF, cell * 2.6, THREE.AdditiveBlending))
      grp.geometry = { dispose() { for (const o of grp.children) o.geometry.dispose() } }; grp.material = { dispose() { for (const o of grp.children) o.material.dispose() } }
      t3.layers.get(layerId).add(grp); t3.fields.set(key, grp); return
    }
    const st = Math.max(1, Math.round(g.nx / 700))
    const max = Math.max(f.max ?? 1, 1e-9), useLog = max > 50
    // a frame that is positive everywhere (shaking, say) is drawn by where it stands in its own range
    const floor = f.min > 0 ? f.min : 0
    // a yes-or-no frame of a service (electricity on) is drawn by where the service is missing
    const service = f.unit === '1' && (f.min ?? 0) === 0 && max === 1 && /_up$|service/.test(name)
    const harmful = service || HARMFUL.test(name)
    const base = new THREE.Color(LAYERS.find((l) => l.id === layerId)?.color ?? '#ffffff'), harm = new THREE.Color(HARM), col = new THREE.Color()
    const P = [], C = []
    for (let r = 0; r < g.ny; r += st) for (let c = 0; c < g.nx; c += st) {
      let v = 0
      for (let dr = 0; dr < st && r + dr < g.ny; dr++) for (let dc = 0; dc < st && c + dc < g.nx; dc++) { const x = data[(r + dr) * g.nx + c + dc]; if (x > v) v = x }
      if (service) v = v > 0.5 ? 0 : 1
      if (!(v > 0)) continue
      const t = Math.min(1, useLog ? Math.log1p(v) / Math.log1p(max) : (v - floor) / Math.max(max - floor, 1e-9)); if (t < 0.02) continue
      const lon = g.lon_w + (c + st / 2) * g.dlon, lat = g.lat_n - (r + st / 2) * g.dlat
      const [x, z] = t3.km(lon, lat); P.push(x, t3.elevAt(lon, lat) + 0.03, z)
      // harm runs from the layer's own colour, where it is slight, to red, where it is worst
      if (harmful) col.copy(base).lerp(harm, service ? 1 : Math.min(1, t * 1.4)); else col.copy(base)
      const b = service ? 0.5 : harmful ? 0.16 + 0.5 * t : 0.07 + 0.6 * t * t; C.push(col.r * b, col.g * b, col.b * b)
    }
    const geo = new THREE.BufferGeometry(); geo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(P), 3)); geo.setAttribute('color', new THREE.BufferAttribute(new Float32Array(C), 3))
    const size = g.dlat * 110.57 * st * 0.85
    const pts = new THREE.Points(geo, new THREE.PointsMaterial({ size, vertexColors: true, transparent: true, opacity: 0.9, blending: THREE.AdditiveBlending, depthWrite: false, sizeAttenuation: true }))
    t3.layers.get(layerId).add(pts); t3.fields.set(key, pts)
  }

  // which frames are wanted now, and at which index; a frame is fetched once and kept
  const cache = new Map()
  let ready = $state(false)
  const wanted = $derived.by(() => {
    const out = []
    for (const G of groups) for (const m of G.modules) {
      const n = frameFor(m); if (!n) continue
      // a frame is drawn only once it has begun: the shaking map is not there before the earthquake
      const begun = cursor === null || (run.fields[n].times?.[0] ?? -Infinity) <= cursor
      const show = begun && isOn(G.id) && isOn(`${m.name}:field`)
      out.push({ layer: G.id, key: m.name, name: n, i: frameIndex(run.fields[n], cursor), show })
    }
    return out
  })
  let drawn = new Map()   // key -> "name#i" currently on screen
  $effect(() => {
    if (!ready) return
    for (const w of wanted) {
      const id = w.show ? `${w.name}#${w.i}` : ''
      if (drawn.get(w.key) === id) continue
      drawn.set(w.key, id)
      if (!w.show) { drawField(w.layer, w.key, w.name, null); continue }
      const ck = `${w.name}#${w.i}`
      const p = cache.get(ck) ?? frame(w.name, w.i); cache.set(ck, p)
      Promise.resolve(p).then((data) => { if (drawn.get(w.key) === id) drawField(w.layer, w.key, w.name, data) }).catch(() => {})
    }
  })

  // the marks, rebuilt when the cursor crosses a change
  let labels = $state([])    // { text, x, y, color, show }
  let labelAnchors = []
  const markSig = $derived(marks.map((m) => (!shown(m, cursor) ? 2 : stateAt(m, cursor)[1] ? 1 : 0)).join('') + '|' + JSON.stringify(off) + '|' + sep)
  $effect(() => {
    void markSig
    if (!ready) return
    const t3 = three
    for (const o of t3.markObjs.values()) { o.parent?.remove(o); o.traverse((c) => { c.geometry?.dispose(); c.material?.dispose() }) }
    t3.markObjs.clear(); labelAnchors = []
    const layerOfModule = new Map(modules.map((m) => [m.name, m.layer]))
    const bySrc = new Map()
    for (const m of marks) { const k = m.src; if (!bySrc.has(k)) bySrc.set(k, []); bySrc.get(k).push(m) }
    for (const [src, list] of bySrc) {
      const layerId = layerOfModule.get(src); const grp = t3.layers.get(layerId); if (!grp) continue
      if (!isOn(layerId) || !isOn(`${src}:marks`)) continue
      const base = new THREE.Color(LAYERS.find((l) => l.id === layerId).color), harm = new THREE.Color(HARM)
      const P = [], C = [], PP = [], PC = [], LP = [], LC = [], GP = [], GC = []
      // a module may carry lines for what it places (its describe's `geometry` table): a road
      // segment's outline, a bridge's span. Then the thing is drawn as its lines, in its state.
      const geom = modules.find((x) => x.name === src)?.tables?.geometry
      const segLines = new Map((geom?.segments ?? []).map((g) => [`segment ${g.id}`, g.lines]))
      const spanLines = new Map((geom?.bridges ?? []).map((g) => [g.name, g.lines]))
      const white = new THREE.Color('#ffffff')
      const draw = (lines, c, b, lift) => { for (const l of lines) for (let k = 0; k + 1 < l.length; k++) { for (const q of [l[k], l[k + 1]]) { const [x, z] = t3.km(q[0], q[1]); GP.push(x, t3.elevAt(q[0], q[1]) + lift, z); GC.push(c.r * b, c.g * b, c.b * b) } } }
      for (const m of list) {
        if (!shown(m, cursor)) continue
        const [, down, status] = stateAt(m, cursor)
        const [x, z] = t3.km(m.lon, m.lat), y0 = t3.elevAt(m.lon, m.lat)
        const c = down ? harm : base, b = down ? 0.9 : 0.4
        if (segLines.has(m.name)) { draw(segLines.get(m.name), c, down ? 0.75 : 0.5, 0.03); continue }
        if (spanLines.get(m.name)?.length) draw(spanLines.get(m.name), down ? harm : white, down ? 1 : 0.95, 0.14)
        if (PINNED.has(m.kind)) {
          const top = y0 + (m.kind === 'hub' ? 1.5 : 0.9)
          LP.push(x, y0, z, x, top, z); LC.push(c.r * 0.5, c.g * 0.5, c.b * 0.5, c.r, c.g, c.b)
          PP.push(x, top, z); PC.push(c.r, c.g, c.b)
          labelAnchors.push({ grp, v: new THREE.Vector3(x, top, z), text: m.name.replace(/ Hospital$| Medical Center$| Bridge$/, ''), color: down ? HARM : LAYERS.find((l) => l.id === layerId).color, status: down ? status : '' })
        } else { P.push(x, y0 + 0.05, z); C.push(c.r * b, c.g * b, c.b * b) }
      }
      const g3 = new THREE.Group()
      const mk = (arr, col, size) => { const geo = new THREE.BufferGeometry(); geo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(arr), 3)); geo.setAttribute('color', new THREE.BufferAttribute(new Float32Array(col), 3)); return new THREE.Points(geo, new THREE.PointsMaterial({ size, vertexColors: true, transparent: true, blending: THREE.AdditiveBlending, depthWrite: false })) }
      if (P.length) g3.add(mk(P, C, 0.11))
      if (GP.length) {
        const gg = new THREE.BufferGeometry(); gg.setAttribute('position', new THREE.BufferAttribute(new Float32Array(GP), 3)); gg.setAttribute('color', new THREE.BufferAttribute(new Float32Array(GC), 3))
        g3.add(new THREE.LineSegments(gg, new THREE.LineBasicMaterial({ vertexColors: true, transparent: true, opacity: 0.95, blending: THREE.AdditiveBlending, depthWrite: false })))
      }
      if (PP.length) {
        g3.add(mk(PP, PC, 0.3))
        const lg = new THREE.BufferGeometry(); lg.setAttribute('position', new THREE.BufferAttribute(new Float32Array(LP), 3)); lg.setAttribute('color', new THREE.BufferAttribute(new Float32Array(LC), 3))
        g3.add(new THREE.LineSegments(lg, new THREE.LineBasicMaterial({ vertexColors: true, transparent: true, opacity: 0.95, blending: THREE.AdditiveBlending, depthWrite: false })))
      }
      grp.add(g3); t3.markObjs.set(src, g3)
    }
  })
  // the area's roads from OpenStreetMap, when a module carries them: faint lines, the heavier the road the brighter
  const ROAD_W = { motorway: 0.34, trunk: 0.28, primary: 0.2, secondary: 0.12, tertiary: 0.08 }
  $effect(() => {
    if (!ready) return
    const t3 = three
    for (const m of modules) {
      const roads = m.tables?.context?.roads; if (!roads?.length) continue
      const key = `${m.name}:roads`
      let obj = t3.fields.get(key)
      if (!obj) {
        // roads a disturbance carries (a fire's) are drawn in the colour of the flows layer, so they do not read as fire
        const col = new THREE.Color(LAYERS.find((l) => l.id === (m.layer === 'across' ? 'flows' : m.layer))?.color ?? '#ffffff'), P = [], C = []
        for (const r of roads) { const b = ROAD_W[r.class] ?? 0.1; for (let k = 0; k + 1 < r.pts.length; k++) for (const q of [r.pts[k], r.pts[k + 1]]) { const [x, z] = t3.km(q[0], q[1]); P.push(x, t3.elevAt(q[0], q[1]) + 0.02, z); C.push(col.r * b, col.g * b, col.b * b) } }
        const geo = new THREE.BufferGeometry(); geo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(P), 3)); geo.setAttribute('color', new THREE.BufferAttribute(new Float32Array(C), 3))
        obj = new THREE.LineSegments(geo, new THREE.LineBasicMaterial({ vertexColors: true, transparent: true, opacity: 0.9, blending: THREE.AdditiveBlending, depthWrite: false }))
        t3.layers.get(m.layer)?.add(obj); t3.fields.set(key, obj)
      }
      obj.visible = isOn(key)
    }
  })
  // What happened between the last moment shown and this one is marked where it happened: a
  // burst for a failure, a gathering for a return, and the ground's own tremor at the event. Only
  // when time moves forward by a step, so dragging the bar far does not set everything off at once.
  let lastCursor = null
  const FX_OF = { tank: 'failSmall', bridge: 'collapse', hospital: 'dark', hub: 'fail', route: null, spot: 'embers', ignition: 'embers', place: 'mark' }
  $effect(() => {
    const t = cursor
    if (!ready || t === null) return
    const prev = lastCursor; lastCursor = t
    if (prev === null || t <= prev || !showFx || !three.visible) return
    const t3 = three, layerOfModule = new Map(modules.map((m) => [m.name, m.layer]))
    if (event !== null && prev < event && event <= t) {
      t3.shake = 1
      t3.fx.emit('dust', { x: 0, y: 0, z: 0 }, { extent: [t3.spanX, t3.spanZ], heightAt: (x, z) => 0.05 })
    }
    if (t - prev > 45 * 86400) return
    const todo = []
    for (const m of marks) {
      const layerId = layerOfModule.get(m.src); if (!layerId || !isOn(layerId) || !isOn(`${m.src}:marks`)) continue
      if (m.from > prev && m.from <= t && FX_OF[m.kind]) todo.push([m, FX_OF[m.kind]])
      for (const c of m.changes) if (c[0] > prev && c[0] <= t) { const fx = c[1] ? FX_OF[m.kind] : m.kind === 'route' || m.kind === 'tank' ? null : 'restore'; if (fx) todo.push([m, fx]) }
    }
    // many at one instant (every tank at the rupture): each is thinned so the pool is not swamped
    const density = todo.length > 160 ? 160 / todo.length : 1
    for (const [m, fx] of todo) {
      const [x, z] = t3.km(m.lon, m.lat)
      t3.fx.emit(fx, { x, y: t3.elevAt(m.lon, m.lat) + (PINNED.has(m.kind) ? 0.5 : 0.08) + t3.layers.get(layerOfModule.get(m.src)).position.y, z }, { density: fx === 'failSmall' ? Math.max(0.35, density) : 1 })
    }
  })
  function placeLabels() {
    if (!three || !showLabels) { if (labels.length) labels = []; return }
    const w = width, h = height, v = new THREE.Vector3(), out = []
    // what has failed is named first; a name that would sit on another is left out
    for (const a of [...labelAnchors].sort((p, q) => (q.status ? 1 : 0) - (p.status ? 1 : 0))) {
      v.copy(a.v); v.y += a.grp.position.y; v.project(three.camera)
      if (v.z > 1 || Math.abs(v.x) > 1 || Math.abs(v.y) > 1) continue
      const x = ((v.x + 1) / 2) * w, y = ((1 - v.y) / 2) * h, len = (a.text.length + (a.status ? a.status.length + 3 : 0)) * 7.2 + 10
      if (out.some((o) => Math.abs(o.y - y) < 12 && x < o.x + o.len && o.x < x + len)) continue
      out.push({ text: a.text, status: a.status, color: a.color, x, y, len })
    }
    labels = out
  }

  // separation: each layer that has something lifts by its place among those present
  $effect(() => {
    if (!ready) return
    const ids = present.map((g) => g.id)
    for (const [id, grp] of three.layers) { const k = ids.indexOf(id); grp.position.y = k > 0 ? k * sep * LIFT_KM : 0; grp.visible = isOn(id) }
  })

  $effect(() => {
    if (!el || !tf || three) return
    let live = true
    frame(terrain, 0).then((elev) => { if (!live) return; build(elev); status = ''; ready = true }).catch((e) => { status = `the ground could not be read: ${e.message}` })
    return () => { live = false }
  })
  $effect(() => { if (ready && three && width) { three.renderer.setSize(width, height, false); three.camera.aspect = width / height; three.camera.updateProjectionMatrix(); three.fx.setViewport(height * three.renderer.getPixelRatio(), three.camera.fov) } })
  $effect(() => {
    if (!wrap) return
    const ro = new ResizeObserver((es) => { width = Math.round(es[0].contentRect.width) }); ro.observe(wrap)
    const io = new IntersectionObserver((es) => { if (three) three.visible = es[0].isIntersecting }); io.observe(wrap)
    if (window.matchMedia?.('(prefers-reduced-motion: reduce)').matches) spin = false
    if (wrap.clientWidth < 640) panel = false   // on a phone the list starts folded so the place is seen first
    return () => { ro.disconnect(); io.disconnect(); if (three) { cancelAnimationFrame(three.raf); three.fx.dispose(); three.controls.dispose(); three.renderer.dispose(); three = null } }
  })

  const when = $derived(cursor === null ? '' : isoTime(cursor))
  const phase = $derived(cursor === null || event === null ? '' : cursor < event ? 'before the event' : `day ${Math.floor((cursor - event) / 86400)} after`)
</script>

<section class="mt-6">
  <div bind:this={wrap} class="relative w-full overflow-hidden rounded-[3px]" style="background:{BG}; height:{height}px">
    <canvas bind:this={el} class="block h-full w-full" style="touch-action:none"></canvas>
    {#if status}<p class="absolute inset-0 flex items-center justify-center text-sm" style="color:#7f9aa8">{status}</p>{/if}

    {#each labels as l}
      <span class="pointer-events-none absolute whitespace-nowrap" style="left:{l.x + 6}px; top:{l.y - 6}px; color:{l.color}; font-size:9.5px; letter-spacing:0.14em; text-transform:uppercase; text-shadow:0 0 4px {BG}">{l.text}{#if l.status}<span style="opacity:0.8"> · {l.status}</span>{/if}</span>
    {/each}

    <div class="absolute left-3 top-3 max-h-[calc(100%-4.5rem)] overflow-y-auto rounded-[3px] text-[12px]" style="background:#0b1620d9; color:#c8d6de; border:1px solid #1d3340; min-width:{panel ? 15 : 0}rem">
      <button class="flex w-full items-center justify-between gap-3 px-3 py-2 text-left" onclick={() => (panel = !panel)} aria-expanded={panel}>
        <span style="letter-spacing:0.14em; text-transform:uppercase; font-size:10.5px">Layers</span><span style="opacity:0.6">{panel ? '–' : '+'}</span>
      </button>
      {#if panel}
        <div class="px-3 pb-3">
          {#each groups as G (G.id)}
            {@const empty = G.modules.length === 0 && G.id !== 'ground'}
            <div class="mt-1.5" style="opacity:{empty ? 0.38 : 1}">
              <label class="flex items-center gap-2" style="cursor:{empty ? 'default' : 'pointer'}">
                <input type="checkbox" disabled={empty} checked={!empty && isOn(G.id)} onchange={() => toggle(G.id)} style="accent-color:{G.color}" />
                <span class="inline-block h-2 w-2 rounded-full" style="background:{G.color}"></span>
                {#if G.n}<span class="mono" style="opacity:0.5; font-size:10px">{G.n}</span>{/if}<span>{G.name}</span>{#if empty}<span style="opacity:0.8">· no module</span>{/if}
              </label>
              {#if G.id === 'ground'}<p class="ml-6" style="opacity:0.6">the relief, from the run's elevation frame</p>{/if}
              {#if isOn(G.id)}
                {#each G.modules as m (m.name)}
                  <div class="ml-6 mt-1">
                    <div class="mono" style="opacity:0.75">{m.name} <span style="opacity:0.6">{m.kind}</span></div>
                    {#if m.frames.length}
                      <label class="flex items-center gap-2"><input type="checkbox" checked={isOn(`${m.name}:field`)} onchange={() => toggle(`${m.name}:field`)} style="accent-color:{G.color}" />
                        {#if m.frames.length > 1}
                          <select class="max-w-[11rem] bg-transparent" style="color:inherit; border-bottom:1px solid #2a4656" value={frameFor(m)} onchange={(e) => (chosen = { ...chosen, [m.name]: e.currentTarget.value })}>
                            {#each m.frames as f (f.name)}<option value={f.name} style="color:#111">{f.title}</option>{/each}
                          </select>
                        {:else}<span>{m.frames[0].title}</span>{/if}
                      </label>
                    {/if}
                    {#if m.roads}
                      <label class="flex items-center gap-2"><input type="checkbox" checked={isOn(`${m.name}:roads`)} onchange={() => toggle(`${m.name}:roads`)} style="accent-color:{G.color}" /><span>the area's roads (OpenStreetMap)</span></label>
                    {/if}
                    {#if marks.some((k) => k.src === m.name)}
                      <label class="flex items-center gap-2"><input type="checkbox" checked={isOn(`${m.name}:marks`)} onchange={() => toggle(`${m.name}:marks`)} style="accent-color:{G.color}" /><span>the things it places ({marks.filter((k) => k.src === m.name).length})</span></label>
                    {/if}
                  </div>
                {/each}
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="absolute bottom-3 left-3 right-3 flex flex-wrap items-end justify-between gap-3 text-[12px]" style="color:#c8d6de">
      <div class="mono" style="text-shadow:0 0 4px {BG}">{when}{#if phase} · <span style="color:{cursor >= event ? HARM : '#c8d6de'}">{phase}</span>{/if}</div>
      <div class="flex flex-wrap items-center gap-3 rounded-[3px] px-3 py-1.5" style="background:#0b1620d9; border:1px solid #1d3340">
        <label class="flex items-center gap-2">layers apart <input type="range" min="0" max="1" step="0.01" bind:value={sep} style="accent-color:#35b3a4; width:7rem" /></label>
        <label class="flex items-center gap-1.5"><input type="checkbox" bind:checked={spin} style="accent-color:#35b3a4" />turn</label>
        <label class="flex items-center gap-1.5"><input type="checkbox" bind:checked={showLabels} style="accent-color:#35b3a4" />names</label>
        <label class="flex items-center gap-1.5"><input type="checkbox" bind:checked={showFx} style="accent-color:#35b3a4" />effects</label>
      </div>
    </div>
  </div>
  <p class="mt-2 max-w-3xl text-xs muted">The modelled area with the vertical stretched {ZS} times. Contour lines are the run's elevation frame, every 20 metres. Each module's map is drawn as points of light in its layer's colour, and red marks what has failed, closed or lost power at the moment shown.{#if hasArrival} Ground a fire has reached is filled dark red, cooling from orange; the bright yellow edge is ground reached in the last two hours of the model's clock, and ground with no fill has not burned.{/if} Drag to turn, scroll to zoom. A layer with no module in this run is listed and empty.</p>
</section>
