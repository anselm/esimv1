<script>
  import Badge from './Badge.svelte'
  import { link } from './router.svelte.js'
  import { isoDate, num } from './fmt.js'
  import Status from './Status.svelte'

  async function load() {
    const r = await fetch('/runs/index.json', { cache: 'no-store' })
    if (!r.ok) return []
    const list = await r.json()
    if (!Array.isArray(list)) return []
    const runs = list.slice().sort((a, b) => (b.start ?? 0) - (a.start ?? 0))
    // best NSE per run, from the optional scores.json (404 means none), and
    // the branch count of an ensemble run (its folder holds ensemble.json)
    await Promise.all(runs.map(async (run) => {
      run.nse = await bestNse(run.id)
      run.branches = await branches(run.id)
    }))
    return runs
  }
  // An ensemble folder has ensemble.json (large: only its existence is checked,
  // by HEAD); its run.json is small and carries kind and branches.
  async function branches(id) {
    try {
      const h = await fetch(`/runs/${encodeURIComponent(id)}/ensemble.json`, { method: 'HEAD', cache: 'no-store' })
      if (!h.ok) return null
      const r = await fetch(`/runs/${encodeURIComponent(id)}/run.json`, { cache: 'no-store' })
      if (!r.ok) return null
      const run = await r.json()
      return run.kind === 'ensemble' && typeof run.branches === 'number' ? run.branches : null
    } catch {
      return null
    }
  }
  async function bestNse(id) {
    try {
      const r = await fetch(`/runs/${encodeURIComponent(id)}/scores.json`, { cache: 'no-store' })
      if (!r.ok) return null
      const list = await r.json()
      const v = Math.max(...(Array.isArray(list) ? list : []).map((s) => s.nse).filter((x) => typeof x === 'number'))
      return Number.isFinite(v) ? v : null
    } catch {
      return null
    }
  }
</script>

<a use:link={{}} class="floating left-3 top-3 hover:text-brand">← projects</a>
<div class="floating right-3 top-3 mono text-xs select-none"><Badge /></div>

{#await load()}
  <p class="muted text-sm">loading…</p>
{:then runs}
  {#if runs.length === 0}
    <p class="muted text-sm">no runs yet</p>
  {:else}
    <div class="label mb-2">{runs.length} loose {runs.length === 1 ? 'run' : 'runs'} · not in any project</div>
    <ul class="divide-y divide-neutral-200 dark:divide-neutral-800">
      {#each runs as r (r.id)}
        <li>
          <a
            use:link={{ run: r.id }}
            class="grid grid-cols-[1fr_auto] gap-x-4 gap-y-1 py-3 sm:grid-cols-[1fr_auto_auto_auto] sm:items-center hover:bg-brand-soft/40 -mx-2 px-2 rounded-[3px]"
          >
            <span class="font-medium">{r.name}</span>
            <span class="mono text-xs muted sm:order-4">{r.digest}</span>
            <span class="mono text-xs muted col-span-2 sm:col-span-1 sm:order-2">
              {isoDate(r.start)} → {isoDate(r.end)}
            </span>
            <span class="flex items-center gap-3 text-xs col-span-2 sm:col-span-1 sm:order-3">
              {#if r.branches != null}
                <span class="mono rounded-[3px] px-1.5 py-px ring-1 ring-neutral-300 dark:ring-neutral-700" title="ensemble run">{num(r.branches)} branches</span>
              {:else}
                <span class="mono muted">{num(r.events)} events</span>
              {/if}
              {#if r.nse != null}
                <span class="mono muted" title="best Nash–Sutcliffe efficiency in scores.json">NSE {r.nse.toFixed(2)}</span>
              {/if}
              <Status closed={r.all_ledgers_closed} />
            </span>
          </a>
        </li>
      {/each}
    </ul>
  {/if}
{:catch e}
  <p class="muted text-sm">no runs yet</p>
{/await}
