<script>
  let { closed } = $props()
</script>

<span class="inline-flex items-center gap-1.5 text-xs" title={closed ? 'all ledgers closed' : 'ledger leak'}>
  <span class="inline-block h-2 w-2 rounded-full {closed ? 'bg-ok' : 'bg-leak'}"></span>
  <span class={closed ? 'text-ok' : 'text-leak'}>{closed ? 'closed' : 'leak'}</span>
</span>
