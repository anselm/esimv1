<script>
  import Badge from './Badge.svelte'
  // One documentation page, rendered from its Markdown; an edit mode that saves through
  // the server. `doc=new` is the form for a new page.
  import { link, go } from './router.svelte.js'
  import { req, hello } from './socket.js'
  let writable = $state(false)
  hello().then((h) => { writable = !h.readonly })
  import { render, titleOf } from './markdown.js'

  let { path } = $props()
  const isNew = $derived(path === 'new')
  let md = $state(null)
  let error = $state(null)
  let editing = $state(false)
  let draft = $state('')
  let newPath = $state('')
  let busy = $state(false)

  async function load() {
    if (isNew) { md = ''; draft = '# Title\n\nWhat this page is for, in a sentence.\n'; editing = true; return }
    try {
      const r = await fetch(`/docs/${path}.md`, { cache: 'no-store' })
      if (!r.ok) throw new Error(`no page at docs/${path}.md`)
      md = await r.text()
      draft = md
    } catch (e) { error = e.message }
  }
  async function save() {
    busy = true; error = null
    const p = isNew ? newPath.trim() : path
    try {
      await req('doc.save', { path: p, markdown: draft })
      if (isNew) go({ doc: p })
      else { md = draft; editing = false }
    } catch (e) { error = e.message } finally { busy = false }
  }
  $effect(() => { load() })
  const html = $derived(md === null ? '' : render(md))
  const pathOk = $derived(/^[a-z0-9][a-z0-9_-]*\/[a-z0-9][a-z0-9_-]*$/.test(newPath.trim()))
</script>

<a use:link={{ docs: true }} class="floating left-3 top-3 hover:text-brand">← docs</a>
<div class="floating right-3 top-3 flex items-center gap-3 mono text-xs select-none">
  {#if !isNew && md !== null && writable}
    <button class="hover:text-brand" onclick={() => { editing = !editing; draft = md }}>{editing ? 'cancel' : 'edit'}</button>
  {/if}
  <Badge />
</div>

{#if error && md === null}
  <p class="text-sm text-leak">{error}</p>
{:else if md === null}
  <p class="muted text-sm">loading…</p>
{:else if editing}
  <section class="max-w-3xl">
    {#if isNew}
      <h1 class="text-xl font-medium">New page</h1>
      <label class="mt-4 block text-sm">
        <span class="label">path</span>
        <input class="field mono mt-1 w-full" bind:value={newPath} placeholder="area/name" />
      </label>
      <p class="mono mt-1 text-xs {pathOk || !newPath ? 'muted' : 'text-leak'}">docs/{newPath || 'area/name'}.md · an area is yours once you write its first page</p>
    {:else}
      <div class="mono text-xs muted">docs/{path}.md</div>
    {/if}
    <textarea class="field mono mt-3 w-full min-h-[32rem] text-xs leading-relaxed" bind:value={draft} spellcheck="false"></textarea>
    {#if error}<p class="mt-2 text-sm text-leak">{error}</p>{/if}
    <div class="mt-3 flex items-center gap-3 text-sm">
      <button class="rounded-[3px] bg-brand px-3 py-1.5 text-white disabled:opacity-50" onclick={save} disabled={busy || !draft.trim() || (isNew && !pathOk)}>{busy ? 'saving…' : 'Save'}</button>
      <span class="text-xs muted">Markdown; the page is a file in the repo and nothing else</span>
    </div>
  </section>
{:else}
  <article class="doc max-w-3xl">
    <div class="mono mb-4 text-xs muted">docs/{path}.md</div>
    {@html html}
  </article>
{/if}
