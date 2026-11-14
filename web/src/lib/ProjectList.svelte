<script>
  $effect(() => { document.title = 'esim' })
  let { area = null } = $props()
  import Badge from './Badge.svelte'
  import Account from './Account.svelte'
  // The front page lists the areas: the top folders, one per person or group, each with how
  // much is in it and the pictures of its featured projects. An area's page (/<area>) lists its
  // projects, ten to a page; /all lists every project under its area. Read from the project
  // server when it is up, from the static index otherwise.
  import { link, go, route } from './router.svelte.js'
  import { req, onChanged, hello } from './socket.js'
  let writable = $state(false)
  let readonly = $state(false)
  hello().then((h) => { writable = !h.readonly; readonly = Boolean(h.readonly && !h.offline) })
  import { isoDate } from './fmt.js'
  import Verdict from './Verdict.svelte'
  import Pager from './Pager.svelte'
  let page = $state(0)
  const SIZE = 10

  let projects = $state(null)
  let error = $state(null)
  let source = $state('')

  async function load() {
    try {
      const r = await req('projects.list')
      projects = r.projects
      source = 'live'
    } catch (e) {
      try {
        const r = await fetch('/projects/index.json', { cache: 'no-store' })
        projects = r.ok ? await r.json() : []
        source = 'static'
      } catch (e2) {
        error = e2.message
      }
    }
    // the latest run's verdict, from its audit.json
    await Promise.all((projects ?? []).map(async (p) => {
      const last = latest(p)
      if (!last) return
      try {
        const r = await fetch(`/projects/${p.path}/runs/${encodeURIComponent(last.id)}/audit.json`, { cache: 'no-store' })
        if (r.ok) p.audit = await r.json()
      } catch { /* no audit */ }
    }))
    projects = projects
  }
  const latest = (p) => (p.runs ?? []).slice().sort((a, b) => (b.start ?? 0) - (a.start ?? 0))[0] ?? null
  const featured = $derived((projects ?? []).filter((p) => p.featured === true))
  // one sentence of the description for a card
  const firstSentence = (s) => { const m = String(s ?? '').match(/^.*?[.!?](\s|$)/); return m ? m[0].trim() : String(s ?? '') }
  const areas = $derived.by(() => {
    const m = new Map()
    for (const p of projects ?? []) {
      const a = p.area ?? p.path.split('/')[0]
      if (area && a !== area) continue
      if (!m.has(a)) m.set(a, [])
      m.get(a).push(p)
    }
    return [...m.entries()].sort(([a], [b]) => a.localeCompare(b))
  })

  $effect(() => {
    load()
    return onChanged(() => load())
  })
</script>

<div class="floating right-3 top-3 flex items-center gap-3 mono text-xs select-none">
  {#if writable}<a use:link={{ isNew: true }} class="hover:text-brand" title="new project">+ project</a>{/if}
  <a use:link={{ kinds: true }} class="hover:text-brand" title="module kinds">kinds</a>
  <a use:link={{ docs: true }} class="hover:text-brand" title="documentation">docs</a>
  <a href="https://github.com/anselm/esimv1" class="hover:text-brand" title="the code and the scenarios, on GitHub">github</a>
  <a href="https://anselm.substack.com" class="hover:text-brand" title="Anselm's writing, on Substack">substack</a>
  <a href="https://x.com/anselm" class="hover:text-brand" title="@anselm on X">x</a>
  <Account /><Badge />
</div>

{#if error}
  <p class="text-sm text-leak">{error}</p>
{:else if projects === null}
  <p class="muted text-sm">loading…</p>
{:else if projects.length === 0}
  <p class="muted text-sm">no projects yet. <a use:link={{ isNew: true }} class="text-brand">Start one.</a></p>
{:else if !route.all && !area}
  <!-- the areas: each a folder of projects -->
  <h1 class="text-xl font-medium">Workspaces</h1>
  <p class="mt-1 max-w-2xl text-sm muted">Each workspace is a person's or a group's folder of scenarios. Open one to see its projects.</p>
  <section class="mt-8 grid gap-8 sm:grid-cols-2">
    {#each areas.slice(page * SIZE, page * SIZE + SIZE) as [a, list] (a)}
      {@const covers = list.filter((p) => p.cover).sort((x, y) => (y.featured === true) - (x.featured === true)).slice(0, 3)}
      <a use:link={{ area: a }} class="group block rounded-[3px] border border-neutral-200 p-4 hover:border-brand dark:border-neutral-800">
        <div class="flex items-baseline justify-between gap-4">
          <h2 class="text-lg font-medium group-hover:text-brand">/{a}</h2>
          <span class="mono text-xs muted">{list.length} {list.length === 1 ? 'project' : 'projects'} · {list.reduce((n, p) => n + (p.runs ?? []).length, 0)} runs</span>
        </div>
        {#if covers.length}
          <div class="mt-3 grid grid-cols-3 gap-2">
            {#each covers as p (p.path)}<img src={`/projects/${p.path}/${p.cover}`} alt={p.title ?? p.path} class="aspect-[4/3] w-full rounded-[3px] object-cover" loading="lazy" />{/each}
          </div>
        {/if}
        <p class="mt-3 text-sm muted line-clamp-2">{list.slice(0, 6).map((p) => p.title ?? p.path.split('/')[1]).join(' · ')}{list.length > 6 ? ' …' : ''}</p>
      </a>
    {/each}
  </section>
  <Pager total={areas.length} size={SIZE} bind:page label="workspaces" />
  <p class="mt-8 text-xs muted"><a use:link={{ all: true }} class="hover:text-brand">all {projects.length} projects in one list →</a></p>
{:else}
  {#if area}<p class="mb-6 text-xs muted"><a use:link={{}} class="hover:text-brand">← workspaces</a> <span class="mono">/ {area}</span></p>
  {:else}<p class="mb-6 text-xs muted"><a use:link={{}} class="hover:text-brand">← workspaces</a></p>{/if}
  {#if area && !areas.length}<p class="muted text-sm">no projects under {area}</p>{/if}
  {#each areas as [area, list] (area)}
    <section class="mb-10">
      <div class="label mb-2">{area} · {list.length} {list.length === 1 ? 'project' : 'projects'}</div>
      <ul class="divide-y divide-neutral-200 dark:divide-neutral-800">
        {#each (area ? list.slice(page * SIZE, page * SIZE + SIZE) : list) as p (p.path)}
          {@const last = latest(p)}
          <li>
            <a use:link={{ project: p.path }} class="block py-3 -mx-2 px-2 rounded-[3px] hover:bg-brand-soft/40">
              <div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
                <span class="font-medium">{p.title ?? p.path}</span>
                <span class="mono text-xs muted">{p.path}</span>
              </div>
              {#if p.description}
                <p class="mt-1 text-sm muted line-clamp-2">{p.description}</p>
              {/if}
              <div class="mt-1.5 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs">
                <span class="mono muted">{(p.kinds ?? []).join(' · ')}</span>
                <span class="mono muted">{(p.runs ?? []).length} {(p.runs ?? []).length === 1 ? 'run' : 'runs'}</span>
                {#if last}
                  <span class="mono muted">{isoDate(last.start)} → {isoDate(last.end)}</span>
                {/if}
                {#if p.audit}
                  <Verdict verdict={p.audit.verdict} />
                {/if}
              </div>
            </a>
          </li>
        {/each}
      </ul>
      {#if area}<Pager total={list.length} size={SIZE} bind:page label="projects" />{/if}
    </section>
  {/each}
{/if}

<p class="mt-12 text-xs muted">
  {#if !readonly}<a use:link={{ runs: true }} class="hover:text-brand">loose runs</a>{/if}
  {#if readonly} a read-only copy: clone <a class="hover:text-brand" href="https://github.com/anselm/esimv1">anselm/esimv1</a> to create, run or edit{/if}
</p>
