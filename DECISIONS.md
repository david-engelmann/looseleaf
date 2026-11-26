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

## D19. Phase 2 checkpoint dispositions

The Phase 2 checkpoint ran the three reviews over Slices 1 to 3,
a red-team re-audit, a spec fidelity review and a standards
review, plus the owner's own verification. What they found and
what was done about it is recorded here. Two findings were
material. The rest were real and are fixed or dispositioned
below.

D18 placed the superseded claims section entry in the promotion
target. The profile places it in the concept that holds the
superseded claim, and the checker enforces the profile, so a
promotion whose replacement landed in a different concept than
the superseded claim reported success and left the bundle
failing its own check. D18's placement sentence is corrected by
this entry. The entry now goes to the concept holding the
superseded claim's curated copy, and to the target only when the
superseded claim was never promoted, which is the case the
registry has always read for a daily-only claim. The profile
also assigns the superseded-by reverse link to the promotion
moment, and promotion now writes it onto the superseded claim's
curated copy in the same pass, refusing when that copy already
names a different replacement.

Promotion's drift refusal was scoped to the named target, while
the checker's duplication and drift rules span the whole curated
layer. Promotion now scans the layer. An identity carried by
another concept refuses outright, and the target's own copy is
compared as before. The comparison still covers the supersedes
link alongside text, quote and sources, a deliberate superset of
the checker's wording-only drift rule, because a carried copy
whose chain link differs is not the same claim.

The writers never write through a symbolic link. The bundle
walker does not follow one, so a layer directory that is a link
holds no concepts the tool can see, and the index and log
writers now refuse it instead of landing files outside the
bundle. Promotion targets come from the walker and were never
exposed.

The promotion reason is validated as one non-empty line before
anything is written, because it lands inside a section entry the
registry harvests, and a reason spanning lines could forge an
entry. A superseded claim whose own text carries the words
Replaced by cannot be recorded in the section's line shape,
which the scanner splits on those words, so that promotion is
refused with the cause named rather than written unreadably.

The quote renderer's escaping is now the registry's own shared
escaping, after the review found the promotion copy escaped only
backslashes and quotes and would have written a multi-line quote
raw across lines. Body edits in the index, log and pass writers
now carry existing lines as raw slices with their own endings,
matching the frontmatter writer's preservation discipline, and a
leading byte order mark rides inside the frontmatter span, so
regeneration no longer strips it. The root index renderer
preserves the body up to and including its heading as a raw
slice, and finds the heading outside fenced code blocks. A log
append on an equal date joins after the section's last entry
rather than its last content line. A promoted block landing in a
concept that carries a superseded claims section but no claim
blocks goes before the section, never inside it. Layer index
links name the path inside the layer directory, so a concept in
a nested subdirectory points at its own file.

Staleness reporting now covers only files inside the bundle's
layers, since a root-level file is not a concept. The slice
scope's phrase about concepts carrying a verified entry and
D18's report rule conflicted, and D18's rule stands, the report
names the newest verification a concept carries and says never
when it carries none. A malformed stale_after no longer vanishes
in silence, because the checker now warns on it under
FM_STALE_AFTER and the profile states the shape the value
carries.

On the standards axis, the integration test harness is extracted
into a shared module after three phases of copies, the log scope
type names exactly the three logs the profile gives, the error
constructors and the line ending rule are shared from the writer
module, and the crate documentation names the full writer
surface. Two review observations are accepted without a change.
The write window in promotion pairs a body computed from the
bundle snapshot with a frontmatter span re-read at write time,
and a host edit inside that window is the contract's concern,
since the host is the bundle's only other writer. The synthesis
contract speaks of promotion into an entity page while the
profile confines claim blocks to the daily and curated layers,
and the implementation follows the profile. The contract phrase
is flagged here for the human gate that owns contract changes,
not amended by this pass.

## D20. Phase 3 design dispositions (2026-11-24)

Phase 3 mechanizes the pass's paperwork, the two frontmatter
stamps, the Synthesis report concept and the re-verification
queue. Its scope was reviewed before building under the three
review shape, a red team pass, a spec fidelity pass against the
design record, the profile and the contract, and a standards
pass. What they found and what changed:

- The queue's membership is the contract's created or changed
  set only. The scope's first draft folded the staleness report
  into the queue, and no source puts stale concepts there. The
  staleness report stays its own mechanic.
- The report is one concept per run, named from the run's stamp
  as synthesis/<date>-<time>.md. The first draft's one file per
  date merged two runs into a single audit unit and left the
  second run's changes carrying the first run's stamp.
- Stamp and date arguments are validated mechanically before
  any write, an actor as one non-empty line and a stamp's date
  as a real calendar date, because the staleness report orders
  verifications as strings and one garbage stamp would falsify
  it permanently.
- Stamps resolve their target through the bundle walker and
  refuse the claim registry, whose frontmatter the registry
  renderer owns outright, so a stamp there would vanish at the
  next regeneration, and any directive concept, which the
  contract human-gates against change.
- The report writer creates a missing synthesis directory and
  regenerates that layer's index in the same operation, so its
  own write never leaves the bundle failing the layout check.
- The actor prohibitions, that a machine actor never stamps a
  human actor and that a pass never verifies a concept it
  changed in the same pass, stay in the contract. A stateless
  command cannot know its caller or what else the caller
  changed, so neither is mechanically enforceable here, and
  this entry records that division openly instead of letting
  the tool pretend to enforce it.
- The queue's overlap with the upkeep engine's general
  mechanism is surfaced to Main under the design record's
  boundary. This tool builds the looseleaf pass's own instance
  only, its membership fixed by the contract.
- Accepted without change, two asymmetries the reviews noted.
  Report appends are not idempotent, matching the log writer,
  and a restamp over another actor's generated stamp proceeds
  without a warning, the diff in the bundle's history being the
  record of the change of hands.

## D21. The pass report writer (2026-11-24)

Phase 3 Slice 2 adds write-report, appending one entry to one
section of the run's Synthesis report concept. The writer
creates the report at synthesis/<date>-<time>.md from the run's
stamp, which is validated as a real calendar date and a real
time before it becomes a file name, with the profile's
frontmatter and all six contract sections in order, so every
later append is a lookup. Its generated stamp is applied
through Slice 1's stamp-generated, so the one construction of
that mapping serves both. Sections are named by pinned tokens
on the LogScope pattern, a section ends at the next level two
heading, and an existing report missing the named section is
refused as malformed. When the synthesis directory is absent
the writer creates it and regenerates that layer's index in
the same operation, through a per-layer regeneration extracted
from the index writer, so the tool's own write never leaves
the bundle failing the layout check. The convention is banked
in PROFILE section 7, and the checker warns REPORT_SHAPE on a
report-named synthesis file whose body lacks one of the six
sections, so the audited artifact is itself auditable.

## D22. The re-verification queue (2026-11-24)

Phase 3 Slice 3 adds queue, printing the re-verification queue
a pass hands on. Its membership is the contract's set and
nothing else, the concepts the pass created or changed in
meaning in the same pass, supplied one by one as --changed
paths after the stamp. Each path is resolved through the
bundle walker and refused when it names no concept, a reserved
file, the claim registry, or a daily file, and duplicates fall
out by resolved path. The output is one line per concept and a
closing count, a list and never an edit, so the host pastes
the printed queue into the report's queue section and the
report carries the queue the tool printed. The general queue
mechanism for any bundle remains the upkeep engine's under
the design record's boundary, surfaced in D20. This command is
the looseleaf pass's own instance.

## D23. Phase 3 checkpoint repairs (2026-11-24)

The phase checkpoint reviews over the landed slices found no
defect that corrupts a bundle, and the repairs below. Stamp
validation now covers the whole stamp through one shared rule,
a real calendar date plus an optional real time with digit
checks and at most a trailing Z, because the staleness report
orders stamps as strings and a garbage tail falsified that
order within the day. The report writer requires the time and
derives its file name from the same rule, so two runs at
different instants can no longer share one report through an
unvalidated tail. A report left unstamped by an interrupted
creation is healed on the next append instead of being appended
over forever. The checker's report shape test strips the file
extension case-insensitively, matching the walker, reads
sections fence-aware, and warns when the six sections stand
out of the contract order the profile states. The report
writer's heading search is fence aware on the same reasoning.
The created report's section headings are rendered from the
same constant the checker reads, so the two cannot drift. The
stamp's record reading now matches the checker's rule exactly,
scalar by and at required, other entries ignored, the whole
verified list validated before the duplicate check runs, and
an empty list refused as the malformed shape the checker flags.
Both stamps also re-read their own rendered output before
writing and refuse when it no longer parses, because wrapping
a record into a list can carry a nested value past the depth
the subset parser reads, and a stamp never writes a document
its own reader cannot take back.
The queue refuses a path carrying a line break, since its
output is pasted into the report as the audited record. The
sibling temp and rename write and the bare segment helper are
shared from the writer module now, and the command line surface
gained end-to-end tests pinning the four subcommands' output
lines and usage exits. Two findings were accepted without
change. The fixed predictable temp path beside each target is
the standing house pattern from the earlier writers, on the
owner's own tree. Two runs sharing one wall clock second still
share one report file, since the name is derived from the
run's stamp and nothing else the tool may use distinguishes
them, and telling such runs apart is the caller's stamp
discipline.

## D24. Phase 4 design dispositions (2026-11-24)

Phase 4 mechanizes concept deprecation and closes the
writer render consolidation, and its design reviews cut the
phase's other proposed command. A record-supersession
command for supersessions born inside the curated layer is
not built. The profile's chain rule puts every claim in a
daily log first and promotion is the only route into the
curated layer, so the flow the command served is one no
source authorizes, and promotion's completion semantics
already finish interrupted supersession paperwork on a
later run. The deprecation command writes the status the
profile and the checker use, deprecated, in one composed
write of the act's three keys, status, superseded_by and
the generated restamp built with the stamps' shared
construction, guarded by a re-read of the composed document
before the single write. Its target ladder is the stamps'
own, directives refused with the rest, since the contract
human-gates any change to a directive. Its replacement is
any managed concept the walker resolves, with no layer
confinement, never the target itself and never a file that
did not read cleanly. A partial act already standing is
completed, a pointer naming a different replacement is
refused as a new decision rather than an execution, a
replacement that is itself deprecated is accepted
deliberately, and the concept's claims are untouched
because the profile keeps the concept and claim axes
separate. The writer consolidation is proven by golden
byte literals captured across the render dispatch's branch
matrix before the change, not by the suite's parse
round-trips, which are blind to the indentation drift that
matters.

## D25. Phase 4 checkpoint repair dispositions (2026-11-24)

The Phase 4 checkpoint reviews found no defect in the
deprecation act's shape, and the repair pass closes the
edges they named. Deprecate now refuses to close a
supersession cycle, walking the replacement's own pointer
chain and refusing when it leads back to the target, since
D24's deliberate acceptance covers chains and a cycle is
not a chain. The repoint comparison normalizes a standing
pointer the way the checker's resolver does, so a
non-canonical pointer naming the same file completes and
is canonicalized instead of drawing a repoint refusal. A
replacement path carrying an anchor is refused, because a
pointer written with one would not resolve under the
checker. A non-scalar status is refused on the same rule
as a non-scalar pointer, that refusal itself being the one
D24 left unrecorded, and a target whose frontmatter does
not parse is now named as such. A synthesis pass report is
refused as a replacement, since a report is a run's record
and not a successor concept, while every other layer the
profile does not exclude stays eligible. The two refusal
ladders the act grew are factored into one shared rule the
stamp targets and the replacement both pass through, and
the act reads the bundle once. Two corrections to D24's
record stand here. The golden pins landed in the writer
module's own tests rather than the integration test file
the scope named, the substance being that they landed
against the unconsolidated code and held through the
consolidation. And the pin matrix covers every shape the
subset can construct, the remaining enumerated cells, a
mapping nested inside a first entry's mapping value and
the collection shapes at deeper indents, being shapes the
parser's depth cap and the profile's nesting bar keep out
of any bundle, with equivalence for them resting on the
consolidation's uniform delegation to the one entry path.

## D26. Phase 5 opening dispositions (2026-11-25)

Phase 5 opens on Main's charge with the README status
correction as its early slice and the design reviews'
dispositions recorded here before any building, as D20 and
D24 did for their phases. The cycle rule's reporting
diverges from the claim precedent deliberately. The claim
rule fires once per cycle at the registry, which is the
claim graph's natural home, and keeps self-supersession
under its own code. The concept rule fires once per
participating concept at that concept's file and treats a
self-pointer as a cycle of one, since the concept checks are
per-file in idiom and a concept cycle has no registry to be
located at. Only the cycle's own members fire, never a
concept whose chain leads into the cycle from outside. The
staleness escaping lives at the print site and covers the
verified stamp beside the path, since both fields of the
printed line can carry a line break out of a quoted
frontmatter scalar. Its one accepted ambiguity is that a
path literally holding a backslash and an n prints the same
as an escaped line break, accepted because the line's
consumer is the person reading the audited record and no
parser reads these lines. The fixture completion stands as
its own slice landing last, since it is surgery on a fixture
the whole suite stands on and the earlier slices assert
finding sets against the fixture as it stands.

## D27. Phase 5 checkpoint dispositions (2026-11-25)

The checkpoint reviews over the Phase 5 diff found the phase
faithful to its scope, and the repairs and corrections stand
here. The cycle finding's message now names the member's own
pointer beside the cycle's membership, as the scope's wording
asked. The staleness and queue closing lines escape the
caller's stamp they echo, since the stamp is validated on its
date alone and a line break in its tail would otherwise forge
a second summary line in the output the phase exists to keep
line-safe. One correction to D26 stands in this entry, since
D26 is landed history. Its accepted-ambiguity note concedes
that a path holding a literal backslash and n prints the same
as an escaped line break, and that is not so. The printer
doubles backslashes first, so the literal prints doubled and
the escaped break prints single, and a reader can reconstruct
every printed path exactly. The concession is withdrawn, and
no reader should reshape the printer to match it. Accepted
without change, with reasons. The cycle walk's cost grows
with the cube of a long chain's length in the worst case, the
same shape as the claim walk it follows, and bundles at the
product's scale sit far below the size where that binds. The
walk skeleton is shared in shape with the claim walk but not
extracted, since the two differ in edge type and reporting
and a third walk is the point where extraction pays. The
check printer and the caller-argument success lines print
their values raw, a boundary kept deliberately, since the
line-safety invariant binds the pass outputs that are pasted
into the audited record, which the closing-line repair now
covers in full. Non-ASCII line separators in file names pass
the escaping untouched, the ASCII pair being the record's
line discipline. The registry stands in the walk's node set,
so a cycle routed through a hand-edited claims file fires on
that file, which is correct detection on a bundle no tool
writes.

## D28. Phase 6 opening dispositions (2026-11-25)

Phase 6 opens on Main's charge with its design reviews'
dispositions recorded here before any building, as D20,
D24 and D26 did for their phases, and it reverses two
banked acceptances by name. D23 accepted the fixed
predictable temp path beside each target as the standing
house pattern from the earlier writers. That acceptance is
reversed. Six implementations of the sibling temp and
rename write stand in the core, the shared helper
write_via_temp serving the stamps and the report writer
and five hand-rolled copies in regenerate_layer,
regenerate_root_index, append_log, write_body and
regenerate_registry, and every one writes its temp with
fs::write, which follows a symlink planted at the temp
path. The five copies route through the one helper, and
the helper removes any file standing at the temp path
before creating its temp with create_new, so a planted
link is unlinked rather than followed and a path planted
in between refuses the write. The removal ignores exactly
the absent case, refuses on any other failure with an
error naming the temp path, and is remove_file only, so
no directory is ever removed and a directory standing at
the temp path refuses the write and stays intact. Create
failures are wrapped in an error naming the temp path as
well. The hardening's one honest delta is recorded, in a
read-only directory the failure now arrives at the removal
step rather than at the rename, with the outcome
unchanged. D27 accepted the check printer and the
caller-argument success lines printing their values raw
under one boundary. The check half of that acceptance is
reversed on the dogfood evidence, since the dogfood
records bank check output as their evidence and the
boundary D27 drew around the pass outputs does not match
where the audited record comes from. The finding's path
and message are escaped at the check print site with the
Phase 5 helper and the core values stay raw. The fields
left unescaped are closed sets, the severity label, the
static rule code and the summary's counts, and a future
change making any of them dynamic reopens this question.
The success half of D27's sentence stands, on this phase's
own reasoning that those lines echo the caller's arguments
back to the caller, and error lines on stderr stay raw as
operator diagnostics. The remaining acceptances stand as
recorded, the one-second report collision, report appends
not idempotent, a restamp proceeding without warning, the
cycle walk's cost and its unextracted skeleton, the
registry in the walk's node set, the write_body window,
and non-ASCII line separators outside the escaping
alphabet.

## D29. Phase 6 checkpoint dispositions (2026-11-25)

The checkpoint reviews over the Phase 6 diff found the
phase faithful to its scope, no defect that lets a write
follow a plant or a printed check line carry a break, and
the repairs below. The temporary write's rename step now
refuses like its other steps, wrapped in an error naming
the step and the temporary path, completing the naming
discipline the scope pinned for the removal and creation
steps and that the write step already followed. Every
refusal from the helper now keeps the underlying failure's
kind instead of re-kinding it to invalid input, since a
permission refusal is not invalid input and a caller may
match on the kind, a behaviour delta D28 did not record
and this entry does. One sentence of D28 is corrected
here, since D28 is landed history. Its read-only
directory sentence says the failure arrives at the removal
step, and that holds only when a stale temporary file
stands there to fail its unlink. With nothing standing,
the removal reports the path absent and proceeds, and the
failure arrives at the creation step. The outcome is
unchanged either way. The test net gains the two helper
consumers it lacked, a plant at the root index writer's
temp path and one at the report writer's, so every
production caller of the helper now stands behind a plant.
The family tests' shared scaffolding, the sentinel plant
and the three closing assertions, is extracted into the
shared test harness the suite already keeps, with the
sentinel file removed by a guard on both the pass and the
panic path. Accepted without change, with reasons. The
planted concept document stands verbatim in both the core
and the CLI test, inherent to a pin that spans two crates
whose harnesses cannot be shared, as the Phase 5
checkpoint recorded for the same pattern. The helper's
call sites keep two idioms, import and full path, each
file consistent with its own standing style for the same
module.

## D30. Phase 7 design dispositions (2026-11-25)

Two standings from the earlier phases are reversed here,
by name. The non-ASCII line separators standing outside
the escaping alphabet, accepted in D27 and carried in
D28's standing list, is reversed. The ground D27 recorded
was a definition, that the ASCII pair is the record's
line discipline, and the readers of the banked records
decide the question the other way, since Python's
splitlines and several editors and log readers treat the
line tabulation, the form feed, the next line control,
the line separator and the paragraph separator as line
boundaries too. The printer's helper now renders all five
as backslash u escapes beside the ASCII pair, and the
Phase 6 ground that widening would alter landed lines is
answered on the banked output, since no landed staleness,
queue or check line carries one of the five, a claim that
rests on the banked transcripts and the prototype bundle's
file names rather than on a mechanical census. Three
further characters splitlines alone treats as boundaries,
the information separators U+001C through U+001E, stand
outside the alphabet by name, since no editor or log
reader the phase can name breaks a line on them. As part
of the same reversal's completion, the re-verification
queue's refusal of a path carrying a line break widens to
the same seven characters the printer escapes, one set
stated in two places, since the queue prints one line per
path and its refusal is the only protection those lines
have. The promotion reason refusal keeps the ASCII pair
deliberately, since a written bundle file is read by the
tool's own parser, whose line discipline is the line feed
alone, and the written record's boundary is the parser's,
not the transcript's.

The second reversal is D29's acceptance of the planted
concept document standing verbatim in two crates' tests
as inherent. The harnesses cannot be shared and the
documents can, since the CLI tests already reach into the
core crate's fixture tree for their bundle copies, and
the phase's second slice moves all three verbatim plant
documents, the check plant, the staleness plant and the
separator plant this entry's phase adds, into shared
fixture files outside the clean fixture, the plant names
standing inline in both crates as the recorded residue.
The same slice takes up D29's second acceptance, the
helper's two call-site idioms, and unifies them on the
import form at the four full-path call sites in
indexes.rs, registry.rs and stamps.rs. The remaining
standings are re-weighed and stand. The one-second report
collision stands on D23's recorded reason, with this
phase's observation added that the report protocol is
several appends under one stamp by design, so a file
already standing at the run's name is the normal case the
writer must join rather than a collision it could detect.
D20's companion acceptances stand on their recorded
reasons, report appends not idempotent matching the log
writer, and a restamp over another actor's generated
stamp proceeding without a warning, the diff in the
bundle's history being the record of the change of hands.
The cycle walk's cubic cost stands on termination and
correctness rather than on bundle scale alone, since the
checker runs over trees whose provenance is not yet
trusted and a crafted long chain makes the walk slow on
input the tool is expected to face, and it terminates by
its position check and is never wrong. The unextracted
walk skeleton stands on its third-walk trigger, which the
chain follow in stamps.rs does not trip, since it walks
toward a named target and extracts no ring. The registry
in the walk's node set stands as correct detection, and
the write_body window stands under the contract's
single-writer term. The success lines and the stderr lines
stand raw on the reasoning D28 recorded.

## D31. Phase 7 checkpoint dispositions (2026-11-25)

The checkpoint reviews over the Phase 7 diff found the
phase faithful to its scope, the spec review with no
findings at any severity and the red team with none, every
printed surface either sealed by the widened alphabet or
standing inside a disposition the phase re-weighed. The
repairs below come from the standards review. The shared
plant helper the second slice added stands once in the
core crate's shared test harness, where this crate keeps
its test scaffolding, instead of once per core test file
that uses it, and the two core copies are removed. The CLI
crate's copy stands, since the crates share no harness and
its base path differs. The queue refusal test now pins
every member of the widened set, the planted line
separator case joined by the remaining four characters
against constructed paths, matching the printer side's
complete pin. One phrase of D30 is noted here, since D30
is landed history. Its second paragraph says this entry's
phase where the register's standing form is this phase.
The sentence's content is accurate and stands.

## D32. Phase 8 design dispositions (2026-11-25)

This phase opens by correcting its own scope draft on
the programme's records. The draft stated that the
registry writer, the re-verification queue and the stamp
commands had only ever run against the clean fixture,
never against a real converted tree. The Phase 6 dogfood
record shows write-registry run over a fresh copy of the
prototype bundle, the registry created on claims.md and
its standing absence resolved in the final check. The
Phase 3 dogfood record shows stamp-generated and
stamp-verified run over the prototype copy and the queue
run over one real concept in it, and the Phase 6 dogfood
ran stamp-generated over the prototype again. The stamps
act on concepts rather than claims in any case, and the
queue never reads a registry, its membership being the
concepts a pass changed, supplied one by one under D22.
The draft's slice, a fresh conversion of the current
tree by the prototype migrator, is withdrawn on four
grounds the design reviews established. The migrator
emits no claim registry and no directives layer. A
directives layer cannot be produced mechanically at all,
directive creation standing at the human gate under the
contract. The migrator's daily sample is three fixed
days, so a rerun's claim bearing layer is the layer every
dogfood already carries. And the migrator's output path
is the prototype directory itself, so a rerun would
overwrite the Phase 1 snapshot and rewrite the loss
ledger, the two records the draft's own never list
promised to protect.

What the phase takes is the narrower ground the reviews
found standing. Phase 6 created the registry as a plant
target and banked no census of its contents, so no
dogfood has examined a generated registry against the
corpus it was generated from, and the queue has never
run over more than the one hand fed concept of Phase 3.
The phase closes both gaps over the existing prototype
under the Phase 1 fixture sharing as approved, on a
fresh copy the dogfood makes, the banked prototype never
written to. The registry closure rests on a mechanical
comparison, a census taken independently from the copy's
daily and curated layers set against the generated
registry entry by entry, with the design record's Phase
1 census of the same tree standing as the independent
expectation. The queue closure rests on a derivation,
the expected membership worked out from the verified
registry and the concepts' stamps before the command
runs and matched against the printed queue line for
line. Findings the run surfaces are classed before any
is acted on, a checker verdict over the converted tree
being a corpus finding that is banked and counted, and a
command's output contradicting the corpus evidence, the
derived expectation or the tool's own contract being a
tool finding, which alone earns a repair or a standing
recorded here. Tool findings past three stop the phase
for a report to Main rather than growing it.

The standings D30 carried are re-weighed and stand, the
one-second report collision on D23's reason with the
multi-append observation, D20's companions on their
recorded reasons, the cycle walk's cost on termination
and correctness with the untrusted provenance caveat, the
unextracted walk skeleton on its third-walk trigger, the
registry in the walk's node set as correct detection,
the write_body window under the single-writer term, and
the success and stderr lines raw on D28's reasoning.
The host writer type key collision logged as L-05 in the
design record stands at risk by its Phase 1 disposition,
the checker re-deriving the expected type from the
directory, and this entry is its first re-weighing since
Phase 1, the Phase 6 and Phase 7 scopes nowhere
mentioning it, its standing resting in part on the
dogfood records' silence, which is named here as the
absence claim it is. The set is recorded as settled, to
be reopened only by a finding with evidence behind it.

## D33. The withheld obligation scoped to carried claims (2026-11-25)

The Phase 8 dogfood surfaced a tool finding, the first
under the triage rule D32 recorded. Over the prototype
copy, write-registry produced a registry the checker
then faulted with two REGISTRY_WITHHELD errors that no
action can clear, each demanding a withheld line for a
supersedes pointer whose target no layer carries. The
corpus carries two such pointers, claims in the daily
layer whose supersedes names the superseded claim in
prose rather than by identity, the source convention's
drift, and the supersession graph already flags each as
CLAIM_SUPERSEDES_DANGLING at its own file. The registry
validator built its withheld expectation from every
supersedes pointer in every layer, while the renderer
presents only claims the bundle carries and the graph
checker scopes its edges to the daily and curated layers
and to carried targets, so a registry the tool itself
wrote could never satisfy the tool's own check on such a
corpus. The repair scopes the validator's expectation to
the renderer's and the graph checker's discipline, the
daily and curated layers and targets a layer carries, so
a dangling pointer is the graph's one finding and creates
no withheld obligation. No finding code, severity or
message changes for any satisfiable case, and the two
prose pointers stand as corpus findings, banked in the
Phase 8 dogfood record with their evidence.

## D34. Phase 8 checkpoint dispositions (2026-11-25)

The checkpoint reviews over the Phase 8 diff found the
phase faithful to its scope, the spec review with no
material or moderate findings and the standards review
with none above a Low. The repairs below come from the
red team's one P2 and the findings it stood beside. The
second repair test D33 carried, planting a claim block
in a bank concept, pinned nothing, since the bundle
loader scans claims only in the daily and curated layers
and the planted block was invisible to the validator, the
renderer and the graph alike before the repair and after
it. The test is removed, and D33's sentence crediting two
tests is corrected here, one test pinning the repair, the
dangling pointer case, and the layer guard in the
validator standing as a statement of the discipline under
the loader's older guarantee rather than as a behaviour
the suite can catch. The same review found the
validator's scan of superseded sections wider than the
renderer's by the reserved curated files, so a section
entry standing only in a layer log would have made the
validator demand a reason the writer never prints, an
obligation of the family D33 repaired. The scan now
skips reserved files as the renderer's does, and a test
pins the case on a writer generated registry, its
withheld line asserted present in its printed form.
Duplicate superseded section entries for one claim
carrying different reasons stand as a recorded residue,
the renderer's map keeping the last entry's reason while
the validator searches by identity and replacement
together, so the two can disagree only on a corpus
carrying such duplicates, and no case stands in any
record. The standards review's Low is corrected here as
the register's convention provides, D32's sentence
resting the queue closure on the verified registry and
the concepts' stamps, where the derivation the dogfood
ran followed the bundle walker's eligibility rules and
the queue's rule consults neither, as the dogfood record
already states. The spec review's three minors stand as
recorded corrections in the dogfood record, the census
premise, the queue movement's supplied set, and the
derivation sentence this entry also corrects.

## D35. Phase 9 design dispositions (2026-11-26)

Phase 9 closes the one member of the validator and
renderer disagreement family D34 left standing and runs
the pass for the first time as the cycle the contract
describes. The residue's mechanics are these. The
renderer's scan of the curated layer's superseded
sections builds its map by the superseded identity alone,
skipping reserved files, and inserts over earlier
entries, so the last entry in bundle order supplies the
date and reason printed beside whichever replacement the
claims' own map names, the first claimant in bundle
order. The validator searches the same entries by
identity and replacement together and demands the first
matching entry's reason in the withheld line. On a corpus
carrying duplicate entries for one claim the two
disagree, the writer printing one supersession's reason
where the checker demands another's, and a registry the
writer generated fails the tool's own check. The
alignment moves the renderer to the validator's keying,
the first entry in bundle order whose identity and
replacement both match the line being printed supplying
its date and reason, with the standing fallback to the
replacement's daily date and no reason when no entry
matches. The move retires the renderer's documented last
entry rule, its comment recording that the last in path
order wins, and adopts the first match both replacements
maps already use, so on a corpus with duplicate matching
entries the printed date moves with the printed reason.
The boundary of the closure is stated as plainly. The
validator reaches a withheld line's replacement by its
prefix and its reason by substring and never checks its
date, so the family closed here is the reason and
replacement disagreement, the date standing outside the
validator's reach by design. The closure is recorded
against the standing D34 left, and two tests pin it with
their plant order fixed in the scope, duplicate entries
naming one replacement and entries naming different
replacements with the other replacement's entry standing
last. The evidence half of the phase is the pass cycle
itself, a pass changing a concept in meaning, the queue
carrying exactly that changed set under D22, the report
recording the pass, a second registry regeneration over
the registry the first one wrote examined line by line,
and the re-verification act standing separate after the
pass closes, on the division D20 records. Its closure
rests on a derivation made before the run, the queue's
membership, the stampability of every concept the run
will stamp, the second regeneration's shape and the final
check's delta share by share, each read from the
baseline's own lines. What the cycle does not close is
recorded with it, a changed set larger than one concept
awaiting a pass that authors changes at scale. The two
prose supersedes pointers banked in Phase 8 stand as
corpus findings still, their treatment complete under
D33 and their repair no act of this product. The
acceptance set stands as D32 as corrected by D34 left it,
settled, to be reopened only by a finding with evidence
behind it.
