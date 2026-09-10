# Implementation scope

Eight ordered task issues own contracts, imports, relation rules, review lifecycle, CLI reports, evaluation and real-data pilot, WASM explorer, and hardening.

The Rust core is pure and consumes versioned JSON. A small Python/DuckDB adapter keeps database bindings out of the WASM dependency tree. It opens the database read-only and rejects an export if the file changes while reading. It does not use Castflow's constructor, which can initialize or migrate schema.

Real evidence is exported to ignored data/ directories with owner-only permissions. Public examples use invented sources and explicit synthetic evidence. Agent-curated real claims remain candidates until an operator reviews them. No model calls are needed for V1; free-text semantic relationships abstain unless reviewed.

A local pilot is not a human-reviewed benchmark. Human-human agreement, semantic extraction recall, real prediction outcomes and model calibration cannot be claimed from synthetic fixtures. These require later operator review and labeled data.

The explorer runs the same bounded graph core in WASM. It reads an explicitly selected local JSON bundle and uploads nothing. There is no unattended scheduler or external publishing action.

