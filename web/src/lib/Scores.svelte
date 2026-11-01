<script>
  // Skill scores from scores.json: one row per (sim, obs) pair.
  import { num } from './fmt.js'

  let { scores = [], series = {} } = $props()

  const f3 = (v) => (v == null || Number.isNaN(v) ? '—' : v.toFixed(3))
  const pbias = (v) => (v == null || Number.isNaN(v) ? '—' : `${v > 0 ? '+' : v < 0 ? '−' : ''}${Math.abs(v).toFixed(1)}%`)
  const unitOf = (s) => series?.[s.sim]?.unit ?? series?.[s.obs]?.unit ?? ''
</script>

<div class="overflow-x-auto">
  <table class="text-xs">
    <thead>
      <tr class="muted">
        <th class="pr-5 text-left font-normal">sim / obs</th>
        <th class="pr-4 text-right font-normal" title="Nash–Sutcliffe efficiency">NSE</th>
        <th class="pr-4 text-right font-normal" title="Kling–Gupta efficiency">KGE</th>
        <th class="pr-4 text-right font-normal" title="Pearson correlation">r</th>
        <th class="pr-4 text-right font-normal" title="percent bias">PBIAS</th>
        <th class="pr-4 text-right font-normal">RMSE</th>
        <th class="pr-4 text-right font-normal">n</th>
        <th class="pr-4 text-right font-normal">mean sim</th>
        <th class="text-right font-normal">mean obs</th>
      </tr>
    </thead>
    <tbody class="mono">
      {#each scores as s (s.sim + '|' + s.obs)}
        <tr class="align-top">
          <td class="pr-5 py-1 whitespace-nowrap">
            <div>{s.sim}</div>
            <div class="muted">{s.obs}</div>
          </td>
          <td class="pr-4 py-1 text-right">{f3(s.nse)}</td>
          <td class="pr-4 py-1 text-right">{f3(s.kge)}</td>
          <td class="pr-4 py-1 text-right">{f3(s.r)}</td>
          <td class="pr-4 py-1 text-right">{pbias(s.pbias_pct)}</td>
          <td class="pr-4 py-1 text-right">{num(+s.rmse.toPrecision(4))}{unitOf(s) ? ` ${unitOf(s)}` : ''}</td>
          <td class="pr-4 py-1 text-right">{num(s.n)}</td>
          <td class="pr-4 py-1 text-right">{num(+s.mean_sim.toPrecision(4))}</td>
          <td class="py-1 text-right">{num(+s.mean_obs.toPrecision(4))}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
