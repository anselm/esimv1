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
