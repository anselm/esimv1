<script>
  import Badge from './Badge.svelte'
  // A new project: a path, a title, a description in the person's words. Claude drafts
  // a manifest from the description against the kinds the kernel knows; the kernel checks
  // it; the person can edit the TOML or paste their own; then the folder is created.
  import { link, go } from './router.svelte.js'
  import { req, onLog } from './socket.js'

  let area = $state('')
  let name = $state('')
  let title = $state('')
  let description = $state('')
  let manifest = $state('')
  let explanation = $state('')
  let check = $state(null)
  let log = $state([])
  let busy = $state(null)   // 'draft' | 'check' | 'create' | null
  let error = $state(null)
  let model = $state('')

  try { area = localStorage.getItem('esim.area') || '' } catch { /* fine */ }

  const slug = (s) => s.toLowerCase().trim().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '').slice(0, 40)
  $effect(() => { if (!name && title) name = slug(title) })
  const path = $derived(`${slug(area) || 'guest'}/${slug(name)}`)
  const pathOk = $derived(/^[a-z0-9][a-z0-9_-]*\/[a-z0-9][a-z0-9_-]*$/.test(path))

  $effect(() => onLog((m) => { if (!m.path) log = [...log.slice(-20), m.line] }))

  async function draft() {
    busy = 'draft'; error = null; log = []
    try {
      const r = await req('project.draft', { description, title })
      manifest = r.manifest
      explanation = r.explanation
      check = r.check
      model = r.model
    } catch (e) { error = e.message } finally { busy = null }
  }
  async function recheck() {
    busy = 'check'; error = null
    try { check = (await req('project.check', { manifest })).check } catch (e) { error = e.message } finally { busy = null }
  }
  async function create(andRun) {
    busy = 'create'; error = null
    try {
      try { localStorage.setItem('esim.area', slug(area)) } catch { /* fine */ }
      await req('project.create', { path, title, description, manifest, by: 'web' + (model ? `, manifest drafted by ${model}` : '') })
      if (andRun) req('project.run', { path }).catch(() => {})
      go({ project: path })
    } catch (e) { error = e.message; busy = null }
  }
</script>

<a use:link={{}} class="floating left-3 top-3 hover:text-brand">← projects</a>
<div class="floating right-3 top-3 mono text-xs select-none"><Badge /></div>

<section class="max-w-3xl">
  <h1 class="text-xl font-medium">New project</h1>
  <p class="mt-1 text-sm muted">A project is a folder: a manifest, your description beside it, and its runs. Anyone can run the folder with the command line; this page only writes it.</p>

  <div class="mt-6 grid gap-4 sm:grid-cols-[1fr_1fr]">
    <label class="text-sm">
      <span class="label">area</span>
      <input class="field mt-1 w-full" bind:value={area} placeholder="your name or a topic" />
    </label>
    <label class="text-sm">
      <span class="label">name</span>
      <input class="field mt-1 w-full" bind:value={name} placeholder="short-name" />
    </label>
  </div>
  <p class="mono mt-1 text-xs {pathOk ? 'muted' : 'text-leak'}">/{path}</p>

  <label class="mt-4 block text-sm">
    <span class="label">title</span>
    <input class="field mt-1 w-full" bind:value={title} placeholder="What this project is, in a few words" />
  </label>

  <label class="mt-4 block text-sm">
    <span class="label">describe it</span>
    <textarea class="field mt-1 w-full min-h-32" bind:value={description} placeholder="What you want to simulate, where, over what time, and what you want to see. Say what matters to you; the draft will say what it can and cannot do with real data."></textarea>
  </label>

  <div class="mt-3 flex flex-wrap items-center gap-3 text-sm">
    <button class="rounded-[3px] bg-brand px-3 py-1.5 text-white disabled:opacity-50" onclick={draft} disabled={busy !== null || description.trim().length < 10}>
      {busy === 'draft' ? 'drafting…' : 'Draft a manifest'}
    </button>
    <span class="text-xs muted">or paste a manifest below</span>
  </div>
  {#if log.length}
    <ul class="mono mt-2 text-xs muted">{#each log as l, i (i)}<li>{l}</li>{/each}</ul>
  {/if}

  {#if explanation}
    <div class="mt-5 rounded-[3px] bg-neutral-50 p-3 text-sm leading-relaxed dark:bg-neutral-900">
      <div class="label mb-1">the draft says{model ? ` (${model})` : ''}</div>
      <p class="whitespace-pre-wrap">{explanation}</p>
    </div>
  {/if}

  <label class="mt-5 block text-sm">
    <span class="label">manifest</span>
    <textarea class="field mono mt-1 w-full min-h-64 text-xs leading-relaxed" bind:value={manifest} spellcheck="false" placeholder="[run]&#10;name = ...&#10;&#10;[[module]]&#10;name = ...&#10;kind = ..."></textarea>
  </label>

  {#if check}
    <div class="mt-3 text-sm">
      {#if check.errors.length}
        <div class="text-leak">{check.errors.length} {check.errors.length === 1 ? 'error' : 'errors'}: the kernel refuses this manifest</div>
        <ul class="mono mt-1 text-xs text-leak">{#each check.errors as e, i (i)}<li>{e}</li>{/each}</ul>
      {:else}
        <div class="text-ok">wiring checks</div>
      {/if}
      {#if check.warnings.length}
        <ul class="mono mt-1 text-xs">{#each check.warnings as w, i (i)}<li>warning: {w}</li>{/each}</ul>
      {/if}
      {#if check.info.length}
        <details class="mt-1 text-xs muted"><summary>{check.info.length} notes</summary><ul class="mono mt-1">{#each check.info as w, i (i)}<li>{w}</li>{/each}</ul></details>
      {/if}
    </div>
  {/if}

  {#if error}
    <p class="mt-3 text-sm text-leak">{error}</p>
  {/if}

  <div class="mt-5 flex flex-wrap items-center gap-3 text-sm">
    <button class="rounded-[3px] px-3 py-1.5 ring-1 ring-neutral-300 disabled:opacity-50 dark:ring-neutral-700" onclick={recheck} disabled={busy !== null || !manifest.trim()}>Check</button>
    <button class="rounded-[3px] px-3 py-1.5 ring-1 ring-neutral-300 disabled:opacity-50 dark:ring-neutral-700" onclick={() => create(false)} disabled={busy !== null || !manifest.trim() || !pathOk}>Create</button>
    <button class="rounded-[3px] bg-brand px-3 py-1.5 text-white disabled:opacity-50" onclick={() => create(true)} disabled={busy !== null || !manifest.trim() || !pathOk || (check && check.errors.length > 0)}>
      {busy === 'create' ? 'creating…' : 'Create and run'}
    </button>
  </div>
</section>
