// The stamp a docs page carries in its first lines: who wrote it, on whose order, who else
// revised it, when it was written and last changed, and its revision number.
const day = (s) => { const d = new Date(s + 'T00:00:00Z'); return isNaN(d) ? s : d.toLocaleDateString('en-GB', { day: 'numeric', month: 'long', year: 'numeric', timeZone: 'UTC' }) }
const KEYS = ['author', 'ordered-by', 'revised-by', 'written', 'revised', 'revision']
export function stampOf(md) {
  const s = {}
  for (const k of KEYS) { const v = (md.match(new RegExp(`<!--\\s*${k}:\\s*(.+?)\\s*-->`)) || [])[1]; if (v) s[k.replace('-by', 'By')] = v }
  return Object.keys(s).length ? s : null
}
// a name that is a workspace handle links to that workspace
export const handleOf = (name) => (/^[A-Z][a-z]+$/.test(name ?? '') && name !== 'Claude' ? name.toLowerCase() : null)
export { day }
