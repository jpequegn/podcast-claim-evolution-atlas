# Podcast Claim Evolution Atlas

Compare claims across podcast episodes without losing their scope, evidence or review history. Rust handles deterministic comparison and review; WASM runs the same engine in a local browser explorer.

## Try it

Requires Rust 1.92+, Node 22.12+ and wasm-pack.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.13.1 --locked
npm ci
npm run build
npm run dev -- --port 5183
```

Open the printed local URL. Start with the 60-claim synthetic corpus, or open a local atlas document with the upload icon. Claims and relationships have separate review gates. Export the document to retain browser decisions; reloading discards unexported work.

## CLI

```sh
mkdir -p data
cargo run -p claim-cli -- init examples/corpus.json --out data/demo.atlas.json
cargo run -p claim-cli -- analyze data/demo.atlas.json
cargo run -p claim-cli -- evaluate data/demo.atlas.json examples/gold.json
cargo run -p claim-cli -- report data/demo.atlas.json --as-of 2026-12-31
```

Outputs never overwrite existing files. See [PROJECT_GUIDE.md](PROJECT_GUIDE.md) for real Castflow imports, review commands, learning exercises, practical uses and extensions.

## Evidence status

The checked-in corpus is synthetic, not a human-reviewed podcast benchmark. The local pilot contains 30 unreviewed claims across ten episodes and seven source labels. Twenty-seven references are timestamped and three are explicitly untimed. Real exports are gitignored and remain private.

Deterministic rules can identify typed conflicts; they cannot establish truth or resolve ambiguous language. Unknown scope and semantic uncertainty abstain. No LLM calls, data uploads, automatic schedules, email or external actions are performed.

The implementation task set is complete when its eight PRs pass. [Source issue #11](https://github.com/jpequegn/project-ideas/issues/11) remains open for the human-reviewed corpus and empirical validation milestone. See [evaluation](docs/EVALUATION.md) and [architecture](docs/ARCHITECTURE.md).
