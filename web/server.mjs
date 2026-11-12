// The project server: a thin layer over the CLI so a browser can list projects, draft a
// manifest from a description, create a project and run it. It owns nothing; every
// project is a folder under ../projects that `esim` writes and anyone can run by hand.
//
// One socket.io connection per client, structured requests {name, payload} with an ack,
// progress and change notices pushed back. Requests:
//   projects.list                       -> rows of projects/index.json (refreshed)
//   project.get {path}                  -> { meta, manifest, runs }
//   kinds                               -> esim kinds --json (cached)
//   project.draft {description, title}  -> { manifest, explanation, check, rounds, model }  (calls Claude)
//   project.check {manifest}            -> { errors, warnings, info }
//   project.create {path, title, description, manifest, by} -> { path }
//   project.run {path, variant?, set?}  -> streams project.log {path, line}; ends with { ok, output }
//   docs.list                           -> { pages: [{ path, title, order }] } and rewrites docs/index.json
//   doc.save {path, markdown}           -> writes docs/<area>/<name>.md (first come first served per area)
//   area.check {area}                   -> { exists, reserved, owner, mine }
// Notices: projects.changed. Identity (ESIM_PUBLIC=1): an attest delegation on the socket
// handshake; `hello` answers with the attest origin and the signed-in {did, handle}.
//
// It also serves the viewer: the built pages from dist/ when `npm run build` has been
// run, and the ../runs and ../projects folders at /runs/ and /projects/. So one process
// is the whole thing in production; in development Vite serves the pages with hot
// reload and proxies the socket here.
//
// Start: node server.mjs   (port 5181; ANTHROPIC_API_KEY in the environment for drafts)

import http from 'node:http'
import { headTags, sitemap } from './lib/meta.mjs'
import readline from 'node:readline'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { spawn, execFile } from 'node:child_process'
import zlib from 'node:zlib'
import { fileURLToPath } from 'node:url'
import { Server } from 'socket.io'
import Anthropic from '@anthropic-ai/sdk'
import { verifyHandshake, ATTEST_URL } from './lib/attest.mjs'

const here = path.dirname(fileURLToPath(import.meta.url))
const ROOT = path.resolve(here, '..')
const ESIM = process.env.ESIM_BIN || path.join(ROOT, 'target/release/esim')
const PORT = Number(process.env.PORT || 5181)
const VERSION = JSON.parse(fs.readFileSync(path.join(here, 'package.json'), 'utf8')).version
const MODEL = process.env.ESIM_MODEL || 'claude-opus-5-5'
// ESIM_READONLY=1: a public copy. Lists, pages, runs and docs are served; creating,
// running, drafting and editing are refused with a plain message. No esim binary is
// needed in this mode: kinds and projects come from the files the last run wrote.
const READONLY = process.env.ESIM_READONLY === '1'
const HAVE_ESIM = fs.existsSync(ESIM)
// ESIM_PUBLIC=1: a public, writable copy. Identity comes from attest (ATTEST_URL, default
// https://attest.monster) on the socket handshake; a signed-in person may create, run and
// write only under the area that is their attest handle, which they claim on first use
// (projects/<handle>/area.toml records the handle and the DID). Without ESIM_PUBLIC the
// server is a private workbench and asks nobody who they are, as before.
const PUBLIC = process.env.ESIM_PUBLIC === '1'
// Area names the viewer and the server need for themselves (routes, system folders),
// refused as areas even when attest has given someone that handle.
const RESERVED_AREAS = new Set('docs doc system runs run new kinds api compare all projects project index assets socket.io guest esim admin'.split(' '))
// Second tier when the Anthropic API is not usable (no key, no credit): a local model
// through ollama, labelled as such in the answer. Third tier is the person's own TOML.
const OLLAMA = process.env.OLLAMA_URL || 'http://127.0.0.1:11434'
const LOCAL_MODEL = process.env.ESIM_LOCAL_MODEL || 'gpt-oss:120b'

function esim(args, onLine) {
  return new Promise((resolve) => {
    const child = spawn(ESIM, args, { cwd: ROOT, env: { ...process.env, ESIM_ROOT: ROOT } })
    let out = ''
    const take = (chunk) => {
      const s = chunk.toString()
      out += s
      if (onLine) for (const line of s.split('\n')) if (line.trim()) onLine(line)
    }
    child.stdout.on('data', take)
    child.stderr.on('data', take)
    child.on('close', (code) => resolve({ ok: code === 0, code, output: out }))
    child.on('error', (e) => resolve({ ok: false, code: -1, output: String(e) }))
  })
}

function esimJson(args) {
  return new Promise((resolve, reject) => {
    execFile(ESIM, args, { cwd: ROOT, env: { ...process.env, ESIM_ROOT: ROOT }, maxBuffer: 64 << 20 }, (err, stdout, stderr) => {
      if (err) return reject(new Error(stderr || err.message))
      try { resolve(JSON.parse(stdout)) } catch (e) { reject(new Error(`bad json from esim ${args[0]}: ${e.message}`)) }
    })
  })
}

let kindsCache = null
async function kinds() {
  if (!kindsCache) {
    if (HAVE_ESIM) kindsCache = await esimJson(['kinds', '--json'])
    else kindsCache = JSON.parse(fs.readFileSync(path.join(ROOT, 'docs/kinds.json'), 'utf8'))
  }
  return kindsCache
}
async function projectsList() {
  if (HAVE_ESIM && !READONLY) return esimJson(['projects', '--json'])
  return JSON.parse(fs.readFileSync(path.join(ROOT, 'projects/index.json'), 'utf8'))
}
const refuse = () => { throw new Error('this copy is read-only: clone the repository to create, run or edit') }

// Who owns an area: projects/<area>/area.toml, written when a signed-in person first
// creates under their handle. An area without one (made by hand, before accounts) is
// unowned: in public mode only the person whose handle it is can claim it.
function areaOwner(area) {
  try {
    const t = fs.readFileSync(path.join(ROOT, 'projects', area, 'area.toml'), 'utf8')
    const get = (k) => (t.match(new RegExp(`^${k}\\s*=\\s*"([^"]*)"`, 'm')) || [])[1] ?? null
    return { handle: get('handle'), did: get('did'), created: get('created') }
  } catch { return null }
}
// What a signed-in person may write: the area that is their handle, claimed on first use.
// Throws with the reason otherwise. Returns the area.
function claimArea(user, area) {
  if (!PUBLIC) return area
  if (!user) throw new Error('sign in through attest to create or run here')
  if (RESERVED_AREAS.has(area)) throw new Error(`"${area}" is reserved`)
  if (area !== user.handle) throw new Error(`you can write under /${user.handle} only; this is /${area}`)
  const owner = areaOwner(area)
  if (owner && owner.did && owner.did !== user.did) throw new Error(`/${area} belongs to another account`)
  if (!owner) {
    const dir = path.join(ROOT, 'projects', area)
    fs.mkdirSync(dir, { recursive: true })
    fs.writeFileSync(path.join(dir, 'area.toml'), `# This area belongs to an attest account: the handle is the folder.\nhandle = "${area}"\ndid = "${user.did}"\ncreated = "${new Date().toISOString().slice(0, 10)}"\n`)
  }
  return area
}

const validPath = (p) => /^[a-z0-9][a-z0-9_-]*\/[a-z0-9][a-z0-9_-]*$/.test(p)

async function checkManifest(text) {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'esim-draft-'))
  const file = path.join(tmp, 'manifest.toml')
  fs.writeFileSync(file, text)
  try {
    // `describe` prints the check report as JSON even when there are errors
    const r = await new Promise((resolve) => {
      execFile(ESIM, ['describe', file], { cwd: ROOT, env: { ...process.env, ESIM_ROOT: ROOT }, maxBuffer: 64 << 20 }, (err, stdout, stderr) => {
        try { resolve(JSON.parse(stdout)) } catch { resolve({ errors: [(stderr || stdout || String(err)).trim()], warnings: [], info: [], modules: [] }) }
      })
    })
    return { errors: r.errors ?? [], warnings: r.warnings ?? [], info: r.info ?? [] }
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true })
  }
}

// ---- the facade: a description becomes a manifest, checked by the kernel, at most three rounds
const GRAMMAR = `A manifest is TOML:

[run]
name = "short-kebab-name"
start = "YYYY-MM-DD"
end = "YYYY-MM-DD"
seed = 1
close_every = "1d"          # ledger closing cadence; "30d" for runs of many years
notes = "one sentence saying what this run is and what it leaves out"

[[module]]
name = "an instance name"   # unique in the manifest; other modules address it by this name
kind = "a kind from the list"
[module.params]
# only parameters the kind lists, with its units

[place]                     # stated once, inherited by every module that does not set the key itself
lat_s = 42.3
lat_n = 43.2
lon_w = -94.2
lon_e = -93.5
res_arcsec = 30             # 30 is a basin in seconds per simulated year; 15 is four times the cost

[audit]
scores = [["sim.series", "obs.series"]]   # pairs of a model series and an observation series to score, when an obs module is present

Rules of the house, which the kernel enforces and you must respect:
- Real terrain and real records only. Nothing synthetic. If the request needs data no kind provides, say so in the explanation and build the nearest honest thing.
- Every module reads only data the registry has. The basin kinds (water.basin, veg.crop, fauna.*, hazard.fire, agents.farms, rules.calendar, nitrogen.basin, sun.irradiance, heat.slab, weather.era5) are built on the Boone River above Webster City, Iowa (42.3 to 43.2 N, 94.2 to 93.5 W, gauge USGS 05481000) and their default parameters are that place. Always write the [place] table so every module shares one window and resolution. Another place needs its own gauge and station parameters and will fetch new tiles; keep to the Boone defaults unless the person names a place and you have the coordinates from them.
- The planetary kinds (carbon.emissions, carbon.pools, energy.twolayer, obs.planet) are zero-dimensional and run from 1850; wire all four together.
- A kind's required reads must be satisfied by another module's owned fields in the same manifest; the kinds list says which fields each owns and reads. The weather module must come before anything that reads wx.* fields; the water module before the crop; the crop before the animals.
- Keep runs affordable: a basin year with all modules is about 15 seconds; ten basin years is minutes. Prefer a season or a year.
- Do not invent parameters. Do not tune parameters to get a wanted answer; say in notes when a parameter is a guess.
- Score what can be scored. Several kinds report an observation series beside a model series (the kinds list shows reports; names with obs and sim, or an obs.* module's reports): water.basin reports hydro.q_sim_daily_m3_s and hydro.q_obs_daily_m3_s, veg.crop reports veg.ndvi_sim_16d and veg.ndvi_obs_16d, nitrogen runs report hydro.no3_sim_mg_l and hydro.no3_obs_mg_l, obs.planet reports obs.co2_ppm and the temperature records. Put every such pair in [audit] scores.
- A manifest may also set [audit] shock_topics and watch for the auditor; leave them out unless asked.

Answer with one TOML block in a \`\`\`toml fence, then a short plain explanation (what was built, what the person asked for that this cannot do, which parameters are guesses). No headings, no bullet lists.`

// One round with Claude; throws on API failure so the caller can fall to the local tier.
async function askClaude(system, messages) {
  const client = new Anthropic()
  const res = await client.messages.create({
    model: MODEL, max_tokens: 16000, system, messages,
    betas: ['server-side-fallback-2026-07-01'], fallbacks: 'default',
  })
  if (res.stop_reason === 'refusal') throw new Error('the model declined to draft this manifest')
  return res.content.filter((b) => b.type === 'text').map((b) => b.text).join('\n')
}

async function askLocal(system, messages) {
  const r = await fetch(`${OLLAMA}/api/chat`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ model: LOCAL_MODEL, stream: false, options: { num_ctx: 65536 },
      messages: [{ role: 'system', content: system.map((b) => b.text).join('\n\n') }, ...messages] }),
  })
  if (!r.ok) throw new Error(`ollama ${r.status}: ${(await r.text()).slice(0, 200)}`)
  const j = await r.json()
  return j.message?.content ?? ''
}

async function draft(description, title, log) {
  const ks = await kinds()
  const system = [
    { type: 'text', text: 'You write manifests for esim, a whole-system simulator: modules on one clock with ledgers that must close. You know only the kinds listed below; their describe output is authoritative for names, parameters, units, fields and topics.\n\n' + GRAMMAR, cache_control: { type: 'ephemeral' } },
    { type: 'text', text: 'The kinds, as `esim kinds --json` prints them:\n' + JSON.stringify(ks), cache_control: { type: 'ephemeral' } },
  ]
  const messages = [{ role: 'user', content: `Project title: ${title || '(none)'}\n\nWhat the person wants to simulate, in their words:\n${description}` }]
  let manifest = null, explanation = '', check = null, rounds = 0
  let tier = process.env.ANTHROPIC_API_KEY ? 'claude' : 'local'
  let model = tier === 'claude' ? MODEL : LOCAL_MODEL
  for (rounds = 1; rounds <= 3; rounds++) {
    log?.(`asking ${model}, round ${rounds}`)
    let text
    try {
      text = tier === 'claude' ? await askClaude(system, messages) : await askLocal(system, messages)
    } catch (e) {
      if (tier === 'claude') {
        log?.(`${MODEL} not usable (${String(e.message).slice(0, 120)}); falling back to ${LOCAL_MODEL} on this machine`)
        tier = 'local'; model = LOCAL_MODEL; rounds--
        continue
      }
      throw e
    }
    const m = text.match(/```toml\s*([\s\S]*?)```/)
    if (!m) throw new Error('no TOML block in the answer')
    manifest = m[1].trim() + '\n'
    explanation = text.replace(/```toml[\s\S]*?```/, '').trim()
    check = await checkManifest(manifest)
    log?.(`check: ${check.errors.length} errors, ${check.warnings.length} warnings`)
    if (check.errors.length === 0) break
    messages.push({ role: 'assistant', content: text })
    messages.push({ role: 'user', content: `esim check rejects that manifest:\n${check.errors.join('\n')}\n\nFix it and answer in the same form.` })
  }
  return { manifest, explanation, check, rounds, model, tier }
}

// ---- static files: dist/ (the built viewer), ../runs, ../projects
const DIST = path.join(here, 'dist')
// The viewer's page with this route's own title, description and share tags written into
// its head (web/lib/meta.mjs), so a link unfurls as what it is. The public origin for the
// absolute URLs the tags need is ESIM_PUBLIC_URL, else the request's host.
function originOf(req) {
  if (process.env.ESIM_PUBLIC_URL) return process.env.ESIM_PUBLIC_URL.replace(/\/$/, '')
  const host = req.headers['x-forwarded-host'] || req.headers.host || `localhost:${PORT}`
  const proto = req.headers['x-forwarded-proto'] || (/^(localhost|127\.)/.test(host) ? 'http' : 'https')
  return `${proto}://${host}`
}
function sendPage(req, res, url) {
  fs.readFile(path.join(DIST, 'index.html'), 'utf8', (err, html) => {
    if (err) { res.statusCode = 503; return res.end('viewer not built') }
    let body = html
    try { body = html.replace(/<title>[^<]*<\/title>/, headTags(ROOT, url, originOf(req))) } catch (e) { console.error('meta', e.message) }
    res.setHeader('Content-Type', 'text/html; charset=utf-8'); res.setHeader('Cache-Control', 'no-cache')
    res.end(body)
  })
}
const TYPES = {
  '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8',
  '.json': 'application/json', '.jsonl': 'application/x-ndjson', '.toml': 'application/toml; charset=utf-8',
  '.f32': 'application/octet-stream', '.md': 'text/markdown; charset=utf-8', '.svg': 'image/svg+xml',
  '.png': 'image/png', '.ico': 'image/x-icon', '.woff2': 'font/woff2', '.map': 'application/json',
}
// Text and raw-float files compress five to ten times; a run page is 18 MB uncompressed.
const COMPRESSIBLE = new Set(['.json', '.jsonl', '.toml', '.md', '.f32', '.js', '.css', '.html', '.svg'])
function sendFile(req, res, file, cache) {
  fs.stat(file, (err, st) => {
    if (err || !st.isFile()) { res.statusCode = 404; return res.end('not found') }
    res.setHeader('Content-Type', TYPES[path.extname(file)] || 'application/octet-stream')
    res.setHeader('Cache-Control', cache)
    res.setHeader('Vary', 'Accept-Encoding')
    const gz = COMPRESSIBLE.has(path.extname(file)) && /\bgzip\b/.test(req.headers['accept-encoding'] || '') && st.size > 1024
    if (gz) { res.setHeader('Content-Encoding', 'gzip'); fs.createReadStream(file).pipe(zlib.createGzip({ level: 6 })).pipe(res) }
    else { res.setHeader('Content-Length', st.size); fs.createReadStream(file).pipe(res) }
  })
}
// A run's log filtered to one topic: `log.jsonl?topic=region.scored` streams only the lines
// carrying that topic, so a view that wants one table need not pull a 19 MB log.
function sendJsonlTopic(req, res, file, topic) {
  fs.stat(file, (err, st) => {
    if (err || !st.isFile()) { res.statusCode = 404; return res.end('not found') }
    res.setHeader('Content-Type', TYPES['.jsonl'])
    res.setHeader('Cache-Control', 'public, max-age=31536000, immutable')
    res.setHeader('Vary', 'Accept-Encoding')
    const needle = `"topic":${JSON.stringify(topic)}`
    const gz = /\bgzip\b/.test(req.headers['accept-encoding'] || '')
    const out = gz ? zlib.createGzip({ level: 6 }) : res
    if (gz) { res.setHeader('Content-Encoding', 'gzip'); out.pipe(res) }
    const rl = readline.createInterface({ input: fs.createReadStream(file), crlfDelay: Infinity })
    rl.on('line', (line) => { if (line.includes(needle)) out.write(line + '\n') })
    rl.on('close', () => out.end())
  })
}
function serveStatic(req, res) {
  const url = decodeURIComponent((req.url || '/').split('?')[0])
  const query = new URLSearchParams((req.url || '').split('?')[1] || '')
  const under = (prefix, root) => {
    const file = path.resolve(root, url.slice(prefix.length))
    if (!file.startsWith(root + path.sep) && file !== root) { res.statusCode = 403; res.end('forbidden'); return true }
    if (file.endsWith('.jsonl') && query.get('topic')) { sendJsonlTopic(req, res, file, query.get('topic')); return true }
    // a run folder is named by its content hashes and never changes: cache it for a year;
    // indexes, projects and docs change and are revalidated
    const immutable = /\/runs\/[^/]+\/(?!index\.json)/.test(url)
    // a path with no extension that names no file is a page of the viewer (/docs/system/limits)
    if (!path.extname(file) && !fs.existsSync(file)) { sendPage(req, res, url); return true }
    sendFile(req, res, file, immutable ? 'public, max-age=31536000, immutable' : 'no-cache')
    return true
  }
  if (url.startsWith('/runs/')) return under('/runs/', path.join(ROOT, 'runs'))
  if (url.startsWith('/projects/')) return under('/projects/', path.join(ROOT, 'projects'))
  if (url.startsWith('/docs/')) return under('/docs/', DOCS)
  if (!fs.existsSync(DIST)) { res.statusCode = 503; return res.end('viewer not built: run `npm run build` in web/, or use the Vite dev server') }
  if (url === '/robots.txt') { res.setHeader('Content-Type', 'text/plain; charset=utf-8'); return res.end(`User-agent: *\nAllow: /\nSitemap: ${originOf(req)}/sitemap.xml\n`) }
  if (url === '/sitemap.xml') { res.setHeader('Content-Type', 'application/xml; charset=utf-8'); return res.end(sitemap(ROOT, originOf(req))) }
  // the viewer: a file from dist/ when it exists, else the page with the route's tags (the routes are paths: /anselm/palisades-2025)
  if (url === '/') return sendPage(req, res, url)
  const file = path.resolve(DIST, '.' + url)
  if (!file.startsWith(DIST + path.sep)) { res.statusCode = 403; return res.end('forbidden') }
  fs.stat(file, (err, st) => {
    if (!err && st.isFile()) return sendFile(req, res, file, url.startsWith('/assets/') ? 'public, max-age=31536000, immutable' : 'no-cache')
    sendPage(req, res, url)
  })
}

// ---- docs: Markdown pages under ../docs/<area>/<name>.md; the index is written for the
// static fallback. A page's title is its first heading; `<!-- order: n -->` on the first
// line orders pages within an area; `<!-- section: name -->` groups them under a heading.
const DOCS = path.join(ROOT, 'docs')
function docsIndex() {
  const pages = []
  if (!fs.existsSync(DOCS)) return pages
  for (const area of fs.readdirSync(DOCS, { withFileTypes: true })) {
    if (!area.isDirectory()) continue
    for (const f of fs.readdirSync(path.join(DOCS, area.name))) {
      if (!f.endsWith('.md')) continue
      const text = fs.readFileSync(path.join(DOCS, area.name, f), 'utf8')
      const title = (text.match(/^#\s+(.+)$/m) || [])[1]?.trim() ?? null
      const order = Number((text.match(/<!--\s*order:\s*(\d+)\s*-->/) || [])[1]) || null
      const section = (text.match(/<!--\s*section:\s*(.+?)\s*-->/) || [])[1] ?? null
      pages.push({ path: `${area.name}/${f.slice(0, -3)}`, title, order, section })
    }
  }
  pages.sort((a, b) => a.path.localeCompare(b.path))
  try { fs.writeFileSync(path.join(DOCS, 'index.json'), JSON.stringify(pages, null, 2)) } catch { /* read-only checkout */ }
  return pages
}
docsIndex()
// the countries in file order (the country_index raster's indices), for the compare view's picker
try {
  const gj = JSON.parse(fs.readFileSync(path.join(ROOT, 'data/cache/natural-earth_ne_110m_admin_0_countries.geojson'), 'utf8'))
  const list = gj.features.map((f) => ({ iso3: f.properties.ISO_A3 === '-99' ? f.properties.ADM0_A3 : f.properties.ISO_A3, name: f.properties.NAME }))
  fs.writeFileSync(path.join(DOCS, 'countries.json'), JSON.stringify(list))
  const rings = []
  for (const f of gj.features) {
    const polys = f.geometry.type === 'Polygon' ? [f.geometry.coordinates] : f.geometry.type === 'MultiPolygon' ? f.geometry.coordinates : []
    for (const poly of polys) for (const ring of poly) rings.push(ring.map(([x, y]) => [Math.round(x * 20) / 20, Math.round(y * 20) / 20]))
  }
  fs.writeFileSync(path.join(DOCS, 'outlines.json'), JSON.stringify(rings))
} catch { /* not fetched yet */ }
// the kinds, for the static fallback of the kinds page (written only where esim exists)
if (HAVE_ESIM) kinds().then((ks) => { try { fs.writeFileSync(path.join(DOCS, 'kinds.json'), JSON.stringify(ks)) } catch { /* read-only */ } }).catch(() => {})

// ---- server
const server = http.createServer(serveStatic)
const io = new Server(server, { cors: { origin: true } })

io.on('connection', (socket) => {
  const reply = (ack, fn) => Promise.resolve().then(fn).then((v) => ack?.({ ok: true, ...v }), (e) => ack?.({ ok: false, error: e.message || String(e) }))

  // identity, once per connection: the handshake's attest field, verified against this
  // connection's origin and confirmed with attest; {did, handle} or nothing, with the reason
  const origin = socket.handshake.headers.origin || ''
  let user = null, authError = ''
  const auth = (async () => {
    if (!PUBLIC) return
    try {
      const a = await verifyHandshake(socket.handshake.auth?.attest, origin)
      if (a) user = { did: a.root, handle: a.handle || '' }
      if (user && !user.handle) { authError = 'your attest account has no handle yet; set one on attest'; user = null }
    } catch (e) { authError = e.message }
  })()
  const who = async () => { await auth; return user }

  socket.on('hello', (_payload, ack) => reply(ack, async () => ({ version: VERSION, readonly: READONLY, public: PUBLIC, attest: PUBLIC ? ATTEST_URL : null, user: await who(), authError, drafts: !READONLY && (Boolean(process.env.ANTHROPIC_API_KEY) || true) })))

  // is this area free, mine, or someone else's? (what a handle sees before creating)
  socket.on('area.check', ({ area } = {}, ack) => reply(ack, async () => {
    if (!/^[a-z0-9][a-z0-9_-]*$/.test(String(area || ''))) throw new Error('an area is lower case letters, digits, - and _')
    const u = await who()
    const owner = areaOwner(area)
    const exists = fs.existsSync(path.join(ROOT, 'projects', area))
    return { area, exists, reserved: RESERVED_AREAS.has(area), owner: owner?.handle ?? null, mine: Boolean(u && (owner ? owner.did === u.did : area === u.handle)) }
  }))

  socket.on('projects.list', (_payload, ack) => reply(ack, async () => ({ projects: await projectsList() })))

  socket.on('project.get', ({ path: p } = {}, ack) => reply(ack, async () => {
    if (!validPath(p)) throw new Error('bad project path')
    const dir = path.join(ROOT, 'projects', p)
    const meta = fs.readFileSync(path.join(dir, 'project.toml'), 'utf8')
    const manifest = fs.readFileSync(path.join(dir, 'manifest.toml'), 'utf8')
    const runs = JSON.parse(fs.readFileSync(path.join(dir, 'runs', 'index.json'), 'utf8'))
    return { meta, manifest, runs }
  }))

  socket.on('kinds', (_payload, ack) => reply(ack, async () => ({ kinds: await kinds() })))

  socket.on('docs.list', (_payload, ack) => reply(ack, async () => ({ pages: docsIndex() })))

  socket.on('doc.save', ({ path: p, markdown } = {}, ack) => reply(ack, async () => {
    if (READONLY) refuse()
    if (!validPath(p)) throw new Error('page path must be area/name in lower case letters, digits, - and _')
    claimArea(await who(), p.split('/')[0])
    if (typeof markdown !== 'string' || !markdown.trim()) throw new Error('an empty page is not saved')
    if (markdown.length > 200_000) throw new Error('page too long')
    const file = path.join(DOCS, `${p}.md`)
    fs.mkdirSync(path.dirname(file), { recursive: true })
    fs.writeFileSync(file, markdown.endsWith('\n') ? markdown : markdown + '\n')
    docsIndex()
    io.emit('docs.changed', { path: p })
    return { path: p }
  }))

  socket.on('project.check', ({ manifest } = {}, ack) => reply(ack, async () => { if (READONLY) refuse(); return { check: await checkManifest(String(manifest || '')) } }))

  socket.on('project.draft', ({ description, title } = {}, ack) => reply(ack, async () => {
    if (READONLY) refuse()
    if (PUBLIC && !(await who())) throw new Error('sign in through attest to draft here')
    if (!description || String(description).trim().length < 10) throw new Error('describe the project in a sentence or more')
    return draft(String(description), String(title || ''), (line) => socket.emit('project.log', { path: null, line }))
  }))

  socket.on('project.create', ({ path: p, title, description, manifest, by } = {}, ack) => reply(ack, async () => {
    if (READONLY) refuse()
    if (!validPath(p)) throw new Error('project path must be area/name in lower case letters, digits, - and _')
    if (!manifest) throw new Error('a manifest is required')
    claimArea(await who(), p.split('/')[0])
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'esim-new-'))
    const file = path.join(tmp, 'manifest.toml')
    fs.writeFileSync(file, manifest)
    try {
      const u = await who()
      const args = ['project', 'new', p, '--manifest', file, '--by', u ? `@${u.handle} via attest${by ? `, ${by}` : ''}` : (by || 'web')]
      if (title) args.push('--title', title)
      if (description) args.push('--description', description)
      const r = await esim(args)
      if (!r.ok) throw new Error(r.output.trim())
      io.emit('projects.changed', { path: p })
      return { path: p, output: r.output }
    } finally {
      fs.rmSync(tmp, { recursive: true, force: true })
    }
  }))

  socket.on('project.run', ({ path: p, variant, set } = {}, ack) => reply(ack, async () => {
    if (READONLY) refuse()
    if (!validPath(p)) throw new Error('bad project path')
    claimArea(await who(), p.split('/')[0])
    const args = ['project', 'run', p]
    if (variant) { if (!/^[a-z0-9][a-z0-9_-]*$/.test(variant)) throw new Error('bad variant name'); args.push('--variant', variant) }
    for (const [k, v] of Object.entries(set || {})) {
      if (!/^[a-z0-9_.-]+$/i.test(k) || String(v).length > 200) throw new Error(`bad override ${k}`)
      args.push('--set', `${k}=${v}`)
    }
    const r = await esim(args, (line) => socket.emit('project.log', { path: p, line }))
    io.emit('projects.changed', { path: p })
    return { ran: r.ok, output: r.output }
  }))
})

server.listen(PORT, '0.0.0.0', () => {
  console.log(`esim ${VERSION}${READONLY ? ' (read-only)' : ''}${PUBLIC ? ` (public; identity from ${ATTEST_URL})` : ''} on :${PORT}, root ${ROOT}, esim ${ESIM}, viewer ${fs.existsSync(DIST) ? 'from dist/' : 'NOT BUILT (npm run build)'}, model ${MODEL} (key ${process.env.ANTHROPIC_API_KEY ? 'set' : 'missing'}), local ${LOCAL_MODEL} at ${OLLAMA}`)
})
