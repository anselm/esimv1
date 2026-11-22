// Object storage behind the site: a file under /projects/ that this machine does not hold is read
// from a bucket (S3 interface; Cloudflare R2 here) and passed on. The bucket stays private: the
// server signs each read, so there is no public address and no cross-origin rule to keep.
// Objects are stored gzipped with their content encoding (deploy/store.py); they are passed on as
// stored to a browser that takes gzip, and the log's topic filter is applied here as for a local file.
//
// Configured by ESIM_STORE (a file of key=value lines: access_key_id, secret_access_key, endpoint,
// bucket), or left off. No dependency: the request signing (AWS Signature Version 4) is below.
import fs from 'node:fs'
import https from 'node:https'
import zlib from 'node:zlib'
import crypto from 'node:crypto'
import readline from 'node:readline'

let conf = null
export function storeFrom(file) {
  if (!file || !fs.existsSync(file)) return null
  const c = Object.fromEntries(fs.readFileSync(file, 'utf8').split('\n').filter((l) => l.includes('=') && !l.startsWith('#')).map((l) => [l.slice(0, l.indexOf('=')).trim(), l.slice(l.indexOf('=') + 1).trim()]))
  if (!c.access_key_id || !c.secret_access_key || !c.endpoint || !c.bucket) return null
  conf = c
  return { bucket: c.bucket }
}

const hmac = (key, s) => crypto.createHmac('sha256', key).update(s).digest()
const sha = (s) => crypto.createHash('sha256').update(s).digest('hex')
function signedGet(key) {
  const host = new URL(conf.endpoint).host, path = '/' + conf.bucket + '/' + key.split('/').map(encodeURIComponent).join('/')
  const now = new Date().toISOString().replace(/[-:]|\.\d{3}/g, ''), day = now.slice(0, 8), empty = sha('')
  const canonical = `GET\n${path}\n\nhost:${host}\nx-amz-content-sha256:${empty}\nx-amz-date:${now}\n\nhost;x-amz-content-sha256;x-amz-date\n${empty}`
  const scope = `${day}/auto/s3/aws4_request`
  const k = hmac(hmac(hmac(hmac('AWS4' + conf.secret_access_key, day), 'auto'), 's3'), 'aws4_request')
  const sig = crypto.createHmac('sha256', k).update(`AWS4-HMAC-SHA256\n${now}\n${scope}\n${sha(canonical)}`).digest('hex')
  return { host, path, headers: { 'x-amz-date': now, 'x-amz-content-sha256': empty, Authorization: `AWS4-HMAC-SHA256 Credential=${conf.access_key_id}/${scope}, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature=${sig}` } }
}

/// Pass the object at `key` to `res`; `topic` filters a log to the lines carrying it. Calls
/// `missing()` when the bucket has no such object (or is unreachable) and nothing was sent.
export function sendFromStore(req, res, key, { topic = null, missing }) {
  if (!conf) return missing()
  const r = signedGet(key)
  https.get({ host: r.host, path: r.path, headers: r.headers }, (up) => {
    if (up.statusCode !== 200) { up.resume(); return missing() }
    const stored = up.headers['content-encoding'] === 'gzip', takes = /\bgzip\b/.test(req.headers['accept-encoding'] || '')
    res.setHeader('Content-Type', up.headers['content-type'] || 'application/octet-stream')
    res.setHeader('Cache-Control', 'no-cache'); res.setHeader('Vary', 'Accept-Encoding'); res.setHeader('X-Esim-From', 'store')
    if (topic) {
      const needle = `"topic":${JSON.stringify(topic)}`, out = takes ? zlib.createGzip({ level: 6 }) : res
      if (takes) { res.setHeader('Content-Encoding', 'gzip'); out.pipe(res) }
      const rl = readline.createInterface({ input: stored ? up.pipe(zlib.createGunzip()) : up, crlfDelay: Infinity })
      rl.on('line', (line) => { if (line.includes(needle)) out.write(line + '\n') }); rl.on('close', () => out.end())
    } else if (stored && takes) { res.setHeader('Content-Encoding', 'gzip'); up.pipe(res) }
    else if (stored) up.pipe(zlib.createGunzip()).pipe(res)
    else up.pipe(res)
  }).on('error', () => missing())
}
