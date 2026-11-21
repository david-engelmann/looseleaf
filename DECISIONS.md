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
