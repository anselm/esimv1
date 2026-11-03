<script>
  import Badge from './Badge.svelte'
  // Two futures side by side: a project's variants, latest run each, under one cursor.
  // Big counters that move with the cursor, the maps, the crossings so far, and a country
  // picker that draws one country's path from the run logs. Built for the 2100 question.
  import { link } from './router.svelte.js'
  import { isoDate, num } from './fmt.js'
  import FieldMap from './FieldMap.svelte'
  import Chart from './Chart.svelte'
  import Timeline from './Timeline.svelte'

  let { path, variants: wanted = null } = $props()
  let runs = $state(null)        // [{ variant, title, id, run, base, log }]
  let error = $state(null)
  let cursor = $state(null)
  let field = $state('people_outside_niche')
  let country = $state(null)     // iso3
  let countryName = $state('')
  let index = $state(null)       // country index frame (Float32Array) and its grid
  let countries = $state(null)   // [{ iso3, name }] by index, from the first log's crossing names and the geojson order

  async function loadJson(url) { const r = await fetch(url, { cache: 'no-store' }); if (!r.ok) throw new Error(`${url}: ${r.status}`); return r.json() }
  async function loadText(url) { const r = await fetch(url, { cache: 'no-store' }); return r.ok ? r.text() : '' }

  // the project's variants and the latest run of each
  async function load() {
    try {
      const meta = await loadText(`/projects/${path}/project.toml`)
      const names = [...meta.matchAll(/\[\[variant\]\][\s\S]*?name\s*=\s*"([^"]+)"(?:[\s\S]*?title\s*=\s*"([^"]+)")?/g)].map((m) => ({ name: m[1], title: m[2] ?? m[1] }))
      const idx = (await loadJson(`/projects/${path}/runs/index.json`)).sort((a, b) => (b.start ?? 0) - (a.start ?? 0))
      const pick = (wanted ?? names.map((n) => n.name).filter((n) => n !== 'hindcast')).slice(0, 3)
      const out = []
      for (const v of pick) {
        const rows = idx.filter((r) => r.variant === v)
        if (!rows.length) continue
        // the newest run folder of that variant
        const row = rows.sort((a, b) => (b.id > a.id ? 1 : -1))[0]
        const base = `/projects/${path}/runs/${encodeURIComponent(row.id)}`
        const run = await loadJson(`${base}/run.json`)
        const audit = await loadJson(`${base}/audit.json`).catch(() => null)
        out.push({ variant: v, title: names.find((n) => n.name === v)?.title ?? v, id: row.id, run, base, audit, log: null })
      }
      if (!out.length) throw new Error('no variant runs yet; run the variants on the project page')
      runs = out
      cursor = Math.max(...out.map((o) => o.run.end))
    } catch (e) { error = e.message }
  }
  $effect(() => { load() })

  const t0 = $derived(runs ? Math.min(...runs.map((o) => o.run.start)) : 0)
  const t1 = $derived(runs ? Math.max(...runs.map((o) => o.run.end)) : 1)

  // value of a series at the cursor
  function at(run, name) {
    const pts = run.series?.[name]?.points
    if (!pts?.length || cursor === null) return null
    let lo = 0, hi = pts.length - 1
    if (pts[0][0] > cursor) return null
    while (lo < hi) { const mid = (lo + hi + 1) >> 1; if (pts[mid][0] <= cursor) lo = mid; else hi = mid - 1 }
    return pts[lo][1]
  }
  const M = (v) => (v === null ? '—' : v >= 1e9 ? `${(v / 1e9).toFixed(2)} bn` : v >= 1e6 ? `${Math.round(v / 1e6)} M` : num(Math.round(v)))
  const deg = (v) => (v === null ? '—' : `${v >= 0 ? '+' : ''}${v.toFixed(2)} °C`)
  const crossingsSoFar = (o) => (o.audit?.crossings ?? []).filter((c) => c.t <= cursor)
  const fields = $derived(runs ? [...new Set(runs.flatMap((o) => Object.keys(o.run.fields ?? {})))].filter((f) => f !== 'country_index').sort() : [])

  // ---- country picker: parse each run's log for the yearly per-country tables (once)
  async function ensureLogs() {
    for (const o of runs) {
      if (o.log) continue
      const text = await loadText(`${o.base}/log.jsonl`)
      const tas = new Map(), heat = new Map()   // iso3 -> [[t, value]]
      let names = new Map()
      for (const line of text.split('\n')) {
        if (!line.includes('"climate.country_tas"') && !line.includes('"people.crossing"')) continue
        let ev; try { ev = JSON.parse(line) } catch { continue }
        if (ev.topic === 'climate.country_tas') {
          for (const [iso, v] of Object.entries(ev.data.by_iso3 ?? {})) {
            if (!tas.has(iso)) tas.set(iso, [])
            tas.get(iso).push([ev.t, v.tas_c])
            if (v.hot_month_c != null) { if (!heat.has(iso)) heat.set(iso, []); heat.get(iso).push([ev.t, v.hot_month_c]) }
          }
        } else if (ev.topic === 'people.crossing') names.set(ev.data.iso3, ev.data.name)
      }
      o.log = { tas, heat, names }
    }
    runs = runs
  }
  // the country under a clicked cell: the country_index frame maps cells to the countries' file order;
  // iso3 for that order comes from the geojson, fetched once
  async function pickAt(cell) {
    const o = runs[0]
    if (!index) {
      const meta = o.run.fields.country_index
      const r = await fetch(`${o.base}/fields/country_index/00000.f32`)
      const buf = await r.arrayBuffer(); const dv = new DataView(buf); const n = buf.byteLength >> 2
      const arr = new Float32Array(n); for (let k = 0; k < n; k++) arr[k] = dv.getFloat32(k * 4, true)
      index = { arr, grid: meta.grid }
    }
    if (!countries) {
      const gj = await loadJson('/docs/countries.json').catch(() => null)
      countries = gj
    }
    const k = index.arr[cell]
    if (Number.isNaN(k) || !countries) return
    const c = countries[k]
    if (!c) return
    country = c.iso3; countryName = c.name
    await ensureLogs()
  }
</script>

<a use:link={{ project: path }} class="floating left-3 top-3 hover:text-brand">← {path}</a>
<div class="floating right-3 top-3 mono text-xs select-none"><Badge /></div>

{#if error}
  <p class="text-sm text-leak">{error}</p>
{:else if !runs}
  <p class="muted text-sm">loading…</p>
{:else}
  <section class="mb-4">
    <h1 class="text-xl font-medium">Two futures, one clock</h1>
    <p class="mt-1 max-w-3xl text-sm muted">Each column is one assumption about emissions after 2024, run through the same planet. Nothing here is a forecast. Drag the time, or press play, and watch the counters: these are people living where the body's limits are passed, not deaths. The model is a few years warm and the thresholds are cited on the run pages.</p>
  </section>

  <Timeline {t0} {t1} bind:cursor />

  <div class="grid gap-6" style="grid-template-columns: repeat({runs.length}, minmax(0, 1fr))">
    {#each runs as o (o.variant)}
      {@const xs = crossingsSoFar(o)}
      <section class="min-w-0">
        <div class="label">{o.variant}</div>
        <div class="text-sm font-medium">{o.title}</div>
        <dl class="mt-3 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
          <dt class="muted">warming since 1850</dt><dd class="mono text-right text-lg tabular-nums">{deg(at(o.run, 'energy.gmst_anom_c'))}</dd>
          <dt class="muted">CO₂</dt><dd class="mono text-right tabular-nums">{at(o.run, 'carbon.co2_ppm') === null ? '—' : `${Math.round(at(o.run, 'carbon.co2_ppm'))} ppm`}</dd>
          <dt class="muted">people outside the human climate niche</dt><dd class="mono text-right text-lg tabular-nums text-leak">{M(at(o.run, 'people.outside_niche'))}</dd>
          <dt class="muted">people living where 31 °C wet-bulb is passed on three days a year</dt><dd class="mono text-right text-lg tabular-nums text-leak">{M(at(o.run, 'wetbulb.people_over_a') ?? at(o.run, 'wetbulb.people_capital_over_a'))}</dd>
          <dt class="muted">people living where 35 °C wet-bulb is passed</dt><dd class="mono text-right tabular-nums">{M(at(o.run, 'wetbulb.people_over_b') ?? at(o.run, 'wetbulb.people_capital_over_b'))}</dd>
          <dt class="muted">countries outside the niche</dt><dd class="mono text-right tabular-nums">{at(o.run, 'people.countries_outside_niche') ?? '—'}</dd>
          <dt class="muted">tipping elements past their central estimate</dt><dd class="mono text-right tabular-nums">{at(o.run, 'tipping.elements_past_central') ?? '—'}</dd>
          <dt class="muted">deep ocean warming</dt><dd class="mono text-right tabular-nums">{deg(at(o.run, 'energy.deep_anom_c'))}</dd>
          <dt class="muted">sea level from expansion alone</dt><dd class="mono text-right tabular-nums">{at(o.run, 'sea.thermal_m') === null ? '—' : `${(at(o.run, 'sea.thermal_m') * 100).toFixed(0)} cm`}</dd>
        </dl>
        {#if o.run.fields?.[field]}
          <div class="mt-4" role="button" tabindex="0" onclick={(e) => { const el = e.currentTarget.querySelector('canvas'); if (!el) return; const r = el.getBoundingClientRect(); const g = o.run.fields[field].grid; const col = Math.floor(((e.clientX - r.left) / r.width) * g.nx); const row = Math.floor(((e.clientY - r.top) / r.height) * g.ny); if (col >= 0 && row >= 0 && col < g.nx && row < g.ny) pickAt(row * g.nx + col) }} onkeydown={() => {}}>
            <FieldMap name={field} field={o.run.fields[field]} base={o.base} {cursor} />
          </div>
        {/if}
        <div class="mt-4">
          <div class="label mb-1">crossed so far · {xs.length}</div>
          <ul class="mono max-h-64 overflow-auto text-xs">
            {#each xs.slice().reverse() as c, i (i)}
              <li class="flex justify-between gap-2 py-0.5 border-t border-neutral-100 dark:border-neutral-900">
                <span class="truncate">{isoDate(c.t).slice(0, 4)} {c.topic === 'tipping.crossing' || c.topic === 'carbon.crossing' ? c.data.title : `${c.data.name}${c.data.capital ? ` (${c.data.capital})` : ''}`}</span>
                <span class="whitespace-nowrap {c.topic === 'tipping.crossing' ? (c.data.level === 'central' ? 'text-leak' : 'muted') : c.topic === 'carbon.crossing' || c.data.kind === 'niche' || c.data.kind?.startsWith('wetbulb') ? 'text-leak' : 'muted'}">{c.topic === 'tipping.crossing' ? `${c.data.level} ${c.data.threshold_c}°` : c.topic === 'carbon.crossing' ? `releases ${Math.round(c.data.gtc_target)} GtC` : c.data.kind === 'niche' ? 'left the niche' : c.data.kind === 'wetbulb_a' ? '31° wet-bulb' : c.data.kind === 'wetbulb_b' ? '35° wet-bulb' : 'hottest month'}</span>
              </li>
            {/each}
          </ul>
        </div>
      </section>
    {/each}
  </div>

  <div class="mt-6 flex flex-wrap items-center gap-3 text-xs">
    <span class="label">map</span>
    {#each fields as f (f)}
      <button class="mono rounded-[3px] px-1.5 py-0.5 ring-1 {field === f ? 'text-brand ring-brand/40' : 'ring-neutral-300 dark:ring-neutral-700'}" onclick={() => { field = f }}>{f}</button>
    {/each}
    <span class="muted">· click a country on a map to follow it</span>
  </div>

  {#if country}
    <section class="mt-8">
      <div class="label mb-2">{countryName} · {country}</div>
      <div class="grid gap-x-6 gap-y-5" style="grid-template-columns: repeat({runs.length}, minmax(0, 1fr))">
        {#each runs as o (o.variant)}
          {#if o.log?.tas.get(country)}
            <Chart name={`${country} annual mean · ${o.variant}`} unit="degC" points={o.log.tas.get(country)} t0={o.run.start} t1={o.run.end} obs={o.log.heat.get(country) ?? null} obsName="hottest month" {cursor} />
          {:else}
            <p class="muted text-sm">loading {o.variant}…</p>
          {/if}
        {/each}
      </div>
    </section>
  {/if}
{/if}
