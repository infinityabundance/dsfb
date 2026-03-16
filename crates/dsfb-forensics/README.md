# dsfb-forensics

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/infinityabundance/dsfb/blob/main/LICENSE)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-forensics/dsfb_forensics_repro.ipynb)

`dsfb-forensics` is the reference specification and audit layer for the Drift-Slew Fusion Bootstrap stack. It wraps the existing `dsfb` observer with a deterministic forensic engine that reconstructs a trust-gated causal graph, compares that graph against a parallel EKF baseline, detects causal-topology shatter events, records silent failures, and writes timestamped audit artifacts to `output-dsfb-forensics/<YYYYMMDD_HHMMSS>/`.

The crate exists to answer a narrow question precisely: when DSFB down-weights or prunes an observation, can the repository produce a replayable, machine-readable explanation of why that happened, whether the causal graph fragmented, whether a stochastic baseline would have accepted the same update anyway, and what the scaling cost of that decision was?

## What The Code Does

For every input step, the crate performs the same ordered pipeline:

1. Loads a deterministic trace from CSV or JSON.
2. Advances a `dsfb::DsfbObserver` and captures residuals, residual envelopes, and raw trust weights.
3. Advances a parallel EKF baseline when `--baseline-comparison on` is selected.
4. Converts raw DSFB weights into a monotone forensic trust score so the audit trail preserves trust descent semantics.
5. Reconstructs a trust-gated causal DAG rooted at the fused DSFB state.
6. Detects a **Shatter Event** when the graph becomes more fragmented because measured slew exceeds the deterministic envelope.
7. Detects a **Silent Failure** when the EKF baseline accepts a measurement that DSFB has pruned or materially down-weighted as structurally inconsistent.
8. Logs per-step complexity metadata and emits the standardized outputs.

The implementation is intentionally explicit and simple. It follows the repository request to use NASA's Power of 10 style constraints: no recursion, bounded loop structure, straightforward data flow, and invariant checks at the loader boundary.

## Theorem And Definition Anchors

Every module and primary public function is annotated with theorem-bank references from the DSFB technical series. The main anchors used by this crate are:

- `CORE-04`: trust-threshold gating induces the causal graph used by the forensic auditor.
- `CORE-08`: anomaly detection implies structural inconsistency, which drives silent-failure reporting.
- `CORE-10`: the full DSFB stack is treated as a deterministic compositional inference algebra.
- `DSFB-07` and `DSFB-08`: residual semantics remain explicit in the trace loader and per-step audit records.
- `DSCD-05` and `DSCD-07`: edge pruning reduces admissibility without creating cycles, which justifies graph fragmentation analysis.
- `TMTR-01`, `TMTR-04`, and `TMTR-10`: the forensic trust score is a monotone descent process with stabilization semantics.

These references are not cosmetic. They define the names of the rule IDs written into `causal_trace.json` and explain why the report talks about trust gating, anomaly soundness, and causal fragmentation in those exact terms.

## Repository Positioning

This crate is intentionally a standalone nested package under `crates/dsfb-forensics`. The repository root workspace manifest is not edited. Build and run it with `--manifest-path`:

```bash
cargo run --release --manifest-path crates/dsfb-forensics/Cargo.toml -- \
  --input-trace crates/dsfb-forensics/fixtures/example_trace.csv \
  --slew-threshold 6.0 \
  --trust-alpha 0.20 \
  --baseline-comparison on \
  --report-format both
```

The crate directly depends on `../dsfb` as a read-only path dependency. The rest of the DSFB family remains untouched and is treated as reference material for terminology, theorem-bank alignment, and audit-layer semantics.

## Input Trace Schema

### CSV

Required columns:

- `dt`
- one or more `measurement_*` columns

Optional columns:

- `step`
- `truth_phi`
- `truth_omega`
- `truth_alpha`

Example:

```csv
step,dt,truth_phi,truth_omega,truth_alpha,measurement_0,measurement_1,measurement_2
0,0.1,0.00,1.00,0.10,0.00,0.02,-0.01
1,0.1,0.10,1.01,0.10,0.11,0.09,0.10
```

### JSON

You can supply either:

- a full document with `channel_names` and `steps`
- or a bare array of steps

Step schema:

```json
{
  "step": 0,
  "dt": 0.1,
  "measurements": [0.0, 0.02, -0.01],
  "truth": {
    "phi": 0.0,
    "omega": 1.0,
    "alpha": 0.1
  }
}
```

If truth is present, the report adds DSFB and EKF phase MAE context. Truth is not required for forensic reasoning.

## CLI

The binary is `dsfb-forensics`.

Arguments:

- `--input-trace <PATH>`
  The CSV or JSON trace to replay.
- `--slew-threshold <FLOAT>`
  The deterministic slew envelope used by the shatter detector.
- `--trust-alpha <FLOAT>`
  The forensic trust floor below which an update is treated as structurally inconsistent.
- `--baseline-comparison <on|off>`
  Enables or disables the EKF baseline observer.
- `--report-format <markdown|json|both>`
  Markdown is always written because `forensic_report.md` is mandatory. `json` and `both` additionally write `forensic_report.json`.

## Output Layout

Every execution creates:

- `output-dsfb-forensics/`
- `output-dsfb-forensics/<timestamp>/`

The timestamp is generated in `YYYYMMDD_HHMMSS` format. If a directory already exists for the current second, the crate waits for the next second rather than overwriting prior output.

Each run writes:

- `causal_trace.json`
  The machine-readable provenance log. Each channel update records `rule_id`, `trust_score`, `causal_depth`, residual data, slew data, prune state, EKF acceptance state, and whether the update participated in a shatter event or silent failure.
- `forensic_report.md`
  The human-readable audit report, including the DSFB Seal of Integrity, reasoning-consistency score, event counts, and complexity bound.
- `forensic_report.json`
  Optional JSON summary written when `--report-format json` or `--report-format both` is selected.

## Seal Semantics

The report awards one of three levels:

- `Level 3`
  No silent failures, no shatter events, and strong reasoning consistency.
- `Level 2`
  Bounded structural debt, but the run remained mostly coherent.
- `Level 1`
  Enough fragmentation or silent-failure activity occurred that the auditor cannot certify strong reasoning integrity.

The seal is not an accuracy score. It is a reasoning-consistency score derived from graph fragmentation, prune activity, and EKF disagreement with DSFB structure.

## Implementation Map

- `src/input.rs`
  Trace loader and schema validation.
- `src/ekf.rs`
  Standard EKF baseline used as the stochastic shadow.
- `src/graph.rs`
  Trust-gated causal DAG reconstruction and fragmentation metrics.
- `src/complexity.rs`
  Per-step Big-O accounting.
- `src/auditor.rs`
  `ForensicAuditor`, shatter detection, silent-failure detection, and run summary generation.
- `src/fs.rs`
  Workspace-root output management and timestamped run directories.
- `src/report.rs`
  Seal assignment and markdown report rendering.
- `src/main.rs`
  CLI entrypoint.

## Colab Reproduction

The notebook [dsfb_forensics_repro.ipynb](./dsfb_forensics_repro.ipynb) clones the repository, installs Rust, builds this crate from scratch, generates a deterministic trace, runs the CLI, loads the emitted JSON outputs with Pandas, and visualizes trust decay, fragmentation, and silent failures with Matplotlib.

Use the Colab badge at the top of this README to open it directly.

## Development Verification

From the repository root:

```bash
cargo test --manifest-path crates/dsfb-forensics/Cargo.toml
```

The included integration test replays `fixtures/example_trace.csv`, runs the audit engine, and verifies that the standardized artifacts can be written to a fresh timestamped output directory.
