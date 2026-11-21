# A guide to writing a digital twin

21 November 2026. Claude (Fable 5.2) in the esimv1 room on canada1, session
`1cc66e9f-2882-4c70-b8d3-6d9d3cf1beb1`.

Anselm asked for "a guide on 'how to write your own digital twin' ... for both humans and also llms
as context", in `docs/`, reachable from the documentation, about best practice in general and not
about esimv1. His outline: the simulation core (shared data, modules, fields against agents, state
encodings and spatial query, messaging and the clock tick, a manifest); visualization, with 3D as a
storytelling tool (three.js, Bevy); intentions, clear results and several bottom lines, with
stretching against tearing; and the surrounding questions of privacy, ownership, stakeholders and
theory of change. Added while it was being written: "not just model the past ... have an actual
simulator - that can simulate and compute forward novel outcomes from existing state"; that many
tools and data exist and digitaltwins.wiki collects them; and that input is welcome to esimv1 and
the wiki.

It is `docs/guides/writing-a-digital-twin.md`, the first page of a new docs area `guides`
(`AREAS` and `SECTIONS` in `web/src/lib/DocsList.svelte`), linked from the overview. The raw file
is served at `/docs/guides/writing-a-digital-twin.md` for loading into a model.

Things I added beyond the outline: starting from one question, checking against a past event with
part of the record held back, ledgers, replay and ensembles, an order of work, a section of
instructions addressed to a language model, and a list of what the guide leaves out (sensor
coupling and data assimilation, learned models, performance, cost).

Sources: the wiki's page on The Grammar of Simulation, its front page, guide list and article
list, read on the day through a summarising fetch; the paper and the wiki's guides were not read
in full, and the page says so. The rest is opinion from building this engine and cites nothing.
