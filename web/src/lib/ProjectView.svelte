<script>
  import Badge from './Badge.svelte'
  // One project: what was asked, the manifest, the runs, and a button to run it again.
  import { link, go } from './router.svelte.js'
  import { req, onLog, onChanged } from './socket.js'
  import { isoDate, num } from './fmt.js'
  import Status from './Status.svelte'
  import Verdict from './Verdict.svelte'

  let { path } = $props()
  let meta = $state(null)
  let manifest = $state('')
  let runs = $state([])
  let error = $state(null)
  let live = $state(false)
  let running = $state(false)
  let log = $state([])

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
      meta = parseMeta(r.meta); manifest = r.manifest; runs = r.runs; live = true
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
    runs = runs.slice().sort((a, b) => (b.start ?? 0) - (a.start ?? 0))
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
<div class="floating right-3 top-3 mono text-xs select-none"><Badge /></div>

{#if error && !meta}
  <p class="text-sm text-leak">{error}</p>
{:else if !meta}
  <p class="muted text-sm">loading…</p>
{:else}
  <section class="mb-8">
    <div class="mono text-xs muted">/{path}</div>
    <h1 class="mt-1 text-xl font-medium">{meta.title ?? path}</h1>
    {#if meta.description}
      <p class="mt-2 max-w-3xl text-sm leading-relaxed">{meta.description}</p>
    {/if}
    <div class="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs muted">
      {#if meta.created}<span>started {meta.created}</span>{/if}
      {#if meta.by}<span>by {meta.by}</span>{/if}
      <span class="mono">{kinds.join(' · ')}</span>
    </div>
  </section>

  <section class="mb-8">
    {#if meta.variants?.length}
      <div class="mb-2 flex flex-wrap items-center justify-between gap-3">
        <div class="label">variants</div>
        <a use:link={{ compare: path }} class="text-sm text-brand">compare the futures →</a>
      </div>
      <ul class="mb-6 divide-y divide-neutral-200 dark:divide-neutral-800">
        {#each meta.variants as v (v.name)}
          <li class="flex flex-wrap items-center justify-between gap-x-4 gap-y-1 py-2">
            <div class="min-w-0">
              <span class="font-medium">{v.title ?? v.name}</span>
              <span class="mono ml-2 text-xs muted">{v.name}</span>
              <div class="mono text-xs muted">{Object.entries(v.set).map(([k, x]) => `${k} = ${x}`).join(' · ')}</div>
            </div>
            <span class="flex items-center gap-3 text-xs">
              <span class="mono muted">{runs.filter((r) => r.variant === v.name).length} runs</span>
              {#if live}<button class="rounded-[3px] bg-brand px-2.5 py-1 text-white disabled:opacity-50" onclick={() => runNow(v.name)} disabled={running}>{running ? '…' : 'Run'}</button>{/if}
            </span>
          </li>
        {/each}
      </ul>
    {/if}
    <div class="mb-2 flex flex-wrap items-center justify-between gap-3">
      <div class="label">{runs.length} {runs.length === 1 ? 'run' : 'runs'}</div>
      {#if live}
        <button class="rounded-[3px] px-3 py-1 text-sm ring-1 ring-neutral-300 disabled:opacity-50 dark:ring-neutral-700" onclick={() => runNow(null)} disabled={running}>{running ? 'running…' : meta.variants?.length ? 'Run as written' : 'Run'}</button>
      {/if}
    </div>
    {#if log.length}
      <pre class="mono mb-3 max-h-48 overflow-auto rounded-[3px] bg-neutral-50 p-2 text-xs leading-relaxed dark:bg-neutral-900">{log.join('\n')}</pre>
    {/if}
    {#if error}<p class="mb-2 text-sm text-leak">{error}</p>{/if}
    {#if runs.length === 0}
      <p class="muted text-sm">not run yet</p>
    {:else}
      <ul class="divide-y divide-neutral-200 dark:divide-neutral-800">
        {#each runs as r (r.id)}
          <li>
            <a use:link={{ project: path, run: r.id }} class="grid grid-cols-[1fr_auto] gap-x-4 gap-y-1 py-2.5 -mx-2 px-2 rounded-[3px] hover:bg-brand-soft/40 sm:grid-cols-[auto_auto_1fr_auto_auto] sm:items-center">
              <span class="text-sm font-medium">{r.variant ?? (r.set && Object.keys(r.set).length ? 'custom' : 'as written')}</span>
              <span class="mono text-xs">{isoDate(r.start)} → {isoDate(r.end)}</span>
              <span class="mono text-xs muted truncate">{r.set && Object.keys(r.set).length ? Object.entries(r.set).map(([k, x]) => `${k}=${x}`).join(' ') : r.digest}</span>
              <span class="mono text-xs muted">{num(r.events)} events</span>
              <span class="flex items-center gap-3">
                {#if r.audit}<Verdict verdict={r.audit.verdict} />{/if}
                <Status closed={r.all_ledgers_closed} />
              </span>
            </a>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <details class="mb-8">
    <summary class="label cursor-pointer">manifest</summary>
    <pre class="mono mt-2 overflow-x-auto rounded-[3px] bg-neutral-50 p-3 text-xs leading-relaxed dark:bg-neutral-900">{manifest}</pre>
  </details>
{/if}
