<!-- order: 8 -->
# Conversation

People will not write manifests by hand for long, and the system is built so they need
not. Most of what looks like infrastructure for people (describes, the check, the catalog
notes, the plain-language information lines) is equally infrastructure for a model: the
aim is that a model can read the toolchain's surface well enough to build a scenario from
a description, and that a person can read what it built.

## How a draft happens

1. The person gives a title and a description in their own words: what, where, over what
   time, and what they want to see.
2. The facade sends the description with `esim kinds --json` (every kind's describe
   under default parameters) and a short grammar of the manifest to a model. The grammar
   carries the house rules: real records only, state the place once, score what can be
   scored, do not invent parameters, say in the notes what the manifest cannot do.
3. The answer's TOML goes through `esim check`. Errors go back to the model with the
   check's words, for up to three rounds.
4. The person sees the manifest, the model's explanation of what it built and what it
   could not do, and the check's findings. They can edit the TOML or paste their own.
5. Create writes the project folder; Run executes it. The project's `by` line records
   which model drafted the manifest.

The model never runs anything and never sees the machine. It writes a document; the
kernel decides whether the document is runnable.

## Tiers

The facade uses the Anthropic API when it is usable, a local model through ollama when
it is not, and the person's own TOML when neither is. The answer is labelled with the
tier that produced it. A local model drafted the first guest project in one round; the
kernel accepted it unchanged except for the scoreboards it had missed, and the grammar
now names those.

## What makes a kind legible to a model

The same things that make it legible to a person: a one-sentence summary that says what
it integrates and what it leaves out; parameter notes that give provenance; required
reads that name the kinds which satisfy them; topic notes that give the data shape. A
kind whose describe is thin will be wired badly by a model and by a person alike. See
[Adding a module](/docs/system/adding-a-module).

## What the conversation is not

It is not the only interface, and it must never become one. Every manifest a model
writes is a file a person can read, check, edit and run from the command line. The
conversation is a lens over structured data; the structure is primary.

## Next

A second turn ("make the summer drier", "add the predators") on a draft that exists, and
the model asking for a place when the description has none.
