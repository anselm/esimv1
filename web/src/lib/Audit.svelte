<script>
  // What the independent auditor found in this run: verdict, tears, shocks and recovery,
  // stocks spent or built, diversity, and what it could not measure here.
  import { isoDate, num } from './fmt.js'
  import Verdict from './Verdict.svelte'

  let { audit } = $props()
  const pct = (v) => (v == null ? '—' : `${v > 0 ? '+' : ''}${v.toFixed(0)}%`)
  const days = (v) => (v == null ? '—' : `${v.toFixed(0)} d`)
  const spent = $derived((audit.stocks ?? []).filter((s) => s.verdict === 'spent down'))
  const built = $derived((audit.stocks ?? []).filter((s) => s.verdict.startsWith('accumulating')))
  const steady = $derived((audit.stocks ?? []).filter((s) => s.verdict === 'steady'))
</script>

<div class="flex flex-wrap items-baseline gap-x-4 gap-y-1">
  <Verdict verdict={audit.verdict} />
  {#each audit.reasons ?? [] as r, i (i)}<span class="text-sm">{r}</span>{/each}
</div>

{#if audit.tears?.length}
  <div class="mt-4">
    <div class="label mb-1">tears</div>
    <ul class="text-sm">
      {#each audit.tears as t, i (i)}
        <li class="py-0.5"><span class="mono text-xs text-leak">{t.kind}</span> <span class="mono text-xs muted">{isoDate(t.t)}</span> {t.note}</li>
      {/each}
    </ul>
  </div>
{/if}

{#if audit.crossings?.length}
  {@const tips = audit.crossings.filter((c) => c.topic === 'tipping.crossing')}
  {@const people = audit.crossings.filter((c) => c.topic === 'people.crossing')}
  <div class="mt-4 grid gap-x-8 gap-y-4 lg:grid-cols-2">
    {#if tips.length}
      <div>
        <div class="label mb-1">tipping thresholds passed</div>
        <table class="mono w-full text-xs">
          <tbody>
            {#each tips as c, i (i)}
              <tr class="border-t border-neutral-100 dark:border-neutral-900">
                <td class="whitespace-nowrap pr-3 py-0.5">{isoDate(c.t).slice(0, 4)}</td>
                <td class="pr-3 py-0.5">{c.data.title}</td>
                <td class="whitespace-nowrap pr-3 py-0.5 {c.data.level === 'central' ? 'text-leak' : 'muted'}">{c.data.level} {c.data.threshold_c}°</td>
                <td class="py-0.5 muted" title={c.data.source}>{c.data.confidence}</td>
              </tr>
            {/each}
          </tbody>
        </table>
        <p class="mt-1 text-[11px] muted">a flag with a citation, not a tip in this run: nothing feeds back yet</p>
      </div>
    {/if}
    {#if people.length}
      <div>
        <div class="label mb-1">countries crossing</div>
        <table class="mono w-full text-xs">
          <tbody>
            {#each people as c, i (i)}
              <tr class="border-t border-neutral-100 dark:border-neutral-900">
                <td class="whitespace-nowrap pr-3 py-0.5">{isoDate(c.t).slice(0, 4)}</td>
                <td class="pr-3 py-0.5">{c.data.name}</td>
                <td class="whitespace-nowrap pr-3 py-0.5 {c.data.kind === 'niche' ? 'text-leak' : 'muted'}">{c.data.kind === 'niche' ? 'outside the niche' : 'hottest month'}</td>
                <td class="whitespace-nowrap py-0.5 text-right">{num(Math.round((c.data.persons ?? 0) / 1e6))} M</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
{/if}

{#if audit.shocks?.length}
  <div class="mt-4">
    <div class="label mb-1">shocks and recovery</div>
    <div class="overflow-x-auto">
      <table class="mono w-full text-xs">
        <thead><tr class="muted text-left"><th class="pr-3 font-normal">shock</th><th class="pr-3 font-normal">stock</th><th class="pr-3 text-right font-normal">drop</th><th class="pr-3 text-right font-normal">recovery</th><th class="pr-3 text-right font-normal">next shock</th><th class="font-normal">verdict</th></tr></thead>
        <tbody>
          {#each audit.shocks as s, i (i)}
            {#each s.responses as r, j (j)}
              <tr class="border-t border-neutral-100 dark:border-neutral-900">
                <td class="whitespace-nowrap pr-3 py-0.5">{#if j === 0}{s.topic} {isoDate(s.t)}{#if s.events > 1} ×{s.events}{/if}{/if}</td>
                <td class="pr-3 py-0.5">{r.series}</td>
                <td class="pr-3 py-0.5 text-right {r.drop_pct > 5 ? '' : 'muted'}">{pct(-r.drop_pct)}</td>
                <td class="pr-3 py-0.5 text-right">{days(r.recovery_days)}</td>
                <td class="pr-3 py-0.5 text-right">{#if j === 0}{days(s.next_shock_days)}{/if}</td>
                <td class="py-0.5 {r.verdict.startsWith('not') ? 'text-leak' : r.verdict === 'unmoved' ? 'muted' : ''}">{r.verdict}</td>
              </tr>
            {/each}
          {/each}
        </tbody>
      </table>
    </div>
  </div>
{/if}

{#if audit.stocks?.length}
  <div class="mt-4 grid gap-x-6 gap-y-3 sm:grid-cols-3">
    {#each [['spent down', spent], ['accumulating', built], ['steady', steady]] as [label, list] (label)}
      <div>
        <div class="label mb-1">{label} · {list.length}</div>
        <ul class="mono text-xs">
          {#each list as s (s.ledger + s.account)}
            <li class="flex justify-between gap-2 py-0.5"><span class="truncate">{s.ledger}/{s.account}</span><span class="tabular-nums {label === 'spent down' ? 'text-leak' : 'muted'}">{pct(s.change_pct)}</span></li>
          {/each}
        </ul>
      </div>
    {/each}
  </div>
{/if}

{#if audit.diversity}
  <div class="mt-4 text-sm">
    <span class="label">diversity</span>
    <span class="mono text-xs">evenness {audit.diversity.evenness_start.toFixed(2)} → {audit.diversity.evenness_end.toFixed(2)} · effective populations {audit.diversity.effective_start.toFixed(2)} → {audit.diversity.effective_end.toFixed(2)} · over {audit.diversity.series.join(', ')}</span>
  </div>
{/if}

{#if audit.populations?.length}
  <div class="mt-3 mono text-xs muted">
    {#each audit.populations as p (p.series)}
      <div>{p.series}: {num(Math.round(p.start))} → {num(Math.round(p.end))}, low {num(Math.round(p.min))} on {isoDate(p.min_at)}{#if p.extinct_at}, gone {isoDate(p.extinct_at)}{/if}</div>
    {/each}
  </div>
{/if}

{#if audit.silent?.length}
  <details class="mt-3 text-xs muted">
    <summary class="cursor-pointer">{audit.silent.length} {audit.silent.length === 1 ? 'thing' : 'things'} the auditor could not measure here</summary>
    <ul class="mt-1">{#each audit.silent as s, i (i)}<li>{s}</li>{/each}</ul>
  </details>
{/if}
