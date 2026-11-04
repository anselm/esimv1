// Path routing, so every view is a shareable URL (query strings until 2026-11-04; the old
// forms are still read, the new ones are written):
//   /                           the projects
//   /<area>                     an area: the projects under one handle
//   /<area>/<name>              one project: its runs and variants, then its story
//   /<area>/<name>/<run id>     one run of that project
//   /<area>/<name>/compare      a project's variants side by side under one cursor
//   /runs                       the loose runs/ folder (the old list); /run/<id> one of its runs
//   /new                        the new-project form
//   /docs                       the documentation index; /docs/<area>/<name> one page (docs/new is the new-page form)
//   /kinds                      every module kind, from its describe
//   /all                        the front page with every project, not only the featured ones
// The top-level words above are reserved and no area may take them (RESERVED_AREAS in server.mjs).
export const route = $state({ area: null, project: null, run: null, runs: false, isNew: false, docs: false, doc: null, kinds: false, compare: null, all: false })

const TOP = new Set(['runs', 'run', 'new', 'docs', 'kinds', 'all', 'compare'])

function read() {
  const q = new URLSearchParams(location.search)
  const parts = location.pathname.split('/').filter(Boolean).map(decodeURIComponent)
  const r = { area: null, project: null, run: null, runs: false, isNew: false, docs: false, doc: null, kinds: false, compare: null, all: false }
  if (parts.length === 0) {
    // the query forms, as written before 2026-11-04 and in links that still carry them
    r.project = q.get('project'); r.run = q.get('run'); r.runs = q.has('runs'); r.isNew = q.has('new')
    r.docs = q.has('docs'); r.doc = q.get('doc'); r.kinds = q.has('kinds'); r.compare = q.get('compare'); r.all = q.has('all')
  } else if (TOP.has(parts[0])) {
    const [top, ...rest] = parts
    if (top === 'runs') r.runs = true
    else if (top === 'run') r.run = rest.join('/') || null
    else if (top === 'new') r.isNew = true
    else if (top === 'kinds') r.kinds = true
    else if (top === 'all') r.all = true
    else if (top === 'docs') { if (rest.length) r.doc = rest.join('/'); else r.docs = true }
  } else if (parts.length === 1) {
    r.area = parts[0]
  } else if (parts.length >= 2) {
    r.project = `${parts[0]}/${parts[1]}`
    if (parts[2] === 'compare') { r.compare = r.project; r.project = null }
    else if (parts[2]) r.run = parts.slice(2).join('/')
  }
  Object.assign(route, r)
}

const enc = (s) => s.split('/').map(encodeURIComponent).join('/')
export function href({ area = null, project = null, run = null, runs = false, isNew = false, docs = false, doc = null, kinds = false, compare = null, all = false } = {}) {
  if (project && run) return `/${enc(project)}/${encodeURIComponent(run)}`
  if (project) return `/${enc(project)}`
  if (area) return `/${encodeURIComponent(area)}`
  if (compare) return `/${enc(compare)}/compare`
  if (run) return `/run/${encodeURIComponent(run)}`
  if (runs) return '/runs'
  if (isNew) return '/new'
  if (doc) return `/docs/${enc(doc)}`
  if (docs) return '/docs'
  if (kinds) return '/kinds'
  if (all) return '/all'
  return '/'
}

export function go(to = {}) {
  history.pushState({}, '', href(to))
  read()
}

// For <a> elements: keep the real href for sharing, route in place on click.
export function link(node, to) {
  const apply = (t) => { node.setAttribute('href', href(t)) }
  apply(to)
  const onclick = (e) => {
    if (e.metaKey || e.ctrlKey || e.shiftKey || e.button !== 0) return
    e.preventDefault()
    go(to)
  }
  node.addEventListener('click', onclick)
  return { update: apply, destroy() { node.removeEventListener('click', onclick) } }
}

if (typeof window !== 'undefined') {
  read()
  addEventListener('popstate', read)
}
