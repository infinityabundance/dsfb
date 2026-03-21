# Public Dataset Dashboard Demo Path

This legacy path now points at the executed NASA public-dataset workflow in
[`../public_dataset_demo.md`](../public_dataset_demo.md).

Use the dedicated one-command demo runner:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bin dsfb-public-dataset-demo -- \
  --phase all
```

The resulting processed CSV pairs can also be routed through the existing replay-to-report CLI:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bin dsfb-forensics-gen -- \
  --observed-csv crates/dsfb-semiotics-engine/data/processed/nasa_milling/observed.csv \
  --predicted-csv crates/dsfb-semiotics-engine/data/processed/nasa_milling/predicted.csv \
  --scenario-id nasa_milling_public_demo \
  --time-column time
```

For deterministic CSV replay, use the processed NASA inputs documented in
[`../public_dataset_demo.md`](../public_dataset_demo.md) with `--dashboard-replay-csv`.
