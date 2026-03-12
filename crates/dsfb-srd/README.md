# dsfb-srd

## Overview

`dsfb-srd` is a deterministic Structural Regime Dynamics (SRD) demonstrator for the paper *Deterministic Causal Architecture for Safety-Critical Autonomous Systems*. It generates a replayable structural event stream, evolves trust from residual envelopes, builds trust-gated causal graphs, sweeps trust thresholds, and exports CSV outputs plus a notebook that reconstructs the figures.

This crate exists to provide deterministic empirical support for the structural claims of the paper, but it is not operational validation of an autonomy system.

## Why this crate exists

The paper makes internal structural claims about how trust gates causal topology:

- trust controls whether causal edges remain admissible,
- connectivity collapses monotonically as the trust threshold rises,
- a structural regime transition appears in the expansion observable `rho(tau)`,
- that transition sharpens as event history size increases,
- degradation and shock intervals are locally detectable from graph statistics.

`dsfb-srd` is a narrow empirical demonstrator for those claims. It is intentionally simple, deterministic, and replayable.

## Relation to the DSFB causal architecture paper

The crate operationalizes a compact structural model rather than a full autonomy stack:

- a deterministic latent signal is generated per event,
- observed values receive deterministic regime-dependent distortion,
- residual envelopes evolve as `e_i = max(decay * e_{i-1}, r_i)`,
- trust evolves as `tau_i = exp(-beta * e_i)`,
- causal edges are admitted only when ordering, windowing, compatibility, structural similarity, and source-trust constraints all hold.

The empirical role of the crate is to show how those deterministic rules induce trust-controlled causal topology and a threshold-sensitive connectivity transition.

## What is empirically demonstrated

- A deterministic event stream with no randomness by default.
- Deterministic trust evolution from residual envelopes.
- Trust-gated DAG construction over structural events.
- Threshold sweeps for the expansion observable `rho(tau)`.
- Finite-size sharpening across `N = 250, 500, 1000, 2000`.
- Time-local connectivity degradation across baseline, degradation, shock, and recovery intervals.

## What is NOT being claimed

- This is not a production autonomy stack.
- This is not a probabilistic simulator.
- This is not an operational flight-data validator.
- This is not evidence that a real aircraft or robot is safe.
- This crate does not validate sensing, estimation, control, or certification claims.

The crate demonstrates deterministic structural behavior only.

## Reproducibility and Run Identity

Each run computes a deterministic `run_id` from configuration parameters only.

- The crate constructs a canonical JSON object from `crate`, `version`, `n_events`, `n_channels`, `causal_window`, `tau_steps`, `shock_start`, `shock_end`, `beta`, and `envelope_decay`.
- It computes a SHA-256 hash of that canonical JSON.
- `config_hash` is the full 64-hex-character SHA-256 digest.
- `run_id` is the first 32 hex characters of that digest.

Two runs with identical configuration parameters will therefore produce identical `run_id` values, even though their timestamp folders differ. This lets results be checked independently while keeping output folders non-overlapping.

## Crate layout

```text
crates/dsfb-srd/
├── Cargo.toml
├── README.md
├── notebooks/
│   └── dsfb_srd_colab.ipynb
└── src/
    ├── compatibility.rs
    ├── config.rs
    ├── event.rs
    ├── experiments.rs
    ├── export.rs
    ├── graph.rs
    ├── lib.rs
    ├── main.rs
    ├── metrics.rs
    ├── signal.rs
    └── trust.rs
```

Because this task avoids editing the repository root manifest, the crate is intentionally standalone. If you later want it in the workspace, add `crates/dsfb-srd` to the root `Cargo.toml` workspace members manually.

## How to run the simulator

Default run:

```bash
cargo run \
  --manifest-path crates/dsfb-srd/Cargo.toml \
  --release \
  --bin dsfb-srd-generate
```

Optional overrides:

```bash
cargo run \
  --manifest-path crates/dsfb-srd/Cargo.toml \
  --release \
  --bin dsfb-srd-generate -- \
  --n-events 2000 \
  --n-channels 4 \
  --tau-steps 101 \
  --shock-start 800 \
  --shock-end 1200 \
  --beta 4.0 \
  --envelope-decay 0.97
```

## Where results are written

Every run writes to a new timestamped directory at the repository root:

```text
output-dsfb-srd/YYYYMMDD-HHMMSS/
```

Example:

```text
output-dsfb-srd/20260312-214530/
```

The crate detects the repository root relative to `crates/dsfb-srd`, creates `output-dsfb-srd/` if needed, allocates a fresh timestamp folder, and writes all CSV outputs there.

## Explanation of CSV outputs

- `run_manifest.csv`: one-row manifest with `run_id`, timestamp folder name, full configuration hash, and primary run parameters.
- `events.csv`: the deterministic primary event stream with latent state, prediction, observation, residual, envelope, trust, and regime label.
- `threshold_sweep.csv`: trust-threshold sweep rows with `rho(tau)` and graph summary statistics across the finite-size experiments.
- `transition_sharpness.csv`: discrete derivatives of `rho(tau)` used to identify the structural transition and its sharpening.
- `time_local_metrics.csv`: windowed connectivity summaries across time for low, critical, and high trust thresholds.
- `graph_snapshot_low.csv`: active edges at a low trust threshold.
- `graph_snapshot_critical.csv`: active edges near the maximal transition sharpness.
- `graph_snapshot_high.csv`: active edges at a high trust threshold.

`threshold_sweep.csv` always includes the paper-facing finite-size set `250, 500, 1000, 2000`. If the primary `--n-events` differs from those sizes, the primary history size is included as an additional sweep size so the event stream and threshold diagnostics remain aligned.

## How to use the Colab notebook

The notebook lives at:

```text
crates/dsfb-srd/notebooks/dsfb_srd_colab.ipynb
```

Workflow:

1. Open the notebook in Colab and run the setup cell.
2. The setup cell clones the DSFB repository into `/content/dsfb` if needed.
3. The setup cell installs Rust with `rustup` if `cargo` is not already available.
4. The setup cell runs `cargo run --manifest-path crates/dsfb-srd/Cargo.toml --release --bin dsfb-srd-generate` from scratch.
5. The notebook then selects the newly created timestamped folder under `output-dsfb-srd/` and uses that fresh run for every figure.
6. Run the remaining cells to produce the figures.

This notebook is intentionally wired to execute the crate and regenerate outputs rather than rely on bundled sample CSVs or manual output uploads. The plotting cells use only `pandas` and `matplotlib`; the setup cell may also invoke `git`, `curl`, and `cargo` inside Colab.

## Interpretation of the figures

- Figure 1 plots `rho(tau)` against the trust threshold for multiple event-history sizes. The expected pattern is monotone connectivity collapse, with a sharper transition for larger histories.
- Figure 2 plots `|drho/dtau|` against the midpoint threshold. The peak identifies the structural transition band and shows how the transition sharpens with system size.
- Figure 3 plots time-local reachable fraction against event index and marks the shock interval. The expected pattern is a local connectivity drop during degradation and shock, with partial recovery afterward.

Those figures support the paper's internal structural claims about trust-gated causal topology. They do not constitute operational validation of an autonomy system.
