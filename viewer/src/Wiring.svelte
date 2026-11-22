<script>
  // The wiring of a run, drawn from what each module declared in its describe: modules
  // as boxes, topics and fields as arrows from the module that writes to the modules
  // that read, ledgers as the shared books between the modules that hold them. Arrows
  // that end in air are outputs nothing in this manifest reads; arrows that climb back
  // up are feedbacks. The drawing computes nothing new: it is the check made visible.
  let { modules = [], info = [] } = $props()

  const graph = $derived.by(() => build(modules))

  function build(mods) {
    const names = mods.map((m) => m.name)
    const idx = new Map(names.map((n, i) => [n, i]))
    const keyOf = (x) => (typeof x === 'string' ? x : x.name)
    // who writes and who reads each topic and field
    const writers = new Map(), readers = new Map()
    const add = (map, k, m, how) => { if (!map.has(k)) map.set(k, []); map.get(k).push({ m, how }) }
    for (const m of mods) {
      for (const t of m.publishes ?? []) add(writers, keyOf(t), m.name, 'topic')
      for (const t of m.sends ?? []) add(writers, keyOf(t), m.name, 'message')
      for (const f of m.fields_owned ?? []) add(writers, keyOf(f), m.name, 'field')
      for (const t of m.subscribes ?? []) add(readers, keyOf(t), m.name, 'topic')
      for (const t of m.accepts ?? []) add(readers, keyOf(t), m.name, 'message')
      for (const f of m.fields_read ?? []) add(readers, keyOf(f), m.name, 'field')
    }
    // edges between modules, one per (from, to) carrying every name on it
    const pair = new Map()
    const dead = [] // written, read by nobody
    for (const [k, ws] of writers) {
      const rs = (readers.get(k) ?? []).filter((r) => !ws.some((w) => w.m === r.m))
      if (!rs.length) { for (const w of ws) dead.push({ m: w.m, name: k, how: w.how }); continue }
      for (const w of ws) for (const r of rs) {
        const id = `${w.m}→${r.m}`
        if (!pair.has(id)) pair.set(id, { from: w.m, to: r.m, names: [], how: w.how })
        pair.get(id).names.push(k)
      }
    }
    const edges = [...pair.values()]
    // back edges by depth-first search in manifest order: those are the feedbacks
    const out = new Map(names.map((n) => [n, []]))
    for (const e of edges) out.get(e.from).push(e)
    const state = new Map()
    const back = new Set()
    const visit = (n) => {
      state.set(n, 1)
      for (const e of out.get(n)) {
        const s = state.get(e.to)
        if (s === 1) back.add(e)
        else if (!s) visit(e.to)
      }
      state.set(n, 2)
    }
    for (const n of names) if (!state.get(n)) visit(n)
    for (const e of edges) e.back = back.has(e)
    // layers: longest path over the forward edges
    const layer = new Map(names.map((n) => [n, 0]))
    for (let i = 0; i < names.length; i++) {
      let moved = false
      for (const e of edges) if (!e.back && layer.get(e.to) < layer.get(e.from) + 1) { layer.set(e.to, layer.get(e.from) + 1); moved = true }
      if (!moved) break
    }
    // sources sit just above their first reader rather than at the top
    const hasIn = new Set(edges.filter((e) => !e.back).map((e) => e.to))
    for (const n of names) if (!hasIn.has(n)) {
      const succ = edges.filter((e) => !e.back && e.from === n).map((e) => layer.get(e.to))
      if (succ.length) layer.set(n, Math.min(...succ) - 1)
    }
    // ledgers: one node per ledger, in a column at the right of the flow
    const ledgers = new Map()
    for (const m of mods) for (const l of m.ledgers ?? []) {
      const k = keyOf(l)
      if (!ledgers.has(k)) ledgers.set(k, { name: k, unit: l.unit ?? '', holders: [] })
      ledgers.get(k).holders.push(m.name)
    }
    // rows by layer; within a row, order by the mean position of neighbours (two sweeps)
    const rows = []
    for (const n of names) { const l = layer.get(n); (rows[l] ??= []).push(n) }
    for (let i = 0; i < rows.length; i++) rows[i] ??= []
    const order = new Map()
    rows.forEach((row) => row.forEach((n, i) => order.set(n, i)))
    const bary = (n, dir) => {
      const nb = edges.filter((e) => !e.back && (dir > 0 ? e.to === n : e.from === n)).map((e) => order.get(dir > 0 ? e.from : e.to))
      return nb.length ? nb.reduce((a, b) => a + b, 0) / nb.length : order.get(n)
    }
    for (const dir of [1, -1, 1]) {
      const seq = dir > 0 ? rows : [...rows].reverse()
      for (const row of seq) {
        const b = new Map(row.map((n) => [n, bary(n, dir)]))
        row.sort((x, y) => b.get(x) - b.get(y) || names.indexOf(x) - names.indexOf(y))
        row.forEach((n, i) => order.set(n, i))
      }
    }
    const LW = ledgers.size ? 156 : 0
    // room grows with density: the busiest row of boxes and the busiest gap of arrows
    const gap = new Map()
    for (const e of edges) if (!e.back) { const r = layer.get(e.from); gap.set(r, (gap.get(r) ?? 0) + 1) }
    const busiest = Math.max(0, ...gap.values())
    const W = Math.max(720, Math.max(...rows.map((r) => r.length)) * 136 + LW, busiest * 96 + LW)
    const RH = busiest > 4 ? 128 : 96, PAD = 24, NH = 34
    const pos = new Map()
    rows.forEach((row, r) => {
      const step = (W - LW - 2 * PAD) / row.length
      row.forEach((n, i) => {
        const kind = mods[idx.get(n)].kind
        const w = Math.max(72, Math.min(step - 10, 7.2 * Math.max(n.length, kind.length * 0.86) + 18))
        pos.set(n, { id: n, label: n, kind, ledger: false, x: PAD + step * (i + 0.5), y: PAD + r * RH + NH / 2, w, h: NH })
      })
    })
    for (const l of ledgers.values()) {
      const ys = l.holders.map((h) => pos.get(h).y)
      const id = `ledger:${l.name}`
      const y = ys.reduce((a, b) => a + b, 0) / ys.length
      // keep ledgers from overlapping one another in the column
      const taken = [...pos.values()].filter((n) => n.ledger).map((n) => n.y)
      let yy = y
      while (taken.some((t) => Math.abs(t - yy) < NH + 10)) yy += NH + 12
      pos.set(id, { id, label: `${l.name} ledger`, sub: l.unit, ledger: true, holders: l.holders, x: W - PAD - 52, y: yy, w: 104, h: NH })
    }
    const H = PAD * 2 + rows.length * RH - (RH - NH) + 24
    return { W, H, RH, nodes: [...pos.values()], edges, dead, ledgers: [...ledgers.values()], pos }
  }

  // Forward arrows leave the bottom of their source and enter the top of their target;
  // one that spans more than one row bows sideways so it does not run through the boxes
  // between. A feedback runs upward beside the forward arrow, offset to its right.
  // how far sideways an edge must bow to clear the boxes in the rows it crosses
  function bow(e) {
    const a = graph.pos.get(e.from), b = graph.pos.get(e.to)
    const span = Math.round(Math.abs(b.y - a.y) / graph.RH)
    if (span <= 1) return 0
    let right = 0, left = 0
    for (let r = 1; r < span; r++) {
      const y = a.y + (b.y - a.y) * (r / span)
      const x = a.x + (b.x - a.x) * (r / span)
      for (const n of graph.nodes) {
        if (n.ledger || Math.abs(n.y - y) > 1) continue
        if (Math.abs(n.x - x) < n.w / 2 + 14) {
          right = Math.max(right, n.x + n.w / 2 + 14 - x)
          left = Math.max(left, x - (n.x - n.w / 2 - 14))
        }
      }
    }
    if (!right && !left) return 0
    // the curve's midpoint sits three quarters of the way to the control points
    return right <= left ? right / 0.75 : -left / 0.75
  }
  function ctrl(e) {
    const a = graph.pos.get(e.from), b = graph.pos.get(e.to)
    if (!e.back) {
      const y0 = a.y + a.h / 2, y1 = b.y - b.h / 2, c = (y1 - y0) / 2, w = bow(e)
      return [a.x, y0, a.x + w, y0 + c, b.x + w, y1 - c, b.x, y1]
    }
    const x0 = a.x + 12, x1 = b.x + 12
    const y0 = a.y - a.h / 2, y1 = b.y + b.h / 2, c = (y1 - y0) / 2
    return [x0, y0, x0 + 30, y0 + c, x1 + 30, y1 - c, x1, y1]
  }
  function path(e) {
    if (!graph.pos.get(e.from) || !graph.pos.get(e.to)) return ''
    const [x0, y0, x1, y1, x2, y2, x3, y3] = ctrl(e)
    return `M${x0},${y0} C${x1},${y1} ${x2},${y2} ${x3},${y3}`
  }
  function mid(e) {
    if (!graph.pos.get(e.from) || !graph.pos.get(e.to)) return { x: 0, y: 0 }
    const [x0, y0, x1, y1, x2, y2, x3, y3] = ctrl(e)
    let t = 0.3
    if (!e.back) {
      // a label sits where its edge has parted from the others: near the target when the
      // source fans out, near the source when the target gathers, in a row gap for a long
      // edge; siblings from one source are spread a little along their curves
      const sib = graph.edges.filter((x) => !x.back && x.from === e.from).sort((x, y) => graph.pos.get(x.to).x - graph.pos.get(y.to).x)
      const k = sib.indexOf(e), n = sib.length
      const span = Math.round(Math.abs(graph.pos.get(e.to).y - graph.pos.get(e.from).y) / graph.RH)
      if (span > 1) t = k % 2 ? 1 - 1 / (2 * span) : 1 / (2 * span)
      else {
        const outs = n, ins = graph.edges.filter((x) => !x.back && x.to === e.to).length
        const base = outs > ins ? 0.78 : ins > outs ? 0.22 : 0.5
        t = n > 1 ? Math.min(0.88, Math.max(0.12, base + (k / (n - 1) - 0.5) * 0.3)) : base
      }
    }
    const u = 1 - t
    return {
      x: u * u * u * x0 + 3 * u * u * t * x1 + 3 * u * t * t * x2 + t * t * t * x3,
      y: u * u * u * y0 + 3 * u * u * t * y1 + 3 * u * t * t * y2 + t * t * t * y3,
    }
  }
  // an output nothing reads: a short arrow from the box's lower left corner into air
  function deadPath(d) {
    const a = graph.pos.get(d.m)
    if (!a) return ''
    const k = graph.dead.filter((x) => x.m === d.m)
    const x = a.x - a.w / 2 + 8 + k.indexOf(d) * 10
    return `M${x},${a.y + a.h / 2} L${x},${a.y + a.h / 2 + 22}`
  }
  const short = (names) => (names.length > 2 ? `${names.slice(0, 2).join(', ')} +${names.length - 2}` : names.join(', '))
  const feedbacks = $derived(graph.edges.filter((e) => e.back))
  const forward = $derived(graph.edges.filter((e) => !e.back))
  // crossings that only the story listens to: the fan-out stated as a fact
  const onlyStory = $derived.by(() => {
    const names = new Set()
    for (const e of graph.edges) for (const n of e.names) if (n.endsWith('.crossing')) names.add(n)
    const r = []
    for (const n of names) {
      const readers = graph.edges.filter((e) => e.names.includes(n)).map((e) => e.to)
      const kinds = readers.map((m) => modules.find((x) => x.name === m)?.kind ?? '')
      if (readers.length && kinds.every((k) => k.startsWith('story.'))) r.push(n)
    }
    return r
  })
</script>

<div class="overflow-x-auto">
  <svg viewBox="0 0 {graph.W} {graph.H}" width={graph.W} height={graph.H} class="wiring block max-w-none text-neutral-800 dark:text-neutral-200" role="img" aria-label="the wiring of this run: modules, the topics and fields between them, their ledgers">
    <defs>
      <marker id="arrow" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
        <path d="M0,0.5 L8,4 L0,7.5 z" fill="currentColor" opacity="0.55" />
      </marker>
      <marker id="arrow-back" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
        <path d="M0,0.5 L8,4 L0,7.5 z" fill="var(--color-brand)" />
      </marker>
      <marker id="arrow-dead" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
        <path d="M0,0.5 L8,4 L0,7.5 z" fill="var(--color-leak)" />
      </marker>
    </defs>

    <!-- ledgers: a book shared by the modules that hold it -->
    {#each graph.nodes.filter((n) => n.ledger) as l (l.id)}
      {#each l.holders as h (h)}
        {@const a = graph.pos.get(h)}
        <line x1={a.x + a.w / 2} y1={a.y} x2={l.x - l.w / 2} y2={l.y} stroke="var(--color-brand)" stroke-width="1" stroke-dasharray="2 3" opacity="0.8" />
      {/each}
    {/each}

    <!-- couplings -->
    {#each forward as e (e.from + e.to)}
      <path d={path(e)} fill="none" stroke="currentColor" stroke-width={e.how === 'field' ? 1 : 1.2} stroke-dasharray={e.how === 'field' ? '4 3' : null} opacity="0.55" marker-end="url(#arrow)" />
      {@const m = mid(e)}
      <text x={m.x} y={m.y} class="edge" text-anchor="middle" dominant-baseline="middle"><title>{e.from} → {e.to}: {e.names.join(', ')}</title>{short(e.names)}</text>
    {/each}

    <!-- feedbacks -->
    {#each feedbacks as e (e.from + e.to)}
      <path d={path(e)} fill="none" stroke="var(--color-brand)" stroke-width="1.4" marker-end="url(#arrow-back)" />
      {@const m = mid(e)}
      <text x={m.x} y={m.y} class="edge brand" text-anchor="start" dominant-baseline="middle"><title>{e.from} → {e.to}: {e.names.join(', ')}</title>{short(e.names)}</text>
    {/each}

    <!-- written, read by nobody: arrows into air -->
    {#each graph.dead as d (d.m + d.name)}
      <path d={deadPath(d)} fill="none" stroke="var(--color-leak)" stroke-width="1.2" marker-end="url(#arrow-dead)"><title>{d.m} writes {d.name}; nothing in this run reads it</title></path>
    {/each}

    <!-- modules and ledgers -->
    {#each graph.nodes as n (n.id)}
      {#if n.ledger}
        <rect x={n.x - n.w / 2} y={n.y - n.h / 2} width={n.w} height={n.h} rx="3" fill="var(--color-brand-soft)" stroke="var(--color-brand)" stroke-width="1" />
        <text x={n.x} y={n.y - 4} text-anchor="middle" dominant-baseline="middle" class="name brand">{n.label}</text>
        <text x={n.x} y={n.y + 9} text-anchor="middle" dominant-baseline="middle" class="kind">{n.sub}</text>
      {:else}
        <rect x={n.x - n.w / 2} y={n.y - n.h / 2} width={n.w} height={n.h} rx="3" class="box" stroke="currentColor" stroke-width="1" />
        <text x={n.x} y={n.y - 4} text-anchor="middle" dominant-baseline="middle" class="name">{n.id}</text>
        <text x={n.x} y={n.y + 9} text-anchor="middle" dominant-baseline="middle" class="kind">{n.kind}</text>
      {/if}
    {/each}
  </svg>
</div>

<div class="mt-3 grid gap-x-8 gap-y-2 text-sm sm:grid-cols-3">
  <div><span class="mono tabular-nums">{forward.length}</span> couplings forward{#if graph.ledgers.length}, <span class="mono tabular-nums">{graph.ledgers.length}</span> shared {graph.ledgers.length === 1 ? 'ledger' : 'ledgers'}{/if}</div>
  <div><span class="mono tabular-nums {feedbacks.length ? 'text-brand' : ''}">{feedbacks.length}</span> {feedbacks.length === 1 ? 'feedback' : 'feedbacks'}{#if feedbacks.length}: {feedbacks.map((e) => `${e.from} → ${e.to}`).join('; ')}{/if}</div>
  <div><span class="mono tabular-nums {graph.dead.length ? 'text-leak' : ''}">{graph.dead.length}</span> outputs nothing reads{#if graph.dead.length}: {graph.dead.map((d) => d.name).join(', ')}{/if}</div>
</div>
{#if onlyStory.length}
  <p class="mt-2 text-sm muted">Crossings only the story listens to: <span class="mono">{onlyStory.join(', ')}</span>. A threshold passed changes a sentence on this page and nothing else in the run.</p>
{/if}
<p class="mt-2 text-xs muted">Drawn from each module's describe: solid arrows are topics, dashed arrows are fields, a teal arrow climbs back to an earlier module, a red arrow ends in air. Reports and frames go to this page and are not drawn.</p>

<style>
  .wiring .box { fill: transparent; opacity: 0.9; }
  .wiring .name { font: 500 11px ui-sans-serif, system-ui, sans-serif; fill: currentColor; }
  .wiring .kind { font: 9.5px ui-monospace, SFMono-Regular, Menlo, monospace; fill: currentColor; opacity: 0.6; }
  .wiring .edge { font: 9.5px ui-monospace, SFMono-Regular, Menlo, monospace; fill: currentColor; opacity: 0.85; paint-order: stroke; stroke: var(--color-page, #fff); stroke-width: 3px; stroke-linejoin: round; }
  .wiring .brand { fill: var(--color-brand); opacity: 1; }
</style>
