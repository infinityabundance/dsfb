# dsfb-tmtr

Deterministic Trust-Monotone Temporal Recursion simulation framework.

## Overview

`dsfb-tmtr` is a deterministic reference implementation of the Trust-Monotone Temporal Recursion (TMTR) operator introduced in the paper *Trust-Monotone Temporal Recursion in Deterministic Structural Dynamics*.

The crate provides a self-contained simulation of bounded temporal recursion inside a DSCD-style observer hierarchy. It operationalizes:

- retroactive refinement of degraded trajectory segments
- bounded forward prediction tubes
- trust-monotone correction propagation
- bounded recursion depth
- preserved causal ordering with no cycle-inducing behavior
- deterministic replayability for paper reproduction

The implementation is intentionally empirical and traceable. It is not production autonomy software.

## Relation to the DSFB Research Stack

This crate focuses on the TMTR layer within the broader deterministic research stack:

- DSFB: deterministic residual estimation
- HRET: hierarchical trust aggregation
- ADD: algebraic deterministic dynamics
- DSCD: deterministic causal topology
- TMTR: bounded temporal recursion and temporal refinement

`dsfb-tmtr` is concerned specifically with TMTR simulation and empirical validation. It does not attempt to implement the full production behavior of the surrounding stack.

## What the Crate Demonstrates

The included scenarios are designed to reproduce the paper’s central claims as empirical artifacts:

- retroactive refinement of past trajectory segments after degraded sensing
- forward prediction tube construction with bounded deterministic intervals
- trust-monotone propagation from higher-trust observers to lower-trust observers only
- bounded recursion depth and convergence stopping
- deterministic replayability under identical configuration
- causal DAG consistency with no backward-time edges and no cycles

These are empirical demonstrations of theoretical claims, not formal proofs.

## Repository Structure

```text
/crates/dsfb-tmtr
  src/                    simulation engine, trust model, causal export, CLI
  notebooks/              Colab notebook for build, execution, plotting, and figures
  tests/                  crate-local regression checks
  Cargo.toml              standalone crate manifest
  README.md               crate documentation
```

At runtime the CLI writes results under:

```text
/output-dsfb-tmtr/YYYY-MM-DD_HH-MM-SS/
```

Each run uses a fresh timestamped directory so prior runs are preserved and artifacts remain auditable.

## Running the Simulation

Run the crate from its own directory:

```bash
cd crates/dsfb-tmtr
cargo run --release -- --scenario all --n-steps 1000
```

You can also target a custom output root:

```bash
cd crates/dsfb-tmtr
cargo run --release -- --scenario disturbance-recovery --output-root ../../output-dsfb-tmtr
```

The CLI writes a new timestamped directory on each run and prints the final output path.

### Workspace Note

This crate is intentionally self-contained and buildable from `crates/dsfb-tmtr` without modifying the monorepo root.

Because the root workspace remains immutable, `cargo run -p dsfb-tmtr` from the repository root is not enabled here. The minimal root change that would be required is adding `"crates/dsfb-tmtr"` to the `[workspace].members` list in [/home/one/dsfb/Cargo.toml](/home/one/dsfb/Cargo.toml). No such change is made by this crate.

## Output Artifacts

Each run emits the following core artifacts:

- `run_manifest.json`
- `config.json`
- `scenario_summary.csv`
- `trajectories.csv`
- `trust_timeseries.csv`
- `residuals.csv`
- `correction_events.csv`
- `prediction_tubes.csv`
- `causal_edges.csv`
- `causal_metrics.csv`
- `notebook_ready_summary.json`

These files expose the trajectory, trust, residual, recursion, and causal quantities needed to inspect TMTR behavior directly.

## Colab Notebook

The notebook at `notebooks/dsfb_tmtr_colab.ipynb` is intended as a reproducible analysis companion. It:

- installs and validates the local environment in Colab
- builds the crate from source
- runs deterministic simulations from scratch
- locates the newest output directory automatically
- loads CSV and JSON artifacts
- generates publication-quality figures into `<run_dir>/figures/`
- prints a concise baseline-versus-TMTR summary

The figure suite includes trajectory reconstruction, retroactive error reduction, trust envelopes, residual convergence, prediction tubes, recursion depth, correction magnitude distribution, causal consistency, and a compact summary comparison panel.

## Determinism and Reproducibility

The simulation is deterministic by design:

- there is no stochastic sampling
- identical configurations yield identical artifact contents
- timestamps affect only the output directory name
- a stable configuration hash is stored in the manifest

This is important for scientific reproducibility and paper review.

## Intended Use

The crate is intended for:

- research exploration of trust-adaptive deterministic temporal recursion
- reproduction of the TMTR paper’s empirical claims
- experimentation with bounded causal-temporal architectures
- educational demonstration of deterministic trust-gated refinement

It is not intended for deployment in safety-critical or production control systems.

## Extending or Integrating the Crate

Possible extension paths include:

- integration with a fuller DSCD runtime
- richer multi-timescale observer hierarchies
- real-sensor or benchmark-data experiments
- domain-specific wrappers for robotics, aerospace, or industrial diagnostics

The current implementation keeps the simulation model simple enough to remain auditable and deterministic.

## License

License information is not yet specified for publication.

## Citation

```bibtex
@misc{debeer_tmtr,
  title        = {Trust-Monotone Temporal Recursion in Deterministic Structural Dynamics},
  author       = {Riaan de Beer},
  year         = {2026},
  note         = {Reference implementation companion crate: dsfb-tmtr},
  howpublished = {Manuscript and reproducibility artifacts}
}
```
