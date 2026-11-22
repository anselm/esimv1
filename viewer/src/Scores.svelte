<script>
  // Skill scores from scores.json: one row per (sim, obs) pair. Each metric is
  // a labelled value so the row wraps on a phone instead of scrolling sideways.
  import { num } from './fmt.js'

  let { scores = [], series = {} } = $props()

  const f3 = (v) => (v == null || Number.isNaN(v) ? '—' : v.toFixed(3))
  const pbias = (v) => (v == null || Number.isNaN(v) ? '—' : `${v > 0 ? '+' : v < 0 ? '−' : ''}${Math.abs(v).toFixed(1)}%`)
  const p4 = (v) => (v == null || Number.isNaN(v) ? '—' : num(+v.toPrecision(4)))
  const unitOf = (s) => series?.[s.sim]?.unit ?? series?.[s.obs]?.unit ?? ''

  const metrics = (s) => [
    ['NSE', f3(s.nse), 'Nash–Sutcliffe efficiency'],
    ['KGE', f3(s.kge), 'Kling–Gupta efficiency'],
    ['r', f3(s.r), 'Pearson correlation'],
    ['PBIAS', pbias(s.pbias_pct), 'percent bias, sim against obs'],
    ['RMSE', `${p4(s.rmse)} ${unitOf(s)}`.trim(), 'root mean square error'],
    ['n', num(s.n), 'paired samples'],
    ['mean sim', `${p4(s.mean_sim)} ${unitOf(s)}`.trim(), ''],
    ['mean obs', `${p4(s.mean_obs)} ${unitOf(s)}`.trim(), ''],
  ]
</script>

<div class="space-y-4">
  {#each scores as s (s.sim + '|' + s.obs)}
    <div class="flex flex-wrap items-start gap-x-6 gap-y-2 text-xs">
      <div class="mono basis-full sm:basis-56 sm:shrink-0">
        <div>{s.sim}</div>
        <div class="muted">{s.obs}</div>
      </div>
      {#each metrics(s) as [label, value, title] (label)}
        <div class="min-w-14" {title}>
          <div class="label">{label}</div>
          <div class="mono mt-0.5">{value}</div>
        </div>
      {/each}
    </div>
  {/each}
</div>
