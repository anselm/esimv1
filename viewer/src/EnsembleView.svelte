<script>
  // The view of an ensemble run (run.json kind "ensemble"): facts from
  // ensemble.json and one band chart per series with spread, grouped by unit.
  // branches/<seed>.json are large and are not loaded here.
  import { isoDate, ms, num } from './fmt.js'
  import Status from './Status.svelte'
  import BandChart from './BandChart.svelte'

  let { run, base } = $props()

  const OBS = 'hydro.q_obs_daily_m3_s'   // observed discharge, identical in every branch
  const SIM = 'hydro.q_sim_daily_m3_s'   // the chart that gets the observed overlay
  const SRC = 'wx.source'                // 0 for reanalysis hours, 1 for forecast-member hours
  const DAY = 86400

  async function load() {
    const r = await fetch(`${base}/ensemble.json`, { cache: 'no-store' })
    if (!r.ok) throw new Error(`ensemble.json ${r.status}`)
    return r.json()
  }

  const hasSpread = (s) => s.p10.some((v, i) => v !== s.p90[i])

  // time at which the median weather source first becomes a forecast member
  function forecastStart(ens) {
    const s = ens.series?.[SRC]
    if (!s) return null
    const i = s.p50.findIndex((v) => v === 1)
    return i < 0 ? null : s.times[i]
  }

  // series with spread, grouped by unit, sorted; the flat ones listed by name
  function split(ens) {
    const groups = new Map()
    const flat = []
    for (const [name, s] of Object.entries(ens.series ?? {})) {
      if (!s.times?.length || !hasSpread(s)) { flat.push(name); continue }
      const u = s.unit ?? ''
      if (!groups.has(u)) groups.set(u, [])
      groups.get(u).push({ name, s })
    }
    for (const g of groups.values()) g.sort((a, b) => a.name.localeCompare(b.name))
    return { groups: [...groups.entries()].sort(([a], [b]) => a.localeCompare(b)), flat: flat.sort() }
  }

  const defaultRange = (s) => (s.times[s.times.length - 1] - s.times[0] > 90 * DAY ? 'last30' : 'all')
</script>

{#await load()}
  <section class="mb-10">
    <h1 class="text-xl font-medium">{run.name}</h1>
    <p class="muted mt-3 text-sm">loading ensemble…</p>
  </section>
{:then ens}
  {@const fc = forecastStart(ens)}
  {@const { groups, flat } = split(ens)}
  <!-- facts -->
  <section class="mb-10">
    <h1 class="text-xl font-medium">{ens.name ?? run.name} <span class="muted text-sm font-normal">ensemble</span></h1>
    <dl class="mt-3 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm sm:grid-cols-[auto_1fr_auto_1fr]">
      <dt class="muted">window</dt>
      <dd class="mono">{isoDate(ens.start ?? run.start)} → {isoDate(ens.end ?? run.end)}</dd>
      <dt class="muted">branches</dt>
      <dd class="mono">{num(ens.branches ?? run.branches)}</dd>
      <dt class="muted">kernel</dt>
      <dd class="mono">{ens.kernel_version ?? run.kernel_version}</dd>
      <dt class="muted">digests</dt>
      <dd class="mono">
        {num(ens.distinct_digests)} distinct
        {#if ens.distinct_digests < (ens.branches ?? run.branches)}
          <span class="text-leak">· {num((ens.branches ?? run.branches) - ens.distinct_digests)} branches repeat another branch's digest</span>
        {/if}
      </dd>
      <dt class="muted">elapsed</dt>
      <dd class="mono">{ms(ens.elapsed_ms ?? run.elapsed_ms)}</dd>
      <dt class="muted">ledgers</dt>
      <dd><Status closed={ens.all_ledgers_closed ?? run.all_ledgers_closed} /></dd>
      {#if fc != null}
        <dt class="muted">forecast from</dt>
        <dd class="mono">{isoDate(fc)}</dd>
      {/if}
    </dl>
  </section>

  <!-- spread per series -->
  {#if groups.length}
    <section class="mb-10">
      <div class="label mb-3">spread across {num(ens.branches ?? run.branches)} branches</div>
      {#each groups as [unit, list] (unit)}
        <div class="mb-6">
          <div class="mono text-xs muted mb-2">{unit}</div>
          <div class="grid gap-x-6 gap-y-5 sm:grid-cols-2">
            {#each list as { name, s } (name)}
              <BandChart
                {name} unit={s.unit} series={s}
                obs={name === SIM ? ens.series[OBS] ?? null : null}
                forecastT={name === SIM ? fc : null}
                initialRange={defaultRange(s)}
              />
            {/each}
          </div>
        </div>
      {/each}
      {#if flat.length}
        <p class="mono text-xs muted mt-2">no spread: {flat.join(', ')}</p>
      {/if}
    </section>
  {:else}
    <p class="muted text-sm">no series with spread</p>
  {/if}
{:catch e}
  <p class="text-sm text-leak">could not load ensemble <span class="mono">{run.name}</span>: {e.message}</p>
{/await}
