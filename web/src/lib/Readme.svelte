<script>
  // A project's README.md, rendered with live figures. Prose is Markdown; a fenced block
  // whose language is `esim` is an embed, and its first word says which:
  //
  //   ```esim
  //   ladder                          the tipping thresholds against the variants' warming
  //   table 2099 series... [=label]   each variant's value of the named series at the start of that year
  //   table 2025-01-08T21:24Z series...  or at a date and time
  //   actors                          a diagram of actors and relationships in the author's words (Actors.svelte)
  //   ```
  //
  // Figures draw from the project's runs and from nothing else: the author writes the
  // story and the runs supply every number in it.
  import { render } from './markdown.js'
  import { latestRuns, seriesAt, yearStart } from './runs.js'
  import Ladder from './Ladder.svelte'
  import Actors from './Actors.svelte'

  let { path, md } = $props()
  const base = $derived(`/projects/${path}`)   // relative images and links in the README resolve here
  let runs = $state(null)
  let error = $state(null)

  // split the Markdown into prose and embeds
  const segments = $derived.by(() => {
    const out = []
    const re = /^```esim[^\n]*\n([\s\S]*?)^```[ \t]*$/gm
    let last = 0, m
    while ((m = re.exec(md))) {
      if (m.index > last) out.push({ kind: 'md', html: render(md.slice(last, m.index), { base }) })
      const words = m[1].trim().split(/\s+/)
      out.push({ kind: words[0], args: words.slice(1), text: m[1].trim() })
      last = m.index + m[0].length
    }
    if (last < md.length) out.push({ kind: 'md', html: render(md.slice(last), { base }) })
    return out
  })
  const needsRuns = $derived(segments.some((s) => s.kind !== 'md' && s.kind !== 'actors'))
  $effect(() => { if (needsRuns && !runs) latestRuns(path, null, { audit: true, describe: true }).then((r) => { runs = r }).catch((e) => { error = e.message }) })

  const elements = $derived.by(() => {
    for (const o of runs ?? []) for (const m of o.describe?.modules ?? []) if (m.kind === 'tipping.elements' && m.tables?.elements?.length) return m.tables.elements
    return []
  })

  // table: `table <year> name[=label] ...`
  function columns(args) {
    return args.slice(1).map((a) => { const m = a.match(/^([^=]+)=(.*)$/); return m ? { name: m[1], label: m[2].replace(/_/g, ' ') } : { name: a, label: a } })
  }
  function fmt(run, name, v) {
    if (v === null || v === undefined) return '—'
    const unit = run.series?.[name]?.unit ?? ''
    if (unit === 'degC') return `${v >= 0 ? '+' : ''}${v.toFixed(2)} °C`
    if (unit === 'ppm') return `${Math.round(v)} ppm`
    if (unit === 'acres') return Math.round(v).toLocaleString('en-US')
    if (unit === 'persons') return v >= 1e9 ? `${(v / 1e9).toFixed(2)} bn` : v >= 1e6 ? `${Math.round(v / 1e6)} M` : Math.round(v).toLocaleString('en-US')
    if (unit === '%') return `${v.toFixed(1)}%`
    if (unit === 'GtC') return `${v.toFixed(1)} GtC`
    if (unit === 'm3' || unit === 'm3/yr') return Math.round(v).toLocaleString('en-US')
    if (unit === 'm') return Math.abs(v) < 2 ? `${(v * 100).toFixed(0)} cm` : `${Math.round(v).toLocaleString('en-US')} m`
    if (unit === 'person-days/yr') return v >= 1e9 ? `${(v / 1e9).toFixed(1)} bn` : `${Math.round(v / 1e6)} M`
    return Number.isInteger(v) ? String(v) : v.toPrecision(3)
  }
</script>

<article class="doc max-w-3xl">
  {#each segments as s, i (i)}
    {#if s.kind === 'md'}
      {@html s.html}
    {:else if s.kind === 'actors'}
      <Actors text={s.text} />
    {:else if error}
      <p class="text-sm text-leak">{error}</p>
    {:else if !runs}
      <p class="muted text-sm">loading the runs…</p>
    {:else if s.kind === 'ladder'}
      <Ladder {runs} {elements} />
    {:else if s.kind === 'table'}
      {@const year = s.args[0]}
      {@const cols = columns(s.args)}
      {@const t = /^\d{4}$/.test(year) ? yearStart(+year) : Math.floor(Date.parse(year) / 1000)}
      <figure class="overflow-x-auto">
        <table class="mono text-xs tabular-nums">
          <thead><tr><th class="label py-1 pr-3 text-left font-normal normal-case tracking-normal">{year}</th>{#each cols as c (c.name)}<th class="label py-1 pr-3 text-right font-normal normal-case tracking-normal">{c.label}</th>{/each}</tr></thead>
          <tbody>
            {#each runs.filter((o) => o.run.end >= t).slice().sort((a, b) => (seriesAt(a.run, 'energy.gmst_anom_c', a.run.end) ?? 0) - (seriesAt(b.run, 'energy.gmst_anom_c', b.run.end) ?? 0)) as o (o.variant)}
              <tr class="border-t border-neutral-100 dark:border-neutral-900"><td class="py-1 pr-3 whitespace-nowrap">{o.variant}</td>{#each cols as c (c.name)}<td class="py-1 pr-3 text-right whitespace-nowrap">{fmt(o.run, c.name, seriesAt(o.run, c.name, t))}</td>{/each}</tr>
            {/each}
          </tbody>
        </table>
      </figure>
    {:else}
      <pre class="text-xs"><code>{s.text}</code></pre>
    {/if}
  {/each}
</article>
