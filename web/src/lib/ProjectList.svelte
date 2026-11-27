<script>
  $effect(() => { document.title = 'esim' })
  let { area = null } = $props()
  import Badge from './Badge.svelte'
  import { newestFirst } from 'esimv1-viewer/runs.js'
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
  import { isoDate } from 'esimv1-viewer/fmt.js'
  import Verdict from 'esimv1-viewer/Verdict.svelte'
  import Pager from './Pager.svelte'
  import HeroGlobe from './HeroGlobe.svelte'
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
  const latest = (p) => (p.runs ?? []).slice().sort(newestFirst)[0] ?? null
  // the few scenarios the front page leads with (featured = true in project.toml), newest first
  const featured = $derived((projects ?? []).filter((p) => p.featured === true).sort((a, b) => String(b.created ?? '').localeCompare(String(a.created ?? ''))))
  // where each scenario is, for the dots on the globe: the middle of the window its manifest gives a module
  let places = $state([])
  $effect(() => {
    const list = projects ?? []
    if (area || route.all || !list.length) return
    Promise.all(list.map(async (p) => {
      try {
        const m = await (await fetch(`/projects/${p.path}/manifest.json`)).json()
        for (const e of m.entities ?? []) { const q = e.params?.place ?? e.place ?? (e.params?.lat_s !== undefined ? e.params : null); if (q && q.lat_s !== undefined) return { lat: (q.lat_s + q.lat_n) / 2, lon: (q.lon_w + q.lon_e) / 2 } }
      } catch { /* no manifest, no dot */ }
      return null
    })).then((r) => { places = r.filter(Boolean) })
  })
  let kindCount = $state(0)
  $effect(() => { fetch('/docs/kinds.json').then((r) => (r.ok ? r.json() : [])).then((k) => { kindCount = (Array.isArray(k) ? k : k.kinds ?? []).length }).catch(() => {}) })
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
  <!-- what this is, in a few lines, over a picture from a real run -->
  {@const lead = featured.find((p) => p.cover) ?? null}
  <HeroGlobe {places} />
  <section class="max-w-xl pt-6 sm:pt-10">
    <div class="mono text-xs uppercase tracking-[0.2em] muted">esim · a simulator for whole places</div>
    <h1 class="mt-5 text-4xl font-light leading-tight tracking-tight sm:text-5xl">Simulation<br />as argument.</h1>
    <!-- this page's file was first committed on 2026-11-01 -->
    <div class="mono mt-3 text-xs muted"><a use:link={{ area: 'anselm' }} class="hover:text-brand">@anselm</a> · 1 November 2026</div>
    <p class="mt-5 text-lg leading-relaxed">A model of a place is a claim about how that place works. esim builds such models from published data, with every assumption open to inspection, and lets anyone push on them and watch what follows.</p>
    <p class="mt-3 leading-relaxed muted">It treats a place as one system: its ground and water, what is built on it, the services that run through it, and the people who live there, all on one clock. The system is assembled from separate parts. In a fire scenario the relief, the land cover, the roads, the buildings, the weather, the dryness of the fuel, the fire, the animals that run from it and the crews that fight it are each a module of their own, and the same parts serve other places and a second fire model. Each scenario runs in seconds to a minute, can be played back and changed, and says plainly what it gets wrong. The aim is a tool that a city or a community can use to see what a decision would do before it is made.</p>
    <div class="mt-6 flex flex-wrap gap-3 text-sm">
      <a use:link={{ doc: 'system/goals' }} class="rounded-[3px] bg-brand px-4 py-2 text-white hover:opacity-90">Where this is going</a>
      <a use:link={{ doc: 'system/overview' }} class="rounded-[3px] border border-neutral-300 bg-white/70 px-4 py-2 hover:border-brand dark:border-neutral-700 dark:bg-neutral-950/70">How it works</a>
      <a use:link={{ kinds: true }} class="rounded-[3px] border border-neutral-300 bg-white/70 px-4 py-2 hover:border-brand dark:border-neutral-700 dark:bg-neutral-950/70">The {kindCount || ''} module kinds</a>
    </div>
    {#if places.length}<p class="mt-5 text-xs muted">The dots on the globe are the places these scenarios model. Earth image: NASA Blue Marble.</p>{/if}
  </section>

  <!-- three facts about the engine, each drawn from what is in the repository -->
  <section class="mt-12 grid max-w-3xl gap-6 border-y border-neutral-200 py-6 sm:grid-cols-3 dark:border-neutral-800">
    <div><div class="mono text-3xl">{projects.length}</div><div class="mt-1 text-sm muted">scenarios so far: fires, a river basin, a harbor, a city after an earthquake, the planet to 2100</div></div>
    <div><div class="mono text-3xl">{kindCount || '—'}</div><div class="mt-1 text-sm muted">kinds of module, each a part that more than one scenario can use: the same relief, land cover, weather and fuel-moisture modules serve five fire scenarios on two fire models</div></div>
    <div><div class="mono text-3xl">{projects.reduce((n, p) => n + (p.runs ?? []).length, 0)}</div><div class="mt-1 text-sm muted">runs on record, every one replayable from its manifest and its seed</div></div>
  </section>

  {#if featured.length}
    <h2 class="mt-12 text-2xl font-medium">Scenarios to start with</h2>
    <section class="mt-5 grid gap-8 sm:grid-cols-2 lg:grid-cols-3">
      {#each featured as p (p.path)}
        <a use:link={{ project: p.path }} class="group block">
          {#if p.cover}<img src={`/projects/${p.path}/${p.cover}`} alt={p.cover_caption ?? p.title ?? p.path} class="aspect-[16/10] w-full rounded-[3px] object-cover" loading="lazy" />{:else}<div class="aspect-[16/10] w-full rounded-[3px] bg-brand-soft/40"></div>{/if}
          <h3 class="mt-3 text-lg font-medium group-hover:text-brand">{p.title ?? p.path}</h3>
          <p class="mt-1 text-sm muted line-clamp-3">{firstSentence(p.description)}</p>
          <div class="mt-1.5 flex flex-wrap items-center gap-x-4 gap-y-1 mono text-xs muted"><span>/{p.path}</span>{#if p.audit}<Verdict verdict={p.audit.verdict} />{/if}</div>
        </a>
      {/each}
    </section>
  {/if}

  <!-- the areas: each a folder of projects -->
  <h2 class="mt-14 text-2xl font-medium">Workspaces</h2>
  <p class="mt-1 max-w-2xl text-sm muted">Each workspace is a person's or a group's folder of scenarios. Open one to see its projects.</p>
  <section class="mt-6 grid gap-8 sm:grid-cols-2">
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
