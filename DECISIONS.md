# DECISIONS

The locked decisions for looseleaf. This record is banked from the Phase 1
design record, which Main confirmed on 2026-11-20 with all four scope
amendments approved. Changes to this record are decisions in their own
right and are gated the same way the synthesis contract gates its own
changes. Nothing in this record is revised silently.

Rename recorded 2026-11-20. This repository was created as faircopy and
renamed to looseleaf the same day, on the owner's ruling that the earlier
name carried too much legal register. The Phase 1 design record and the
loss ledger stand as the historical gate record under the name they were
written with and are not rewritten.

## D1. The product

Looseleaf is a memory system for AI agents, stored as plain files the user
owns. The store is a bundle of Markdown files with YAML frontmatter that
follows the Open Knowledge Format (OKF) at version 0.3. Looseleaf is a
profile of that standard plus a synthesis discipline. It is not a memory
server, it holds no database, and reading and search are left to the
existing consumers of the format.

The name is the product in one word. A looseleaf binder holds pages that
can be rearranged, replaced and removed. That is the shape of the memory
itself. Daily pages are added as the days accumulate, curated pages are
replaced as the facts change, and stale pages are retired by supersession
rather than vanishing.

## D2. The two legs

The field of file-native agent memory is crowded and the generic seat is
taken several times over. Looseleaf stands on two legs and no others.

The first leg is OKF v0.3 nativity. None of the incumbent file memory
products is built on the standard. Looseleaf's bundle layout is a profile
of OKF v0.3, its trust fields carry real values under the standard's own
rules, and its bundles validate with the official validator.

The second leg is the synthesis discipline as a published and testable
method. What a synthesis pass may promote, what it may supersede, what
provenance every surviving claim carries and what stays human-gated are
written down as a contract that any host agent can execute and any
auditor can check. The discipline is proven on a real memory tree, the
Muse memory system of the repository owner, whose shape the Phase 1
conversion test ran against in full.

## D3. Ceded ground

Ground held by others is ceded, not contested.

Markdown memory with compaction and supersession is conceded to
Palinode, which shipped it first, with git versioning, rollback and
hybrid search in one tool. Supersession enforced at search time inside a
memory server is conceded to the rwnalds Engram build, which withholds
superseded notes and explains each withholding. Reading and searching an
OKF bundle over MCP is conceded to okf-agent-memory. The general OKF
upkeep engine is the engine seat's ground and the OKF trust auditor is
the auditor seat's ground. Looseleaf consumes their primitives and
checks rather than reimplementing them.

The claim that survives the concessions is narrow. Nobody else enforces
supersession in the standard's own vocabulary, in the files themselves,
with claim-level chains and no server in the loop.

## D4. The claim block convention and the registry

OKF v0.3 supersession is per concept with a single target. The memory
system looseleaf is built for supersedes at the level of the individual
claim, and the standard has no home for that. This is loss ledger entry
L-11 from the design phase, the headline finding of the Phase 1 gate.

The claim block convention carries it. A claim is a list item in a daily
or curated file with a kind and confidence tag, a stable identity that is
never reused, a verbatim quote, and its sources. A claim is never edited
in place once a later daily file exists. A correction is a new claim that
supersedes the old one, with both identities, the date and a one-line
reason recorded.

The claim registry is a generated file at the bundle root that joins
every claim chain in the bundle. The registry is a looseleaf convention
layered on OKF v0.3. It is stated as such wherever it is described and is
never presented as part of the standard itself. A superseded claim
appears in the registry as withheld, with its reason and its replacement
named, never silently dropped.

## D5. Standing directives are first-class concepts

The standing orders a memory runs under are stored as concepts of their
own type in a directives layer. A memory system that cannot represent its
own standing orders as data is missing its most load-bearing content.
This was Amendment A of the design record, approved at the gate on the
evidence of loss ledger entry L-14.

## D6. The synthesis judgment half is a skill contract

The deterministic core never calls a model. The judgment half of the
synthesis pass ships as a written skill contract in this repository,
which a host agent executes and an auditor can check. This was Amendment
C of the design record, approved at the gate.

## D7. Migration scope

The general migrator is not looseleaf's. The prototype migrator built in
the design phase stands as a test instrument for the Muse memory shape
only. A converter for the one source shape looseleaf is the product for is
a candidate to hand to the engine seat. That handoff is
provisional until that seat's own gate, and no second migration home is
built in the meantime. This was Amendment D of the design record,
approved at the gate.

## D8. Stack

A Rust workspace on the 2024 edition. The core crate is looseleaf-core and
the command line crate is looseleaf. The core is standard library only.
External crates are confined to adapters, the command line crate,
benchmarks and dev dependencies, and any exception is requested by name
with its reason. The toolchain is pinned in rust-toolchain.toml and
Cargo.lock is committed. Licensing is MIT OR Apache-2.0. The bundle
writer is byte-stable and preserves frontmatter keys it does not own.

## D9. Human gates

The following acts are never taken by the tool or by a synthesis pass on
its own authority. Creating, changing or retiring a standing directive.
Naming a relationship on a person file, which is recorded only in the
owner's own words. Superseding a claim that carries a human ruling, on
agent inference alone. Deprecating a whole concept. Deleting any file.
The first synthesis pass over any new corpus. Any change to the synthesis
contract itself.

## D10. The stated risk

Looseleaf's edge depends on the Open Knowledge Format becoming a real
interchange layer that other tools read and write. The standard is young
and that outcome is not guaranteed. If the format stays a niche
specification, looseleaf's remaining ground is the synthesis contract and
the claim convention as a method one person's agents follow, which is a
smaller product than the one designed here. This risk is accepted with
eyes open and is restated rather than buried.

A second stated risk rides with the layout. A host agent that writes
Markdown from its own templates can collide with the frontmatter type
key the profile depends on. The layout checker re-stamps the type from
the directory a file sits in, and any live memory tree stays the source
of truth until a separate cutover decision is made.

## D11. Repository transport

This repository carries its own per-repo SSH deploy key, generated on the
fleet pattern and registered to this repository alone. The repository is
not added to the existing GitHub App or token installation. Git transport
runs over SSH with that key and spends none of the REST pool.

## D12. Checkpoints

Major checkpoints take three reviews before landing. An independent
red-team re-audit of the finished state, an independent line-by-line
review of the diff on the two axes of spec fidelity and standards, and
the owner's own verification pass with the key checks re-run firsthand.
A single green run is not a review.

## D13. Writer conventions

Added 2026-11-21 with the writer slice. The frontmatter serializer
renders the canonical block form defined in PROFILE.md section 5 and
quotes a scalar only when the plain form would not parse back to the
same text, so a parsed value always renders to the same bytes. Edits
to an existing document go through a line-preserving upsert that
replaces only the target key's lines, which is the mechanism behind
the unknown-key preservation D8 requires. The claim registry format
is owned by the tool outright. Regeneration takes its generated stamp
as an input rather than reading a clock, so the same bundle state
with the same stamp renders byte-identically, and the write lands
through a sibling temporary file renamed over the target, so a failed
write never truncates the registry that was there.

## D14. Checkpoint fix pass dispositions

Added 2026-11-23 with the Phase 2 opening fix pass, which closed the
findings of the three checkpoint reviews over the checker and writer
slices. Most findings were plain repairs. The ones that are decisions
rather than repairs are recorded here.

- The layout rules the profile states as rules are errors, including
  a layer that carries no index, a daily or curated layer that
  carries no log, and a daily file whose name is not a calendar date.
  The warnings that remain are the advisory set. A directory that is
  not a profile layer, a Markdown file at the bundle root, a person
  body missing part of its six section shape, a log whose sections
  run out of newest first order, an extension key without the prefix,
  a concept whose status and replacement disagree, a supersession
  fork, and a symbolic link the walker did not follow.
- The claim kind and confidence vocabularies in PROFILE.md section 3
  are closed. A tag outside them is an error, not a new kind by use.
- The nesting rule in PROFILE.md section 5 is structural. Block
  collections nest one level under a key, a sequence item may carry
  one mapping, and the values inside those collections stay inline.
  Flow collections nest at most three deep, a cap that exists so a
  crafted value cannot recurse the parser off its stack.
- D13 stands as written. The one place the writer had drifted past
  it, quoting any scalar that contains a double quote, was brought
  back into line, because an interior double quote parses back
  unchanged in the plain form.
- The walker follows no symbolic links and skips only `.git`
  directories. Everything else in the tree is read, a file it cannot
  read is reported as unreadable rather than as empty, and a skipped
  link is reported rather than silent.
- The registry check compares identity sets in both directions and
  reads the withheld presentation in the rendered format, because a
  registry that names a claim no layer carries is as stale as one
  that drops a claim the layers carry.
- A supersession fork is a warning and a supersession cycle is an
  error. The registry renderer names the first claimant in bundle
  order while a fork is resolved, so rendering stays deterministic.

## D15. Index regeneration

Added 2026-11-23 with Phase 2 Slice 1. Layer indexes are generated
files in the shape the clean fixture already carried by hand, a
heading and one linked bullet per concept, and the renderer
reproduces the fixture's indexes byte for byte. Only layer
directories present in the bundle are written, because creating a
layer directory is a layout act rather than a writer act. A concept
with no usable title lists under its file stem, and the checker
flags the missing title on its own rule. The root index stays human
owned in this slice. Its frontmatter is the owner's, and its body
regeneration is the line-preserving edit of Slice 2.

## D16. Divergences from the official OKF skills

Added 2026-11-23 under the applicability audit of the same date. The
synthesis contract and the profile were read against the two official
Agent Skills that shipped with OKF v0.3, okf-generate and okf-consume,
including the consume skill's trust reference. Where a host agent
holding the official skills would read or write a looseleaf bundle
differently than this product expects, the divergence and its
disposition are recorded here. The claim level machinery is this
product's own by design, loss L-11 in the Phase 1 record, and is out
of scope for this entry except where the official skills touch it.

- Index entry shape. Okf-generate writes each index entry as a linked
  title followed by the concept's description, grouped under
  headings. Looseleaf layer indexes are a heading and one bare linked
  bullet per concept, byte stable across regenerations.
  Disposition, ours stands. Index entry format is not one of the
  standard's conformance rules, the description lives in the
  concept's frontmatter where consumers read it, and byte stability
  is what makes a bad regeneration visible in a diff.
- The claim registry reads as a concept. Claims.md carries the type
  Claim Registry, so an okf-consume host treats it as a concept and
  may cite it as knowledge. Disposition, accepted. Its description
  states that it is a generated index, its content is a truthful
  index of the bundle's own claim chains, and carrying a type is
  what keeps the file visible to standard tooling and to this
  product's own layout rules.
- Claim level provenance. Okf-generate cites a specific claim with a
  body footnote whose label is a sources entry id. Looseleaf carries
  claim provenance in claim blocks and the registry, so a consume
  host sees concept level sources only and no claim level citations.
  Disposition, by design under L-11. The standard has no home for
  claim level supersession. The pass keeps each concept's body
  current, so a consumer is never served superseded claim text as
  current body content.
- Sources containment. Okf-consume treats a path that resolves
  outside the bundle as an external reference to weigh, and a
  resource containing spaces as a scope descriptor rather than a
  path. The profile requires a sources resource to resolve inside
  the bundle or be an absolute URL, and the checker errors
  otherwise. Disposition, a deliberate tightening, and it stands.
  This product's provenance must be re-readable by the pass that
  verifies quotes against their sources, and unverifiable provenance
  is the defect the checker exists to catch. The dogfood run against
  the Phase 1 prototype bundle showed the rule working, 45 sources
  named the living source tree the bundle was converted from and
  none resolved in bundle.
- Generated on format only edits. Okf-generate keeps the existing
  generated value when a change only fixes format, whitespace or
  links, and restamps it on a change in meaning. The profile stamps
  the pass in generated without drawing that line. Disposition,
  adopted into the synthesis contract. The pass restamps generated
  only when it creates a concept or changes one in meaning, so a
  cosmetic edit never lapses a verification that was current against
  the earlier stamp.
- The verified self stamp. Okf-generate never records a verification
  on content its own run generated. The contract had the pass
  stamping its actor for checks it truly ran, without excluding
  content the same pass created or changed. Disposition, adopted
  into the synthesis contract. The pass never stamps a verification
  on a concept it created or changed in meaning in the same pass.
  Those concepts enter the re-verification queue instead.
- Checked and aligned, no divergence recorded. The root index
  carries frontmatter with a title, a scope description and a
  license, and consumers choose bundles by that description. Layer
  index and log files carry no frontmatter. Log entries sit under
  dated headings, newest first. Status and superseded_by carry the
  standard's meanings, including following superseded_by from a
  deprecated concept to its replacement. Verified written as one
  mapping or as a list of mappings is read natively by the consume
  trust rules, which treat a single mapping as a list of one.
  Unknown keys are preserved by every writer on both sides. Path
  fields written with a leading slash resolve from the bundle root,
  and the checker's resolver accepts that form.

## D17. Root index regeneration and log appends

Added 2026-11-23 with Phase 2 Slice 2. The root index body is a
generated layer list. Regenerating it preserves the frontmatter
span and the heading line byte for byte and rewrites only what
sits below the heading, one bullet per layer directory present in
the bundle, in profile order, rendered as a link to the layer
directory followed by the layer's concept count. A hand written
description in the old bullet shape does not survive regeneration,
because the list is generated in full and a half generated list is
how indexes drift. A root index with no frontmatter or no heading
is refused, since both are the checker's findings to report and
the owner's to supply, never the writer's to invent.

Log appends are the one dated mechanical edit. An append whose
date equals the log's newest section joins that section after its
last entry. A newer date opens a new section at the top. An older
date is refused, because placing it would edit history into order
rather than append to it. Appends exist only for the three logs
the profile names, the bundle log and the curated and daily layer
logs. A missing one of those is created with the heading the clean
fixture carries, so a checker finding about the missing file does
not block the append that repairs it. An entry is one non-empty
line, rendered as one bullet, and every date is a real calendar
date given as an argument, never a clock reading.

## D18. Synthesis pass mechanics

Added 2026-11-24 with Phase 2 Slice 3. The pass module holds the
three deterministic steps inside one synthesis pass. Staleness is
a report, never an edit. A concept is stale when its stale_after
date is earlier than the date of a stamp the caller supplies, the
report names the newest verification the concept carries, and the
tool reads no clock to produce it.

Promotion copies one claim from the daily layer into a named
curated concept with its identity, quote and sources unchanged,
landing after the target's last claim block so a promoted claim
never falls inside the superseded claims section. When the
curated layer already carries the identity, promotion compares
text, quote, sources and the supersedes link, and any difference
is drift, refused with nothing written, because overwriting drift
is how a memory product silently rewrites its own past. An
identical claim is a no-op, except that a supersession entry the
section is missing is still written, so an interrupted promotion
completes on a later pass instead of reporting itself done.

When a promotion carries a supersedes link it needs its one-line
reason, and the target's superseded claims section gains the
entry in the exact line shape the registry scanner reads, dated
by the daily file the promoted claim lives in. The entry's text
is the superseded claim's own text, read from wherever in the
bundle it lives. The daily file is never touched, because the
daily layer is append-only, and a daily claim carrying a reverse
link is refused outright, since the daily layer never holds one.

A promoted block renders its quote sub-item always as a
double-quoted scalar, the shape every quote in the daily files
and the curated layer already carries, because a quote is an
excerpt whose exact wording is the evidence and the quoted form
keeps a comma or a colon inside it unambiguous to any reader.
The first hosted run of this slice caught the renderer using the
frontmatter writer's plain form for a quote containing a comma,
which parses back correctly under this product's own scanner and
still broke the established shape.
