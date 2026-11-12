// Formatting helpers. Times in the data are Unix seconds (UTC).

export const isoDate = (s) => new Date(s * 1000).toISOString().slice(0, 10)

export const isoTime = (s) => new Date(s * 1000).toISOString().slice(0, 16).replace('T', ' ') + 'Z'

export function num(v) {
  if (v === null || v === undefined) return '—'
  if (typeof v !== 'number') return String(v)
  if (Number.isInteger(v)) return v.toLocaleString('en-US')
  if (Math.abs(v) >= 1000) return v.toLocaleString('en-US', { maximumFractionDigits: 2 })
  return v.toLocaleString('en-US', { maximumSignificantDigits: 6 })
}

export function compact(data) {
  if (data === null || data === undefined) return ''
  if (typeof data !== 'object') return String(data)
  return Object.entries(data)
    .map(([k, v]) => `${k}=${typeof v === 'number' ? num(v) : JSON.stringify(v)}`)
    .join(' ')
}

export const ms = (v) => (v == null ? '—' : v < 1000 ? `${v} ms` : `${(v / 1000).toFixed(2)} s`)

// a moment relative to a run's event: "6 days before", "2 min in", "day 25"
export function relTime(t, event) {
  if (event == null || t == null) return ''
  const d = t - event, a = Math.abs(d)
  if (a < 60) return d < 0 ? 'the minute before' : 'the first minute'
  if (a < 3600) { const m = Math.round(a / 60); return d < 0 ? `${m} min before` : `${m} min in` }
  if (a < 86400) { const h = Math.floor(a / 3600); return d < 0 ? `${h} h before` : `${h} h in` }
  const days = Math.floor(a / 86400)
  return d < 0 ? `${days} day${days === 1 ? '' : 's'} before` : `day ${days}`
}
