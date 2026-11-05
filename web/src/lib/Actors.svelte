<script>
  // A diagram of actors and the relationships between them, in the author's own words.
  // The README writes it as text and this draws it, so the picture and the prose cannot
  // disagree:
  //
  //   ```esim
  //   actors
  //   weather: the wind, the dry air, the sun
  //   fire
  //   weather -> fire: drives the head downwind
  //   fire -> weather: makes its own wind (not in the model)
  //   ```
  //
  // A line `name: words` declares an actor with a caption; `a -> b: words` a relationship,
  // read along the arrow. A relationship whose words end in a parenthesis beginning "not"
  // is one the world has and the model lacks: it is drawn dashed, with the words kept, so
  // the gaps are on the page beside the couplings. The actor with the most arrows sits in
  // the middle and the rest stand around it in the order written.
  let { text = '' } = $props()


  const graph = $derived.by(() => parse(text))

  function parse(src) {
    const actors = [], edges = []
    const byName = new Map()
    const add = (name) => {
      if (!byName.has(name)) { const a = { name, caption: '' }; byName.set(name, a); actors.push(a) }
      return byName.get(name)
    }
    for (const raw of src.split('\n').slice(1)) {
      const line = raw.trim()
      if (!line) continue
      const e = line.match(/^(.+?)\s*->\s*(.+?)\s*(?::\s*(.*))?$/)
      if (e) {
        let words = e[3] ?? ''
        const gap = words.match(/\((not\b[^)]*)\)\s*$/)
        if (gap) words = words.slice(0, gap.index).trim()
        add(e[1]); add(e[2])
        edges.push({ from: e[1], to: e[2], words, missing: !!gap, note: gap ? gap[1] : '' })
        continue
      }
      const a = line.match(/^([^:]+?)\s*(?::\s*(.*))?$/)
      if (a) add(a[1]).caption = a[2] ?? ''
    }
    // the hub: the actor with the most arrows; the rest on an ellipse, clockwise from the top
    const degree = new Map(actors.map((a) => [a.name, 0]))
    for (const e of edges) { degree.set(e.from, degree.get(e.from) + 1); degree.set(e.to, degree.get(e.to) + 1) }
    const hub = actors.slice().sort((a, b) => degree.get(b.name) - degree.get(a.name))[0]
    const ring = actors.filter((a) => a !== hub)
    // the picture grows with the ring so the arrows stay long enough for their words
    const big = ring.length > 6
    const W = big ? 900 : 720, H = big ? 660 : 520, RX = W / 2 - 95, RY = H / 2 - 70
    const pos = new Map()
    if (hub) pos.set(hub.name, { x: W / 2, y: H / 2 })
    ring.forEach((a, i) => {
      const t = -Math.PI / 2 + (2 * Math.PI * i) / Math.max(1, ring.length)
      pos.set(a.name, { x: W / 2 + RX * Math.cos(t), y: H / 2 + RY * Math.sin(t) })
    })
    const nodes = actors.map((a) => {
      const cap = lines(a.caption)
      const w = Math.max(72, 8 * a.name.length + 24, ...cap.map((l) => 5.9 * l.length + 18))
      return { ...a, ...pos.get(a.name), cap, w, h: 28 + 11 * cap.length, hub: a === hub }
    })
    const at = new Map(nodes.map((n) => [n.name, n]))
    // each arrow from border to border; a pair in both directions is parted sideways
    const drawn = edges.map((e) => {
      const a = at.get(e.from), b = at.get(e.to)
      const dx = b.x - a.x, dy = b.y - a.y, len = Math.hypot(dx, dy) || 1
      const ux = dx / len, uy = dx === 0 && dy === 0 ? 0 : dy / len
      const twin = edges.some((o) => o.from === e.to && o.to === e.from)
      const side = twin ? 9 : 0
      const px = -uy * side, py = ux * side
      const clip = (n, sx, sy) => Math.min(n.w / 2 / Math.max(1e-6, Math.abs(sx)), n.h / 2 / Math.max(1e-6, Math.abs(sy)))
      const x1 = a.x + ux * (clip(a, ux, uy) + 4) + px, y1 = a.y + uy * (clip(a, ux, uy) + 4) + py
      const x2 = b.x - ux * (clip(b, ux, uy) + 6) + px, y2 = b.y - uy * (clip(b, ux, uy) + 6) + py
      // the words sit at the arrow's middle, turned to its angle, reading left to right
      // whichever way it points; a short arrow does not cut them off
      const ltr = x2 >= x1
      const mx = (x1 + x2) / 2, my = (y1 + y2) / 2
      const angle = (Math.atan2(y2 - y1, x2 - x1) * 180) / Math.PI + (ltr ? 0 : 180)
      // words longer than the arrow go on two lines
      const full = e.words + (e.missing ? ` (${e.note})` : '')
      const room = Math.hypot(x2 - x1, y2 - y1) + 30
      const rows = 5.3 * full.length <= room ? [full] : split(full)
      // a lone arrow carries its words above it; one of a pair carries them on its outer
      // side, which is its own right hand, and "below" in text terms only when drawn its way
      const below = twin && ltr
      const dys = rows.map((_, i) => (below ? 10 + 10 * i : -3 - 10 * (rows.length - 1 - i)))
      return { ...e, x1, y1, x2, y2, mx, my, angle, rows, dys, id: `actors-${edges.indexOf(e)}` }
    })
    return { nodes, edges: drawn, W, H }
  }

  function split(s) {
    const words = s.split(' ')
    let best = 1, diff = Infinity
    for (let i = 1; i < words.length; i++) {
      const d = Math.abs(words.slice(0, i).join(' ').length - words.slice(i).join(' ').length)
      if (d < diff) { diff = d; best = i }
    }
    return [words.slice(0, best).join(' '), words.slice(best).join(' ')]
  }

  function lines(s, n = 26) {
    if (!s) return []
    const out = [], words = s.split(/\s+/)
    let cur = ''
    for (const w of words) { if ((cur + ' ' + w).trim().length > n && cur) { out.push(cur); cur = w } else cur = (cur + ' ' + w).trim() }
    if (cur) out.push(cur)
    return out.slice(0, 2)
  }
</script>

<figure class="my-4 overflow-x-auto">
  <svg viewBox="0 0 {graph.W} {graph.H}" class="w-full h-auto min-w-[560px] text-neutral-900 dark:text-neutral-100" role="img" aria-label="the actors and their relationships">
    <defs>
      <marker id="actors-head" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
        <path d="M 0 0 L 10 5 L 0 10 z" fill="currentColor" />
      </marker>
    </defs>
    {#each graph.edges as e (e.id)}
      <line x1={e.x1} y1={e.y1} x2={e.x2} y2={e.y2} stroke="currentColor" stroke-width="1.1" stroke-dasharray={e.missing ? '4 3' : null} opacity={e.missing ? 0.45 : 0.8} marker-end="url(#actors-head)" />
      {#each e.rows as row, i (i)}
        <text transform="translate({e.mx} {e.my}) rotate({e.angle})" text-anchor="middle" font-size="10" fill="currentColor" opacity={e.missing ? 0.6 : 0.9} font-style={e.missing ? 'italic' : null} dy={e.dys[i]}>{row}</text>
      {/each}
    {/each}
    {#each graph.nodes as n (n.name)}
      <rect x={n.x - n.w / 2} y={n.y - n.h / 2} width={n.w} height={n.h} rx="3" fill="var(--color-page)" stroke={n.hub ? 'var(--color-brand)' : 'currentColor'} stroke-opacity={n.hub ? 1 : 0.5} stroke-width={n.hub ? 1.4 : 1} />
      <text x={n.x} y={n.y - n.h / 2 + 14} text-anchor="middle" dominant-baseline="middle" font-size="13" fill="currentColor">{n.name}</text>
      {#each n.cap as l, i (i)}
        <text x={n.x} y={n.y - n.h / 2 + 29 + i * 11} text-anchor="middle" dominant-baseline="middle" font-size="9.5" fill="currentColor" opacity="0.6" class="mono">{l}</text>
      {/each}
    {/each}
  </svg>
</figure>
