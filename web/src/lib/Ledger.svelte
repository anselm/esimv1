<script>
  import { isoTime, num } from './fmt.js'

  let { ledger } = $props()

  const closes = $derived(ledger.closes ?? [])
  const leaks = $derived(closes.filter((c) => !c.ok).length)
  const last = $derived(closes[closes.length - 1])
  const accounts = $derived(
    last ? Object.keys(last.balances).sort() : (ledger.accounts ?? []).slice().sort(),
  )
</script>

<div>
  <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1 text-sm">
    <span class="font-medium">{ledger.name}</span>
    <span class="mono text-xs muted">{ledger.unit}</span>
    <span class="mono text-xs muted">{num(ledger.transfers)} transfers</span>
    <span class="mono text-xs muted">tol {ledger.tolerance}</span>
    <span class="mono text-xs {leaks ? 'text-leak' : 'text-ok'}">
      {closes.length} closes · {leaks ? `${leaks} leak${leaks === 1 ? '' : 's'}` : 'all closed'}
    </span>
  </div>

  <!-- one square per close -->
  <div class="mt-2 flex flex-wrap gap-[3px]" role="img" aria-label="{closes.length} ledger closes, {leaks} leaks">
    {#each closes as c, i (c.t)}
      <span
        class="h-3 w-3 rounded-[2px] {c.ok ? 'bg-ok/80' : 'bg-leak'}"
        title="{isoTime(c.t)} · {c.ok ? 'ok' : 'LEAK'} · max_leak {c.max_leak}"
      ></span>
    {/each}
  </div>

  {#if last}
    <table class="mt-3 text-xs">
      <thead>
        <tr class="muted">
          <th class="pr-4 text-left font-normal">account</th>
          <th class="pr-4 text-right font-normal">balance ({ledger.unit})</th>
          <th class="text-right font-normal">leak</th>
        </tr>
      </thead>
      <tbody class="mono">
        {#each accounts as a (a)}
          <tr>
            <td class="pr-4 py-0.5">{a}{ledger.stocks?.includes(a) ? ' *' : ''}</td>
            <td class="pr-4 py-0.5 text-right">{num(last.balances[a])}</td>
            <td class="py-0.5 text-right {last.leaks?.[a] ? 'text-leak' : 'muted'}">
              {a in (last.leaks ?? {}) ? num(last.leaks[a]) : ''}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
    <div class="mt-1 text-[11px] muted">final balances at {isoTime(last.t)} · * stock</div>
  {/if}
</div>
