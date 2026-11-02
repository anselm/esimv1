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
// Notices: projects.changed.
//
// It also serves the viewer: the built pages from dist/ when `npm run build` has been
// run, and the ../runs and ../projects folders at /runs/ and /projects/. So one process
// is the whole thing in production; in development Vite serves the pages with hot
// reload and proxies the socket here.
//
// Start: node server.mjs   (port 5181; ANTHROPIC_API_KEY in the environment for drafts)

import http from 'node:http'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { spawn, execFile } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { Server } from 'socket.io'
import Anthropic from '@anthropic-ai/sdk'

const here = path.dirname(fileURLToPath(import.meta.url))
const ROOT = path.resolve(here, '..')
const ESIM = process.env.ESIM_BIN || path.join(ROOT, 'target/release/esim')
const PORT = Number(process.env.PORT || 5181)
const MODEL = process.env.ESIM_MODEL || 'claude-opus-5-5'
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
  if (!kindsCache) kindsCache = await esimJson(['kinds', '--json'])
  return kindsCache
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
const TYPES = {
  '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8',
  '.json': 'application/json', '.jsonl': 'application/x-ndjson', '.toml': 'application/toml; charset=utf-8',
  '.f32': 'application/octet-stream', '.md': 'text/markdown; charset=utf-8', '.svg': 'image/svg+xml',
  '.png': 'image/png', '.ico': 'image/x-icon', '.woff2': 'font/woff2', '.map': 'application/json',
}
function sendFile(res, file, cache) {
  fs.stat(file, (err, st) => {
    if (err || !st.isFile()) { res.statusCode = 404; return res.end('not found') }
    res.setHeader('Content-Type', TYPES[path.extname(file)] || 'application/octet-stream')
    res.setHeader('Cache-Control', cache)
    fs.createReadStream(file).pipe(res)
  })
}
function serveStatic(req, res) {
  const url = decodeURIComponent((req.url || '/').split('?')[0])
  const under = (prefix, root) => {
    const file = path.resolve(root, url.slice(prefix.length))
    if (!file.startsWith(root + path.sep) && file !== root) { res.statusCode = 403; res.end('forbidden'); return true }
    sendFile(res, file, 'no-store')
    return true
  }
  if (url.startsWith('/runs/')) return under('/runs/', path.join(ROOT, 'runs'))
  if (url.startsWith('/projects/')) return under('/projects/', path.join(ROOT, 'projects'))
  if (url.startsWith('/docs/')) return under('/docs/', DOCS)
  if (!fs.existsSync(DIST)) { res.statusCode = 503; return res.end('viewer not built: run `npm run build` in web/, or use the Vite dev server') }
  // the viewer: a file from dist/ when it exists, else index.html (routes are query strings, so this is rare)
  const file = path.resolve(DIST, '.' + (url === '/' ? '/index.html' : url))
  if (!file.startsWith(DIST + path.sep)) { res.statusCode = 403; return res.end('forbidden') }
  fs.stat(file, (err, st) => {
    if (!err && st.isFile()) return sendFile(res, file, url.startsWith('/assets/') ? 'public, max-age=31536000, immutable' : 'no-cache')
    sendFile(res, path.join(DIST, 'index.html'), 'no-cache')
  })
}

// ---- docs: Markdown pages under ../docs/<area>/<name>.md; the index is written for the
// static fallback. A page's title is its first heading; `<!-- order: n -->` on the first
// line orders pages within an area.
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
      pages.push({ path: `${area.name}/${f.slice(0, -3)}`, title, order })
    }
  }
  pages.sort((a, b) => a.path.localeCompare(b.path))
  try { fs.writeFileSync(path.join(DOCS, 'index.json'), JSON.stringify(pages, null, 2)) } catch { /* read-only checkout */ }
  return pages
}
docsIndex()
// the kinds, for the static fallback of the kinds page
kinds().then((ks) => { try { fs.writeFileSync(path.join(DOCS, 'kinds.json'), JSON.stringify(ks)) } catch { /* read-only */ } }).catch(() => {})

// ---- server
const server = http.createServer(serveStatic)
const io = new Server(server, { cors: { origin: true } })

io.on('connection', (socket) => {
  const reply = (ack, fn) => Promise.resolve().then(fn).then((v) => ack?.({ ok: true, ...v }), (e) => ack?.({ ok: false, error: e.message || String(e) }))

  socket.on('projects.list', (_payload, ack) => reply(ack, async () => ({ projects: await esimJson(['projects', '--json']) })))

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
    if (!validPath(p)) throw new Error('page path must be area/name in lower case letters, digits, - and _')
    if (typeof markdown !== 'string' || !markdown.trim()) throw new Error('an empty page is not saved')
    if (markdown.length > 200_000) throw new Error('page too long')
    const file = path.join(DOCS, `${p}.md`)
    fs.mkdirSync(path.dirname(file), { recursive: true })
    fs.writeFileSync(file, markdown.endsWith('\n') ? markdown : markdown + '\n')
    docsIndex()
    io.emit('docs.changed', { path: p })
    return { path: p }
  }))

  socket.on('project.check', ({ manifest } = {}, ack) => reply(ack, async () => ({ check: await checkManifest(String(manifest || '')) })))

  socket.on('project.draft', ({ description, title } = {}, ack) => reply(ack, async () => {
    if (!description || String(description).trim().length < 10) throw new Error('describe the project in a sentence or more')
    return draft(String(description), String(title || ''), (line) => socket.emit('project.log', { path: null, line }))
  }))

  socket.on('project.create', ({ path: p, title, description, manifest, by } = {}, ack) => reply(ack, async () => {
    if (!validPath(p)) throw new Error('project path must be area/name in lower case letters, digits, - and _')
    if (!manifest) throw new Error('a manifest is required')
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'esim-new-'))
    const file = path.join(tmp, 'manifest.toml')
    fs.writeFileSync(file, manifest)
    try {
      const args = ['project', 'new', p, '--manifest', file, '--by', by || 'web']
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
    if (!validPath(p)) throw new Error('bad project path')
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
  console.log(`esim on :${PORT}, root ${ROOT}, esim ${ESIM}, viewer ${fs.existsSync(DIST) ? 'from dist/' : 'NOT BUILT (npm run build)'}, model ${MODEL} (key ${process.env.ANTHROPIC_API_KEY ? 'set' : 'missing'}), local ${LOCAL_MODEL} at ${OLLAMA}`)
})
