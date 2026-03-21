# Replay-To-Report Forensics CLI

The crate now includes a dedicated post-flight utility:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bin dsfb-forensics-gen -- \
  --observed-csv crates/dsfb-semiotics-engine/tests/fixtures/observed_fixture.csv \
  --predicted-csv crates/dsfb-semiotics-engine/tests/fixtures/predicted_fixture.csv \
  --scenario-id fixture_csv \
  --time-column time \
  --output-dir /tmp/dsfb-forensics
```

This is the hero workflow for turning a clearly documented observed/predicted CSV pair into the
normal deterministic artifact bundle:

- PDF report
- PNG and SVG figures
- CSV and JSON audit tables
- ZIP bundle

If the host supports it, add `--open` to attempt opening the generated PDF automatically:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bin dsfb-forensics-gen -- \
  --observed-csv crates/dsfb-semiotics-engine/tests/fixtures/observed_fixture.csv \
  --predicted-csv crates/dsfb-semiotics-engine/tests/fixtures/predicted_fixture.csv \
  --scenario-id fixture_csv \
  --time-column time \
  --open
```

When no platform opener is available, the command reports `open_status=unsupported:...` instead of
silently failing. This is a post-flight forensics convenience layer over the same deterministic
engine and artifact exporter; it is not a field-validation or certification claim.
