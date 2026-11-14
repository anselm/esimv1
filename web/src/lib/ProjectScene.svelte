<script>
  // The project's place in three dimensions at the top of its page: the newest run of the
  // project's first variant, in the layers view, with a timeline of its own. When the project has
  // no run the view can draw (no elevation frame, or runs made before modules carried layers),
  // nothing is shown and the page keeps its cover picture.
  import Layers3D from './Layers3D.svelte'
  import Timeline from './Timeline.svelte'
  import { latestRuns } from './runs.js'
  import { link } from './router.svelte.js'

  let { path, shown = $bindable(false) } = $props()
  let item = $state(null)
  let cursor = $state(null)
  const eventOf = (cs) => (cs ?? []).find((c) => c.data?.kind === 'rupture' || c.data?.kind === 'ignition')?.t ?? null

  $effect(() => {
    let live = true
    item = null; shown = false
    latestRuns(path, null, { audit: true, describe: true }).then((rows) => {
      if (!live) return
      const r = rows.find((x) => x.run?.fields?.terrain_elev_m && x.describe?.modules?.some((m) => m.layer))
      if (!r) return
      r.run.describe = r.describe; r.run.audit = r.audit
      const ev = eventOf(r.audit?.crossings)
      // open a week after the event, when there is one, so the first sight is the aftermath
      cursor = ev !== null ? Math.min(r.run.end, ev + 7 * 86400) : r.run.end
      item = r; shown = true
    }).catch(() => {})
    return () => { live = false }
  })
  const crossings = $derived(item?.audit?.crossings ?? [])
  const moments = $derived([...new Set([...Object.values(item?.run?.fields ?? {}).flatMap((f) => f.times ?? []), ...crossings.map((c) => c.t)])].sort((a, b) => a - b))
</script>

{#if item}
  <section class="mb-10">
    <Layers3D run={item.run} base={item.base} {cursor} event={eventOf(crossings)} />
    <div class="mt-2">
      <Timeline t0={item.run.start} t1={item.run.end} bind:cursor autoplay={false} {moments} event={eventOf(crossings)} />
    </div>
    <p class="mt-1 text-xs muted">The run shown is <a use:link={{ project: path, run: item.id }} class="text-brand hover:underline">{item.title}</a>. Press play or drag the bar to move through it; open the run for its figures.</p>
  </section>
{/if}
