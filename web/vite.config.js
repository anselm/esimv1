import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))

// Serve the sibling ../runs folder at /runs/ in dev and preview.
// A production deploy copies runs/ into dist/runs/ (not done here).
function serveRuns() {
  const root = path.resolve(here, '../runs')
  const types = {
    '.json': 'application/json',
    '.jsonl': 'application/x-ndjson',
    '.toml': 'application/toml; charset=utf-8',
  }
  const handler = (req, res, next) => {
    if (!req.url || !req.url.startsWith('/runs/')) return next()
    const rel = decodeURIComponent(req.url.split('?')[0].slice('/runs/'.length))
    const file = path.resolve(root, rel)
    if (!file.startsWith(root + path.sep)) {
      res.statusCode = 403
      return res.end('forbidden')
    }
    fs.stat(file, (err, st) => {
      if (err || !st.isFile()) {
        res.statusCode = 404
        return res.end('not found')
      }
      res.setHeader('Content-Type', types[path.extname(file)] || 'application/octet-stream')
      res.setHeader('Cache-Control', 'no-store')
      fs.createReadStream(file).pipe(res)
    })
  }
  return {
    name: 'esim-serve-runs',
    configureServer(server) { server.middlewares.use(handler) },
    configurePreviewServer(server) { server.middlewares.use(handler) },
  }
}

export default defineConfig({
  plugins: [tailwindcss(), svelte(), serveRuns()],
})
