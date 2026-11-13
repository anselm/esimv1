# The writing guidance, enumerated

2026-11-12, evening. Anselm read the Portland page and found nearly every sentence poor:
"Something is terribly wrong with the writing overall." "Have we over-constrained the guidance
on avoiding claudisms?" Then: "please enumerate your guidance around text production - i am
going to ammend it." This entry is that list. Written by Claude Code in the esim room on
canada1, session 0bad2b71 (transcript under
`~/.claude/projects/-home-anselm-projects-2026-earth-simulator-1/`). It rests on the files
named below, each opened today, and on a scan of the session transcript for when each was last
read. The Portland README is unchanged. Nothing here is pushed.

## Where the guidance lives

Nine places. The second column is where to edit. The third says how the text reaches a session.

| # | Source | Edit at | Reaches me |
|---|--------|---------|-----------|
| 1 | The session's system prompt, "Writing for the user" | not on disk; I found no setting of yours that adds it, so I take it to be the product's and cannot confirm | every turn; governs chat replies only |
| 2 | Global instructions | `~/projects/2026/harness/claude/CLAUDE.md` | every turn |
| 3 | Skill `writing-style` | `~/projects/2026/harness/claude/skills/writing-style/SKILL.md` | when invoked; in this session once, on 2026-11-03 |
| 4 | The primer's line-edit rules, which the skill calls the canonical ban list | `~/projects/2027/orbital/orbital-primer/claude.md` | only if opened; first opened in this session today |
| 5 | Feedback memories, twelve on prose | `~/projects/2026/harness/claude/memory/*.md` | the one-line index every turn; a file's body only if opened; none was opened here before today |
| 6 | This project's own pages | `docs/system/telling.md`, `docs/system/honesty.md`, the root `README.md`, `CLAUDE.md` | `CLAUDE.md` every turn; the others when opened |
| 7 | Heckler, the linter | `~/projects/2027/heckler/bin/heckler.mjs` | when run |
| 8 | The wiki's style page, status proposed, not yet read by you | `~/projects/2026/twin-compendium/data/about/style.md` | not in force in this room |
| 9 | Your essay "How to write less bad" | anselm.substack.com, 2026-09-09 | through the skill's summary of it |

## What each one says

**1. The system prompt (chat replies).** Lead with the answer. One idea per sentence, about
twenty words, with a verb. A new sentence instead of a semicolon. No em-dashes, parentheticals
or arrows. State facts, not my reasoning. Never use a name I made up during the session. Expand
an uncommon acronym at first use. Code and numbers out of the prose; a number goes in a small
table. Lists for parallel items. No headers under about 500 words. Stop when the content stops.

**2. Global instructions.** Never invent a fact, date, URL, author or title; say "I guessed".
Nothing publishes under your byline without your say. Real images or none. Durable thinking
goes in dated devlog entries. For public prose, use the writing-style skill; the essay is the
standard and heckler is "a rough filter".

**3. The writing-style skill.**
- Every public text gets an adversarial pass before it ships.
- Machine drafts, human voices.
- Philosophy at the top and bottom only, never woven through the numbers.
- The ban list: dry but not dull; no cinematic verbs; no empty intensifiers; no "it's not just
  X, it's Y"; no em-dashes; no self-congratulation; no groveling; a conversation between
  equals, not a lecture.
- Your reference voice: question volleys at the reader, first person, feelings named, a
  register that swings from lyrical to technical to colloquial, pieces built on an emotional arc.
- Slop patterns to cut, from the essay (the skill says thirteen and lists twelve): the kicker, the empty pivot, hedging and
  throat-clearing, the metronome, no lungs, threes on autopilot, the bow, false symmetry,
  sunk-cost polish, even effort, pointer words opening consecutive sentences, uniform altitude.
- Three checks: negate each sentence; cash out each metaphor; argue against each sentence.
- What you want: metaphor cashed out, breath-based pacing, humour, varied rhythm, first person
  where it fits, one reader at a time.
- Run heckler on any public draft before showing you.

**4. The primer's line-edit rules.** The skill carries this file's bans and leaves out the rest.
What it leaves out:
- An audience: "intelligent, curious readers with a STEM interest but not necessarily domain
  expertise".
- "Use plain sentences, active voice, and concrete examples."
- "Vary sentence length ... do not flatten everything into short declaratives."
- Keep an honest passive: "A forced actor-subject is worse than an honest passive."
- "Generation and editing are separate modes." The adversarial line-edit is an on-demand pass,
  "not something to apply to every draft while ideas are still forming".
- Precision over near-synonyms. Credit the originator of a term.
- "The rules serve the sentence; never sand prose into something correct but charmless."

**5. The memories.** Your words, by date.

| Date | File | The rule |
|------|------|----------|
| 2026-07-31 | `doc-style-terse` | devnotes and accountant documents short; lead with the decision |
| 2026-09-03 | `writing-voice-is-human-work` | a linter does not give tenor; deliver the structure and facts and stop; do not fake voice; beware "volunteer words for volunteer thoughts" |
| 2026-09-21 | `public-copy-no-hedging` | no mincing qualifications, no forensics, no status on a public page; status goes in the devlog |
| 2026-09-22 | `surface-copy-plain-rigour-beneath` | the surface is a plain story hook; rigour moves one layer down |
| 2026-09-23 | `articles-never-tell-reader-what-to-think` | no telling the reader how to feel; no moralising last line |
| 2026-09-28 | `feedback-no-watered-down-prose` | when editing your prose keep the edge and the specifics |
| 2026-09-28 | `humble-public-copy` | leave you out of public copy unless cited |
| 2026-10-19 | `feedback-no-strawman-contrast` | describe the thing, never the thing it is not; no kickers; the reader is a guest |
| 2026-10-20 | `feedback-noun-list-then-swelling-clause` | flat lists; no final clause that swells and ends on "it" |
| 2026-10-29 | `feedback-slides-compression-reads-stilted` | compression into fragments reads stilted; whole sentences; no inside thoughts on a public surface |
| 2026-11-04 | `feedback-deep-histories-totality` | every topic as priming, event, aftermath |
| 2026-11-11 | `feedback-no-announcing-or-grading-sentences` | cut sentences that announce or grade what comes next |

Two more bear on register: `feedback-goals-before-technique` (answer at the goal level, mechanism
below the fold) and `self-taught-wants-lineage` (name the precedents).

The index file is over its size limit, so its last twelve lines never load. Two of them are
`writing-voice-is-human-work` and `feedback-slides-compression-reads-stilted`.

**6. This project's own pages.** I wrote these, from your asks.
- `docs/system/honesty.md`: "The project page says 'an assumption, never a forecast', in those
  words." "Every milestone entry and every project description leads with what the model gets
  wrong before what it gets right." "Where the engine cannot yet compute a thing, the page says so."
- `docs/system/telling.md`: most readers are novices; three parts (priming, event, aftermath);
  the actors figure; "the project page is a story"; "the register is sober"; a first paragraph
  "that says what the page is, who wrote it and when, and that nothing on it is a forecast"; what
  each run gets wrong "said in the same breath as what it gets right"; a closing reading list
  with the day each source was read and what is only recalled.
- Root `README.md`: "Lead with what is wrong."
- `CLAUDE.md`: no prose rule of its own. It is 5,820 words of my own compressed notes, loaded
  every turn.

**7. Heckler.** Ten patterns: "not just X but Y", stock verbs, "plays a vital role",
"it is worth noting", formal connectives, consultant filler, the em-dash aside, the bow, empty
intensifiers, exclamation marks. It also measures rhythm and function-word distance from your
corpus, and counts three things as good: question volleys, first person, a concrete number.
Run today on the Portland page it reports "no paragraph-level tells found".

**8. The wiki's style page.** Not in force here, and the only place with rules for building a
sentence: subject then verb, soon; known first, new last; short words; a term explained in a
clause at first use; a sentence over 45 words is split. It also has "What goes in": a claim that
could be false, the figure, who says so, how sure once, one argument once.

## What the list adds up to

**Mostly prohibitions.** In the set that was in force when the Portland page was written (2, 3,
the memory index, 6 and 7) I count about twenty-six prohibitions and about twelve positive
instructions. The count is mine and rough. Half of the positive ones describe your essay voice,
which I am told elsewhere not to imitate.

**No rule says how to build a sentence, and none names the reader of a project page.** The chat
rules do both, and the chat reads plainly. The page had no such rule and came out at a median of
34 words a sentence, 15 sentences over 60 words, 14 openings of the form "What ... is", 33
semicolons, and "DOGAMI" 19 times and never spelled out.

**Every check is negative.** A page passes by not containing a listed shape. Nothing asks
whether a stranger could follow it. Heckler passes the page you found poor throughout.

**The rules contradict each other, and I resolved each conflict toward the newer or nearer text.**
- Honesty against plain copy. `honesty.md` and `telling.md` tell the page to lead with what is
  wrong, to say so in the same breath, and to say "never a forecast" in those words. Your
  feedback of 2026-09-21 and 2026-09-22 says no forensics and no status on a public page. The
  Portland page follows the first: it confesses in every paragraph and narrates its own revisions.
  "A scenario, not a forecast" is that instruction carried out, and it is also the foil shape
  you banned on 2026-10-19.
- Voice. One memory says deliver facts and structure and leave voice to you. The skill
  describes your lyrical register, and `telling.md` says the page is a story. The Portland lede
  performs a voice.
- Questions and first person. The skill and heckler count them as good. The wiki page bans
  them. Nothing says which a project page here is.
- Length. Terse on one side; deep history, the same-breath caveat and the reading list on the
  other. The page is 5,250 words.
- When to apply the bans. The primer says after drafting. The skill dropped that line, and I
  apply them while drafting.

## Why the Portland page reads as it does

1. **Each ban left a workaround.** No em-dash and no parenthesis gave the semicolon chain. No
   foil, no announcement and no pointer word gave the pseudo-cleft: "The thing Portland decided
   this week is about a six-mile strip". Avoiding a blunt claim gave coy pointing: "the
   afternoon this page is about" for a date, "a tsunami with no earthquake behind it" for a
   tsunami nobody in Japan felt the cause of.
2. **I wrote in the session's private dialect.** "The thread", "the window", "the record", "the
   bundle", "says so". The largest sample of prose in my context is my own: the 5,820 words of
   `CLAUDE.md`, the commit messages, the summaries. The page is in that register.
3. **The page is a devlog.** "Now computes rather than arranges", "no longer reports". That is
   the honesty rule applied sentence by sentence, on a public page.
4. **About nine patches in a day and no rewrite.** Each round added clauses to standing sentences.
   Numbers from superseded runs are still on the page.
5. **Vague phrasing hid a gap.** "The thing Portland decided" asserts a vote I never confirmed.
   The record file says the result was not recorded when read. A plain sentence would have had
   to name the vote and its count, and I would have seen I did not have it.

So: yes, over-constrained, and in a specific way. The constraints are nearly all negative, they
are applied while drafting, and nothing positive stands against them. Some of the trouble is
not the guidance: your plain-surface and no-forensics feedback was on file and I did not open it.

## The five points

1. **"A scenario, not a forecast."** It means nobody can say when the fault will next break, so
   we chose a date and ran the earthquake the state plans for. It should say that.
2. **DOGAMI** is the Oregon Department of Geology and Mineral Industries. Spelled out once, then
   "the state's geologists" or the acronym.
3. **"Because there was no Portland."** Dropped. The point worth making is yours: the same
   rupture now meets bridges from 1908, tanks built on average in 1954 and 615,852 buildings,
   most of them older than the science that found the fault.
4. **"That night."** Not a participle; it is an adverb of time and grammatical. It is still
   wrong for the sentence: it points back across a colon to a date two clauses earlier, and it
   blurs a fact. The wave reached Japan about ten hours after the rupture, near midnight on
   27 January by Japan's calendar. That figure is recalled from Satake's 1996 paper, not reread.
5. **"The afternoon this page is about."** It avoids naming a date. "The scenario's date is 327
   years later."

One sentence, before and after:

> The thing Portland decided this week is about a six-mile strip of the west bank of the
> Willamette in the north of the city, between the Fremont Bridge and Linnton, which the state
> calls the Critical Energy Infrastructure Hub.

> More than 90 percent of Oregon's liquid fuel is stored at one tank farm on the Willamette in
> northwest Portland. The state calls it the Critical Energy Infrastructure Hub. On 12 November
> 2026 Portland's council was due to vote on Ordinance 2026-312, which would make each terminal
> there cut its storage capacity by at least 20 percent by December 2036. I have not seen the
> result.

## Wrong on the live page, apart from style

- It says the ordinance passed. Unverified. The first reading passed 11 to 0 on 5 November; I
  found no report of the 12 November vote.
- Expected deaths appear as 765, 1,000 and 670 without saying which is which.
- One bullet has about 150 million gallons leaving the tanks and another has 61 million.
- "What is arranged rather than computed: the lifelines" contradicts the base run, which
  computes them.
- "The next rungs" lists poles and routes that are built.
- "Eleven of twelve close on the first day"; the base run has ten.
- "About four hundred tanks"; the record has 415 active and the model 587.
- The hub's north end is given as Linnton; the Oregonian says the southern tip of Sauvie Island.

## Candidates for the amendment

These are proposals, not rules. They are yours to take or leave.

1. Name the reader and the job for each surface. For a project page: a curious person who
   knows neither the subject nor simulation, who wants to know what would happen and how far to
   trust the answer.
2. Put one positive rule above the bans: say who did what, with the name, the date and the
   number; subject then verb, soon; a term or acronym explained at first use; no label coined in
   the session; split a sentence past about thirty words.
3. Restore the primer's line: draft first, lint after. The bans become a second pass.
4. Settle honesty against plain copy. One suggestion: the page says what the model cannot do
   once, in one section near the end; revision history and status live in the devlog and on the
   limits page.
5. Settle voice for project pages: plain reference register, no performed lede, your hand for
   anything that should sound like you.
6. After three rounds of patches, rewrite the page whole from a fact sheet. A fresh writer
   given only the fact sheet and the reader would break the self-imitation.
7. Cut this project's `CLAUDE.md` to orientation and move its history to the devlog. Trim the
   memory index so the last lines load; that file is the manager's.
8. If heckler is to catch this, it needs sentence length, "What ... is" openings, undefined
   acronyms, "this page", and semicolons per thousand words. It would still be a negative test.
