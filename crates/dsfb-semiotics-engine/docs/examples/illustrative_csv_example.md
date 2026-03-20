# Illustrative CSV-Driven Example

This example demonstrates the CSV ingestion path using the small public example files committed in:

- `examples/data/illustrative_observed.csv`
- `examples/data/illustrative_predicted.csv`

These files are intentionally small and auditable. They are included to exercise the CSV pipeline reproducibly. They are not field-validation data.

## Exact CLI Run

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --input-mode csv \
  --observed-csv crates/dsfb-semiotics-engine/examples/data/illustrative_observed.csv \
  --predicted-csv crates/dsfb-semiotics-engine/examples/data/illustrative_predicted.csv \
  --scenario-id illustrative_csv_example \
  --time-column time \
  --envelope-mode fixed \
  --envelope-base 0.6
```

## Example Library Run

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --example run_csv_fixture
```

## Expected Behavior

- The same residual -> sign -> syntax -> grammar -> semantics pipeline used for synthetic scenarios is executed.
- Artifacts are written into a fresh timestamped run directory.
- The report and manifest mark the run as `csv`.
- Outputs remain illustrative and deterministic; they do not constitute field validation.

## Why This Example Exists

- to provide a reproducible CSV-driven workflow in version control
- to make ingestion assumptions inspectable
- to give reviewers a small end-to-end example that does not depend on network access
