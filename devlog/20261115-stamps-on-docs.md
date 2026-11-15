# Stamps on the documentation pages

15 November 2026. Written by Claude (Claude Code session in the esimv1 room on canada1), on Anselm's order.

## The ask

Anselm, of https://esimv1.exe.xyz/docs: "each page should indicate who wrote it (indicate claude as well) and if
revised or ordered by anybody else (anselm for now) and also the date and revision (most of the pages will be at
revision 1)." And of the home page: "even the home page should be dated - with @anselm and the date the file was
first created - @anselm can link to /anselm folder (like underneath the title or somewhere)."

His reason, which is the reason for the devlog too: "i tend to try very hard to avoid documentation or file rot -
and i found that in general it's a good pattern to try to datestamp files in some way ... when I look back at
projects that are 30 years old and cannot make heads or tails out of what is most important, what is current, what
the revisions are, what was the order of thinking at the time." README.md and TODO.md are his exceptions, as known
conventions.

## What was done

- All 24 Markdown files under `docs/` carry comment lines at the top: `author`, `ordered-by`, `revised-by` where
  it applies, `written`, `revised`, `revision`. The stamp is in the file, so it survives without the repository.
- The dates came from git: each file's first and latest commit (`git log --follow`). Several pages moved on
  4 November, and `--follow` was trusted across that move.
- Every page starts at revision 1 whatever its earlier changes; the count begins here. The RFC is at 4, its draft
  number. `revised-by: Anselm` is on the goals page only, whose goal sentence is his wording.
- The server writes the stamp into `docs/index.json`. A page shows it under its path, with a person's name as a link
  to their workspace; the index shows the date and revision beside each page.
- The two generated event pages get their stamp from `esim events --write`, with fixed dates in the generator.
- The home page says `@anselm · 1 November 2026` under the title, linked to `/anselm`. The date is the first commit
  of the page's file, `web/src/lib/ProjectList.svelte`; the repository itself began on 31 October.
- The rule is written in `docs/system/workspace.md` (Stamps) and in `CLAUDE.md`.

## What is left

- Nothing checks that `revised` is moved when a page changes. A check would compare it with the file's latest
  commit, leaving out commits that touch only the stamp lines.
- The dates in the generator for the event pages are fixed, so they will not follow a change to the tables.
- Project READMEs, `page.html` and `run.html` files carry no stamp. Project pages show a byline from `project.toml`.
- `author: Claude` does not say which model or session; the devlog entries of each day do.
