# Illustrative External-Bank Example

This example shows how to run the crate with a small external heuristic-bank artifact instead of the builtin fallback.

The goal is architectural inspection, not stronger scientific claims. The example bank is intentionally minimal and is only suitable for smoke tests and reviewer-visible provenance checks.

## Command

From the repository root:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --scenario nominal_stable \
  --bank-mode external \
  --bank-path crates/dsfb-semiotics-engine/tests/fixtures/external_bank_minimal.json
```

Strict validation is the default. If you need a review-only authoring run, use:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --scenario nominal_stable \
  --bank-mode external \
  --bank-path crates/dsfb-semiotics-engine/tests/fixtures/external_bank_minimal.json \
  --bank-validation-mode permissive
```

## What To Inspect

The run exports:

- `json/loaded_heuristic_bank_descriptor.json`
- `json/heuristic_bank_validation.json`
- `csv/heuristic_bank_validation.csv`
- `run_metadata.json`
- `manifest.json`

These artifacts expose the bank schema version, bank version, bank source kind, content hash, and validation result.

## Interpretation

- This is not field validation.
- This is not a better bank than the builtin reference bank.
- It is a deterministic demonstration that the runtime can load a typed external bank artifact, validate it, and record exactly which bank was used for the run.
