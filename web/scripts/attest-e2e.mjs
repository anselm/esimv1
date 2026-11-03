// End-to-end sign-in through attest on a public copy, with headless Chrome's virtual authenticator over CDP. Chrome cannot
// open the sign-in popup under CDP, so the test takes attest's redirect flow, the one a phone without popups takes: it opens
// attest's login page in a second tab with this page's device key, origin and return address, registers a passkey there, and
// attest sends that tab back with the session in the URL fragment. Then: the handshake signs the page in, the new-project form
// locks the area to the handle and says the area is free, a project is created under it (area.toml records the DID), creating
// under another area is refused, and signing out leaves the form asking for a sign-in. Adapted from matchmaker's test
// (2026-09-29). Needs both servers running:
//   (attest)  ATTEST_DB=/somewhere/attest.sqlite OAUTH=off GRAPH_IMPORT=off PORT=8100 node server/index.mjs   (in the attest repo)
//   (esim)    ESIM_PUBLIC=1 ATTEST_URL=http://localhost:8100 PORT=5183 node web/server.mjs                      (after npm run build)
// Usage: node web/scripts/attest-e2e.mjs [site=http://localhost:5183] [service=http://localhost:8100]
// The project it creates under projects/<handle>/ is deleted at the end; the test account stays on the local attest.
import WebSocket from 'ws'
import { spawn } from 'node:child_process'
import { existsSync, readdirSync, readFileSync, rmSync } from 'node:fs'
import { homedir } from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')
const pwDir = `${homedir()}/.cache/ms-playwright`
const pw = existsSync(pwDir) ? readdirSync(pwDir).filter((d) => d.startsWith('chromium-')).sort().map((d) => `${pwDir}/${d}/chrome-linux64/chrome`).filter(existsSync).pop() : null
const CHROME = process.env.CHROME || pw || 'chromium'
const site = process.argv[2] || 'http://localhost:5183', service = process.argv[3] || 'http://localhost:8100'
if (!/^https?:\/\/(localhost|127\.0\.0\.1)(:|\/|$)/.test(service)) { console.error('refusing to create test accounts on ' + service + '; run against a local attest'); process.exit(2) }
const port = 9500 + Math.floor(Math.random() * 100)
const chrome = spawn(CHROME, ['--headless=new', '--no-sandbox', '--disable-gpu', `--remote-debugging-port=${port}`, '--user-data-dir=/tmp/esim-attest-e2e-' + port, 'about:blank'], { stdio: 'ignore' })
for (let i = 0; i < 40; i++) { try { await fetch(`http://127.0.0.1:${port}/json/version`); break } catch { await new Promise((r) => setTimeout(r, 250)) } }
const { webSocketDebuggerUrl } = await (await fetch(`http://127.0.0.1:${port}/json/version`)).json()
const ws = new WebSocket(webSocketDebuggerUrl); await new Promise((r) => ws.on('open', r)); let id = 0; const sessions = new Map()
const send = (method, params = {}, sessionId) => new Promise((res, rej) => { const i = ++id; const h = (m) => { const j = JSON.parse(m); if (j.id === i) { ws.off('message', h); j.error ? rej(new Error(j.error.message)) : res(j.result) } }; ws.on('message', h); ws.send(JSON.stringify({ id: i, method, params, sessionId })) })
ws.on('message', (m) => { const j = JSON.parse(m); if (j.method === 'Target.attachedToTarget') sessions.set(j.params.targetInfo.targetId, j.params.sessionId); if (j.method === 'Runtime.exceptionThrown') console.error('  EXCEPTION', j.params.exceptionDetails.exception?.description?.split('\n')[0]); if (j.method === 'Runtime.consoleAPICalled' && (j.params.type === 'error' || j.params.type === 'warning')) console.error('  console.' + j.params.type, j.params.args.map((a) => a.value || a.description).join(' ').slice(0, 200)) })
const sleep = (ms) => new Promise((r) => setTimeout(r, ms)); let fails = 0; const ok = (c, m) => { console.log(c ? '  ✓' : '  ✗', m); if (!c) fails++ }
const evalIn = async (s, expression) => { const r = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true }, s); if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || 'eval failed'); return r.result.value }
const waitFor = async (s, expr, ms = 10000) => { const t0 = Date.now(); while (Date.now() - t0 < ms) { try { if (await evalIn(s, expr)) return true } catch {} await sleep(200) } return false }
const tab = async (url) => { const { targetId } = await send('Target.createTarget', { url: 'about:blank' }); await sleep(400); const s = sessions.get(targetId); await send('Runtime.enable', {}, s); await send('Page.enable', {}, s); await send('Page.navigate', { url }, s); await sleep(1800); return s }
const set = (sel, v) => `(() => { const el = document.querySelector(${JSON.stringify(sel)}); el.value = ${JSON.stringify(v)}; el.dispatchEvent(new Event('input', { bubbles: true })); return true })()`
await send('Target.setAutoAttach', { autoAttach: true, waitForDebuggerOnStart: false, flatten: true })

console.log('sign in through attest at', service, 'from', site)
const page = await tab(site + '/?new')
ok(await waitFor(page, `document.body.innerText.includes('sign in through attest (top right)')`), 'signed out, the new-project form asks for a sign-in')
ok(await waitFor(page, `localStorage.getItem('esim.attest') === ${JSON.stringify(service)}`), 'the server named its attest and the page stored it')
const dev = JSON.parse(await evalIn(page, `(async () => { const A = await import(${JSON.stringify(service + '/attest-core.js')}); const d = await A.deviceKey(); return JSON.stringify({ did: d.did, jwk: d.jwk }); })()`))
ok(dev.did.startsWith('did:key:zDna'), 'page holds a device key ' + dev.did.slice(0, 24) + '…')
const login = await tab(`${service}/login?device=${encodeURIComponent(dev.did)}&key=${encodeURIComponent(Buffer.from(JSON.stringify(dev.jwk)).toString('base64url'))}&origin=${encodeURIComponent(site)}&return=${encodeURIComponent(site + '/?new')}`)
await send('WebAuthn.enable', { enableUI: false }, login)
await send('WebAuthn.addVirtualAuthenticator', { options: { protocol: 'ctap2', transport: 'internal', hasResidentKey: true, hasUserVerification: true, isUserVerified: true, automaticPresenceSimulation: true } }, login)
const handle = 'es' + Math.random().toString(36).slice(2, 7)
await evalIn(login, `document.getElementById('handle').value = ${JSON.stringify(handle)}; document.getElementById('register').click(); true`)
ok(await waitFor(login, `location.origin === ${JSON.stringify(site)}`, 15000), 'registered @' + handle + ' on attest, delegated to the page\'s key, and attest sent the tab back to ' + site)
ok(await waitFor(login, `location.hash === ''`, 15000), 'fragment consumed')
ok(await waitFor(login, `document.body.innerText.includes('@' + ${JSON.stringify(handle)})`, 15000), 'handshake accepted: the corner shows @' + handle)
ok(await waitFor(login, `document.querySelector('input[readonly]')?.value === ${JSON.stringify(handle)}`, 10000), 'the area field is locked to the handle')
ok(await waitFor(login, `document.body.innerText.includes('is free; it becomes yours')`, 10000), 'the server says the area is free')
// create a project under the handle with a small manifest
const manifest = readFileSync(path.join(ROOT, 'manifests/m0-buckets.toml'), 'utf8')
await evalIn(login, set('input[placeholder="your name or a topic"] ~ input, input.field:not([readonly])', 'e2e buckets'))
await evalIn(login, `(() => { const inputs = [...document.querySelectorAll('input.field')]; const t = inputs.find(i => !i.readOnly); t.value = 'e2e buckets'; t.dispatchEvent(new Event('input', { bubbles: true })); const ta = [...document.querySelectorAll('textarea')].pop(); ta.value = ${JSON.stringify(manifest)}; ta.dispatchEvent(new Event('input', { bubbles: true })); return true })()`)
ok(await waitFor(login, `document.body.innerText.includes('/' + ${JSON.stringify(handle)} + '/e2e-buckets')`), 'the path reads /' + handle + '/e2e-buckets')
await evalIn(login, `[...document.querySelectorAll('button')].find(b => b.textContent.trim() === 'Create').click(); true`)
ok(await waitFor(login, `location.search === '?project=' + encodeURIComponent(${JSON.stringify(handle + '/e2e-buckets')}).replace('%2F', '/')  || location.search.includes('project=' + ${JSON.stringify(handle)})`, 20000), 'the project was created and the page moved to it')
const areaToml = path.join(ROOT, 'projects', handle, 'area.toml')
const area = existsSync(areaToml) ? readFileSync(areaToml, 'utf8') : ''
ok(area.includes(`handle = "${handle}"`), 'projects/' + handle + '/area.toml records the handle')
const S = JSON.parse(await evalIn(login, `localStorage.getItem('attest:session')`))
ok(area.includes(`did = "${S.root}"`), 'and the account DID ' + S.root.slice(0, 20) + '…')
const projToml = path.join(ROOT, 'projects', handle, 'e2e-buckets', 'project.toml')
ok(existsSync(projToml) && readFileSync(projToml, 'utf8').includes(`@${handle} via attest`), 'project.toml says who created it, by handle')
// another area is refused, even with the field unlocked by hand
await send('Page.navigate', { url: site + '/?new' }, login); await sleep(2500)
await evalIn(login, `(() => { const a = document.querySelector('input[readonly]'); a.removeAttribute('readonly'); a.value = 'anselm'; a.dispatchEvent(new Event('input', { bubbles: true })); const inputs = [...document.querySelectorAll('input.field')]; const t = inputs[inputs.length - 1]; t.value = 'e2e intruder'; t.dispatchEvent(new Event('input', { bubbles: true })); const ta = [...document.querySelectorAll('textarea')].pop(); ta.value = ${JSON.stringify(manifest)}; ta.dispatchEvent(new Event('input', { bubbles: true })); return true })()`)
await sleep(300)
await evalIn(login, `[...document.querySelectorAll('button')].find(b => b.textContent.trim() === 'Create').click(); true`)
ok(await waitFor(login, `document.body.innerText.includes('you can write under /' + ${JSON.stringify(handle)} + ' only')`, 10000), 'creating under /anselm is refused with the reason')
ok(!existsSync(path.join(ROOT, 'projects', 'anselm', 'e2e-intruder')), 'and nothing was written there')
// sign out
await evalIn(login, `[...document.querySelectorAll('button')].find(b => b.title === 'sign out').click(); true`); await sleep(2500)
ok(await evalIn(login, `!localStorage.getItem('attest:session')`), 'sign out cleared the attest session')
ok(await waitFor(login, `document.body.innerText.includes('sign in through attest (top right)')`), 'and the form asks for a sign-in again')
// a tampered session is refused with the reason
const bad = { ...S, id: S.id.replace(/^./, (c) => (c === '0' ? '1' : '0')) }
await evalIn(login, `localStorage.setItem('attest:session', ${JSON.stringify(JSON.stringify(bad))}); true`); await send('Page.navigate', { url: site + '/?new' }, login); await sleep(2500)
ok(await waitFor(login, `document.querySelector('button[title*="does not match"]') !== null`, 10000), 'a tampered delegation id is refused and the reason is on the sign-in control')
// cleanup
rmSync(path.join(ROOT, 'projects', handle), { recursive: true, force: true })
console.log(fails ? `${fails} FAILED` : 'all passed', '· test account @' + handle + ' remains on the local attest at ' + service + '; projects/' + handle + ' deleted')
ws.close(); chrome.kill(); process.exit(fails ? 1 : 0)
