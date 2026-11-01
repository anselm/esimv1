<script>
  import { go } from './router.svelte.js'
  import { isoDate, num, ms } from './fmt.js'
  import Status from './Status.svelte'
  import Ledger from './Ledger.svelte'
  import Chart from './Chart.svelte'
  import Scores from './Scores.svelte'
  import FieldMap from './FieldMap.svelte'
  import EventLog from './EventLog.svelte'
  import EnsembleView from './EnsembleView.svelte'

  let { id } = $props()
  const base = $derived(`/runs/${encodeURIComponent(id)}`)

  async function load() {
    const r = await fetch(`${base}/run.json`, { cache: 'no-store' })
    if (!r.ok) throw new Error(`run.json ${r.status}`)
    const run = await r.json()
    run.scores = await loadScores()
    return run
  }
  // scores.json is optional: a 404 (or malformed file) means no SCORES section.
  async function loadScores() {
    try {
      const r = await fetch(`${base}/scores.json`, { cache: 'no-store' })
      if (!r.ok) return null
      const list = await r.json()
      return Array.isArray(list) && list.length ? list : null
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
    const out = new Map()   // sim name -> obs name
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
</script>

<a
  href={location.pathname}
  onclick={(e) => { e.preventDefault(); go(null) }}
  class="floating left-3 top-3 hover:text-brand"
  aria-label="back to runs"
>← runs</a>
<div class="floating right-3 top-3 mono text-xs muted select-none">esim</div>

{#await load()}
  <p class="muted text-sm">loading…</p>
{:then run}
{#if run.kind === 'ensemble'}
  <!-- an ensemble run: facts and band charts come from ensemble.json -->
  <EnsembleView {run} {base} />
{:else}
  <!-- facts -->
  <section class="mb-10">
    <h1 class="text-xl font-medium">{run.name}</h1>
    <dl class="mt-3 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm sm:grid-cols-[auto_1fr_auto_1fr]">
      <dt class="muted">window</dt>
      <dd class="mono">{isoDate(run.start)} → {isoDate(run.end)}</dd>
      <dt class="muted">seed</dt>
      <dd class="mono">{run.seed}</dd>
      <dt class="muted">kernel</dt>
      <dd class="mono">{run.kernel_version}</dd>
      <dt class="muted">events</dt>
      <dd class="mono">{num(run.delivered)} delivered of {num(run.events)}</dd>
      <dt class="muted">digest</dt>
      <dd class="mono">{run.log_digest}</dd>
      <dt class="muted">elapsed</dt>
      <dd class="mono">{ms(run.elapsed_ms)}</dd>
      <dt class="muted">ledgers</dt>
      <dd><Status closed={run.all_ledgers_closed} /></dd>
    </dl>
  </section>

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

  <!-- scores: sim against observation -->
  {#if run.scores}
    <section class="mb-10">
      <div class="label mb-3">scores</div>
      <Scores scores={run.scores} series={run.series} />
    </section>
  {/if}

  <!-- maps: field snapshots -->
  {#if run.fields && Object.keys(run.fields).length}
    <section class="mb-10">
      <div class="label mb-3">maps</div>
      <div class="grid gap-x-6 gap-y-6 sm:grid-cols-2 lg:grid-cols-3">
        {#each Object.keys(run.fields).sort() as f (f)}
          <FieldMap name={f} field={run.fields[f]} {base} />
        {/each}
      </div>
    </section>
  {/if}

  <!-- series -->
  {#if run.series && Object.keys(run.series).length}
    {@const paired = pairs(run)}
    <section class="mb-10">
      <div class="label mb-3">series</div>
      {#each byUnit(run.series, paired) as [unit, list] (unit)}
        <div class="mb-6">
          <div class="mono text-xs muted mb-2">{unit}</div>
          <div class="grid gap-x-6 gap-y-5 sm:grid-cols-2">
            {#each list as s (s.name)}
              <Chart
                name={s.name} unit={s.unit} points={s.points} t0={run.start} t1={run.end}
                obs={s.obs ? run.series[s.obs].points : null} obsName={s.obs}
              />
            {/each}
          </div>
        </div>
      {/each}
    </section>
  {/if}

  <!-- events -->
  <section class="mb-10">
    <div class="label mb-3">events</div>
    {#await loadText('log.jsonl')}
      <p class="muted text-sm">loading log…</p>
    {:then text}
      {#if text === null}
        <p class="muted text-sm">no log</p>
      {:else}
        <EventLog {text} />
      {/if}
    {/await}
  </section>

{/if}

  <!-- manifest -->
  <section class="mb-10">
    <div class="label mb-3">manifest</div>
    {#await loadText('manifest.toml')}
      <p class="muted text-sm">loading…</p>
    {:then toml}
      {#if toml === null}
        <p class="muted text-sm">no manifest</p>
      {:else}
        <pre class="mono overflow-x-auto rounded-[3px] bg-neutral-50 p-3 text-xs leading-relaxed dark:bg-neutral-900">{toml}</pre>
      {/if}
    {/await}
  </section>
{:catch e}
  <p class="text-sm text-leak">could not load run <span class="mono">{id}</span>: {e.message}</p>
{/await}
