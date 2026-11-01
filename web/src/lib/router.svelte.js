// Query-string routing: `?run=<id>` is a run view, bare `/` is the list.
export const route = $state({ run: null })

function read() {
  route.run = new URLSearchParams(location.search).get('run')
}

export function go(run) {
  const url = run ? `?run=${encodeURIComponent(run)}` : location.pathname
  history.pushState({}, '', url)
  read()
}

if (typeof window !== 'undefined') {
  read()
  addEventListener('popstate', read)
}
