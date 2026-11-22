// A picture for each module kind, made from a real run that uses it: the kind's first map
// frame at the end of the run, or, for a kind with no map, its first series over the run.
// Written to docs/kinds/<kind>.png with docs/kinds/index.json saying which project and which
// output each came from. Nothing here is illustration: every pixel is a run's own number.
//   node web/scripts/kind-pictures.mjs        (from the repository root; runs must exist locally)
import fs from 'node:fs'
import path from 'node:path'
import zlib from 'node:zlib'
import { LAYERS } from '../../viewer/src/layers.js'

const ROOT = process.cwd()
const OUT = path.join(ROOT, 'docs/kinds')
const W = 480, H = 270, BG = [7, 13, 21]
const hex = (h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16))
const kinds = JSON.parse(fs.readFileSync(path.join(ROOT, 'docs/kinds.json'), 'utf8'))
const colorOf = Object.fromEntries(kinds.map((k) => [k.kind, hex(LAYERS.find((l) => l.id === k.layer)?.color ?? '#c8d6de')]))
const read = (p) => { try { return JSON.parse(fs.readFileSync(p, 'utf8')) } catch { return null } }

function png(rgb) {
  const crcT = new Uint32Array(256).map((_, n) => { let c = n; for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1; return c >>> 0 })
  const crc = (b) => { let c = 0xffffffff; for (const x of b) c = crcT[(c ^ x) & 255] ^ (c >>> 8); return (c ^ 0xffffffff) >>> 0 }
  const chunk = (type, data) => { const len = Buffer.alloc(4); len.writeUInt32BE(data.length); const td = Buffer.concat([Buffer.from(type), data]); const c = Buffer.alloc(4); c.writeUInt32BE(crc(td)); return Buffer.concat([len, td, c]) }
  const head = Buffer.alloc(13); head.writeUInt32BE(W, 0); head.writeUInt32BE(H, 4); head[8] = 8; head[9] = 2
  const raw = Buffer.alloc((W * 3 + 1) * H)
  for (let y = 0; y < H; y++) { raw[y * (W * 3 + 1)] = 0; rgb.copy(raw, y * (W * 3 + 1) + 1, y * W * 3, (y + 1) * W * 3) }
  return Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk('IHDR', head), chunk('IDAT', zlib.deflateSync(raw, { level: 9 })), chunk('IEND', Buffer.alloc(0))])
}
const canvas = () => { const b = Buffer.alloc(W * H * 3); for (let i = 0; i < W * H; i++) { b[i * 3] = BG[0]; b[i * 3 + 1] = BG[1]; b[i * 3 + 2] = BG[2] } return b }
const put = (b, x, y, c, a = 1) => { if (x < 0 || y < 0 || x >= W || y >= H) return; const i = (y * W + x) * 3; for (let k = 0; k < 3; k++) b[i + k] = Math.min(255, Math.round(b[i + k] * (1 - a) + c[k] * a)) }

/// A map frame, fitted into the picture: brightness by the value, in the layer's colour.
function drawField(dir, name, f, color) {
  const idx = (f.times?.length ?? 1) - 1
  let buf; try { buf = fs.readFileSync(path.join(dir, 'fields', name, String(idx).padStart(5, '0') + '.f32')) } catch { return null }
  const { nx, ny } = f.grid, data = new Float32Array(buf.buffer, buf.byteOffset, buf.byteLength >> 2)
  if (data.length !== nx * ny) return null
  let lo = Infinity, hi = -Infinity, n = 0
  for (const v of data) if (Number.isFinite(v)) { if (v < lo) lo = v; if (v > hi) hi = v; if (v !== 0) n++ }
  if (!(hi > lo) || n < 12) return null
  const floor = lo > 0 ? lo : 0, log = hi - floor > 50
  const scale = Math.min(W / nx, H / ny), w = Math.round(nx * scale), h = Math.round(ny * scale), ox = (W - w) >> 1, oy = (H - h) >> 1
  const b = canvas()
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    // the largest value among the source cells this pixel covers, so thin features survive
    const c0 = Math.floor(x / scale), c1 = Math.max(c0 + 1, Math.floor((x + 1) / scale)), r0 = Math.floor(y / scale), r1 = Math.max(r0 + 1, Math.floor((y + 1) / scale))
    let v = -Infinity
    for (let r = r0; r < r1 && r < ny; r++) for (let c = c0; c < c1 && c < nx; c++) { const q = data[r * nx + c]; if (Number.isFinite(q) && q > v) v = q }
    if (!(v > floor)) continue
    const t = Math.min(1, log ? Math.log1p(v - floor) / Math.log1p(hi - floor) : (v - floor) / (hi - floor))
    put(b, ox + x, oy + y, color.map((k) => k + (255 - k) * Math.max(0, t - 0.75) * 2), 0.18 + 0.82 * t)
  }
  return b
}
/// A series over the run, as a line.
function drawSeries(s, color) {
  const pts = (s.points ?? []).filter((p) => Number.isFinite(p[1]))
  if (pts.length < 3) return null
  let lo = Infinity, hi = -Infinity; for (const p of pts) { if (p[1] < lo) lo = p[1]; if (p[1] > hi) hi = p[1] }
  if (!(hi > lo)) return null
  const t0 = pts[0][0], t1 = pts[pts.length - 1][0]; if (!(t1 > t0)) return null
  const b = canvas(), m = 22
  const X = (t) => m + ((t - t0) / (t1 - t0)) * (W - 2 * m), Y = (v) => H - m - ((v - lo) / (hi - lo)) * (H - 2 * m)
  for (let x = m; x < W - m; x++) put(b, x, H - m, color, 0.25)
  for (let i = 1; i < pts.length; i++) {
    const x0 = X(pts[i - 1][0]), y0 = Y(pts[i - 1][1]), x1 = X(pts[i][0]), y1 = Y(pts[i][1]), n = Math.max(1, Math.ceil(Math.hypot(x1 - x0, y1 - y0)))
    for (let k = 0; k <= n; k++) { const x = Math.round(x0 + ((x1 - x0) * k) / n), y = Math.round(y0 + ((y1 - y0) * k) / n); put(b, x, y, color, 1); put(b, x, y + 1, color, 0.6); put(b, x, y - 1, color, 0.35) }
  }
  return b
}

fs.mkdirSync(OUT, { recursive: true })
const index = {}
const projects = read(path.join(ROOT, 'projects/index.json')) ?? []
// candidates: every project's newest run, featured projects first
const runs = []
for (const p of projects.slice().sort((a, b) => (b.featured === true) - (a.featured === true))) {
  const dir0 = path.join(ROOT, 'projects', p.path, 'runs')
  const rows = (read(path.join(dir0, 'index.json')) ?? []).slice().sort((a, b) => (b.start ?? 0) - (a.start ?? 0) || (b.id > a.id ? 1 : -1))
  for (const row of rows.slice(0, 3)) { const dir = path.join(dir0, row.id); const run = read(path.join(dir, 'run.json')), describe = read(path.join(dir, 'describe.json')); if (run && describe) runs.push({ p, row, dir, run, describe }) }
}
for (const k of kinds.map((x) => x.kind)) {
  let best = null
  for (const pass of ['map', 'series']) {
    for (const r of runs) {
      const m = (r.describe.modules ?? []).find((x) => x.kind === k); if (!m) continue
      if (pass === 'map') {
        const titles = new Map((m.views ?? []).filter((v) => v.kind === 'map').map((v) => [v.source, v.title]))
        const names = [...new Set([...titles.keys(), ...(m.frames ?? []).map((f) => f.name)])].filter((n) => n !== 'terrain_elev_m' && r.run.fields?.[n])
        for (const n of names) { const b = drawField(r.dir, n, r.run.fields[n], colorOf[k]); if (b) { best = { b, what: titles.get(n) ?? n.replace(/_/g, ' '), kind: 'a map at the end of the run', r }; break } }
      } else {
        const titles = new Map((m.views ?? []).filter((v) => v.kind === 'chart' || v.kind === 'counter').map((v) => [v.source, v.title]))
        const names = [...new Set([...titles.keys(), ...(m.reports ?? []).map((f) => f.name)])].filter((n) => r.run.series?.[n])
        for (const n of names) { const b = drawSeries(r.run.series[n], colorOf[k]); if (b) { best = { b, what: titles.get(n) ?? n, kind: 'a series over the run', r }; break } }
      }
      if (best) break
    }
    if (best) break
  }
  if (!best) { console.log('no picture  ', k); continue }
  fs.writeFileSync(path.join(OUT, `${k}.png`), png(best.b))
  index[k] = { img: `/docs/kinds/${k}.png`, what: best.what, how: best.kind, project: best.r.p.path, title: best.r.p.title ?? best.r.p.path, variant: best.r.row.variant ?? null }
  console.log('picture     ', k.padEnd(22), best.r.p.path, '·', best.what)
}
fs.writeFileSync(path.join(OUT, 'index.json'), JSON.stringify(index, null, 1))
console.log(`${Object.keys(index).length} of ${kinds.length} kinds have a picture`)
