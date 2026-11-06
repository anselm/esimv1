<script>
  let { area = null } = $props()
  import Badge from './Badge.svelte'
  import Account from './Account.svelte'
  // The front page: the featured scenarios as cards (a project whose project.toml says
  // `featured = true`, with its cover picture), and on ?all every project under its area,
  // each a folder under projects/<area>/<name>. Read from the project server when it is
  // up, from the static index otherwise.
  import { link, go, route } from './router.svelte.js'
  import { req, onChanged, hello } from './socket.js'
  let writable = $state(false)
  let readonly = $state(false)
  hello().then((h) => { writable = !h.readonly; readonly = Boolean(h.readonly && !h.offline) })
  import { isoDate } from './fmt.js'
  import Verdict from './Verdict.svelte'

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
  <Account /><Badge />
</div>

{#if error}
  <p class="text-sm text-leak">{error}</p>
{:else if projects === null}
  <p class="muted text-sm">loading…</p>
{:else if projects.length === 0}
  <p class="muted text-sm">no projects yet. <a use:link={{ isNew: true }} class="text-brand">Start one.</a></p>
{:else if !route.all && !area && featured.length}
  <!-- the featured scenarios: one picture, one sentence each -->
  <section class="mb-10 grid gap-8 sm:grid-cols-2">
    {#each featured as p (p.path)}
      <a use:link={{ project: p.path }} class="group block">
        {#if p.cover}
          <img src={`/projects/${p.path}/${p.cover}`} alt={p.cover_caption ?? p.title ?? p.path} class="aspect-[12/5] w-full rounded-[3px] object-cover" loading="lazy" />
        {:else}
          <div class="aspect-[12/5] w-full rounded-[3px] bg-brand-soft/40"></div>
        {/if}
        <h2 class="mt-3 text-lg font-medium group-hover:text-brand">{p.title ?? p.path}</h2>
        <p class="mt-1 text-sm muted">{firstSentence(p.description)}</p>
        <div class="mt-1.5 flex flex-wrap items-center gap-x-4 gap-y-1 mono text-xs muted">
          <span>/{p.path}</span>
          <span>{(p.runs ?? []).length} {(p.runs ?? []).length === 1 ? 'run' : 'runs'}</span>
          {#if p.audit}<Verdict verdict={p.audit.verdict} />{/if}
        </div>
      </a>
    {/each}
  </section>
  <p class="text-xs muted"><a use:link={{ all: true }} class="hover:text-brand">all {projects.length} projects →</a></p>
{:else}
  {#if area}<p class="mb-6 text-xs muted"><a use:link={{}} class="hover:text-brand">← projects</a> <span class="mono">/ {area}</span></p>
  {:else if featured.length}<p class="mb-6 text-xs muted"><a use:link={{}} class="hover:text-brand">← featured</a></p>{/if}
  {#if area && !areas.length}<p class="muted text-sm">no projects under {area}</p>{/if}
  {#each areas as [area, list] (area)}
    <section class="mb-10">
      <div class="label mb-2">{area} · {list.length} {list.length === 1 ? 'project' : 'projects'}</div>
      <ul class="divide-y divide-neutral-200 dark:divide-neutral-800">
        {#each list as p (p.path)}
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
    </section>
  {/each}
{/if}

<p class="mt-12 text-xs muted">
  {#if !readonly}<a use:link={{ runs: true }} class="hover:text-brand">loose runs</a>{/if}
  {#if readonly} a read-only copy: clone <a class="hover:text-brand" href="https://github.com/anselm/esimv1">anselm/esimv1</a> to create, run or edit{/if}
</p>
