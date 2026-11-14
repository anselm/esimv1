// A small effects engine for the 3D views: pooled particles drawn as one additive point cloud,
// spawned in bursts at world coordinates from named recipes. It exists to mark a moment on the
// map (a tank failing, a bridge closing, a hospital losing power, power coming back) where the
// eye already is, without a panel or a legend. One draw call, a fixed pool, nothing allocated per
// frame; when nothing is alive the update returns at once.
//
//   const fx = createEffects(scene, { capacity: 6000 })
//   fx.emit('fail', { x, y, z }, { color: '#ff4d3d', scale: 1 })   // a recipe by name, or a spec, or a list of specs
//   fx.update(dt)                                                    // seconds since the last frame
//
// A spec: { count, shape: 'up' | 'sphere' | 'ring' | 'inward' | 'area', speed: [lo, hi], life: [lo, hi] (seconds),
// size: [start, end] (world units), colors: [start, end], gravity, drag, swirl (radians a second about the
// vertical through the origin), radius (where 'inward' and 'area' particles start), spread (0 to 1, the cone of 'up') }.
// Units are the scene's: these recipes assume about a kilometre to the unit.
import * as THREE from 'three'

export const RECIPES = {
  // something breaks and lets go: a hot burst upward that cools to red and falls back
  fail: [{ count: 16, shape: 'up', spread: 0.75, speed: [0.5, 1.5], life: [0.8, 1.7], size: [0.34, 0.03], colors: ['#ffe2a0', '#ff3b2b'], gravity: -1.1, drag: 0.8 }],
  // a small one, for many at once
  failSmall: [{ count: 6, shape: 'up', spread: 0.9, speed: [0.3, 0.9], life: [0.6, 1.3], size: [0.22, 0.02], colors: ['#ffd27a', '#ff3b2b'], gravity: -0.9, drag: 0.9 }],
  // a span goes down: a ring runs out along the ground and dust drops
  collapse: [
    { count: 44, shape: 'ring', speed: [0.9, 1.5], life: [0.7, 1.2], size: [0.3, 0.04], colors: ['#ffffff', '#ff4d3d'], drag: 1.6 },
    { count: 26, shape: 'sphere', speed: [0.2, 0.7], life: [0.9, 1.6], size: [0.2, 0.02], colors: ['#ffcf9a', '#7a2a22'], gravity: -1.4, drag: 0.4 },
  ],
  // the lights go out: light is drawn inward and down, and dies
  dark: [{ count: 34, shape: 'inward', radius: 1.0, speed: [0.8, 1.3], life: [0.7, 1.1], size: [0.22, 0.02], colors: ['#fff0b8', '#ff3b2b'], swirl: 2.6, gravity: -0.5 }],
  // something comes back: light gathers inward, turning, and rises
  restore: [
    { count: 40, shape: 'inward', radius: 1.3, speed: [0.9, 1.5], life: [0.8, 1.3], size: [0.05, 0.26], colors: ['#35b3a4', '#ffffff'], swirl: 4.2 },
    { count: 14, shape: 'up', spread: 0.15, speed: [0.6, 1.2], life: [0.9, 1.5], size: [0.2, 0.02], colors: ['#ffffff', '#7fdcff'], drag: 0.5, delay: 0.7 },
  ],
  // a place is marked: a quiet ring
  mark: [{ count: 28, shape: 'ring', speed: [0.5, 0.8], life: [0.8, 1.2], size: [0.2, 0.03], colors: ['#ffffff', '#35b3a4'], drag: 1.2 }],
  // fire: embers rising
  embers: [{ count: 22, shape: 'up', spread: 0.5, speed: [0.3, 1.0], life: [1.2, 2.6], size: [0.2, 0.02], colors: ['#ffd27a', '#ff3b2b'], gravity: 0.25, drag: 0.5, swirl: 0.8 }],
  // the ground shakes: dust lifts everywhere at once and hangs
  dust: [{ count: 520, shape: 'area', speed: [0.05, 0.35], life: [1.6, 3.4], size: [0.14, 0.02], colors: ['#c8d6de', '#35b3a4'], gravity: -0.05, drag: 0.6 }],
}

export function createEffects(parent, { capacity = 6000 } = {}) {
  const N = capacity
  const pos = new Float32Array(N * 3), vel = new Float32Array(N * 3), org = new Float32Array(N * 2)
  const c0 = new Float32Array(N * 3), c1 = new Float32Array(N * 3), col = new Float32Array(N * 3)
  const age = new Float32Array(N), life = new Float32Array(N), s0 = new Float32Array(N), s1 = new Float32Array(N), size = new Float32Array(N)
  const grav = new Float32Array(N), drag = new Float32Array(N), swirl = new Float32Array(N)
  let next = 0, alive = 0
  const geo = new THREE.BufferGeometry()
  geo.setAttribute('position', new THREE.BufferAttribute(pos, 3).setUsage(THREE.DynamicDrawUsage))
  geo.setAttribute('aColor', new THREE.BufferAttribute(col, 3).setUsage(THREE.DynamicDrawUsage))
  geo.setAttribute('aSize', new THREE.BufferAttribute(size, 1).setUsage(THREE.DynamicDrawUsage))
  geo.boundingSphere = new THREE.Sphere(new THREE.Vector3(), 1e6)   // never culled: the pool is everywhere
  const mat = new THREE.ShaderMaterial({
    transparent: true, depthWrite: false, blending: THREE.AdditiveBlending,
    uniforms: { uScale: { value: 400 } },
    vertexShader: 'attribute vec3 aColor; attribute float aSize; uniform float uScale; varying vec3 vColor; void main() { vColor = aColor; vec4 mv = modelViewMatrix * vec4(position, 1.0); gl_PointSize = aSize * uScale / max(0.001, -mv.z); gl_Position = projectionMatrix * mv; }',
    fragmentShader: 'varying vec3 vColor; void main() { vec2 d = gl_PointCoord - 0.5; float r = dot(d, d) * 4.0; if (r > 1.0) discard; float a = (1.0 - r); gl_FragColor = vec4(vColor * a * a, 1.0); }',
  })
  const points = new THREE.Points(geo, mat)
  points.frustumCulled = false; points.renderOrder = 10
  parent.add(points)
  const rnd = (r) => r[0] + Math.random() * (r[1] - r[0])
  const A = new THREE.Color(), B = new THREE.Color()

  function one(spec, o, opt) {
    const k = opt.scale ?? 1, n = Math.max(1, Math.round((spec.count ?? 12) * (opt.density ?? 1)))
    A.set(opt.colors?.[0] ?? spec.colors?.[0] ?? '#ffffff'); B.set(opt.colors?.[1] ?? spec.colors?.[1] ?? '#ffffff')
    for (let j = 0; j < n; j++) {
      const i = next; next = (next + 1) % N
      if (life[i] <= 0 || age[i] >= life[i]) alive++
      const th = Math.random() * Math.PI * 2, sp = rnd(spec.speed ?? [0.5, 1]) * k
      let px = o.x, py = o.y, pz = o.z, vx = 0, vy = 0, vz = 0
      if (spec.shape === 'ring') { vx = Math.cos(th) * sp; vz = Math.sin(th) * sp; vy = 0.05 * sp }
      else if (spec.shape === 'sphere') { const u = Math.random() * 2 - 1, s = Math.sqrt(1 - u * u); vx = s * Math.cos(th) * sp; vy = Math.abs(u) * sp; vz = s * Math.sin(th) * sp }
      else if (spec.shape === 'inward') { const R = (spec.radius ?? 1) * k * (0.6 + Math.random() * 0.4); px += Math.cos(th) * R; pz += Math.sin(th) * R; py += Math.random() * 0.5 * k; vx = -Math.cos(th) * sp; vz = -Math.sin(th) * sp; vy = 0.15 * sp }
      else if (spec.shape === 'area') { px += (Math.random() - 0.5) * (opt.extent?.[0] ?? 10); pz += (Math.random() - 0.5) * (opt.extent?.[1] ?? 10); py = opt.heightAt ? opt.heightAt(px, pz) : o.y; vy = sp; vx = (Math.random() - 0.5) * sp; vz = (Math.random() - 0.5) * sp }
      else { const w = (spec.spread ?? 0.5) * Math.random(); vx = Math.cos(th) * w * sp; vz = Math.sin(th) * w * sp; vy = sp }
      pos[i * 3] = px; pos[i * 3 + 1] = py; pos[i * 3 + 2] = pz
      vel[i * 3] = vx; vel[i * 3 + 1] = vy; vel[i * 3 + 2] = vz
      org[i * 2] = o.x; org[i * 2 + 1] = o.z
      c0[i * 3] = A.r; c0[i * 3 + 1] = A.g; c0[i * 3 + 2] = A.b; c1[i * 3] = B.r; c1[i * 3 + 1] = B.g; c1[i * 3 + 2] = B.b
      age[i] = -(spec.delay ?? 0) - Math.random() * 0.12; life[i] = rnd(spec.life ?? [1, 1.5])
      s0[i] = (spec.size?.[0] ?? 0.2) * k; s1[i] = (spec.size?.[1] ?? 0.02) * k
      grav[i] = (spec.gravity ?? 0) * k; drag[i] = spec.drag ?? 0; swirl[i] = spec.swirl ?? 0
    }
  }
  return {
    object: points,
    get alive() { return alive },
    /// `what` is a recipe's name, a spec, or a list of specs; `at` is { x, y, z } in the parent's coordinates.
    emit(what, at, opt = {}) { const specs = typeof what === 'string' ? RECIPES[what] ?? [] : Array.isArray(what) ? what : [what]; for (const s of specs) one(s, at, opt) },
    setViewport(heightPx, fovDeg) { mat.uniforms.uScale.value = (0.55 * heightPx) / (2 * Math.tan((fovDeg * Math.PI) / 360)) },
    update(dt) {
      if (alive <= 0) return
      const h = Math.min(dt, 0.05); let n = 0
      for (let i = 0; i < N; i++) {
        if (life[i] <= 0) continue
        age[i] += h
        if (age[i] >= life[i]) { life[i] = 0; size[i] = 0; continue }
        n++
        if (age[i] < 0) { size[i] = 0; continue }   // waiting for its delay
        const k = i * 3, d = Math.max(0, 1 - drag[i] * h)
        vel[k] *= d; vel[k + 1] = vel[k + 1] * d + grav[i] * h; vel[k + 2] *= d
        pos[k] += vel[k] * h; pos[k + 1] += vel[k + 1] * h; pos[k + 2] += vel[k + 2] * h
        if (swirl[i]) { const a = swirl[i] * h, cs = Math.cos(a), sn = Math.sin(a), dx = pos[k] - org[i * 2], dz = pos[k + 2] - org[i * 2 + 1]; pos[k] = org[i * 2] + dx * cs - dz * sn; pos[k + 2] = org[i * 2 + 1] + dx * sn + dz * cs }
        const t = age[i] / life[i], fade = t < 0.12 ? t / 0.12 : 1 - (t - 0.12) / 0.88
        size[i] = s0[i] + (s1[i] - s0[i]) * t
        col[k] = (c0[k] + (c1[k] - c0[k]) * t) * fade; col[k + 1] = (c0[k + 1] + (c1[k + 1] - c0[k + 1]) * t) * fade; col[k + 2] = (c0[k + 2] + (c1[k + 2] - c0[k + 2]) * t) * fade
      }
      alive = n
      geo.attributes.position.needsUpdate = true; geo.attributes.aColor.needsUpdate = true; geo.attributes.aSize.needsUpdate = true
    },
    dispose() { parent.remove(points); geo.dispose(); mat.dispose() },
  }
}
