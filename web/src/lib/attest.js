// Identity = attest (https://attest.monster). Sign-in opens a small popup on attest, where the person's passkey signs a
// 30-day delegation to a key this browser made for this origin (non-extractable, in IndexedDB). attest's own client library
// does that work; it is loaded from the service so there is one copy, always current. The server says which attest to use
// (the `hello` answer). What is kept here: the session attest hands back (its own localStorage key) and the attest origin.
// Adapted from matchmaker/src/lib/attest.js on 2026-11-03.
const KEY = 'esim.attest'
let origin = '', libs = null
export const attestOrigin = () => { if (origin) return origin; try { return localStorage.getItem(KEY) || '' } catch { return '' } }
export function setAttestOrigin(o) {
  if (!o || o === attestOrigin()) { origin = o || origin; return }
  origin = o; libs = null
  try { localStorage.setItem(KEY, o) } catch { /* fine */ }
}
async function lib() {
  const o = attestOrigin()
  if (!o) throw new Error('no identity service configured')
  if (!libs) libs = Promise.all([import(/* @vite-ignore */ o + '/attest-core.js'), import(/* @vite-ignore */ o + '/lib/verify.mjs')]).then(([A, V]) => ({ A, V })).catch((e) => { libs = null; throw new Error('cannot load the sign-in library from ' + o + ' (' + (e?.message || e) + ')') })
  return libs
}
export async function session() { try { return (await lib()).A.session() } catch { return null } }
// The socket handshake: the delegation plus the device key's signature over {type, del, origin, at}, fresh on every (re)connect.
export async function handshake() {
  if (!attestOrigin()) return null
  let L
  try { L = await lib() } catch { return null }
  const s = L.A.session()
  if (!s) return null
  const dev = await L.A.deviceKey()
  if (dev.did !== s.delegation.device) { L.A.clearSession(); return null }
  const at = new Date().toISOString()
  const sig = await L.V.signObject(dev.privateKey, { type: 'esim-auth', del: s.id, origin: location.origin, at })
  return { del: s.id, delegation: s.delegation, at, sig, salt: s.salt }
}
// Sign in on attest. A popup where one can work (open it synchronously in the tap handler and pass it, so Safari allows it),
// else a redirect to attest that comes back here with the session in the URL fragment (home-screen web apps, blocked popups).
export const standalone = () => { try { return matchMedia('(display-mode: standalone)').matches || navigator.standalone === true } catch { return false } }
export function openPopup() { if (standalone()) return null; try { return window.open('about:blank', 'attest-login', 'popup,width=420,height=560') } catch { return null } }
export async function signIn(popup) { const L = await lib(); return L.A.signIn(popup ? { popup } : { mode: 'redirect' }) }
export const returning = () => /^#attest-session=/.test(location.hash || '')
export async function signOut() { try { (await lib()).A.clearSession() } catch { /* fine */ } }
