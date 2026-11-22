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

// an event's local time in words: '2027-03-26 14:00 PDT' -> 'Friday 26 March 2027 at 14:00 PDT'
export function eventWords(local) {
  const m = /^(\d{4})-(\d{2})-(\d{2})[ T](\d{2}:\d{2})\s*(\S*)/.exec(local ?? '')
  if (!m) return local ?? ''
  const d = new Date(Date.UTC(+m[1], +m[2] - 1, +m[3]))
  const days = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday']
  const months = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December']
  return `${days[d.getUTCDay()]} ${+m[3]} ${months[+m[2] - 1]} ${m[1]} at ${m[4]}${m[5] ? ' ' + m[5] : ''}`
}

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
