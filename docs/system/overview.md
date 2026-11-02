<!-- order: 1 -->
# What this is

A whole-system simulator: many small pieces of science on one clock, each reading real
records and writing into shared state, with accounts that must balance at every seam. A
person, or more often a conversation with a model, arranges the pieces into a manifest;
the kernel runs it; the result is a run folder anyone can replay, score, audit and look
at.

It is a bucket of tools, not a product. Everything is a file or a command: the module
kinds are code in one repository today and will be code at URLs tomorrow; a manifest is
a TOML file; a project is a folder; a run is a folder; a page like this is a Markdown
file. The viewer and the server only read and write those files, and nothing on them is
needed to use the tools. That is deliberate. The people who build engines and models in
this field are not competitors; the aim is to give them and anyone else a place where
pieces from different hands can be checked against each other and against the record.

## The shape, in one paragraph

Records come in through a registry that pins every file it fetches by hash. Modules read
them and integrate forward, exchanging events and shared fields and moving conserved
quantities through ledgers. A manifest names the modules, their parameters, the place and
the window; manifests can include other manifests and projects can run a manifest under
named variants. Every run writes its log, its series, its map frames and its ledgers; an
auditor reads the finished folder and says what the numbers mean. Scores compare model
series to observed ones. The viewer shows a project's runs with their maps under one
time cursor.

## Where to start reading

- [Terminology](?doc=system/terminology), because the words have been fuzzy.
- [Honesty](?doc=system/honesty), the rules that make the rest worth doing.
- [Modules](?doc=system/modules), [Layers](?doc=system/layers), [Manifests](?doc=system/manifests), [Projects and runs](?doc=system/projects-and-runs).
- [Conversation](?doc=system/conversation), how a scenario gets built without hand-writing TOML.
- [Data](?doc=system/data), [Scoring and tearing](?doc=system/scoring-and-tearing), [Evolution](?doc=system/evolution).
- [Workspace](?doc=system/workspace), how the folders are laid out and how to behave in a shared one.
- [Time](?doc=system/time), [Vocabulary](?doc=system/vocabulary), [Adding a module](?doc=system/adding-a-module), [Limits today](?doc=system/limits).

The design brief and every decision since are dated entries in the repository's
`devlog/` folder. This documentation describes the system as it stands; the devlog says
why it stands that way.
