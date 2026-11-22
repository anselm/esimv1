// A small Markdown renderer for the docs pages: headings, paragraphs, lists, fenced
// code, tables, links, emphasis and inline code. No dependency; no raw HTML passes
// through (everything is escaped first), so a page written by anyone is safe to render.

const esc = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')

// A relative image or link source resolves against `base` (a project folder), so a
// README can show a picture that sits beside it.
function src(h, base) {
  if (/^(https?:\/\/|\/|\?|#|mailto:)/.test(h)) return h
  if (base && /^[\w.\-\/]+$/.test(h) && !h.includes('..')) return `${base.replace(/\/$/, '')}/${h}`
  return '#'
}

function inline(s, base) {
  let out = esc(s)
  out = out.replace(/`([^`]+)`/g, (_, c) => `<code>${c}</code>`)
  out = out.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
  out = out.replace(/(^|[^*])\*([^*\n]+)\*/g, '$1<em>$2</em>')
  // ![alt](file): an image; relative to the page's base folder
  out = out.replace(/!\[([^\]]*)\]\(([^)\s]+)\)/g, (_, a, h) => `<img src="${src(h, base)}" alt="${a}" loading="lazy">`)
  // [text](href): same-site links route in place; others open normally
  out = out.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_, t, h) => `<a href="${src(h, base)}">${t}</a>`)
  return out
}

export function render(md, { base = '' } = {}) {
  const inl = (t) => inline(t, base)
  // HTML comments carry page metadata (order) and are never shown
  const lines = md.replace(/<!--[\s\S]*?-->/g, '').replace(/\r\n/g, '\n').split('\n')
  const html = []
  let i = 0
  const para = []
  const flush = () => { if (para.length) { html.push(`<p>${inl(para.join(' '))}</p>`); para.length = 0 } }
  while (i < lines.length) {
    const line = lines[i]
    if (/^```/.test(line)) {
      flush()
      const lang = line.slice(3).trim()
      const buf = []
      i++
      while (i < lines.length && !/^```/.test(lines[i])) buf.push(lines[i++])
      i++
      html.push(`<pre data-lang="${esc(lang)}"><code>${esc(buf.join('\n'))}</code></pre>`)
      continue
    }
    const h = line.match(/^(#{1,4})\s+(.*)$/)
    if (h) {
      flush()
      const id = h[2].toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')
      html.push(`<h${h[1].length} id="${id}">${inl(h[2])}</h${h[1].length}>`)
      i++
      continue
    }
    if (/^\s*[-*]\s+/.test(line) || /^\s*\d+\.\s+/.test(line)) {
      flush()
      const ordered = /^\s*\d+\./.test(line)
      const items = []
      // an indented line continues the item above it, even when it happens to start with
      // a number and a period ("2040. The first decades…"); an unindented one starts an item
      while (i < lines.length && (/^\s*[-*]\s+/.test(lines[i]) || /^\s*\d+\.\s+/.test(lines[i]) || (/^\s{2,}\S/.test(lines[i]) && items.length))) {
        if (/^\s{2,}\S/.test(lines[i]) && items.length && !/^\s{2,}[-*]\s+/.test(lines[i])) items[items.length - 1] += ' ' + lines[i].trim()
        else items.push(lines[i].replace(/^\s*([-*]|\d+\.)\s+/, ''))
        i++
      }
      html.push(`<${ordered ? 'ol' : 'ul'}>${items.map((t) => `<li>${inl(t)}</li>`).join('')}</${ordered ? 'ol' : 'ul'}>`)
      continue
    }
    if (/^\|/.test(line) && i + 1 < lines.length && /^\|?\s*:?-{2,}/.test(lines[i + 1])) {
      flush()
      const cells = (l) => l.replace(/^\||\|$/g, '').split('|').map((c) => c.trim())
      const head = cells(line)
      i += 2
      const rows = []
      while (i < lines.length && /^\|/.test(lines[i])) rows.push(cells(lines[i++]))
      html.push(`<table><thead><tr>${head.map((c) => `<th>${inl(c)}</th>`).join('')}</tr></thead><tbody>${rows.map((r) => `<tr>${r.map((c) => `<td>${inl(c)}</td>`).join('')}</tr>`).join('')}</tbody></table>`)
      continue
    }
    if (/^>\s?/.test(line)) {
      flush()
      const buf = []
      while (i < lines.length && /^>\s?/.test(lines[i])) buf.push(lines[i++].replace(/^>\s?/, ''))
      html.push(`<blockquote>${inl(buf.join(' '))}</blockquote>`)
      continue
    }
    if (/^\s*$/.test(line)) { flush(); i++; continue }
    if (/^(-{3,}|\*{3,})\s*$/.test(line)) { flush(); html.push('<hr>'); i++; continue }
    para.push(line.trim())
    i++
  }
  flush()
  return html.join('\n')
}

/// The first H1 of a page, for lists.
export function titleOf(md) {
  const m = md.match(/^#\s+(.+)$/m)
  return m ? m[1].trim() : null
}
