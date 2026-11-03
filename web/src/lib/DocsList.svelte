<script>
  import Badge from './Badge.svelte'
  import Account from './Account.svelte'
  // The documentation: Markdown pages under docs/<area>/<name>.md, areas first come first
  // served like projects. The index is docs/index.json, written by the server.
  import { link } from './router.svelte.js'
  import { req, hello } from './socket.js'
  let writable = $state(false)
  hello().then((h) => { writable = !h.readonly })

  let pages = $state(null)
  let error = $state(null)

  async function load() {
    try {
      pages = (await req('docs.list')).pages
    } catch {
      try {
        const r = await fetch('/docs/index.json', { cache: 'no-store' })
        pages = r.ok ? await r.json() : []
      } catch (e) { error = e.message }
    }
  }
  const areas = $derived.by(() => {
    const m = new Map()
    for (const p of pages ?? []) {
      const a = p.path.split('/')[0]
      if (!m.has(a)) m.set(a, [])
      m.get(a).push(p)
    }
    // system first, then the rest alphabetically
    return [...m.entries()].sort(([a], [b]) => (a === 'system' ? -1 : b === 'system' ? 1 : a.localeCompare(b)))
  })
  $effect(() => { load() })
</script>

<a use:link={{}} class="floating left-3 top-3 hover:text-brand">← projects</a>
<div class="floating right-3 top-3 flex items-center gap-3 mono text-xs select-none">
  {#if writable}<a use:link={{ doc: 'new' }} class="hover:text-brand" title="new page">+ page</a>{/if}
  <Account /><Badge />
</div>

<h1 class="text-xl font-medium">Documentation</h1>
<p class="mt-1 max-w-2xl text-sm muted">The <a use:link={{ kinds: true }} class="text-brand">kinds</a> page lists every module a manifest can build on. Pages here are Markdown files under <span class="mono">docs/&lt;area&gt;/&lt;name&gt;.md</span>. The <span class="mono">system</span> area describes the simulator; any other area is whoever claims it first, as with projects.</p>

{#if error}
  <p class="mt-6 text-sm text-leak">{error}</p>
{:else if pages === null}
  <p class="mt-6 muted text-sm">loading…</p>
{:else}
  {#each areas as [area, list] (area)}
    <section class="mt-8">
      <div class="label mb-2">{area} · {list.length} {list.length === 1 ? 'page' : 'pages'}</div>
      <ul class="divide-y divide-neutral-200 dark:divide-neutral-800">
        {#each list.slice().sort((a, b) => (a.order ?? 99) - (b.order ?? 99) || a.path.localeCompare(b.path)) as p (p.path)}
          <li>
            <a use:link={{ doc: p.path }} class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 py-2.5 -mx-2 px-2 rounded-[3px] hover:bg-brand-soft/40">
              <span class="font-medium">{p.title ?? p.path}</span>
              <span class="mono text-xs muted">{p.path}</span>
            </a>
          </li>
        {/each}
      </ul>
    </section>
  {/each}
{/if}
