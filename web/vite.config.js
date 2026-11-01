import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))

// Serve the sibling ../runs and ../projects folders at /runs/ and /projects/ in dev and
// preview. A production deploy copies them into dist/ (not done here).
function serveFolder(prefix, root) {
  const types = {
    '.json': 'application/json',
    '.f32': 'application/octet-stream',
    '.md': 'text/markdown; charset=utf-8',
    '.jsonl': 'application/x-ndjson',
    '.toml': 'application/toml; charset=utf-8',
  }
  const handler = (req, res, next) => {
    if (!req.url || !req.url.startsWith(prefix)) return next()
    const rel = decodeURIComponent(req.url.split('?')[0].slice(prefix.length))
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
    name: `esim-serve-${prefix.replaceAll('/', '')}`,
    configureServer(server) { server.middlewares.use(handler) },
    configurePreviewServer(server) { server.middlewares.use(handler) },
  }
}

export default defineConfig({
  plugins: [
    tailwindcss(), svelte(),
    serveFolder('/runs/', path.resolve(here, '../runs')),
    serveFolder('/projects/', path.resolve(here, '../projects')),
  ],
  server: {
    // the project server (server.mjs) speaks socket.io on 5181
    proxy: { '/socket.io': { target: 'http://127.0.0.1:5181', ws: true } },
  },
})
