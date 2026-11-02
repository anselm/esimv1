<script>
  import Badge from './Badge.svelte'
  // One documentation page, rendered from its Markdown; an edit mode that saves through
  // the server. `doc=new` is the form for a new page.
  import { link, go } from './router.svelte.js'
  import { req } from './socket.js'
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
  {#if !isNew && md !== null}
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

<style>
  /* prose for rendered pages: sober, system fonts, space not lines */
  .doc :global(h1) { font-size: 1.375rem; font-weight: 500; margin: 0 0 0.75rem; }
  .doc :global(h2) { font-size: 1.05rem; font-weight: 500; margin: 2rem 0 0.5rem; }
  .doc :global(h3) { font-size: 0.95rem; font-weight: 500; margin: 1.5rem 0 0.4rem; }
  .doc :global(p) { font-size: 0.925rem; line-height: 1.65; margin: 0.6rem 0; }
  .doc :global(li) { font-size: 0.925rem; line-height: 1.6; margin: 0.25rem 0 0.25rem 1.25rem; list-style: disc; }
  .doc :global(ol li) { list-style: decimal; }
  .doc :global(code) { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 0.85em; background: rgb(0 0 0 / 0.05); padding: 0 0.25em; border-radius: 3px; }
  :global(.dark) .doc :global(code), .doc :global(pre code) { background: transparent; }
  .doc :global(pre) { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 0.8rem; line-height: 1.5; padding: 0.75rem; border-radius: 3px; overflow-x: auto; margin: 0.75rem 0; background: rgb(0 0 0 / 0.04); }
  @media (prefers-color-scheme: dark) { .doc :global(pre) { background: rgb(255 255 255 / 0.06); } .doc :global(code) { background: rgb(255 255 255 / 0.08); } }
  .doc :global(table) { font-size: 0.85rem; border-collapse: collapse; margin: 0.75rem 0; }
  .doc :global(th), .doc :global(td) { text-align: left; padding: 0.25rem 1rem 0.25rem 0; vertical-align: top; border-top: 1px solid rgb(128 128 128 / 0.2); }
  .doc :global(th) { font-weight: 500; color: rgb(115 115 115); border-top: none; }
  .doc :global(a) { color: var(--color-brand); }
  .doc :global(blockquote) { margin: 0.75rem 0; padding-left: 0.75rem; border-left: 2px solid rgb(128 128 128 / 0.3); color: rgb(115 115 115); }
  .doc :global(hr) { border: none; border-top: 1px solid rgb(128 128 128 / 0.2); margin: 1.5rem 0; }
</style>
