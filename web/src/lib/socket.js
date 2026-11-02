// One socket to the project server (web/server.mjs, proxied by Vite at /socket.io).
// Requests are {name, payload} with an ack; the server pushes `project.log` lines and
// `projects.changed` notices. When the server is down, `req` rejects and the pages fall
// back to the static files Vite serves (projects/index.json, run folders).
import { io } from 'socket.io-client'

let socket = null
export function sock() {
  if (!socket) socket = io({ path: '/socket.io', transports: ['websocket', 'polling'], reconnectionDelayMax: 5000 })
  return socket
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

// What the server is: version and whether this copy is read-only. Resolved once.
let helloPromise = null
export function hello() {
  if (!helloPromise) helloPromise = req('hello', {}, 5000).catch(() => ({ ok: false, readonly: true, offline: true }))
  return helloPromise
}
