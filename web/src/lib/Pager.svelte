<script>
  // Pages of a list: earlier and later, and where you are. Shown only when there is more than
  // one page. The list's owner slices by `page` and `size`.
  let { total, size = 12, page = $bindable(0), label = 'page' } = $props()
  const pages = $derived(Math.max(1, Math.ceil(total / size)))
  $effect(() => { if (page > pages - 1) page = pages - 1 })
</script>

{#if pages > 1}
  <nav class="mt-3 flex items-center gap-4 mono text-xs" aria-label="pages">
    <button class="rounded-[3px] px-2 py-1 hover:text-brand disabled:opacity-30" disabled={page === 0} onclick={() => (page = Math.max(0, page - 1))}>← earlier</button>
    <span class="muted">{label} {page + 1} of {pages} · {total} in all</span>
    <button class="rounded-[3px] px-2 py-1 hover:text-brand disabled:opacity-30" disabled={page >= pages - 1} onclick={() => (page = Math.min(pages - 1, page + 1))}>later →</button>
  </nav>
{/if}
