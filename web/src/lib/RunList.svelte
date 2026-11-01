<script>
  import { go } from './router.svelte.js'
  import { isoDate, num } from './fmt.js'
  import Status from './Status.svelte'

  async function load() {
    const r = await fetch('/runs/index.json', { cache: 'no-store' })
    if (!r.ok) return []
    const list = await r.json()
    return Array.isArray(list) ? list.slice().sort((a, b) => (b.start ?? 0) - (a.start ?? 0)) : []
  }
</script>

<div class="floating right-3 top-3 mono text-xs muted select-none">esim</div>

{#await load()}
  <p class="muted text-sm">loading…</p>
{:then runs}
  {#if runs.length === 0}
    <p class="muted text-sm">no runs yet</p>
  {:else}
    <div class="label mb-2">{runs.length} {runs.length === 1 ? 'run' : 'runs'}</div>
    <ul class="divide-y divide-neutral-200 dark:divide-neutral-800">
      {#each runs as r (r.id)}
        <li>
          <a
            href="?run={encodeURIComponent(r.id)}"
            onclick={(e) => { e.preventDefault(); go(r.id) }}
            class="grid grid-cols-[1fr_auto] gap-x-4 gap-y-1 py-3 sm:grid-cols-[1fr_auto_auto_auto] sm:items-center hover:bg-brand-soft/40 -mx-2 px-2 rounded-[3px]"
          >
            <span class="font-medium">{r.name}</span>
            <span class="mono text-xs muted sm:order-4">{r.digest}</span>
            <span class="mono text-xs muted col-span-2 sm:col-span-1 sm:order-2">
              {isoDate(r.start)} → {isoDate(r.end)}
            </span>
            <span class="flex items-center gap-3 text-xs col-span-2 sm:col-span-1 sm:order-3">
              <span class="mono muted">{num(r.events)} events</span>
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
