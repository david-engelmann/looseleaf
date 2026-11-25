# looseleaf

Looseleaf is a memory system for AI agents. It keeps what an agent knows in plain Markdown files that the user owns and can read, edit and move with no service running.

An agent's working notes pile up fast and go stale faster. Looseleaf gives those notes a fixed shape and a maintenance discipline. The shape is a bundle of Markdown files with YAML frontmatter that follows the Open Knowledge Format at version 0.3. The discipline is a synthesis pass that reads the daily logs, promotes what still holds into curated memory and retires what no longer holds by superseding it with a pointer to its replacement. A looseleaf binder holds pages that can be rearranged, replaced and removed, and a memory kept this way works the same way. New pages go in as the days accumulate, settled pages are replaced as the facts change, and stale pages come out under stated rules rather than by drift.

Every claim in the bundle carries its provenance. A claim keeps a stable identity, the exact words it came from and a link to the message or document behind it. When a claim is corrected, the old wording stays in the daily log untouched and a new claim supersedes it. A generated registry at the root of the bundle holds the whole chain, so any reader can see what changed, when it changed and what replaced it. A superseded claim is presented as withheld, with its reason and its replacement named, never silently dropped.

Nothing in the bundle needs looseleaf itself to stay readable. The files are plain Markdown. Any consumer of the Open Knowledge Format can open the bundle, and a person can read the whole memory with nothing but a text editor.

## What is in the bundle

- **index.md and log.md** are the reserved files the format itself defines. The index names the bundle and its format version. The log records bundle level changes.
- **daily** holds one log per day. Daily files are append only. Nothing rewrites them, not the tool and not the synthesis pass.
- **curated** holds the long term memory, one file per topic. This is the only layer the synthesis pass rewrites.
- **people and groups** hold one file per person and per group the agent works with.
- **directives** holds the standing orders the memory runs under, kept as data in the bundle rather than as prose buried in other files.
- **bank** holds distilled memory banks, one file per bank.
- **synthesis** holds one report per synthesis pass. A pass that left no report is treated as a pass that never ran.
- **claims.md** is the generated claim registry. The tool rebuilds it from the daily and curated layers. Nobody edits it by hand.

## How it is used

The looseleaf command line tool checks a bundle against the profile, rebuilds the claim registry, the layer indexes and the root index, appends dated entries to the bundle and layer logs, reports which concepts have gone stale, and promotes claims from the daily logs into the curated layer. It also stamps the frontmatter the synthesis contract binds the pass to, writes the report concept each pass run is audited against, prints the re-verification queue a pass hands on, marks a concept deprecated in favour of its replacement, and reports anything in the bundle that breaks the rules. The synthesis pass itself is a written contract that a host agent follows, and the tool checks the result against that contract. The tool never calls a model. Judgment lives in the contract, where a person can read it and an auditor can test it.

Some acts stay with the person who owns the memory. The tool will not retire a standing directive, name a relationship on a person file, delete a file or run the first pass over a new collection of files on its own. Those calls belong to the owner, and the contract says so.

## Status

The bundle checker, the claim registry writer, layer and root index regeneration, dated log appends, and the synthesis pass mechanics have landed and pass the hosted gate. The crates are not yet published.
