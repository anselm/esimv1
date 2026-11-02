# 2026-11-01 — Documentation pages, and whether to add accounts now

Anselm asked for a documentation link beside "+ project": wiki-like, Markdown files on a
first-come-first-served path like projects, with system-level documentation to start,
and a list of what it should cover with an invitation to think about what was missing.
Then, mid-work, whether to add real accounts through his authentication service at
attest.monster now, or focus on the 2100 scenario first. Agent: Claude (Fable 5.1),
earth-simulator-1 room on canada1.

## The pages

`docs/<area>/<name>.md`. The viewer lists them (`?docs`), renders them (`?doc=area/name`)
with a small renderer that escapes everything and passes no raw HTML through, and edits
or creates them through the server (`doc.save`), which also writes `docs/index.json` for
the static fallback. The `system` area is the simulator's own documentation; any other
area is whoever writes there first, as with projects.

Sixteen system pages, in reading order: what this is; terminology; honesty; modules;
layers; manifests; projects and runs; conversation; data; scoring and tearing;
evolution; workspace; time; vocabulary; adding a module; limits today. His list covered
the first eleven. The five added are the ones I thought were missing: the honesty rules
as a first-class page rather than a paragraph in a README, because everything else rests
on them; time and ordering, which every module author gets wrong once; the vocabulary,
because topic and field names are the real interoperability surface and nothing governed
them in writing; a recipe for adding a module, since the pool is meant to grow from many
hands; and a page that states what the models get wrong today, to be read before any
number and rewritten as it changes.

Two choices in the terminology page worth noting: "kernel" replaces "harness" and
"module" replaces "compute module" and "agent"; and "scenario" is kept as the
conversational word for a manifest plus its assumptions, with the manifest as the
artefact. "Patch" is given a definition, because the planetary picture needs it.

## Accounts: the recommendation

Not yet, and not because the idea is wrong. The attest.monster design is right for this:
a passkey identity whose handle is the area, so a user can only create under
`projects/<handle>` and `docs/<handle>`, with everything else staying files. It is about
a day of work on the server (verify the token, map handle to area, refuse writes outside
it) and nothing in the layout moves. The swear-word filter belongs on area and project
names when they become public, and nowhere else.

Three reasons to do the 2100 scenario first:

1. Accounts only matter on a public machine, and this one is reachable only over the
   private network. Accounts and the public deployment are one piece of work, not two,
   and the deployment was deferred this morning for good reasons.
2. The first thing a visitor with an account would see is a planet running half a degree
   warm in 2024 with crossing years twenty years early. The two causes (no aerosols; a
   carbon cycle that keeps too much airborne) are known, bounded and on the list. A
   visitor who sees them fixed learns what the project is; one who sees them unfixed
   learns the opposite.
3. The area convention already gives what accounts give for the people who can reach the
   machine today, which is one person.

So: aerosols and the box carbon cycle, then humidity, then accounts together with the
deployment, in that order. If he decides otherwise it is a day and the design above
holds.
