// Writes docs/country-outlines.json: each country's rings by ISO3 from the cached Natural Earth
// 50m file (the same outlines the score.regions module keys on), thinned for the globe.
// Usage: node web/scripts/country-outlines.mjs   (from the repo root)
import fs from 'node:fs'
const src = 'data/cache/natural-earth_ne_50m_admin_0_countries.geojson'
const g = JSON.parse(fs.readFileSync(src, 'utf8'))
const out = {}
let rings = 0, pts = 0
for (const f of g.features) {
  const p = f.properties
  let iso3 = p.ISO_A3 && p.ISO_A3 !== '-99' ? p.ISO_A3 : p.ADM0_A3
  if (!iso3 || iso3 === '-99') continue
  const polys = f.geometry.type === 'Polygon' ? [f.geometry.coordinates] : f.geometry.type === 'MultiPolygon' ? f.geometry.coordinates : []
  const keep = []
  for (const poly of polys) {
    const ring = poly[0]                                   // outer ring only
    if (ring.length < 8) continue
    const step = ring.length > 400 ? 3 : ring.length > 120 ? 2 : 1
    const thin = ring.filter((_, i) => i % step === 0).map(([x, y]) => [Math.round(x * 100) / 100, Math.round(y * 100) / 100])
    if (thin.length < 6) continue
    keep.push(thin); rings++; pts += thin.length
  }
  if (keep.length) out[iso3] = (out[iso3] ?? []).concat(keep)
}
fs.writeFileSync('docs/country-outlines.json', JSON.stringify(out))
console.log(Object.keys(out).length, 'countries', rings, 'rings', pts, 'points', fs.statSync('docs/country-outlines.json').size, 'bytes')
