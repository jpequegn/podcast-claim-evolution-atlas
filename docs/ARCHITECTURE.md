# Architecture and limits

## Boundaries

- claim-core owns contracts, exact arithmetic, rules, review transitions, temporal queries, reports and evaluation. It has no clock, network, file I/O or database dependency.
- claim-cli bounds reads, invokes the core and creates new owner-only output files.
- scripts/export_castflow.py isolates DuckDB access in a read-only Python adapter. It records the database snapshot hash, query digest, selected IDs and evidence content digest.
- claim-wasm exposes the same serialized operations to a TypeScript/Vite explorer. Browser imports stay in memory and export through a local download.

The workspace consolidates the proposed separate formats crate into claim-core to avoid splitting tightly coupled schema/validation code. Python rather than native Rust handles DuckDB to reuse the installed Castflow-compatible driver and avoid database build weight.

## Evidence and identity

Version 1 bundles carry claim IDs, structured values, explicit scope/time, modality and stance, retention labels, extractor attribution, confidence metadata and referenced evidence IDs. Evidence records distinguish synthetic fixtures, timestamped transcripts, untimed transcripts and summary-only material. No confidence value participates in deterministic contradiction rules.

Canonical hashes serialize through sorted JSON objects. Arrays retain order; this is a versioned application encoding, not an RFC 8785 conformance claim. Text is not Unicode-normalized; aliases must map directly to canonical identifiers without chains. Original speaker wording remains in evidence. Exact numbers support at most six decimal places and twelve integer digits. Recognized units are ratio/percent, ms/s/min, count and usd. No exchange-rate or dimensional guessing.

## Relationships

Candidate blocking uses canonical subject, exact predicate and topic. Only asserted facts, definitions and predictions with compatible scope are mechanically classified. Unknown scope, partial time overlap, unrecognized units, causal statements and unstructured semantic differences abstain. Different definitions are distinct from contradiction. Boolean polarity is explicit; negative numeric constraints abstain because their complements can be disjoint sets.

Proposals sort by source assertion date and claim ID. All edges follow that total order, so arbitrary backward edges and cycles cannot be imported. Semantic SUPPORTS decisions are reviewer classifications, not automatic entailment. Resolution and supersession are terminal dispositions of reviewed disagreements, not automatically inferred claim truth states.

## Review records

Each review binds the full bundle digest, prior review digest, expected revision, target, decision, reviewer, reason, date and cited evidence. Replaying the chain revalidates all transitions. Both claims must be approved before accepting a relation. Closing an accepted contradiction requires additional evidence. Review timestamps cannot precede cited evidence or prior decisions.

This is a local single-operator document protocol. A person with file access can rewrite a whole document and recompute its hashes; the chain detects accidental/mismatched edits but is not a signature, authentication system or tamper-proof audit service. Old revisions can branch into separate documents. Reconcile explicitly through diff, not shared concurrent writes. Exported browser reviews survive only in the downloaded file.

## Bounds and privacy

500 claims, 1000 evidence records, 5000 candidate pairs, 2000 reviews, 2 MB bundle inputs, 4 MB atlas documents. Exports select at most 20 episodes and 1000 segments. Larger corpora should be split into topical bundles. Limits stop processing rather than silently truncate.

The exporter fails if a selected episode is missing, the DB changes, output exists or the selection is too large. Missing, zero-width or unusable rounded timestamps become untimed transcript evidence, never fabricated times. Full transcript text is not exported; small excerpts and source references permit later verification.

Local native files are created mode 0600; export directories mode 0700. Retention and sensitivity are metadata, not encryption or automatic deletion. Browser downloads follow browser/OS permissions. The Vite development server exposes development assets on loopback; do not publish it or expose it to LAN access.

## Not implemented or not validated

- Live model extraction/adjudication, embeddings, entity discovery or semantic confidence calibration.
- A human-reviewed 60-claim real-data gold set, human-human agreement or measured extraction precision/recall.
- Automatic truth judgments, inferred speaker expertise, source-independence verification or recommendations about people.
- General knowledge-graph integration, arbitrary relationship imports, context migrations or multi-user collaboration.
- Automatic prediction resolution, task scheduling, email or project creation.

The shipped local pilot has 30 unreviewed candidates, including three untimed references. project-ideas #11 should remain open for the human-reviewed acceptance criteria. The completed implementation task set is distinct from that validation milestone.

## References

- [Rust WASM target](https://doc.rust-lang.org/stable/rustc/platform-support/wasm32-unknown-unknown.html)
- [wasm-pack build](https://rustwasm.github.io/docs/wasm-pack/commands/build.html)
- [W3C PROV-O](https://www.w3.org/TR/prov-o/): entity, activity, attribution and derivation informed the contracts; this is not a full RDF implementation.
