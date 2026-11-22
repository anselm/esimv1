<script>
  // A page written in HTML with the site's views placed in it by tag. The author writes any
  // HTML at all (headings, prose, columns, tabs, their own styles and scripts) and puts
  // `<esim-map field="deaths">`, `<esim-chart series="...">`, `<esim-scene>` and the rest where
  // a view should be. A tag's `run` attribute names a variant or a run id; without it the tag
  // shows the page's own run, or the project's first variant. Tags of one run share one moment.
  // `<esim-tabs>` turns its `<section title="...">` children into tabs. The HTML is trusted: it
  // comes from the repository, and its scripts run.
  import { mount, unmount } from 'svelte'
  import NbView from './NbView.svelte'
  import { loadRun } from './notebook.svelte.js'

  let { src, path, fixed = null } = $props()
  let el = $state()
  let error = $state(null)

  $effect(() => {
    if (!el || !src) return
    let live = true; const mounted = []
    fetch(src, { cache: 'no-store' }).then((r) => { if (!r.ok) throw new Error(`${src}: ${r.status}`); return r.text() }).then((html) => {
      if (!live) return
      el.innerHTML = html
      const runs = new Map()
      const ctxOf = (key) => { const k = key ?? ''; if (!runs.has(k)) runs.set(k, loadRun(path, key, fixed)); return runs.get(k) }
      // tabs first, so the views inside them are mounted where they will be shown
      for (const tabs of el.querySelectorAll('esim-tabs')) {
        const panes = [...tabs.children].filter((c) => c.tagName === 'SECTION')
        const bar = document.createElement('div'); bar.className = 'nb-tabs'; bar.setAttribute('role', 'tablist')
        const pick = (i) => { panes.forEach((p, k) => { p.hidden = k !== i }); [...bar.children].forEach((b, k) => b.setAttribute('aria-selected', String(k === i))); window.dispatchEvent(new Event('resize')) }
        panes.forEach((p, i) => { const b = document.createElement('button'); b.type = 'button'; b.setAttribute('role', 'tab'); b.textContent = p.getAttribute('title') ?? `Tab ${i + 1}`; p.removeAttribute('title'); b.onclick = () => pick(i); bar.appendChild(b) })
        tabs.prepend(bar); pick(0)
      }
      for (const node of el.querySelectorAll('*')) {
        const tag = node.tagName.toLowerCase()
        if (!tag.startsWith('esim-') || tag === 'esim-tabs') continue
        const attrs = Object.fromEntries([...node.attributes].map((a) => [a.name, a.value]))
        const text = node.textContent.trim(); node.textContent = ''
        mounted.push(mount(NbView, { target: node, props: { tag, attrs, ctx: ctxOf(attrs.run), path, text } }))
      }
      // the page's own scripts: a script set as HTML does not run, so each is made again
      for (const old of el.querySelectorAll('script')) { const s = document.createElement('script'); for (const a of old.attributes) s.setAttribute(a.name, a.value); s.textContent = old.textContent; old.replaceWith(s) }
    }).catch((e) => { if (live) error = e.message })
    return () => { live = false; for (const m of mounted) { try { unmount(m) } catch {} } }
  })
</script>

{#if error}<p class="text-sm text-leak">{error}</p>{/if}
<div bind:this={el} class="doc notebook"></div>

<style>
  .notebook { max-width: none; }
  .notebook :global(esim-scene), .notebook :global(esim-timeline), .notebook :global(esim-map), .notebook :global(esim-chart), .notebook :global(esim-story), .notebook :global(esim-modules), .notebook :global(esim-module), .notebook :global(esim-wiring), .notebook :global(esim-readme), .notebook :global(esim-tabs) { display: block; }
  .notebook :global(esim-counter) { display: inline-block; margin: 0 1.75rem 0.75rem 0; }
  .notebook :global(.nb-tabs) { display: flex; flex-wrap: wrap; gap: 0.25rem 1.25rem; border-bottom: 1px solid color-mix(in srgb, currentColor 18%, transparent); margin: 1.5rem 0 1.25rem; }
  .notebook :global(.nb-tabs button) { padding: 0.5rem 0; margin-bottom: -1px; border-bottom: 2px solid transparent; font-size: 0.95rem; opacity: 0.65; }
  .notebook :global(.nb-tabs button[aria-selected='true']) { border-bottom-color: var(--color-brand); color: var(--color-brand); opacity: 1; }
  .notebook :global(.nb-cols) { display: grid; gap: 1.5rem 2.5rem; grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr)); align-items: start; }
  .notebook :global(.nb-wide) { max-width: none; }
  .notebook :global(esim-scene .absolute p) { font-size: inherit; line-height: inherit; margin: 0; }
  .notebook :global(p), .notebook :global(h1), .notebook :global(h2), .notebook :global(h3), .notebook :global(ul), .notebook :global(ol) { max-width: 46rem; }
</style>
