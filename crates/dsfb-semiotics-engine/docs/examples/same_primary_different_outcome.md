# Same Primary Behavior, Different Structural Outcome

This note highlights one of the crate's central deterministic claims:

apparent first-order similarity does not guarantee the same structural or semantic outcome.

In short: same primary behavior can still lead to a different outcome once higher-order structure
is considered.

Two reproducible paths already in the crate show this cleanly:

1. Synthetic magnitude-matched pair
2. NASA Bearings paper Figure 9

## Synthetic Pair

Run:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -- \
  --scenario magnitude_matched_admissible \
  --scenario magnitude_matched_detectable
```

Interpretation:

- primary residual scale is intentionally similar
- one case stays admissible
- the other develops a persistent outward relation to the envelope and becomes detectable

## NASA Bearings Paper Figure 9

Regenerate:

```bash
cargo run --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --bin dsfb-public-dataset-demo -- \
  --dataset nasa_bearings --phase all
```

Interpretation:

- two within-run NASA Bearings windows are matched on similar primary residual magnitude
- their meta-residual slew structure differs materially
- their downstream grammar outcome differs materially

This is the paper-facing within-run version of the same argument:

primary behavior alone is insufficient.
