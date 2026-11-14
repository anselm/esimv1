<script>
  // The Earth, turning slowly, half off the right edge of the front page. Taken from the
  // orbital-sim site's home page (orbital-sim.exe.xyz, website/client/src/lib/Globe.svelte), with
  // one change: the dots are the places this site's scenarios model, read from their manifests,
  // where the original had placeholder cities. Decoration behind the page; it takes no input.
  import * as THREE from 'three'

  let { places = [] } = $props()   // [{ lat, lon }]
  let container = $state(null)
  let api = null

  const xyz = (lat, lon, r = 1) => {
    const phi = (90 - lat) * (Math.PI / 180), theta = (lon + 180) * (Math.PI / 180)
    return new THREE.Vector3(-r * Math.sin(phi) * Math.cos(theta), r * Math.cos(phi), r * Math.sin(phi) * Math.sin(theta))
  }

  $effect(() => {
    if (!container) return
    const w = container.clientWidth, h = container.clientHeight
    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true })
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2)); renderer.setSize(w, h)
    Object.assign(renderer.domElement.style, { display: 'block', width: '100%', height: '100%' })
    container.appendChild(renderer.domElement)
    const scene = new THREE.Scene()
    // the camera sits back so the sphere fills about three quarters of the frame, looking down a little
    const camera = new THREE.PerspectiveCamera(45, w / h, 0.1, 100)
    camera.position.set(0, 0.3, 3.2); camera.lookAt(0, 0, 0)
    // low ambient light and one sun from the upper left, so the terminator crosses the visible half
    scene.add(new THREE.AmbientLight(0xffffff, 0.12))
    const sun = new THREE.DirectionalLight(0xffffff, 1.8); sun.position.set(-5, 2, 2); scene.add(sun)
    const tilt = new THREE.Group(); tilt.rotation.z = 23.4 * (Math.PI / 180); scene.add(tilt)   // the Earth's axial tilt
    const tex = new THREE.TextureLoader().load('/assets/earth.jpg'); tex.colorSpace = THREE.SRGBColorSpace
    const globe = new THREE.Mesh(new THREE.SphereGeometry(1, 64, 64), new THREE.MeshStandardMaterial({ map: tex, roughness: 0.9, metalness: 0 }))
    globe.rotation.y = -Math.PI / 2 + (105 * Math.PI) / 180   // start with North America facing forward, where most scenarios are
    tilt.add(globe)
    // the atmosphere: a slightly larger sphere seen from inside, glowing at the limb
    const atmoMat = new THREE.MeshBasicMaterial({ color: 0x2266cc, side: THREE.BackSide, transparent: true, opacity: 0.28, depthWrite: false, blending: THREE.AdditiveBlending })
    tilt.add(new THREE.Mesh(new THREE.SphereGeometry(1.02, 64, 64), atmoMat))
    const dots = new THREE.Group(); globe.add(dots)
    const dotGeo = new THREE.SphereGeometry(0.02, 12, 12), dotMat = new THREE.MeshBasicMaterial({ color: 0xffe2a0, transparent: true, opacity: 0.95 })
    const haloGeo = new THREE.SphereGeometry(0.045, 12, 12), haloMat = new THREE.MeshBasicMaterial({ color: 0xff6a4d, transparent: true, opacity: 0.35, depthWrite: false })
    api = { setPlaces(list) { dots.clear(); for (const p of list) { for (const [g, m] of [[haloGeo, haloMat], [dotGeo, dotMat]]) { const d = new THREE.Mesh(g, m); d.position.copy(xyz(p.lat, p.lon, 1.012)); dots.add(d) } } } }
    api.setPlaces(places)
    const still = window.matchMedia?.('(prefers-reduced-motion: reduce)').matches
    let raf, visible = true
    const tick = () => { raf = requestAnimationFrame(tick); if (!visible) return; if (!still) globe.rotation.y += 0.0003; renderer.render(scene, camera) }   // about six minutes a turn
    tick()
    const ro = new ResizeObserver(() => { const w2 = container.clientWidth, h2 = container.clientHeight; camera.aspect = w2 / h2; camera.updateProjectionMatrix(); renderer.setSize(w2, h2) }); ro.observe(container)
    const onVis = () => { visible = !document.hidden }; document.addEventListener('visibilitychange', onVis)
    return () => { cancelAnimationFrame(raf); ro.disconnect(); document.removeEventListener('visibilitychange', onVis); dotMat.dispose(); haloMat.dispose(); atmoMat.dispose(); renderer.domElement.remove(); renderer.dispose(); api = null }
  })
  $effect(() => { const list = places; if (api) api.setPlaces(list) })
</script>

<div aria-hidden="true" class="pointer-events-none fixed right-0 top-1/2 -z-10 aspect-square w-[min(80vw,80vh)] -translate-y-1/2 translate-x-1/2 select-none overflow-hidden rounded-full opacity-25 sm:opacity-60">
  <div bind:this={container} style="width:100%;height:100%"></div>
</div>
