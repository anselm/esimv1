<script>
  import Badge from './Badge.svelte'
  import Account from './Account.svelte'
  // Every module kind a person or a model can build on, from the kinds' own describes:
  // what it integrates and leaves out, its parameters with provenance, what it reads,
  // owns, publishes and subscribes, its ledgers, outputs and data.
  import { link } from './router.svelte.js'
  import { req } from './socket.js'

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

  // family = the prefix before the dot, which is a rough layer
  const families = $derived.by(() => {
    const m = new Map()
    const needle = q.trim().toLowerCase()
    for (const k of kinds ?? []) {
      if (needle && !JSON.stringify(k).toLowerCase().includes(needle)) continue
      const f = k.kind.split('.')[0]
      if (!m.has(f)) m.set(f, [])
      m.get(f).push(k)
    }
    return [...m.entries()]
  })
  const fmt = (v) => (v === null || v === undefined ? '—' : typeof v === 'object' ? JSON.stringify(v) : String(v))
  const n = (a) => (a ?? []).length
</script>

<a use:link={{}} class="floating left-3 top-3 hover:text-brand">← projects</a>
<div class="floating right-3 top-3 flex items-center gap-3 mono text-xs select-none">
  <a use:link={{ docs: true }} class="hover:text-brand">docs</a>
  <Account /><Badge />
</div>

<h1 class="text-xl font-medium">Kinds</h1>
<p class="mt-1 max-w-2xl text-sm muted">The module kinds a manifest can instantiate, described by themselves. A kind says what it integrates and what it leaves out, where its numbers come from, what it reads and owns. <span class="mono">esim kinds --json</span> prints the same.</p>
<input class="field mt-4 w-64" bind:value={q} placeholder="filter" aria-label="filter kinds" />

{#if error}
  <p class="mt-6 text-sm text-leak">{error}</p>
{:else if kinds === null}
  <p class="mt-6 muted text-sm">loading…</p>
{:else}
  {#each families as [family, list] (family)}
    <section class="mt-8">
      <div class="label mb-2">{family} · {list.length}</div>
      <ul class="divide-y divide-neutral-200 dark:divide-neutral-800">
        {#each list as k (k.kind)}
          <li class="py-3">
            <button class="w-full text-left" onclick={() => { open = open === k.kind ? null : k.kind }}>
              <div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
                <span class="mono font-medium">{k.kind}</span>
                <span class="mono text-xs muted">priority {k.priority} · {k.dt} · {n(k.params)} params · owns {n(k.fields_owned)} · reads {n(k.fields_read)} · {n(k.assets)} assets</span>
              </div>
              <p class="mt-1 text-sm leading-relaxed">{k.summary}</p>
            </button>
            {#if open === k.kind}
              <div class="mt-3 grid gap-x-8 gap-y-4 text-xs lg:grid-cols-2">
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
