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
  // sections in reading order; any other section follows alphabetically, pages with none last
  const SECTIONS = ['Start here', 'Where this is going', 'The machine', 'Projects and the record', 'Telling it', 'Changing it', 'Building a twin']
  // what each area of the documentation is, said once above its sections
  const AREAS = { system: ['The system', 'How the engine works, what it holds, and how to use and change it.'], events: ['The future', 'Lists of events ahead: the dated ones, and the wild cards a run may draw.'], rfc: ['Standards', 'Proposals for formats that other engines could share.'], guides: ['Guides', 'How to do the work, whatever engine you use. Written for people and for language models to read as context.'] }
  const byOrder = (a, b) => (a.order ?? 99) - (b.order ?? 99) || a.path.localeCompare(b.path)
  function sectioned(list) {
    const m = new Map()
    for (const p of list) { const k = p.section ?? ''; if (!m.has(k)) m.set(k, []); m.get(k).push(p) }
    const rank = (k) => (k === '' ? 1e6 : SECTIONS.includes(k) ? SECTIONS.indexOf(k) : 1e3)
    return [...m.entries()].sort(([a], [b]) => rank(a) - rank(b) || a.localeCompare(b)).map(([k, v]) => [k, v.slice().sort(byOrder)])
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

<a use:link={{}} class="floating left-3 top-3 hover:text-brand">← workspaces</a>
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
  <!-- reference pages that are drawn from the engine itself -->
  <section class="mt-8 grid gap-3 sm:grid-cols-3">
    <a use:link={{ kinds: true }} class="rounded-[3px] border border-neutral-200 p-3 hover:border-brand dark:border-neutral-800"><div class="font-medium">Module kinds</div><p class="mt-0.5 text-xs muted">Every kind of module a scenario can use, by layer, described by itself.</p></a>
    <a use:link={{ doc: 'system/sources' }} class="rounded-[3px] border border-neutral-200 p-3 hover:border-brand dark:border-neutral-800"><div class="font-medium">Data sources</div><p class="mt-0.5 text-xs muted">Every dataset the engine reads: where it comes from, its licence, who uses it.</p></a>
    <a use:link={{ doc: 'system/limits' }} class="rounded-[3px] border border-neutral-200 p-3 hover:border-brand dark:border-neutral-800"><div class="font-medium">Limits today</div><p class="mt-0.5 text-xs muted">The known faults of each model, kept current.</p></a>
  </section>
  {#each areas as [area, list] (area)}
    <!-- an area: a large heading with a rule; its sections: small coloured capitals; its pages: plain rows, indented -->
    <section class="mt-12">
      <div class="flex items-baseline justify-between gap-4 border-b-2 border-neutral-900 pb-1.5 dark:border-neutral-100">
        <h2 class="text-2xl font-medium">{AREAS[area]?.[0] ?? area}</h2>
        <span class="mono text-xs muted">docs/{area} · {list.length} {list.length === 1 ? 'page' : 'pages'}</span>
      </div>
      {#if AREAS[area]}<p class="mt-2 max-w-2xl text-sm muted">{AREAS[area][1]}</p>{/if}
      {#each sectioned(list) as [section, pages] (section)}
        <div class="mt-7 grid gap-x-6 sm:grid-cols-[11rem_1fr]">
          <h3 class="pt-2 text-xs font-semibold uppercase tracking-[0.14em] text-brand">{section || 'Other pages'}</h3>
          <ul class="border-l border-neutral-200 dark:border-neutral-800">
            {#each pages as p (p.path)}
              <li>
                <a use:link={{ doc: p.path }} class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-0.5 py-2 pl-4 pr-2 hover:bg-brand-soft/40">
                  <span class="text-[15px]">{p.title ?? p.path}</span>
                  <span class="mono text-[11px] muted">{p.path.split('/').slice(1).join('/')}{#if p.revised || p.written}{' · '}{p.revised ?? p.written}{/if}{#if p.revision}{' · '}r{p.revision}{/if}</span>
                </a>
              </li>
            {/each}
          </ul>
        </div>
      {/each}
    </section>
  {/each}
{/if}
