## Evidence Ledger

Every public-facing numeric claim in this repository should map to one command, one artifact, or one generated section.

| Claim Surface | Generated From | Artifact |
|---------------|----------------|----------|
| README results table | `cargo run --bin dsfb-regenerate-public-artifacts` | `docs/generated/README_RESULTS.md` |
| Full evaluation narrative | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/evaluation_results.txt` |
| Demo output | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/demo-output.txt` |
| Sensitivity sweep table | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/sensitivity_sweep.csv` |
| Scenario CSV: Clock Drift | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/clock_drift.csv` |
| Scenario CSV: Partial Partition | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/partial_partition.csv` |
| Scenario CSV: Channel Backpressure | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/channel_backpressure.csv` |
| Scenario CSV: Async Starvation | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/async_starvation.csv` |
| Paper TeX results table | `cargo run --bin dsfb-regenerate-public-artifacts` | `paper/generated/results_summary.tex` |
| Audit contract summary | `cargo run --bin dsfb-regenerate-public-artifacts` | `docs/generated/AUDIT_CONTRACT.md` |
| Paper TeX audit contract | `cargo run --bin dsfb-regenerate-public-artifacts` | `paper/generated/audit_contract.tex` |
| Claim ledger | `cargo run --bin dsfb-regenerate-public-artifacts` | `docs/generated/CLAIM_LEDGER.md` |
