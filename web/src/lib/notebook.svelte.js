// What a notebook page shares between its views: one run, loaded once, and one moment in it.
// Every `esim-` tag on a page that names the same run reads the same context, so a timeline
// placed anywhere moves every map, chart and counter of that run.
import { latestRuns } from './runs.js'

export class RunCtx {
  run = $state(null)       // run.json, with describe and audit attached
  base = $state('')        // where the run's files are
  id = $state('')
  cursor = $state(null)    // the moment shown, Unix seconds
  error = $state(null)
  get crossings() { return this.run?.audit?.crossings ?? [] }
  get event() { return this.crossings.find((c) => c.data?.kind === 'rupture' || c.data?.kind === 'ignition')?.t ?? null }
  get moments() { return [...new Set([...Object.values(this.run?.fields ?? {}).flatMap((f) => f.times ?? []), ...this.crossings.map((c) => c.t)])].sort((a, b) => a - b) }
}

const json = async (url) => { const r = await fetch(url, { cache: 'no-store' }); if (!r.ok) return null; const v = await r.json(); return Array.isArray(v) ? (v.length ? v : null) : v }

/// The context for a run named on a page of project `path`: by variant name, by run id, or, when
/// the page is itself a run's page, that run (`fixed`: { id, base }).
export function loadRun(path, key, fixed) {
  const ctx = new RunCtx()
  ;(async () => {
    try {
      let base, id
      if (fixed && (!key || key === fixed.id)) { base = fixed.base; id = fixed.id }
      else {
        const idx = (await json(`/projects/${path}/runs/index.json`)) ?? []
        const byId = key ? idx.find((r) => r.id === key) : null
        if (byId) id = byId.id
        else {
          const rows = await latestRuns(path, key ? [key] : null, { audit: false })
          if (!rows.length) throw new Error(key ? `no run of ${key}` : 'no runs yet')
          id = rows[0].id
        }
        base = `/projects/${path}/runs/${encodeURIComponent(id)}`
      }
      const run = await json(`${base}/run.json`)
      if (!run) throw new Error(`run ${id} could not be read`)
      ;[run.audit, run.describe, run.scores] = await Promise.all([json(`${base}/audit.json`), json(`${base}/describe.json`), json(`${base}/scores.json`)])
      ctx.base = base; ctx.id = id; ctx.run = run
      const ev = (run.audit?.crossings ?? []).find((c) => c.data?.kind === 'rupture' || c.data?.kind === 'ignition')?.t ?? null
      ctx.cursor = ev !== null ? Math.min(run.end, ev + 7 * 86400) : run.end
    } catch (e) { ctx.error = e.message }
  })()
  return ctx
}
