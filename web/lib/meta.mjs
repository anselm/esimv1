// The tags a page carries for whoever reads the HTML without running the viewer: a link
// unfurled on LinkedIn, X, Slack or a search engine. The viewer is one page routed by
// path, so the server writes these per route into the built index.html before sending
// it (server.mjs sendPage). Every route gets a title, a description, a canonical URL and
// an image: a project's card (deploy/cards.sh composes projects/<area>/<name>/card.png
// from its cover, 1200 by 630), else its cover, else the site's card.
import fs from 'node:fs'
import path from 'node:path'

const SITE = 'esim'
const TAGLINE = 'A whole-system simulator: small modules of science on one clock, each reading real records, with ledgers that must balance. Scenarios, not forecasts, with their limits stated.'

const esc = (s) => String(s ?? '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
const clip = (s, n = 280) => { const t = String(s ?? '').replace(/\s+/g, ' ').trim(); return t.length <= n ? t : t.slice(0, n - 1).replace(/\s+\S*$/, '') + '…' }

let cache = { projects: null, docs: null, at: 0 }
function indexes(root) {
  const now = Date.now()
  if (cache.projects && now - cache.at < 5000) return cache
  const read = (f) => { try { return JSON.parse(fs.readFileSync(path.join(root, f), 'utf8')) } catch { return [] } }
  cache = { projects: read('projects/index.json'), docs: read('docs/index.json'), at: now }
  return cache
}

// the first paragraph of a Markdown page, without its comments, heading and markup
function firstParagraph(file) {
  let text
  try { text = fs.readFileSync(file, 'utf8') } catch { return '' }
  const paras = text.replace(/<!--[\s\S]*?-->/g, '').split(/\n\s*\n/).map((p) => p.trim()).filter((p) => p && !p.startsWith('#') && !p.startsWith('```') && !p.startsWith('|') && !p.startsWith('- ') && !p.startsWith('!['))
  return (paras[0] ?? '').replace(/\[([^\]]*)\]\([^)]*\)/g, '$1').replace(/[*_`]/g, '')
}

// what a route is about
export function describe(root, url) {
  const { projects, docs } = indexes(root)
  const parts = url.split('/').filter(Boolean).map(decodeURIComponent)
  const page = { title: SITE, description: TAGLINE, image: '/card.png', type: 'website', path: url }
  const top = parts[0]
  if (parts.length === 0 || top === 'all') return page
  if (top === 'docs') {
    if (parts.length === 1) return { ...page, title: `Documentation · ${SITE}`, description: 'How the simulator works and what it does not do: the kernel, the module kinds, manifests, projects and runs, the honesty rules, and the limits to read before any number.' }
    const p = parts.slice(1).join('/')
    const d = docs.find((x) => x.path === p)
    const desc = firstParagraph(path.join(root, 'docs', p + '.md'))
    return { ...page, title: `${d?.title ?? p} · ${SITE}`, description: clip(desc) || page.description, type: 'article' }
  }
  if (top === 'kinds') return { ...page, title: `Module kinds · ${SITE}`, description: 'Every module kind the simulator has, from its own declaration: what it reads, writes, publishes and accounts for, with every parameter named, defaulted and sourced.' }
  if (top === 'runs' || top === 'run') return { ...page, title: `Runs · ${SITE}` }
  if (top === 'new') return { ...page, title: `New project · ${SITE}` }
  if (parts.length === 1) {
    const n = projects.filter((p) => p.area === top).length
    return { ...page, title: `${top} · ${SITE}`, description: `${n ? `${n} project${n === 1 ? '' : 's'}` : 'The projects'} under ${top} on ${SITE}: ${TAGLINE.charAt(0).toLowerCase()}${TAGLINE.slice(1)}` }
  }
  const [area, name] = parts
  const p = projects.find((x) => x.area === area && x.name === name)
  if (!p) return page
  const dir = path.join(root, 'projects', area, name)
  const image = fs.existsSync(path.join(dir, 'card.png')) ? `/projects/${area}/${name}/card.png` : p.cover ? `/projects/${area}/${name}/${p.cover}` : '/card.png'
  const title = p.title ?? `${area}/${name}`
  const description = clip(p.description || firstParagraph(path.join(dir, 'README.md')) || TAGLINE)
  const base = { ...page, title: `${title} · ${SITE}`, description, image, type: 'article', created: p.created, author: area }
  if (parts[2] === 'compare') return { ...base, title: `${title}, the variants compared · ${SITE}`, description: clip(`${title}: every variant of the project side by side under one time cursor. ${p.description ?? ''}`) }
  if (parts[2]) {
    const r = (p.runs ?? []).find((x) => x.id === parts[2])
    if (!r) return base
    const when = (t) => new Date(t * 1000).toISOString().slice(0, 10)
    const what = r.variant ? `the ${r.variant} variant` : `seed ${r.seed}`
    return { ...base, title: `${title}: ${r.variant ?? `run ${r.id}`} · ${SITE}`, description: clip(`One run of ${title}, ${what}, ${when(r.start)} to ${when(r.end)}${r.verdict ? `, verdict: ${r.verdict}` : ''}. ${p.description ?? ''}`) }
  }
  return base
}

// the <head> block for a route; origin is the public origin the links are made absolute with
export function headTags(root, url, origin) {
  const m = describe(root, url)
  const abs = (u) => (u.startsWith('http') ? u : origin + u)
  const canonical = abs(m.path)
  const image = abs(m.image)
  const tags = [
    `<title>${esc(m.title)}</title>`,
    `<meta name="description" content="${esc(m.description)}" />`,
    `<link rel="canonical" href="${esc(canonical)}" />`,
    `<link rel="icon" href="/favicon.svg" type="image/svg+xml" />`,
    `<meta property="og:site_name" content="${SITE}" />`,
    `<meta property="og:type" content="${m.type}" />`,
    `<meta property="og:title" content="${esc(m.title)}" />`,
    `<meta property="og:description" content="${esc(m.description)}" />`,
    `<meta property="og:url" content="${esc(canonical)}" />`,
    `<meta property="og:image" content="${esc(image)}" />`,
    `<meta property="og:image:width" content="1200" />`,
    `<meta property="og:image:height" content="630" />`,
    `<meta property="og:image:alt" content="${esc(m.title)}" />`,
    `<meta name="twitter:card" content="summary_large_image" />`,
    `<meta name="twitter:title" content="${esc(m.title)}" />`,
    `<meta name="twitter:description" content="${esc(m.description)}" />`,
    `<meta name="twitter:image" content="${esc(image)}" />`,
  ]
  if (m.created) tags.push(`<meta property="article:published_time" content="${esc(m.created)}" />`)
  const ld = m.type === 'article'
    ? { '@context': 'https://schema.org', '@type': 'Article', headline: m.title, description: m.description, image, url: canonical, datePublished: m.created, author: { '@type': 'Person', name: m.author === 'anselm' ? 'Anselm Hook' : m.author }, publisher: { '@type': 'Organization', name: SITE, url: origin } }
    : { '@context': 'https://schema.org', '@type': 'WebSite', name: SITE, description: m.description, url: origin }
  tags.push(`<script type="application/ld+json">${JSON.stringify(ld).replace(/</g, '\\u003c')}</script>`)
  return tags.join('\n    ')
}

// every page a crawler may index, one line each
export function sitemap(root, origin) {
  const { projects, docs } = indexes(root)
  const urls = ['/', '/docs', '/kinds']
  for (const p of projects) { urls.push(`/${p.area}/${p.name}`); if ((p.runs ?? []).length > 1) urls.push(`/${p.area}/${p.name}/compare`) }
  for (const a of new Set(projects.map((p) => p.area))) urls.push(`/${a}`)
  for (const d of docs) urls.push(`/docs/${d.path}`)
  return `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls.map((u) => `  <url><loc>${esc(origin + u)}</loc></url>`).join('\n')}\n</urlset>\n`
}
