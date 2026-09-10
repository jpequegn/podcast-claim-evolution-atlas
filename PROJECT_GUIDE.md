# Project guide

## What it does

The atlas compares structured claims while preserving their original wording and evidence. It distinguishes direct typed conflicts from changed definitions, different valid times, narrower conditions and insufficient evidence. It gives you a local timeline, a relationship review matrix, an immutable-by-convention sequence of review documents and a report of changes between snapshots.

The default dataset is synthetic. It exercises the software without making claims about real speakers, finance or health. The local Castflow pilot is separate and unreviewed.

## Start the explorer

Requires Rust 1.92+, Node 22.12+ and wasm-pack 0.13.1+.

```sh
git clone https://github.com/jpequegn/podcast-claim-evolution-atlas.git
cd podcast-claim-evolution-atlas
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.13.1 --locked
npm ci
npm run build
npm run dev -- --port 5183
```

Open the URL printed by Vite. The default is a synthetic corpus of 60 claims. Select a topic, open a claim, inspect its scope and evidence, and approve or reject it with a decision note. On Relationships, both claims must be approved before a relationship can be accepted. A reviewer can replace an abstention with a supported semantic classification, but that is a human judgment, not a rule-engine result.

The upload icon reads a local atlas JSON document in your browser; it does not upload it. The download icon exports the full bundle and review chain. Decisions live in browser memory until exported. Reloading discards unexported edits. The dev server binds to loopback. Do not expose it to a network or serve the repository root publicly.

## CLI walkthrough

```sh
mkdir -p data
cargo run -p claim-cli -- init examples/basic.json --out data/first.atlas.json
cargo run -p claim-cli -- validate data/first.atlas.json
cargo run -p claim-cli -- analyze data/first.atlas.json
cargo run -p claim-cli -- timeline data/first.atlas.json task
cargo run -p claim-cli -- review data/first.atlas.json examples/approve-claim.json --out data/reviewed.atlas.json
cargo run -p claim-cli -- diff data/first.atlas.json data/reviewed.atlas.json
cargo run -p claim-cli -- report data/reviewed.atlas.json --as-of 2026-09-10 --previous data/first.atlas.json --out data/report.md
```

Every output path must be new. Review requests carry the exact current revision. Old documents remain valid historical snapshots, so separate files can fork; this is not a multi-user database. Compare forks before continuing. A rejected or corrected claim is imported in a new bundle revision; it is not silently overwritten within an existing review chain.

Approval records who accepted the representation and its evidence, not that the assertion is universally true. Resolution or supersession applies to an accepted disagreement and requires additional evidence beyond the original pair. It never follows automatically from age, popularity or silence.

## Your local pilot

During implementation, the main Castflow database was opened read-only and ten episodes were selected, including 2384. Thirty claims were curated as candidates. The local checkout contains:

- data/pilot-export-v2/: manifest and bounded evidence references.
- data/pilot-claims.json: agent-curated claims with unknown scope left explicit.
- data/pilot-v2.atlas.json: importable explorer document.
- data/pilot-v2-report.md: initial report.

These files are not in GitHub. No real claim has been automatically approved. Three transcript segments from episode 2603 lack usable timestamp ranges and are marked transcript_untimed; the other 27 have valid ranges.

A good first session is to open the pilot document, filter by source, check five claims against their original episodes, and improve population, definitions and time qualifiers before deciding relationships. Keep separate predictions, recommendations and quoted opinions. The source issue's human-reviewed dataset is still an explicit remaining step.

## Export your own Castflow selection

Use a stable snapshot or stop the writer before exporting. The exporter rejects a database that changes during the read. It never calls Castflow schema initialization and never writes to the database.

```sh
uv run --python 3.12 --with duckdb==1.4.1 python scripts/export_castflow.py --db /absolute/path/p3.duckdb --episodes 2384,2488 --segments 1293967,1342465 --out data/selected-export
cargo run -p claim-cli -- import data/selected-export data/selected-claims.json --title "Reviewed selection" --out data/selected.atlas.json
```

The claims file is an array matching examples/basic.json's claims contract, with evidence_ids taken from the exported JSONL. The title does not confer review authority. All imported claims start as candidates.

Exports contain short excerpts and full-segment digests, not full transcripts. A digest establishes which text version was referenced, not factual truth or lawful redistribution. Keep real exports private and use the original source for listening or reading.

## Typical usage

Once a week, curate a small set of relevant claims, compare scope before language, and review only relationships that matter to an active question. Generate a report against last week's document. Use disagreements to guide reading, not as automatic investment, health or engineering decisions.

Useful questions include: which agent-evaluation claims concern final answers versus intermediate tool behavior? Are long-context advocates describing the same task and model generation as critics? Has a later source supplied evidence that actually resolves an earlier disagreement?

## Learning exercises

1. Change a synthetic boolean value and observe a direct conflict.
2. Change the population instead and observe abstention.
3. Compare 500 milliseconds with one second using different inequality operators.
4. Change valid time and distinguish temporal change from contradiction.
5. Approve both claims, accept a relationship and inspect the review hashes.
6. Try to close a disagreement using only its original evidence; it must fail.
7. Add independent evidence in a new bundle, then review an explicit closure.
8. Compare native and browser WASM results using npm run parity.
9. Import a zero-width transcript segment and inspect its untimed evidence badge.

## Innovative extensions

- A prediction-resolution calendar that asks for later evidence without auto-closing claims.
- A reviewed connection to active project-ideas, so unresolved claims become research questions rather than automatic project suggestions.
- Cross-checks against primary papers and later outcome data, with source quality kept explicit.
- A provider-neutral extraction service that only proposes claims and preserves exact spans; evaluate it against independently labeled real data.
- Signed review receipts and explicit bundle migrations for collaborative use.
- A provenance-graph export into Podgraph, leaving this app focused on claim comparison.

## Verification

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
uv run --python 3.12 --with duckdb==1.4.1 python -m unittest discover -s scripts -p 'test_*.py'
npm ci
npm run build
npm run parity
wasm-pack test --node crates/claim-wasm
```

The synthetic evaluation reports expected abstention and missing human/model metrics honestly. See docs/EVALUATION.md and docs/ARCHITECTURE.md for the evidence limits and operating boundaries.

