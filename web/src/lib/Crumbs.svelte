<script>
  // A path as breadcrumbs, every fragment a link: / anselm / palisades-cells / <run>
  // (Anselm, 2026-11-04: "make fragments clickable so i can go to /anselm").
  import { link } from './router.svelte.js'
  let { path, run = null, floating = false } = $props()
  const [area, name] = $derived(path.split('/'))
</script>

<span class={`mono ${floating ? '' : 'text-xs muted'}`}>
  <a use:link={{}} class="hover:text-brand">/</a>
  <a use:link={{ area }} class="hover:text-brand">{area}</a>
  {#if name}<span class="muted">/</span> <a use:link={{ project: path }} class="hover:text-brand">{name}</a>{/if}
  {#if run}<span class="muted">/</span> <span class="muted">{run.length > 18 ? run.slice(-8) : run}</span>{/if}
</span>
