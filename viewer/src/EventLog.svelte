<script>
  import { isoTime, compact } from './fmt.js'

  let { text } = $props()
  const LIMIT = 200

  const events = $derived.by(() => {
    const out = []
    for (const line of text.split('\n')) {
      if (!line.trim()) continue
      try { out.push(JSON.parse(line)) } catch { /* skip malformed line */ }
    }
    return out
  })
  const topics = $derived([...new Set(events.map((e) => e.topic))].sort())

  let topic = $state('')
  let q = $state('')

  const filtered = $derived.by(() => {
    const needle = q.trim().toLowerCase()
    return events.filter((e) => {
      if (topic && e.topic !== topic) return false
      if (!needle) return true
      return `${e.src} ${e.to ?? ''} ${e.topic} ${compact(e.data)}`.toLowerCase().includes(needle)
    })
  })
  const shown = $derived(filtered.slice(-LIMIT))
  const summary = $derived(
    (filtered.length > LIMIT ? `last ${LIMIT} of ` : '') +
      filtered.length.toLocaleString('en-US') +
      (topic || q.trim() ? ' matching' : ' events'),
  )
</script>

<div class="mb-2 flex flex-wrap items-center gap-2 text-xs">
  <select bind:value={topic} class="field" aria-label="topic">
    <option value="">all topics</option>
    {#each topics as t (t)}<option value={t}>{t}</option>{/each}
  </select>
  <input bind:value={q} placeholder="filter" class="field w-40" aria-label="text filter" />
  <span class="mono muted">
    {summary}
  </span>
</div>

{#if shown.length === 0}
  <p class="muted text-sm">no matching events</p>
{:else}
  <div class="overflow-x-auto">
    <table class="mono w-full text-xs">
      <thead>
        <tr class="muted text-left">
          <th class="pr-3 font-normal">time</th>
          <th class="pr-3 text-right font-normal">seq</th>
          <th class="pr-3 font-normal">src → to</th>
          <th class="pr-3 font-normal">topic</th>
          <th class="font-normal">data</th>
        </tr>
      </thead>
      <tbody>
        {#each shown as e (e.seq)}
          <tr class="border-t border-neutral-100 dark:border-neutral-900">
            <td class="whitespace-nowrap pr-3 py-0.5 muted">{isoTime(e.t)}</td>
            <td class="pr-3 py-0.5 text-right muted">{e.seq}</td>
            <td class="whitespace-nowrap pr-3 py-0.5">{e.src} → {e.to ?? '·'}</td>
            <td class="whitespace-nowrap pr-3 py-0.5 {e.topic.startsWith('kernel.') ? 'text-brand' : ''}">{e.topic}</td>
            <td class="py-0.5 muted">{compact(e.data)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}
