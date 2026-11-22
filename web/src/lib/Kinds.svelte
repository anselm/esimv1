<script>
  import Badge from './Badge.svelte'
  import Account from './Account.svelte'
  // Every module kind a person or a model can build on, from the kinds' own describes:
  // what it integrates and leaves out, its parameters with provenance, what it reads,
  // owns, publishes and subscribes, its ledgers, outputs and data.
  import { link } from './router.svelte.js'
  import { req } from './socket.js'
  import { LAYERS } from 'esimv1-viewer/layers.js'

  let kinds = $state(null)
  let error = $state(null)
  let q = $state('')
  let open = $state(null)

  async function load() {
    try { kinds = (await req('kinds')).kinds }
    catch {
      try { const r = await fetch('/docs/kinds.json', { cache: 'no-store' }); kinds = r.ok ? await r.json() : [] }
      catch (e) { error = e.message }
    }
  }
  $effect(() => { load() })
  // a picture per kind, made from a real run that uses it (web/scripts/kind-pictures.mjs)
  let pictures = $state({})
  $effect(() => { fetch('/docs/kinds/index.json').then((r) => (r.ok ? r.json() : {})).then((j) => { pictures = j }).catch(() => {}) })

  // by layer, in the layers' reading order; a kind with no layer comes last
  const groups = $derived.by(() => {
    const needle = q.trim().toLowerCase()
    const list = (kinds ?? []).filter((k) => !needle || JSON.stringify(k).toLowerCase().includes(needle))
    const out = LAYERS.map((L) => ({ ...L, kinds: list.filter((k) => k.layer === L.id) }))
    const rest = list.filter((k) => !LAYERS.some((L) => L.id === k.layer))
    if (rest.length) out.push({ id: '', name: 'No layer', color: '#888', kinds: rest })
    return out.filter((g) => g.kinds.length || !needle)
  })
  const firstSentence = (t) => { const m = String(t ?? '').match(/^.*?[.!?](\s|$)/); return m ? m[0].trim() : String(t ?? '') }
  const fmt = (v) => (v === null || v === undefined ? '—' : typeof v === 'object' ? JSON.stringify(v) : String(v))
  const n = (a) => (a ?? []).length
</script>

<a use:link={{ docs: true }} class="floating left-3 top-3 hover:text-brand">← docs</a>
<div class="floating right-3 top-3 flex items-center gap-3 mono text-xs select-none">
  <a use:link={{ docs: true }} class="hover:text-brand">docs</a>
  <Account /><Badge />
</div>

<h1 class="text-xl font-medium">Module kinds</h1>
<p class="mt-1 max-w-2xl text-sm muted">Every kind of module a scenario can use, grouped by the layer of the world it belongs to and described by itself: what it does and leaves out, where its numbers come from, what it reads and writes. Open a card for the detail. A layer with no kind yet is listed, because the gap is part of the picture. Each card's picture is an output of a real run that uses the kind: a map at the end of the run, or a series over it.</p>
<input class="field mt-4 w-64" bind:value={q} placeholder="filter" aria-label="filter kinds" />

{#if error}
  <p class="mt-6 text-sm text-leak">{error}</p>
{:else if kinds === null}
  <p class="mt-6 muted text-sm">loading…</p>
{:else}
  {#each groups as G (G.id)}
    <section class="mt-10">
      <div class="flex items-baseline gap-3 border-b-2 pb-1.5" style="border-color:{G.color}">
        <span class="inline-block h-3 w-3 rounded-full" style="background:{G.color}"></span>
        <h2 class="text-2xl font-medium">{G.name}</h2>
        {#if G.n}<span class="mono text-xs muted">{G.n}</span>{/if}
        <span class="mono text-xs muted">{G.kinds.length} {G.kinds.length === 1 ? 'kind' : 'kinds'}</span>
      </div>
      {#if !G.kinds.length}<p class="mt-3 text-sm muted">No module kind yet.</p>{/if}
      <ul class="mt-4 grid gap-4 md:grid-cols-2">
        {#each G.kinds as k (k.kind)}
          <li class="rounded-[3px] border border-neutral-200 dark:border-neutral-800 {open === k.kind ? 'md:col-span-2' : ''}" style="border-top:3px solid {G.color}">
            <button class="block w-full p-4 text-left" onclick={() => { open = open === k.kind ? null : k.kind }} aria-expanded={open === k.kind}>
              {#if pictures[k.kind] && open !== k.kind}
                <img src={pictures[k.kind].img} alt="{pictures[k.kind].what}, from {pictures[k.kind].title}" class="mb-3 aspect-[16/9] w-full rounded-[3px] object-cover" loading="lazy" />
                <div class="mb-2 text-[11px] muted">{pictures[k.kind].what}: {pictures[k.kind].how}, from <span class="mono">{pictures[k.kind].project}</span></div>
              {/if}
              <div class="mono text-base font-medium">{k.kind}</div>
              <p class="mt-1.5 text-sm leading-relaxed">{open === k.kind ? k.summary : firstSentence(k.summary)}</p>
              <div class="mt-2 mono text-[11px] muted">{k.dt} · {n(k.params)} settings · writes {n(k.fields_owned)} · reads {n(k.fields_read)} · {n(k.assets)} data</div>
            </button>
            {#if open === k.kind}
              <div class="grid gap-x-8 gap-y-4 px-4 pb-4 text-xs lg:grid-cols-2">
                <div>
                  <div class="label mb-1">parameters</div>
                  <table class="mono w-full">
                    <tbody>
                      {#each k.params ?? [] as p (p.name)}
                        <tr class="border-t border-neutral-100 align-top dark:border-neutral-900">
                          <td class="whitespace-nowrap pr-3 py-0.5">{p.name}</td>
                          <td class="whitespace-nowrap pr-3 py-0.5">{fmt(p.default)}{p.unit ? ` ${p.unit}` : ''}</td>
                          <td class="py-0.5 muted {/guess|recalled/i.test(p.note ?? '') ? 'text-leak' : ''}">{p.note}</td>
                        </tr>
                      {/each}
                    </tbody>
                  </table>
                </div>
                <div class="space-y-3">
                  {#if n(k.fields_read)}<div><div class="label mb-1">reads</div><div class="mono">{#each k.fields_read as f (f.name)}<div>{f.name} <span class="muted">{f.unit} · {f.required ? 'required' : 'optional'}{f.on ? ` · on ${f.on}` : ''}{f.note ? ` · ${f.note}` : ''}</span></div>{/each}</div></div>{/if}
                  {#if n(k.fields_owned)}<div><div class="label mb-1">owns</div><div class="mono">{#each k.fields_owned as f (f.name)}<div>{f.name} <span class="muted">{f.unit}</span></div>{/each}</div></div>{/if}
                  {#if n(k.subscribes)}<div><div class="label mb-1">subscribes</div><div class="mono">{#each k.subscribes as t (t.name)}<div>{t.name} <span class="muted">{t.note}</span></div>{/each}</div></div>{/if}
                  {#if n(k.publishes)}<div><div class="label mb-1">publishes</div><div class="mono">{#each k.publishes as t (t.name)}<div>{t.name} <span class="muted">{t.note}</span></div>{/each}</div></div>{/if}
                  {#if n(k.sends) || n(k.accepts)}<div><div class="label mb-1">directed</div><div class="mono">{#each k.sends ?? [] as t, i (i)}<div>sends {t.topic} → {t.to}</div>{/each}{#each k.accepts ?? [] as t (t.name)}<div>accepts {t.name} <span class="muted">{t.note}</span></div>{/each}</div></div>{/if}
                  {#if n(k.ledgers)}<div><div class="label mb-1">ledgers</div><div class="mono">{#each k.ledgers as l (l.name)}<div>{l.name} <span class="muted">{l.unit} · tolerance {l.tolerance}</span></div>{/each}</div></div>{/if}
                  {#if n(k.frames) || n(k.reports)}<div><div class="label mb-1">outputs</div><div class="mono">{#each k.frames ?? [] as f (f.name)}<div>map {f.name} <span class="muted">{f.unit}</span></div>{/each}{#each k.reports ?? [] as r (r.name)}<div>series {r.name} <span class="muted">{r.unit}</span></div>{/each}</div></div>{/if}
                  {#if n(k.assets)}<div><div class="label mb-1">data</div><div class="mono">{#each k.assets as a (a.key)}<div>{a.key} <span class="muted">{a.note}</span></div>{/each}</div></div>{/if}
                  {#if n(k.notes)}<div><div class="label mb-1">notes</div>{#each k.notes as t, i (i)}<p class="text-sm leading-relaxed">{t}</p>{/each}</div>{/if}
                </div>
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    </section>
  {/each}
{/if}
