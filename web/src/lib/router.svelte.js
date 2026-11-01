// Query-string routing, so every view is a shareable URL:
//   /                           the projects
//   ?project=area/name          one project: its description, manifest and runs
//   ?project=area/name&run=id   one run of that project
//   ?run=id                     a run in the loose runs/ folder (the old list)
//   ?runs                       the loose runs/ folder itself
//   ?new                        the new-project form
export const route = $state({ project: null, run: null, runs: false, isNew: false })

function read() {
  const q = new URLSearchParams(location.search)
  route.project = q.get('project')
  route.run = q.get('run')
  route.runs = q.has('runs')
  route.isNew = q.has('new')
}

export function href({ project = null, run = null, runs = false, isNew = false } = {}) {
  const q = new URLSearchParams()
  if (project) q.set('project', project)
  if (run) q.set('run', run)
  if (runs) q.set('runs', '')
  if (isNew) q.set('new', '')
  const s = q.toString().replace(/=(&|$)/g, '$1')
  return s ? `?${s}` : location.pathname
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
