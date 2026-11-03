// One socket to the project server (web/server.mjs, proxied by Vite at /socket.io).
// Requests are {name, payload} with an ack; the server pushes `project.log` lines and
// `projects.changed` notices. When the server is down, `req` rejects and the pages fall
// back to the static files Vite serves (projects/index.json, run folders).
import { io } from 'socket.io-client'
import { handshake } from './attest.js'

let socket = null
export function sock() {
  // auth is a function so every (re)connect carries a freshly signed attest handshake, or nothing
  if (!socket) socket = io({ path: '/socket.io', transports: ['websocket', 'polling'], reconnectionDelayMax: 5000, auth: (cb) => handshake().then((a) => cb(a ? { attest: a } : {})).catch(() => cb({})) })
  return socket
}

// After a sign-in or sign-out: reconnect so the handshake carries the new state, ask hello
// again, and tell the page (onIdentity) who it is now.
const identityFns = new Set()
export function onIdentity(fn) { identityFns.add(fn); return () => identityFns.delete(fn) }
export async function reconnect() {
  helloPromise = null
  const s = sock()
  s.disconnect()
  s.connect()
  const h = await hello()
  for (const fn of identityFns) fn(h)
  return h
}

export function req(name, payload = {}, timeoutMs = 600_000) {
  return new Promise((resolve, reject) => {
    const s = sock()
    const timer = setTimeout(() => reject(new Error(`${name}: no answer from the project server`)), timeoutMs)
    if (!s.connected) {
      // give a fresh page one second to connect before giving up
      const t0 = Date.now()
      const wait = () => {
        if (s.connected) return send()
        if (Date.now() - t0 > 8000) { clearTimeout(timer); return reject(new Error('project server not reachable')) }
        setTimeout(wait, 100)
      }
      return wait()
    }
    send()
    function send() {
      s.emit(name, payload, (res) => {
        clearTimeout(timer)
        if (!res) return reject(new Error(`${name}: empty answer`))
        if (res.ok) resolve(res)
        else reject(new Error(res.error || `${name} failed`))
      })
    }
  })
}

export function onLog(fn) {
  const s = sock()
  s.on('project.log', fn)
  return () => s.off('project.log', fn)
}

export function onChanged(fn) {
  const s = sock()
  s.on('projects.changed', fn)
  return () => s.off('projects.changed', fn)
}

// What the server is: version, whether this copy is read-only, which attest it trusts, and who
// this socket is signed in as. Resolved once per connection.
let helloPromise = null
export function hello() {
  if (!helloPromise) helloPromise = req('hello', {}, 5000).catch(() => ({ ok: false, readonly: true, offline: true }))
  return helloPromise
}
