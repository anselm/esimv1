<script>
  // The front page: projects, each a folder under projects/<area>/<name>. Read from the
  // project server when it is up, from the static index otherwise.
  import { link, go } from './router.svelte.js'
  import { req, onChanged } from './socket.js'
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
  const areas = $derived.by(() => {
    const m = new Map()
    for (const p of projects ?? []) {
      const a = p.area ?? p.path.split('/')[0]
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
  <a use:link={{ isNew: true }} class="hover:text-brand" title="new project">+ project</a>
  <span class="muted">esim</span>
</div>

{#if error}
  <p class="text-sm text-leak">{error}</p>
{:else if projects === null}
  <p class="muted text-sm">loading…</p>
{:else if projects.length === 0}
  <p class="muted text-sm">no projects yet. <a use:link={{ isNew: true }} class="text-brand">Start one.</a></p>
{:else}
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
  <a use:link={{ runs: true }} class="hover:text-brand">loose runs</a>
  {#if source === 'static'} · project server is down; showing the files{/if}
</p>
