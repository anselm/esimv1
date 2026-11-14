<script>
  import Badge from './Badge.svelte'
  import { newestFirst } from './runs.js'
  import Account from './Account.svelte'
  // One project: what was asked, the manifest, the runs, and a button to run it again.
  import { link, go } from './router.svelte.js'
  import { req, onLog, onChanged, hello } from './socket.js'
  import { isoDate, num } from './fmt.js'
  import Status from './Status.svelte'
  import Verdict from './Verdict.svelte'
  import Readme from './Readme.svelte'
  import Crumbs from './Crumbs.svelte'
  import ProjectScene from './ProjectScene.svelte'
  import Pager from './Pager.svelte'
  import Notebook from './Notebook.svelte'
  // a project may lay its own page out: page.html beside its manifest
  let pageSrc = $state(null)
  $effect(() => { const u = `/projects/${path}/page.html`; pageSrc = null; fetch(u, { method: 'HEAD', cache: 'no-store' }).then((r) => { if (r.ok && (r.headers.get('content-type') ?? '').includes('html')) pageSrc = u }).catch(() => {}) })
  let runPage = $state(0)
  // the page's one list: each variant with its runs (newest first), then runs of no variant
  const rows = $derived.by(() => {
    const by = new Map()
    for (const r of runs ?? []) { const k = r.variant ?? ''; if (!by.has(k)) by.set(k, []); by.get(k).push(r) }
    const out = (meta?.variants ?? []).map((v) => ({ key: `v:${v.name}`, name: v.name, title: v.title ?? v.name, set: v.set ?? {}, runs: by.get(v.name) ?? [] }))
    const named = new Set((meta?.variants ?? []).map((v) => v.name))
    for (const [k, list] of by) if (!named.has(k)) out.push({ key: `r:${k}`, name: k, title: k ? k : list[0]?.set && Object.keys(list[0].set).length ? 'A custom run' : 'The scenario as written', set: k ? {} : list[0]?.set ?? {}, runs: list })
    if (!out.length && meta) out.push({ key: 'r:', name: '', title: 'The scenario as written', set: {}, runs: [] })
    return out
  })

  let { path } = $props()
  $effect(() => { if (meta?.title) document.title = `${meta.title} · esim` })
  let sceneShown = $state(false)
  let meta = $state(null)
  let manifest = $state('')
  // the program a project was made from, when it was a JavaScript or JSON manifest: kept
  // under source/ beside the resolved document, named by manifest.json's from block
  let source = $state(null)
  let runs = $state([])
  let error = $state(null)
  let live = $state(false)
  let running = $state(false)
  let log = $state([])
  let readme = $state(null)      // the project's README.md, when it has one

  // project.toml is small: flat key = "value" lines, then [[variant]] tables with a
  // [variant.set] of "module.param" = value lines.
  function parseMeta(text) {
    const out = { variants: [] }
    let cur = null, inSet = false
    const unq = (v) => v.replace(/^"(.*)"$/s, '$1').replace(/\\"/g, '"').replace(/\\n/g, '\n').replace(/\\\\/g, '\\')
    for (const raw of text.split('\n')) {
      const line = raw.trim()
      if (!line || line.startsWith('#')) continue
      if (line === '[[variant]]') { cur = { set: {} }; out.variants.push(cur); inSet = false; continue }
      if (line === '[variant.set]') { inSet = true; continue }
      const m = line.match(/^("?)([\w.\-]+)\1\s*=\s*(.*)$/)
      if (!m) continue
      const [, , k, v] = m
      if (cur && inSet) cur.set[k] = unq(v)
      else if (cur) cur[k] = unq(v)
      else out[k] = unq(v)
    }
    return out
  }

  async function load() {
    try {
      const r = await req('project.get', { path })
      meta = parseMeta(r.meta); manifest = r.manifest; runs = r.runs; live = !(await hello()).readonly
    } catch (e) {
      try {
        const [m, t, rr] = await Promise.all([
          fetch(`/projects/${path}/project.toml`, { cache: 'no-store' }).then((r) => r.text()),
          fetch(`/projects/${path}/manifest.toml`, { cache: 'no-store' }).then((r) => r.text()),
          fetch(`/projects/${path}/runs/index.json`, { cache: 'no-store' }).then((r) => (r.ok ? r.json() : [])),
        ])
        meta = parseMeta(m); manifest = t; runs = rr; live = false
      } catch (e2) { error = e2.message }
    }
    runs = runs.slice().sort(newestFirst)
    fetch(`/projects/${path}/manifest.json`, { cache: 'no-store' }).then((r) => (r.ok ? r.json() : null)).then(async (doc) => {
      const from = doc?.from
      if (!from || !['javascript', 'json'].includes(from.language)) return
      const name = String(from.entry).split('/').pop()
      const r = await fetch(`/projects/${path}/source/${name}`, { cache: 'no-store' })
      if (r.ok) source = { name, language: from.language, text: await r.text(), sources: Object.keys(from.sources || {}) }
    }).catch(() => {})
    fetch(`/projects/${path}/README.md`, { cache: 'no-store' }).then((r) => (r.ok ? r.text() : null)).then((t) => { readme = t && t.trim() ? t : null }).catch(() => {})
    await Promise.all(runs.map(async (run) => {
      try {
        const r = await fetch(`/projects/${path}/runs/${encodeURIComponent(run.id)}/audit.json`, { cache: 'no-store' })
        if (r.ok) run.audit = await r.json()
      } catch { /* none */ }
    }))
    runs = runs
  }

  async function runNow(variant = null) {
    running = true; log = []; error = null
    try {
      const r = await req('project.run', { path, variant })
      if (!r.ran) error = 'the run failed; see the log'
      await load()
    } catch (e) { error = e.message } finally { running = false }
  }

  $effect(() => {
    load()
    const a = onLog((m) => { if (m.path === path) log = [...log.slice(-40), m.line] })
    const b = onChanged((m) => { if (m.path === path && !running) load() })
    return () => { a(); b() }
  })
  const kinds = $derived([...manifest.matchAll(/^kind\s*=\s*"([^"]+)"/gm)].map((m) => m[1]))
</script>

<a use:link={{}} class="floating left-3 top-3 hover:text-brand">← projects</a>
<div class="floating right-3 top-3 mono text-xs select-none"><Account /><Badge /></div>

{#if error && !meta}
  <p class="text-sm text-leak">{error}</p>
{:else if !meta}
  <p class="muted text-sm">loading…</p>
{:else}
  <section class="mb-8">
    <Crumbs {path} />
    <h1 class="mt-1 text-xl font-medium">{meta.title ?? path}</h1>
    {#if meta.description}
      <p class="mt-2 max-w-3xl text-sm leading-relaxed">{meta.description}</p>
    {/if}
    <div class="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs muted">
      {#if meta.created}<span>started {meta.created}</span>{/if}
      {#if meta.by}<span>by <a use:link={{ area: path.split('/')[0] }} class="hover:text-brand">{meta.by}</a></span>{/if}
      <span class="mono">{kinds.join(' · ')}</span>
    </div>
  </section>

  <!-- the place in 3D from the newest run, when a run can show it; otherwise the cover picture -->
  {#if pageSrc}<section class="mb-10"><Notebook src={pageSrc} {path} /></section>{:else}<ProjectScene {path} bind:shown={sceneShown} />{/if}
  {#if meta.cover && !sceneShown && !pageSrc}
    <!-- a depiction of the scenario, a file beside the manifest named in project.toml -->
    <figure class="mb-10 max-w-3xl">
      <img src={`/projects/${path}/${meta.cover}`} alt={meta.cover_caption ?? meta.title ?? path} class="w-full rounded-[3px]" />
      {#if meta.cover_caption}<figcaption class="mt-1.5 text-xs muted">{meta.cover_caption}</figcaption>{/if}
    </figure>
  {/if}

  <section class="mb-8">
    <!-- One list. A row is a version of the scenario (a variant, or the scenario as written) with its
         newest run, and the thing to do with it is on the row: open the run. Earlier runs of the same
         version fold under it. (Until 2026-11-14 this was two lists, variants and then runs.) -->
    <div class="mb-1 flex flex-wrap items-baseline justify-between gap-3">
      <h2 class="text-2xl font-medium">Runs</h2>
      <span class="flex items-center gap-4 text-sm">
        {#if rows.filter((r) => r.runs.length).length > 1}<a use:link={{ compare: path }} class="text-brand hover:underline">compare them side by side →</a>{/if}
        {#if live}<button class="rounded-[3px] px-3 py-1 ring-1 ring-neutral-300 disabled:opacity-50 dark:ring-neutral-700" onclick={() => runNow(null)} disabled={running}>{running ? 'running…' : 'Run as written'}</button>{/if}
      </span>
    </div>
    <p class="mb-4 max-w-2xl text-sm muted">{rows.length > 1 ? `Each run is the scenario played through once. These are ${rows.length} versions of it, each with one thing changed. Open one to watch it and read its numbers.` : 'A run is the scenario played through once. Open it to watch it and read its numbers.'}</p>
    {#if log.length}
      <pre class="mono mb-3 max-h-48 overflow-auto rounded-[3px] bg-neutral-50 p-2 text-xs leading-relaxed dark:bg-neutral-900">{log.join('\n')}</pre>
    {/if}
    {#if error}<p class="mb-2 text-sm text-leak">{error}</p>{/if}
    <ul class="grid gap-3 lg:grid-cols-2">
      {#each rows.slice(runPage * 10, runPage * 10 + 10) as v (v.key)}
        {@const r = v.runs[0] ?? null}
        <li class="flex flex-col rounded-[3px] border border-neutral-200 p-4 dark:border-neutral-800">
          <div class="flex items-start justify-between gap-4">
            <div class="min-w-0">
              <div class="font-medium leading-snug">{v.title}</div>
              {#if v.name}<div class="mono mt-0.5 text-xs muted">{v.name}</div>{/if}
            </div>
            {#if r}
              <a use:link={{ project: path, run: r.id }} class="shrink-0 rounded-[3px] bg-brand px-3.5 py-1.5 text-sm text-white hover:opacity-90">Open run →</a>
            {:else if live}
              <button class="shrink-0 rounded-[3px] bg-brand px-3.5 py-1.5 text-sm text-white disabled:opacity-50" onclick={() => runNow(v.name || null)} disabled={running}>{running ? '…' : 'Run it'}</button>
            {:else}
              <span class="shrink-0 text-xs muted">not run yet</span>
            {/if}
          </div>
          {#if Object.keys(v.set ?? {}).length}
            <div class="mono mt-2 text-[11px] leading-relaxed muted">changes: {Object.entries(v.set).map(([k, x]) => `${k} = ${x}`).join(' · ')}</div>
          {/if}
          {#if r}
            <div class="mt-auto flex flex-wrap items-center gap-x-4 gap-y-1 pt-3 text-xs">
              <span class="mono muted">{isoDate(r.start)} → {isoDate(r.end)}</span>
              <span class="mono muted">{num(r.events)} events</span>
              {#if r.audit}<Verdict verdict={r.audit.verdict} />{/if}
              <Status closed={r.all_ledgers_closed} />
              {#if live}<button class="hover:text-brand disabled:opacity-50" onclick={() => runNow(v.name || null)} disabled={running}>run again</button>{/if}
            </div>
            {#if v.runs.length > 1}
              <details class="mt-2 text-xs">
                <summary class="cursor-pointer muted">{v.runs.length - 1} earlier {v.runs.length === 2 ? 'run' : 'runs'}</summary>
                <ul class="mt-1">
                  {#each v.runs.slice(1) as e (e.id)}
                    <li><a use:link={{ project: path, run: e.id }} class="mono hover:text-brand">{e.id}</a>{#if e.set && Object.keys(e.set).length}<span class="mono muted"> · {Object.entries(e.set).map(([k, x]) => `${k}=${x}`).join(' ')}</span>{/if}</li>
                  {/each}
                </ul>
              </details>
            {/if}
          {/if}
        </li>
      {/each}
    </ul>
    {#if !rows.length}<p class="muted text-sm">not run yet</p>{/if}
    <Pager total={rows.length} size={10} bind:page={runPage} label="versions" />
  </section>

  {#if readme && !pageSrc}
    <!-- the story under the runs it is read from (Anselm, 2026-11-04: the variants and runs first) -->
    <section class="mb-10">
      {#key readme}<Readme {path} md={readme} />{/key}
    </section>
  {/if}

  {#if source}
    <details class="mb-4" open>
      <summary class="label cursor-pointer">source <span class="mono normal-case opacity-60">{source.name}</span></summary>
      <p class="mt-2 text-xs opacity-70">The program this project was made from, in {source.language === 'javascript' ? 'JavaScript' : 'JSON'}{source.sources.length ? `, with ${source.sources.join(', ')}` : ''}. The manifest below is what it resolved to.</p>
      <pre class="mono mt-2 overflow-x-auto rounded-[3px] bg-neutral-50 p-3 text-xs leading-relaxed dark:bg-neutral-900">{source.text}</pre>
    </details>
  {/if}
  <details class="mb-8">
    <summary class="label cursor-pointer">manifest</summary>
    <pre class="mono mt-2 overflow-x-auto rounded-[3px] bg-neutral-50 p-3 text-xs leading-relaxed dark:bg-neutral-900">{manifest}</pre>
  </details>
{/if}
