<script>
  // The auditor's verdict as a status: icon and word, never colour alone.
  let { verdict } = $props()
  const look = {
    holds: ['●', 'text-ok', 'holds'],
    stretching: ['◐', 'text-brand', 'stretching'],
    tearing: ['○', 'text-leak', 'tearing'],
  }
  const v = $derived(look[verdict] ?? ['·', 'muted', verdict ?? '—'])
</script>

<span class="mono inline-flex items-center gap-1 text-xs {v[1]}" title="auditor's verdict"><span aria-hidden="true">{v[0]}</span>{v[2]}</span>
