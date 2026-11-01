<script>
  import { go } from './router.svelte.js'
  import { isoDate, num, ms } from './fmt.js'
  import Status from './Status.svelte'
  import Ledger from './Ledger.svelte'
  import Chart from './Chart.svelte'
  import FieldMap from './FieldMap.svelte'
  import EventLog from './EventLog.svelte'

  let { id } = $props()
  const base = $derived(`/runs/${encodeURIComponent(id)}`)

  async function load() {
    const r = await fetch(`${base}/run.json`, { cache: 'no-store' })
    if (!r.ok) throw new Error(`run.json ${r.status}`)
    return r.json()
  }
  async function loadText(name) {
    const r = await fetch(`${base}/${name}`, { cache: 'no-store' })
    return r.ok ? r.text() : null
  }

  // Group series by unit so comparable charts sit together.
  function byUnit(series) {
    const groups = new Map()
    for (const [name, s] of Object.entries(series ?? {})) {
      const u = s.unit ?? ''
      if (!groups.has(u)) groups.set(u, [])
      groups.get(u).push({ name, ...s })
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
    <section class="mb-10">
      <div class="label mb-3">series</div>
      {#each byUnit(run.series) as [unit, list] (unit)}
        <div class="mb-6">
          <div class="mono text-xs muted mb-2">{unit}</div>
          <div class="grid gap-x-6 gap-y-5 sm:grid-cols-2">
            {#each list as s (s.name)}
              <Chart name={s.name} unit={s.unit} points={s.points} t0={run.start} t1={run.end} />
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
