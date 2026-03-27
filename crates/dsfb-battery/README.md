# Deterministic Residual-Based Early Indication of Battery Degradation
## DSFB Battery Health Monitoring

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-battery/notebooks/dsfb_battery_demo.ipynb)



### (NASA PCoE B0005 Evaluation)

## What this shows

This repository evaluates a deterministic residual-based signal for indicating battery degradation transitions.

### On the NASA PCoE B0005 dataset:

A residual-derived signal is computed from observed battery behavior
A simple capacity threshold (85%) is used as a baseline
End-of-life (80%) is used as a reference

The residual-based signal indicates the degradation transition earlier than the threshold baseline in this case.

### Observed result (B0005)

| Metric                     | Cycle |
|--------------------------|------:|
| DSFB signal trigger       |    38 |
| 85% capacity threshold    |    79 |
| End-of-life (80%)         |   101 |
| Lead vs threshold         |    41 |
| Lead vs EOL               |    63 |

*Observed on NASA PCoE B0005; no generalization claimed.*  
*Reproducible using this crate and the provided Colab notebook (deterministic replay).*
The DSFB signal reflects a change in residual structure and should be interpreted as an early indication of transition, not a direct estimate of capacity or remaining useful life.

## How it works (brief)
Computes residual-like quantities from battery signals
Extracts local drift and slew structure
Detects transitions in signal behavior

## Key properties:

deterministic (replayable)
read-only (non-interfering)
does not replace existing BMS or estimators
How to reproduce
Colab: [link]
Notebook: notebooks/phase2_battery_validation.ipynb
Run: cargo run --release
Scope
Single-cell evaluation (B0005)
Offline analysis
Demonstration of behavior, not general proof
A standalone Rust crate implementing the DSFB (Drift–Slew Fusion Bootstrap) structural semiotics engine for battery health monitoring. The crate interprets capacity fade, resistance drift, and knee-onset acceleration in lithium-ion battery data as structured diagnostic signs, producing typed early-warning signals with deterministic auditability. It operates as an interpretive augmentation layer over existing BMS estimation pipelines — it does not replace probabilistic estimators or physics-based models.


## Mathematical Specification

All functions implement named formal objects from the paper:

> Riaan de Beer, *DSFB Structural Semiotics Engine for Battery Health Monitoring: A Deterministic Early-Warning Framework for Capacity Fade, Internal Resistance Drift, and Knee-Onset Detection in Safety-Critical Energy Storage Systems*, Version 1.0, 2026.

### Definition 1: Residual Sign Tuple

The sign tuple at cycle *k*:

```
σ_k = (r_k, d_k, s_k)
```

where:

- Residual: `r_k = y_k − ŷ_k` (capacity deviation from healthy-window nominal)
- Drift (windowed first difference):
  ```
  d_k = (1/W) Σ_{i=0}^{W−1} (r_{k−i} − r_{k−i−1}) = (r_k − r_{k−W}) / W
  ```
- Slew (windowed second difference):
  ```
  s_k = (1/W) Σ_{i=0}^{W−1} (d_{k−i} − d_{k−i−1}) = (d_k − d_{k−W}) / W
  ```

### Definition 2: Battery Grammar State

Three-level finite-state classification at each cycle:

- **Admissible:** `|r_k| ≤ ρ` and no persistent outward drift
- **Boundary:** `|r_k| > 0.8ρ`, or persistent outward drift (L_d consecutive cycles), or persistent slew with drift
- **Violation:** `|r_k| > ρ` (envelope exit)

### Definition 3: Admissibility Envelope

Constructed from the healthy baseline window of N_h cycles:

```
μ_y^(0) = (1/N_h) Σ_{k=1}^{N_h} y_k
σ_y^(0) = sqrt( (1/(N_h−1)) Σ_{k=1}^{N_h} (y_k − μ_y^(0))² )
ρ_y = 3 σ_y^(0)
```

Admissible iff `|r_k^(y)| ≤ ρ_y`.

### Definition 4: Typed Heuristic Bank Entry

```
H_j = (P_j, R_j, A_j, I_j, U_j)
```

- P = Pattern descriptor (channel, drift, slew, temporal signatures)
- R = Regime scope (operating conditions)
- A = Admissibility assumptions
- I = Candidate interpretation (typed degradation motif)
- U = Ambiguity/uncertainty notes

### Proposition 3: Operational Knee-Transition Criterion

Grammar transition to acceleration state requires all of:

```
d_k > θ_d  for L_d consecutive indices
s_k > θ_s  for L_s consecutive indices
Regime assumptions unchanged over interval
```

### Theorem 1: Discrete-Time Finite Envelope Exit Under Sustained Outward Drift

Under sustained outward drift η with envelope expansion κ (where η > κ):

```
k* − k_0 ≤ ⌈ g_{k_0} / (η − κ) ⌉
```

where `g_{k_0} = ρ − |r_{k_0}|` is the initial admissibility gap and k* is the first envelope exit time. For static envelope (κ = 0):

```
t* ≤ ⌈ ρ / η ⌉
```

### Proposition 1: Envelope Invariance Under Inward-Compatible Evolution

If `r_{k_0} ≤ ε_{k_0}` and `r_{k+1} − r_k ≤ ε_{k+1} − ε_k` for all k ≥ k_0, then `r_k ≤ ε_k` for all k ≥ k_0.

### Law 1: Battery Structural Detectability Principle

Detectability of battery degradation is governed by structural separation of residual trajectories relative to regime-dependent admissibility envelopes, not by state-of-health magnitude alone.

## NASA PCoE Battery Dataset

The Stage II proof-of-concept uses the **NASA Prognostics Center of Excellence (PCoE) Battery Dataset**, specifically Cell B0005:

- **Chemistry:** 18650 lithium-ion
- **Protocol:** Constant-current discharge at 2A, charge at 1.5A, 24°C ambient
- **Cycles:** 168 discharge cycles to failure
- **Source:** NASA Ames Research Center, Prognostics Center of Excellence
- **URL:** https://www.nasa.gov/intelligent-systems-division/discovery-and-systems-health/pcoe/pcoe-data-set-repository/

Data is extracted from the original MATLAB `.mat` file using `tools/extract_nasa_b0005.py`.

## Detection Comparison Methodology

The crate compares two detection methods:

1. **DSFB Structural Alarm:** First cycle where grammar state leaves Admissible — `k_alarm = inf{k : Γ_k ∈ {Boundary, Violation}}`
2. **Threshold Baseline:** First cycle where capacity drops below 85% of initial capacity

Both are measured against end-of-life (EOL = 80% of initial capacity):

```
Δk_lead = k_EOL − k_alarm
```

**Honest scope:** This is a Stage II proof-of-concept on a single cell. All preprocessing choices are declared in advance (Section 8 of the paper). The demonstration does not claim universal chemistry transfer, unique mechanism identifiability, or that DSFB replaces probabilistic or physics-based methods. Results are reported as observed.

## Crate Module Structure

| File | Description |
|------|-------------|
| `src/lib.rs` | Library root: module declarations, re-exports, CSV data loader |
| `src/types.rs` | Type definitions: `SignTuple`, `GrammarState`, `ReasonCode`, `EnvelopeParams`, `PipelineConfig`, `Theorem1Result` |
| `src/math.rs` | Mathematical core: residual construction, drift/slew computation, envelope parameterization, Theorem 1 exit bound |
| `src/detection.rs` | Detection engine: grammar state evaluation, reason code assignment, full pipeline, threshold baseline, Theorem 1 verification |
| `src/export.rs` | Artifact export: CSV trajectory, JSON detection results |
| `src/bin/dsfb_battery_demo.rs` | CLI binary: runs full pipeline on B0005 data, exports to timestamped output folder |
| `tools/extract_nasa_b0005.py` | Python script: downloads NASA PCoE dataset, extracts B0005 capacity data to CSV |
| `notebooks/dsfb_battery_demo.ipynb` | Colab notebook: full pipeline, 12 figures, Theorem 1 verification, artifact export |

## Output Artifact Structure

Each run creates a timestamped folder:

```
outputs/dsfb_battery_YYYYMMDD_HHMMSS/
  fig01_capacity_fade.png
  fig02_residual_trajectory.png
  fig03_drift_trajectory.png
  fig04_slew_trajectory.png
  fig05_admissibility_envelope.png
  fig06_grammar_state_timeline.png
  fig07_detection_comparison.png
  fig08_theorem1_verification.png
  fig09_semiotic_projection.png
  fig10_cumulative_drift.png
  fig11_lead_time_comparison.png
  fig12_heuristics_bank_entry.png
  semiotic_trajectory.csv
  stage2_detection_results.json
  dsfb_battery_figures_YYYYMMDD_HHMMSS.pdf
  dsfb_battery_artifacts_YYYYMMDD_HHMMSS.zip
```

## Build and Run

```bash
cd crates/dsfb-battery

# Extract NASA data (requires scipy)
pip install scipy
python3 tools/extract_nasa_b0005.py

# Build
cargo build --release

# Run demo
cargo run --release --bin dsfb-battery-demo

# Run tests
cargo test
```

## IP Notice

This paper is published under CC BY 4.0. The CC BY 4.0 license applies to the text and figures of this paper as a written work and to all prior and subsequent papers in the DSFB series published by the same author; it does not constitute a license to the theoretical framework, formal constructions, or methods described herein. Reference implementations, Rust crates, Colab notebooks, and all associated code artifacts are released under the Apache 2.0 license. The Apache 2.0 license applies solely to those software artifacts as executable and distributable works; it does not constitute a license to the underlying theoretical framework, mathematical architecture, formal constructions, or supervisory methods from which those artifacts derive. The theoretical framework, formal constructions, mathematical architecture, and supervisory methods described in this paper and in all papers in the DSFB series constitute proprietary Background IP of Invariant Forge LLC. Commercial deployment, integration, sublicensing, or derivative use of the framework—including re-derivation by abstraction, equivalent reformulation, notation substitution, or domain translation—requires a separate written license from Invariant Forge LLC. Licensing: licensing@invariantforge.net

## Citation

```bibtex
@article{debeer2026dsfb_battery,
  title={DSFB Structural Semiotics Engine for Battery Health Monitoring:
         A Deterministic Early-Warning Framework for Capacity Fade,
         Internal Resistance Drift, and Knee-Onset Detection in
         Safety-Critical Energy Storage Systems},
  author={de Beer, Riaan},
  year={2026},
  note={Version 1.0},
  url={https://github.com/infinityabundance/dsfb/tree/main/crates/dsfb-battery},
  license={CC BY 4.0 (text), Apache 2.0 (code)}
}
```
