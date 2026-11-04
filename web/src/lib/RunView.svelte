<script>
  import Badge from './Badge.svelte'
  // One run, composed from its manifest: the kernel's header and time cursor first, then
  // each module's own views in the manifest's order (from describe.json, written by the
  // run), then the auditor's reading, the scores, the ledgers, and the raw events and
  // manifest behind disclosures. No module is special-cased here; a module that wants a
  // counter, a map or a list declares it in its describe.
  import { link } from './router.svelte.js'
  import Crumbs from './Crumbs.svelte'
  import { isoDate, num, ms } from './fmt.js'
  import Status from './Status.svelte'
  import Ledger from './Ledger.svelte'
  import Chart from './Chart.svelte'
  import Scores from './Scores.svelte'
  import FieldMap from './FieldMap.svelte'
  import EventLog from './EventLog.svelte'
  import EnsembleView from './EnsembleView.svelte'
  import Timeline from './Timeline.svelte'
  import Audit from './Audit.svelte'
  import ModuleViews from './ModuleViews.svelte'
  import Wiring from './Wiring.svelte'

  let { id, base, back = {}, backLabel = 'back' } = $props()
  let cursor = $state(null)
  // roads, place names and city limits a module fetched for its window (its describe's `context` table)
  const contextOf = (run) => (run?.describe?.modules ?? []).map((m) => m.tables?.context).find(Boolean) ?? null
  let wantLog = $state(false)   // the event log is ten megabytes; fetched when its panel opens

  async function load() {
    const r = await fetch(`${base}/run.json`, { cache: 'no-store' })
    if (!r.ok) throw new Error(`run.json ${r.status}`)
    const run = await r.json()
    ;[run.scores, run.audit, run.describe] = await Promise.all([loadJson('scores.json'), loadJson('audit.json'), loadJson('describe.json')])
    // the cursor starts at the beginning and plays, so the tip-over is seen, not just the end state
    cursor = run.start
    return run
  }
  async function loadJson(name) {
    try {
      const r = await fetch(`${base}/${name}`, { cache: 'no-store' })
      if (!r.ok) return null
      const v = await r.json()
      return Array.isArray(v) ? (v.length ? v : null) : v
    } catch {
      return null
    }
  }
  async function loadText(name) {
    const r = await fetch(`${base}/${name}`, { cache: 'no-store' })
    return r.ok ? r.text() : null
  }

  // Frames and series no module's views claimed (older runs without describe.json, or
  // outputs a describe does not list) are shown after the modules, grouped by unit.
  function unclaimed(run) {
    const claimedMaps = new Set(), claimedSeries = new Set()
    for (const d of run.describe?.modules ?? []) {
      const vs = d.views?.length ? d.views : [...(d.frames ?? []).map((f) => ({ kind: 'map', source: f.name })), ...(d.reports ?? []).map((r) => ({ kind: 'chart', source: r.name }))]
      for (const v of vs) { if (v.kind === 'map') claimedMaps.add(v.source); else if (v.kind === 'chart' || v.kind === 'counter') { claimedSeries.add(v.source); if (v.obs) claimedSeries.add(v.obs) } }
      for (const f of d.frames ?? []) claimedMaps.add(f.name)
      for (const r of d.reports ?? []) claimedSeries.add(r.name)
    }
    const maps = Object.keys(run.fields ?? {}).filter((f) => !claimedMaps.has(f)).sort()
    const paired = new Map()
    for (const sc of run.scores ?? []) if (run.series?.[sc.sim] && run.series?.[sc.obs] && !paired.has(sc.sim)) paired.set(sc.sim, sc.obs)
    const hidden = new Set(paired.values())
    const groups = new Map()
    for (const [name, s] of Object.entries(run.series ?? {})) {
      if (claimedSeries.has(name) || hidden.has(name)) continue
      const u = s.unit ?? ''
      if (!groups.has(u)) groups.set(u, [])
      groups.get(u).push({ name, ...s, obs: paired.get(name) ?? null })
    }
    for (const g of groups.values()) g.sort((a, b) => a.name.localeCompare(b.name))
    return { maps, series: [...groups.entries()].sort(([a], [b]) => a.localeCompare(b)) }
  }
</script>

{#if back.project}
  <span class="floating left-3 top-3"><a use:link={back} class="hover:text-brand">←</a> <Crumbs path={back.project} run={id} floating /></span>
{:else}
  <a use:link={back} class="floating left-3 top-3 hover:text-brand">← {backLabel}</a>
{/if}
<div class="floating right-3 top-3 mono text-xs select-none"><Badge /></div>

{#await load()}
  <p class="muted text-sm">loading…</p>
{:then run}
{#if run.kind === 'ensemble'}
  <EnsembleView {run} {base} />
{:else}
  {@const rest = unclaimed(run)}
  {@const crossings = run.audit?.crossings ?? []}

  <!-- the kernel: facts, one line -->
  <section class="mb-4">
    <h1 class="text-xl font-medium">{run.name}</h1>
    <div class="mt-1 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs mono muted">
      <span>{isoDate(run.start)} → {isoDate(run.end)}</span>
      <span>seed {run.seed}</span>
      <span>{num(run.delivered)} events</span>
      <span>{ms(run.elapsed_ms)}</span>
      <span>{run.log_digest}</span>
      <Status closed={run.all_ledgers_closed} />
    </div>
  </section>

  {#if cursor !== null}
    <Timeline t0={run.start} t1={run.end} bind:cursor autoplay={true} />
  {/if}

  <!-- the modules, in the manifest's order, each with the views it declared -->
  {#each run.describe?.modules ?? [] as d (d.name)}
    <ModuleViews describe={d} {run} {base} {cursor} {crossings} context={contextOf(run)} />
  {/each}

  <!-- what no module claimed -->
  {#if rest.maps.length}
    <section class="mb-10">
      <div class="label mb-3">other maps</div>
      <div class="grid gap-x-6 gap-y-6 {rest.maps.length === 1 ? '' : rest.maps.length === 2 ? 'sm:grid-cols-2' : 'sm:grid-cols-2 lg:grid-cols-3'}">
        {#each rest.maps as f (f)}
          <FieldMap name={f} field={run.fields[f]} {base} {cursor} context={contextOf(run)} />
        {/each}
      </div>
    </section>
  {/if}
  {#if rest.series.length}
    <section class="mb-10">
      <div class="label mb-3">other series</div>
      {#each rest.series as [unit, list] (unit)}
        <div class="mb-6">
          <div class="mono text-xs muted mb-2">{unit}</div>
          <div class="grid gap-x-6 gap-y-5 sm:grid-cols-2">
            {#each list as s (s.name)}
              <Chart name={s.name} unit={s.unit} points={s.points} t0={run.start} t1={run.end} obs={s.obs ? run.series[s.obs].points : null} obsName={s.obs} {cursor} />
            {/each}
          </div>
        </div>
      {/each}
    </section>
  {/if}

  <!-- the wiring: the check made visible -->
  {#if run.describe?.modules?.length}
    <section class="mb-10">
      <div class="label mb-1">wiring</div>
      <p class="mb-3 text-sm muted">The relationships between the modules of this run: who writes what, who reads it, and which books they share. Drawn from what each module declares, so a relationship the world has and the run lacks shows as a missing arrow.</p>
      <Wiring modules={run.describe.modules} info={run.describe.info ?? []} />
    </section>
  {/if}

  <!-- the auditor -->
  {#if run.audit}
    <section class="mb-10">
      <div class="label mb-2">audit</div>
      <Audit audit={run.audit} />
    </section>
  {/if}

  <!-- scores: sim against observation -->
  {#if run.scores}
    <section class="mb-10">
      <div class="label mb-3">scores</div>
      <Scores scores={run.scores} series={run.series} />
    </section>
  {/if}

  <!-- ledgers -->
  {#if run.ledgers?.length}
    <section class="mb-10">
      <div class="label mb-3">ledgers</div>
      <div class="space-y-6">
        {#each run.ledgers as l (l.name)}
          <Ledger ledger={l} />
        {/each}
      </div>
    </section>
  {/if}

  <!-- events, behind a disclosure -->
  <details class="mb-6" ontoggle={(e) => { if (e.currentTarget.open) wantLog = true }}>
    <summary class="label cursor-pointer">events <span class="mono normal-case tracking-normal muted">· {num(run.events)} lines, loaded when opened</span></summary>
    <div class="mt-3">
      {#if wantLog}
        {#await loadText('log.jsonl')}
          <p class="muted text-sm">loading log…</p>
        {:then text}
          {#if text === null}
            <p class="muted text-sm">no log</p>
          {:else}
            <EventLog {text} />
          {/if}
        {/await}
      {/if}
    </div>
  </details>
{/if}

  <details class="mb-10">
    <summary class="label cursor-pointer">manifest</summary>
    {#await loadText('manifest.toml')}
      <p class="muted text-sm">loading…</p>
    {:then toml}
      {#if toml === null}
        <p class="muted text-sm">no manifest</p>
      {:else}
        <pre class="mono mt-2 overflow-x-auto rounded-[3px] bg-neutral-50 p-3 text-xs leading-relaxed dark:bg-neutral-900">{toml}</pre>
      {/if}
    {/await}
  </details>
{:catch e}
  <p class="text-sm text-leak">could not load run <span class="mono">{id}</span>: {e.message}</p>
{/await}
