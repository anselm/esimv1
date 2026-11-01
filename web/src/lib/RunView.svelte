<script>
  // One run, maps first. A single time cursor drives every map frame and marks every
  // chart. Then what the auditor found, the scores, the charts, the ledgers in brief,
  // and the raw events and manifest behind disclosure.
  import { link } from './router.svelte.js'
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

  let { id, base, back = {}, backLabel = 'back' } = $props()
  let cursor = $state(null)

  async function load() {
    const r = await fetch(`${base}/run.json`, { cache: 'no-store' })
    if (!r.ok) throw new Error(`run.json ${r.status}`)
    const run = await r.json()
    ;[run.scores, run.audit] = await Promise.all([loadJson('scores.json'), loadJson('audit.json')])
    cursor = run.end
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

  // A score pairs sim with obs; when both series exist they share one chart,
  // drawn under the sim's name, and the obs series is not charted on its own.
  function pairs(run) {
    const out = new Map()
    for (const sc of run.scores ?? []) {
      if (run.series?.[sc.sim] && run.series?.[sc.obs] && !out.has(sc.sim)) out.set(sc.sim, sc.obs)
    }
    return out
  }
  // Group series by unit so comparable charts sit together.
  function byUnit(series, paired) {
    const hidden = new Set(paired.values())
    const groups = new Map()
    for (const [name, s] of Object.entries(series ?? {})) {
      if (hidden.has(name)) continue
      const u = s.unit ?? ''
      if (!groups.has(u)) groups.set(u, [])
      groups.get(u).push({ name, ...s, obs: paired.get(name) ?? null })
    }
    for (const g of groups.values()) g.sort((a, b) => a.name.localeCompare(b.name))
    return [...groups.entries()].sort(([a], [b]) => a.localeCompare(b))
  }
  const fieldNames = (run) => Object.keys(run.fields ?? {}).sort()
</script>

<a use:link={back} class="floating left-3 top-3 hover:text-brand">← {backLabel}</a>
<div class="floating right-3 top-3 mono text-xs muted select-none">esim</div>

{#await load()}
  <p class="muted text-sm">loading…</p>
{:then run}
{#if run.kind === 'ensemble'}
  <EnsembleView {run} {base} />
{:else}
  {@const maps = fieldNames(run)}
  {@const paired = pairs(run)}

  <!-- facts, one line -->
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
    <Timeline t0={run.start} t1={run.end} bind:cursor />
  {/if}

  <!-- maps, first -->
  {#if maps.length}
    <section class="mb-10">
      <div class="grid gap-x-6 gap-y-6 {maps.length === 1 ? '' : maps.length === 2 ? 'sm:grid-cols-2' : 'sm:grid-cols-2 lg:grid-cols-3'}">
        {#each maps as f (f)}
          <FieldMap name={f} field={run.fields[f]} {base} {cursor} />
        {/each}
      </div>
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

  <!-- series -->
  {#if run.series && Object.keys(run.series).length}
    <section class="mb-10">
      <div class="label mb-3">series</div>
      {#each byUnit(run.series, paired) as [unit, list] (unit)}
        <div class="mb-6">
          <div class="mono text-xs muted mb-2">{unit}</div>
          <div class="grid gap-x-6 gap-y-5 sm:grid-cols-2">
            {#each list as s (s.name)}
              <Chart
                name={s.name} unit={s.unit} points={s.points} t0={run.start} t1={run.end}
                obs={s.obs ? run.series[s.obs].points : null} obsName={s.obs} {cursor}
              />
            {/each}
          </div>
        </div>
      {/each}
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
  <details class="mb-6">
    <summary class="label cursor-pointer">events</summary>
    <div class="mt-3">
      {#await loadText('log.jsonl')}
        <p class="muted text-sm">loading log…</p>
      {:then text}
        {#if text === null}
          <p class="muted text-sm">no log</p>
        {:else}
          <EventLog {text} />
        {/if}
      {/await}
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
