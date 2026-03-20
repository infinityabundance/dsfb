# Contributing

`dsfb-semiotics-engine` is intended to remain a deterministic, auditable reference implementation. Contributions should improve clarity, reproducibility, and reviewability without increasing scientific claims.

## Engineering Standards

- Keep the layered architecture intact: `residual -> sign -> syntax -> grammar -> semantics -> artifacts`.
- Prefer explicit types and small pure functions over hidden state or ad hoc string logic.
- Preserve deterministic ordering in exports, hashes, and reported candidate sets.
- Do not introduce probabilistic models, ML classifiers, Bayesian wording, or unique-cause claims.
- Treat semantic bank entries as constrained retrieval rules, not diagnoses.
- Keep outward wording conservative in code, tests, README text, reports, and CLI help.

## Quality Gate

Run the full local gate before submitting changes:

```bash
just qa
```

Equivalent direct commands:

```bash
cargo fmt --check --manifest-path Cargo.toml
cargo clippy --manifest-path Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path Cargo.toml
cargo doc --manifest-path Cargo.toml --no-deps
```

For an end-to-end artifact smoke run:

```bash
just smoke
```

## Extension Guidance

When adding syntax metrics:

- make the deterministic formula explicit
- document the exact meaning in code and README text
- avoid names that imply broader theoretical coverage than the implementation computes
- add regression tests for both expected and conservative fallback behavior

When adding heuristic-bank entries:

- keep entries typed and explicit
- add scope, admissibility, regime, provenance, compatibility, and applicability notes
- explain compatibility and ambiguity outwardly rather than silently resolving them
- add tests for the new motif, its edge cases, and its interaction with existing compatible or incompatible motifs

When changing outputs:

- preserve existing bundle structure unless a change is additive and justified
- keep JSON/CSV field ordering and naming stable where possible
- update README and report wording in the same change so code and documentation remain aligned

## Snapshot Workflow

Snapshot tests are intentionally small and human-readable. Refresh them only when a change is intentional and reviewed:

```bash
DSFB_UPDATE_SNAPSHOTS=1 cargo test --manifest-path Cargo.toml --test snapshots
```

Then inspect the updated files under `tests/snapshots/` before committing them.
