---
name: looseleaf-synthesis
description: Run a synthesis pass over a looseleaf bundle. Consolidates daily claim logs into curated memory under the looseleaf contract, promoting claims with provenance intact, superseding what no longer holds, regenerating indexes and the claim registry, and writing the pass report the run is audited against.
---

# Looseleaf synthesis

A synthesis pass turns daily accumulation into curated memory. This
skill is the contract for that pass. It binds any agent that runs one,
the way the profile in `docs/PROFILE.md` binds the files. A pass that
cannot show its work against this contract did not happen, and its
edits are reverted until it can.

## Inputs and outputs

Inputs are the daily logs since the last pass, the current curated
layer, the people, group and topic pages, the directive concepts and
the last pass report.

Outputs are edits to the curated layer and the entity pages, the
regenerated indexes and claim registry, entries appended to the bundle
log, one Synthesis report concept for the run, and a re-verification
queue for the checker. The pass never outputs edits to daily files.

Before editing anything, run `looseleaf check <bundle>` and record the
findings. A pass starts from the checker's picture of the bundle, not
from its own impression of it.

## What the pass may do

- Promote a claim from a daily log into the curated layer or an entity
  page, carrying its claim identity, its quote and its sources
  unchanged. Promotion is a copy with provenance, never a paraphrase
  that keeps the identity. If the wording must change, the promoted
  text is a new claim that supersedes, and the chain says so.
- Supersede a curated claim when a newer claim replaces it, recording
  the old text, the old identity, the replacing identity, the date and
  a one-line reason in the concept's superseded claims section, and
  updating the claim registry in the same pass.
- Update entity pages. New facts enter Facts with their claim anchors.
  Events enter History. Open threads open and close as the evidence
  lands. The relationship section changes only under the human gate
  below.
- Regenerate the people and group indexes in rank order, the bundle
  index and the claim registry. Regeneration is deterministic. The
  same bundle state produces byte-identical indexes, which is what
  makes a bad regeneration visible in a diff.
- Assign `stale_after` under a fixed rule set and no other way. A claim
  stating a count, a status or a holder of a role receives a review
  date one pass cycle after the date it was last confirmed. A claim
  stating a standing decision or a preference receives none unless its
  own text names a date. A date assigned outside these rules is a
  contract breach, because the standard forbids inventing review dates.
- Flag shape and health findings without fixing them silently. A bank
  file breaching its shape rule, a concept whose size makes it
  unreadable as one concept, a person page whose type key went
  missing, a chain that does not terminate. Flags land in the pass
  report and the affected concept is named.
- Propose new topic concepts when a curated concept outgrows its
  grain, and propose directive lifts when standing orders are found
  buried in prose. Proposals land in the pass report for the human
  gate, not in the bundle.

## What the pass may never do

- Edit, reorder or delete anything in a daily file. Daily files are
  the evidence layer. A pass that rewrites its evidence can prove
  anything and therefore proves nothing.
- Delete a claim. Claims are superseded or left alone. The chain is
  the memory of being wrong, and deleting it is the one unforgivable
  edit in a memory system.
- Stamp a human verification that did not happen. The pass is a
  machine actor. It stamps its own actor, `process:looseleaf-synthesis`,
  for checks it truly ran, and never a human actor.
- Promote a claim whose quote does not verify against its source.
  Re-read the source message or file passage first. A quote that fails
  to match blocks that promotion and lands in the report as a
  provenance defect in the source log, a finding about the extraction
  layer, not something to quietly repair.
- Rewrite a claim's identity. Identities are permanent. Rewording is
  supersession, with a new identity for the new text and the chain
  linking them.
- Act on a directive it disagrees with by editing around it.
  Directives bind the pass the way they bind any reader. Changing one
  is human gated.

## What stays human gated

The pass proposes and a human disposes on each of these.

- Creating, changing or retiring a Directive concept.
- Naming a relationship on a person page. A relationship is recorded
  only in the owner's own words. The pass records observed facts and
  leaves the naming blank where the human has not named it.
- Superseding a claim that records a human ruling, where the
  replacement is agent inference rather than a newer human statement.
- Deprecating a whole concept, and any deletion of any file, at any
  time.
- The first pass over any new source corpus, and any change to this
  contract itself.

## The pass report

Every run writes one Synthesis concept listing promotions with claim
identities, supersessions with both identities and the reason, indexes
regenerated, flags raised, promotions blocked and why, and the
re-verification queue it emitted. After writing the report, run
`looseleaf check <bundle>` again and cite the result in the report.
The report is the unit the three-review checkpoint rule audits. A run
with no report is treated as a run that did not happen.
