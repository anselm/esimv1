// A project's variants and the latest run of each, read from the project folder: the
// shape the compare page and a README's embeds both need. Everything comes from files
// the server already serves; nothing is computed that a run did not report.

async function loadJson(url) { const r = await fetch(url, { cache: 'no-store' }); if (!r.ok) throw new Error(`${url}: ${r.status}`); return r.json() }
async function loadText(url) { const r = await fetch(url, { cache: 'no-store' }); return r.ok ? r.text() : '' }

/// The [[variant]] tables of project.toml: name, title and the set in order.
export function variantsOf(meta) {
  return [...meta.matchAll(/\[\[variant\]\][\s\S]*?name\s*=\s*"([^"]+)"(?:[\s\S]*?title\s*=\s*"([^"]+)")?/g)].map((m) => ({ name: m[1], title: m[2] ?? m[1] }))
}

/// [{ variant, title, id, base, run, audit, describe }] for the newest run of each wanted
/// variant (all of them, in project order, when `wanted` is null). `describe` is the
/// run's describe.json: the modules as they described themselves for that run.
export async function latestRuns(path, wanted = null, { audit = true, describe = false } = {}) {
  const meta = await loadText(`/projects/${path}/project.toml`)
  const names = variantsOf(meta)
  const idx = await loadJson(`/projects/${path}/runs/index.json`).catch(() => [])
  const pick = wanted ?? names.map((n) => n.name)
  const out = []
  for (const v of pick) {
    const rows = idx.filter((r) => r.variant === v)
    if (!rows.length) continue
    const row = rows.sort((a, b) => (b.start ?? 0) - (a.start ?? 0) || (b.id > a.id ? 1 : -1))[0]
    const base = `/projects/${path}/runs/${encodeURIComponent(row.id)}`
    const [run, au, de] = await Promise.all([
      loadJson(`${base}/run.json`),
      audit ? loadJson(`${base}/audit.json`).catch(() => null) : null,
      describe ? loadJson(`${base}/describe.json`).catch(() => null) : null,
    ])
    out.push({ variant: v, title: names.find((n) => n.name === v)?.title ?? v, id: row.id, base, run, audit: au, describe: de })
  }
  return out
}

/// The value of a series at or before time t (Unix seconds), or null.
export function seriesAt(run, name, t) {
  const pts = run.series?.[name]?.points
  if (!pts?.length || t === null || t === undefined) return null
  if (pts[0][0] > t) return null
  let lo = 0, hi = pts.length - 1
  while (lo < hi) { const mid = (lo + hi + 1) >> 1; if (pts[mid][0] <= t) lo = mid; else hi = mid - 1 }
  return pts[lo][1]
}

/// Unix seconds at the start of a year (UTC).
export const yearStart = (y) => Date.UTC(y, 0, 1) / 1000
/// The year of a Unix time.
export const yearOf = (t) => new Date(t * 1000).getUTCFullYear()
