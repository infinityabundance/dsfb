# dsfb-computer-graphics

[![Crates.io](https://img.shields.io/crates/v/dsfb-computer-graphics.svg)](https://crates.io/crates/dsfb-computer-graphics)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`dsfb-computer-graphics` is a Rust research crate that implements a **deterministic supervisory layer** for temporal graphics pipelines, grounded in the DSFB (Drift–Slew Fusion Bootstrap) structural semiotics framework. It does not replace temporal anti-aliasing, denoising, or neural reconstruction. It attaches to an existing pipeline, interprets the residual process between predictions and observations, and produces trust, integrity, and intervention signals that regulate temporal history reuse.

The canonical empirical target is **DSFB-TRG++**: trust-regulated temporal reuse supervision on real Unreal Engine frame captures. The strongest current result is the DSFB+strong-heuristic hybrid, which reduces ROI MAE from `0.00657 ± 0.00247` (strong heuristic alone) to `0.00501 ± 0.00178` on the frozen five-capture benchmark. Pure DSFB does not outperform the strong heuristic on its own in the current evaluation.

> **"The experiment is intended to demonstrate behavioral differences rather than establish optimal performance."**

---

## Table of Contents

- [What DSFB Is (and Is Not)](#what-dsfb-is-and-is-not)
- [Mathematical Framework](#mathematical-framework)
  - [Residual Stack](#residual-stack)
  - [Uncertainty Proxy and Normalization](#uncertainty-proxy-and-normalization)
  - [Residual Envelopes](#residual-envelopes)
  - [Admissibility: Smoothstep Thresholds](#admissibility-smoothstep-thresholds)
  - [Structural Grammar](#structural-grammar)
  - [Estimator Integrity Signal](#estimator-integrity-signal)
  - [Directional Anisotropy](#directional-anisotropy)
  - [DSFB Trust Field](#dsfb-trust-field)
  - [DSFB-TRG++: Trust-Regulated Temporal Blending](#dsfb-trg-trust-regulated-temporal-blending)
- [Using the Crate](#using-the-crate)
  - [CLI](#cli)
  - [Library API](#library-api)
- [Benchmark Results](#benchmark-results)
- [GPU Timing](#gpu-timing)
- [Citation](#citation)
- [IP Notice and Licensing](#ip-notice-and-licensing)

---

## What DSFB Is (and Is Not)

DSFB is a **read-only supervisory observer**. Disabling it restores identical baseline behavior.

| What DSFB does | What DSFB does not do |
|---|---|
| Interprets residuals between prediction and observation | Replace rendering, denoising, TAA, or path tracing |
| Produces a per-pixel trust field T(u) ∈ [0,1] | Provide calibrated posterior probabilities |
| Detects hidden estimator stress via integrity signal | Guarantee perceptual quality improvements |
| Classifies residual behavior by structural grammar | Beat strong heuristics with pure DSFB in the current eval |
| Regulates temporal reuse weight via trust (DSFB-TRG++) | Perform engine-integrated production-grade profiling |

DSFB operates on signals the host pipeline already produces: current-frame color, reprojected history, depth, motion vectors, and normals. It does not alter those signals. Insertion point: post-reprojection, pre-blending.

---

## Mathematical Framework

All definitions follow the companion paper. Let `u ∈ Ω` denote a pixel site and `t` a discrete frame index.

### Residual Stack

The primary residual is the discrepancy between observation and prediction:

```
r_t(u) = y_t(u) − ŷ_t(u)
```

For temporal reuse, the multi-channel **residual stack** is:

```
         ┌ r_t^color(u)   ┐
         │ r_t^depth(u)   │
r_t(u) = │ r_t^motion(u)  │  ∈ ℝ^k
         │ r_t^normal(u)  │
         └ r_t^feature(u) ┘
```

Where, for the temporal reuse instantiation:

```
r_t^color(u)  = y_t(u)       − h_{t−1}(Π(u))    — appearance disagreement
r_t^depth(u)  = d_t(u)       − d_{t−1}(Π(u))    — geometric support disagreement
r_t^motion(u) = Π_{t−1→t}(u) − Π̂(u)             — correspondence disagreement
r_t^normal(u) = n_t(u)       − n_{t−1}(Π(u))    — orientation disagreement
```

A scalar residual alone is insufficient for supervisory tasks. Grouped and correlated residual structures are often important when disturbances are correlated rather than isolated.

### Uncertainty Proxy and Normalization

The uncertainty proxy `Σ_t(u) ∈ ℝ^{k×k}` (symmetric positive definite) normalizes the residual stack. In the current crate, a **diagonal surrogate** is used, built from three scalar proxies computed per-pixel with zero heap allocation:

- **Local contrast proxy** — neighborhood color variance (8-connected, allocation-free `for_each_neighbor`)
- **Neighborhood inconsistency proxy** — mean absolute deviation across the 8-connected neighborhood
- **Motion disagreement proxy** — screen-space motion-vector spread

The **normalized residual**:

```
r̃_t(u) = Σ_t(u)^{−1/2} · r_t(u)
```

The **normalized inconsistency statistic** (Mahalanobis-like):

```
q_t(u) = r̃_t(u)ᵀ r̃_t(u) = r_t(u)ᵀ Σ_t(u)^{−1} r_t(u)
```

This statistic captures normalized magnitude but not full structure. It is necessary but not sufficient for trust assignment.

For the diagonal proxy with channel-wise exponential averaging (forgetting factor `ρ_k`):

```
σ²_{k,t}(u) = ρ_k · σ²_{k,t−1}(u) + (1 − ρ_k) · |r_{k,t}(u)|²
```

### Residual Envelopes

A first-order envelope tracks recent residual magnitude per channel:

```
s_{k,t+1}(u) = ρ_k · s_{k,t}(u) + (1 − ρ_k) · |r_{k,t}(u)|,    0 < ρ_k < 1
```

The envelope accumulates recent history, provides a memory state for suppression and recovery, and supports monotone trust mappings.

A temporal residual history window:

```
R_t(u) = { r̃_τ(u) | τ ∈ [t−T, t] }
```

This history supports structural descriptors: persistence, drift, bounded-slew behavior, impulsive departures, and recurrent motif structure.

### Admissibility: Smoothstep Thresholds

Admissibility is always **regime-relative**, never a global scalar threshold. Each proxy channel has a `SmoothstepThreshold` with lower bound `τ_lo` and upper bound `τ_hi`. For a raw signal `s ≥ 0`, the **rejection weight** is:

```
φ(s) = smoothstep((s − τ_lo) / (τ_hi − τ_lo))

smoothstep(x) = clamp(x, 0, 1)² · (3 − 2 · clamp(x, 0, 1))
```

- `s ≤ τ_lo`: fully admissible → φ = 0
- `s ≥ τ_hi`: fully rejected → φ = 1
- Intermediate: smooth continuous rejection weight

The admissibility state space includes normalized residual, temporal increment, proxy, inconsistency statistic, integrity signal, anisotropy, and envelopes:

```
X_adm = { (r̃_t(u), Δr̃_t(u), Σ_t(u), q_t(u), m_t(u), κ_t(u), s_t(u)) }
```

A valid admissibility family satisfies: closed, computationally testable, perturbation-stable, regime-indexed.

### Structural Grammar

The grammar classifies the residual history into one of seven structural labels `S`:

| Label | Typical residual organization | Default supervisory tendency |
|---|---|---|
| σ_nom | Admissible, low persistence, low integrity excursion | Keep / trust reuse |
| σ_occ | Abrupt depth + history inconsistency, local trust collapse | Flush or strong downweight |
| σ_mv | Motion mismatch without consistent depth support | Distrust correspondence, conservative fallback |
| σ_hist | Persistent moderate mismatch, stale history build-up | Reset or decay history |
| σ_spec | View-dependent instability, grouped feature inconsistency | Conservative reconstruction, reduced enhancement |
| σ_thin | Fragile support near thin geometry or undercoverage | Downweight, caution |
| σ_unstable | Oscillatory suppression/recovery, high integrity churn | Stabilize, damp, log, fallback |

The crate's classifier maps per-pixel proxy signals to this label family via a fixed priority ordering:

```
σ*(u) = σ_mv   if φ_m(p_m(u)) > φ_c(p_c(u)) and φ_m > τ_mv
       σ_occ   if depth-discontinuity signal exceeds threshold
       σ_thin  if thin-geometry flag is active at u
       σ_hist  if neighborhood inconsistency exceeds threshold
       σ_nom   otherwise
```

`σ_unstable` and `σ_spec` are reserved in the grammar but require temporal persistence tracking across multiple frames (planned for a future release).

**Structural states in code** (`StructuralState` enum):
- `Nominal` — residuals admissible, no structural intervention needed
- `DisocclusionLike` — depth/history inconsistency suggests invalid history
- `UnstableHistory` — persistent moderate mismatch, stale accumulation
- `MotionEdge` — motion-vector inconsistency dominates

### Estimator Integrity Signal

The integrity signal detects **hidden estimator stress** — proxy evolution inconsistent with the declared regime, even when raw residual magnitude is masked by temporal blending or denoising.

The proxy-slew statistic (covariance-growth rate):

```
m_t(u) = tr(Σ_t(u)^{−1} · Σ̇_t(u))
```

where `Σ̇_t(u) = Σ_t(u) − Σ_{t−1}(u)`. For the diagonal proxy, this is cheap to compute:

```
m_t(u) = Σ_k  σ̇²_{k,t}(u) / σ²_{k,t}(u)
```

This is interpretable as a first-order **uncertainty-volume growth rate** (approximates `Δ log det(Σ_t)`).

**Interpretation:**
- Large positive `m_t(u)`: local uncertainty expanding — correspondence failure, specular ambiguity, neural feature instability
- Large negative `m_t(u)`: abrupt contraction — possibly benign (correction) or suspicious (overconfident collapse)
- Persistent oscillation: estimator churn, repeated regime switching

The regime-relative integrity excursion:

```
e_t(u) = max(0, (|m_t(u)| − γ_nom(ρ_t(u))) / γ_scale(ρ_t(u)))
```

Smoothed integrity state (forgetting factor `ζ`):

```
ē_t(u) = ζ · ē_{t−1}(u) + (1 − ζ) · e_t(u)
```

**Key theorem:** under nominal bounded slew assumption `|m_t| ≤ γ_nom`, a regime shift at time `t*` is detectable via the integrity channel whenever `tr(Σ_{t*}^{−1} · Σ̇_{t*}) > γ_nom`. This can detect regime shifts even when raw residual magnitude remains moderate.

### Directional Anisotropy

For the diagonal proxy, the **channel-dominance anisotropy ratio** is:

```
κ_t(u) = max_k σ²_{k,t}(u) / (min_k σ²_{k,t}(u) + ε)
```

- `κ_t ≈ 1`: near-isotropic uncertainty
- Large `κ_t`: strong directional concentration (unresolved transport modes, correspondence failure direction, thin geometry)

The **directional growth rate** from the leading eigenvalue:

```
g_t(u) = (λ_{1,t}(u) − λ_{1,t−1}(u)) / (λ_{1,t−1}(u) + ε)
```

Joint interpretation with the integrity signal:

| e_t | κ_t | Interpretation | Supervisory implication |
|---|---|---|---|
| Low | Low | Stable, diffuse uncertainty | Nominal operation |
| Low | High | Stable but structured difficulty | Cautious reuse or targeted refinement |
| High | Low | Diffuse instability | Global suppression or fallback |
| High | High | Rapidly growing structured failure | Aggressive intervention, logging, rerouting |

### DSFB Trust Field

The trust field `T_t(u) ∈ [0,1]` aggregates all diagnostics into a single supervisory scalar:

```
T_t(u) = Γ_T(q_t(u), ē_t(u), κ_t(u), s_t(u), ρ_t(u), Φ_t(H_t(u)))
```

A practical multiplicative form:

```
T_t(u) = exp(−α_q · q_t) · exp(−α_e · ē_t) · exp(−α_κ · ψ_κ) · exp(−α_s · ψ_s) · ω_{Φ_t}
```

where `ω_{Φ_t} ∈ (0, 1]` is a grammar-dependent weight. In the crate, trust is computed as:

```rust
let hazard = parameters.weights.residual * residual_gate
    + parameters.weights.depth * depth_gate
    + parameters.weights.normal * normal_gate
    + parameters.weights.motion * motion_gate
    + parameters.weights.neighborhood * neighborhood_gate
    + parameters.weights.thin * thin_gate
    + parameters.weights.grammar * grammar_component;

let trust = 1.0 - hazard.clamp(0.0, 1.0);
```

**Suppression is faster than recovery** — trust is lost quickly when structural inconsistency appears and regained more cautiously after stability returns. Anomaly memory `M_t(u)` tracks persistence:

```
M_t(u) = λ · M_{t−1}(u) + (1 − λ) · χ_t(u)
```

**Finite-time detectability theorem:** under monotone trust response and persistent inadmissibility over interval `[t0, t0+L]`, there exists a finite `t* ≤ t0 + L` such that `T_{t*}(u) ≤ τ_crit` for any prescribed threshold. Structural incompatibility relative to regime — not raw residual magnitude alone — drives detectability.

### DSFB-TRG++: Trust-Regulated Temporal Blending

DSFB-TRG++ replaces ad hoc TAA heuristics with trust-regulated blending. The standard temporal accumulation formula is:

```
ŷ_t(u) = α_t(u) · y_t(u) + (1 − α_t(u)) · h_{t−1}(Π_{t−1→t}(u))
```

In DSFB-TRG++, the blending weight is governed by the trust field:

```
α_t(u) = 1 − T_t(u)
```

- **High trust** (`T_t ≈ 1`): strong history reuse, small `α_t`
- **Low trust** (`T_t ≈ 0`): rely on current estimate, large `α_t`

The intervention ladder:

| Trust level | Intervention |
|---|---|
| `T_t ≥ τ_high` | keep — maintain reuse |
| `τ_mid ≤ T_t < τ_high` | downweight — reduce reuse contribution |
| `τ_low ≤ T_t < τ_mid` | decay — attenuate accumulated history |
| `T_t < τ_low` | flush — discard history |

Grammar overrides: `σ_occ` → immediate flush; `σ_unstable` → decay or fallback; `σ_hist` → decay.

**DSFB-TRG++ algorithm per pixel:**
1. Compute residual stack `r_t(u)`
2. Construct uncertainty proxy `Σ_t(u)` from lightweight ensemble
3. Compute diagnostics: `q_t(u)`, `e_t(u)`, `κ_t(u)`, `g_t(u)`
4. Evaluate admissibility and structural grammar label `Φ_t`
5. Update trust `T_t(u)` and anomaly memory `M_t(u)`
6. Select intervention and apply blending or reset

---

## Using the Crate

### CLI

Build and run in release mode:

```bash
cd crates/dsfb-computer-graphics
cargo build --release
```

**Run the canonical Unreal-native replay** (strict path — requires real Unreal exports):

```bash
WGPU_BACKEND=vulkan cargo run --release -- run-unreal-native \
  --manifest examples/unreal_native_capture_manifest.json \
  --output generated/unreal_native_runs
```

This validates a `dsfb_unreal_native_v1` manifest, runs DSFB temporal supervision on the imported captures, and writes a timestamped evidence bundle.

**Run the full demo suite** (synthetic + Unreal-native):

```bash
cargo run --release -- run-all --output generated/all_runs
```

**Run Demo A only** (temporal reuse comparison):

```bash
cargo run --release -- run-demo-a --output generated/demo_a
```

**Run the scenario suite** (10 deterministic synthetic scenarios):

```bash
cargo run --release -- run-scenario --all --output generated/scenarios
```

**Run GPU timing and resolution scaling study**:

```bash
WGPU_BACKEND=vulkan cargo run --release -- run-timing --output generated/timing
WGPU_BACKEND=vulkan cargo run --release -- run-resolution-scaling --output generated/scaling
```

**Validate a completed evidence bundle**:

```bash
cargo run --release -- validate-final --output generated/final_bundle
```

**Prepare datasets** (Davis optical flow / Sintel):

```bash
cargo run --release -- prepare-davis --input /path/to/davis --output data/davis
cargo run --release -- prepare-sintel --input /path/to/sintel --output data/sintel
```

Available subcommands: `run-unreal-native`, `run-all`, `run-demo-a`, `run-demo-b`, `run-ablations`, `run-scenario`, `run-timing`, `run-resolution-scaling`, `run-sensitivity`, `run-demo-b-efficiency`, `run-gpu-path`, `run-realism-suite`, `import-engine-native`, `run-engine-native-replay`, `import-external`, `export-evaluator-handoff`, `validate`, `validate-final`, `generate-scene`, `prepare-davis`, `prepare-sintel`.

### Library API

The crate exposes its supervision core as a library (`dsfb_computer_graphics`).

**Add to `Cargo.toml`:**

```toml
[dependencies]
dsfb-computer-graphics = "0.1"
```

**Core supervision function:**

```rust
use dsfb_computer_graphics::{
    supervise_temporal_reuse,
    HostTemporalInputs,
    HostSupervisionProfile,
    default_host_realistic_profile,
};

// Build inputs from your pipeline's per-frame buffers
let inputs = HostTemporalInputs {
    current_color:       &current_color_field,
    reprojected_history: &history_field,
    motion_vectors:      &motion_field,
    current_depth:       &depth_field,
    reprojected_depth:   &prev_depth_field,
    current_normals:     Some(&normals_field),
    reprojected_normals: Some(&prev_normals_field),
    visibility_hint:     None,
    thin_hint:           None,
};

// Select a supervision profile
// "host realistic" uses residual + depth + normal + neighborhood + thin + grammar
let profile = default_host_realistic_profile(
    0.05,  // alpha_min — minimum blending weight
    0.95,  // alpha_max — maximum blending weight
);

// Run the supervisory layer
let outputs = supervise_temporal_reuse(&inputs, &profile);

// Use the outputs
let trust_field        = outputs.trust;        // T_t(u) ∈ [0,1]
let alpha_field        = outputs.alpha;        // blending weight per pixel
let intervention_field = outputs.intervention; // hazard signal ∈ [0,1]
let residual_field     = outputs.residual;     // raw residual (L1 color)
let state_field        = outputs.state;        // StructuralState per pixel
let proxies            = outputs.proxies;      // individual proxy fields
```

**Available supervision profiles:**

```rust
use dsfb_computer_graphics::{
    default_host_realistic_profile,   // residual + depth + normal + neighborhood + thin + grammar
    synthetic_visibility_profile,     // adds synthetic visibility hint (research/debug)
    motion_augmented_profile,         // adds motion disagreement proxy
};
```

**Custom profile:**

```rust
use dsfb_computer_graphics::{
    HostSupervisionProfile,
    host_realistic_parameters,
};

let mut parameters = host_realistic_parameters();
parameters.alpha_range.min = 0.1;
parameters.alpha_range.max = 0.9;
parameters.weights.grammar = 0.3;

let profile = HostSupervisionProfile {
    id: "my_profile".to_string(),
    label: "Custom supervisory profile".to_string(),
    description: "...".to_string(),
    modulate_alpha: true,
    use_depth_proxy: true,
    use_normal_proxy: true,
    use_motion_proxy: true,
    use_neighborhood_proxy: true,
    use_thin_proxy: true,
    use_history_instability: true,
    use_grammar: true,
    parameters,
};
```

**Run the full evidence pipeline programmatically:**

```rust
use dsfb_computer_graphics::{run_all, run_demo_a, run_demo_b, run_unreal_native};

run_all(&output_dir)?;
run_demo_a(&output_dir)?;
run_unreal_native(&manifest_path, &output_dir)?;
```

**Scene generation and validation:**

```rust
use dsfb_computer_graphics::{
    generate_scene_artifacts,
    validate_artifact_bundle,
    export_evaluator_handoff,
};

let scene = generate_scene_artifacts(&scene_config, &output_dir)?;
validate_artifact_bundle(&bundle_path)?;
export_evaluator_handoff(&bundle_path, &handoff_dir)?;
```

---

## Benchmark Results

All numbers from the frozen five-capture Unreal-native benchmark. ROI is defined as pixels where baseline error exceeds 15% of local contrast, computed once from `fixed_alpha` and held fixed across all methods. ROI coverage: **50.60% ± 18.61%** (nearly a global structural error measure). Reference source: exported higher-resolution `reference_color` (Unreal engine proxy, not path-traced ground truth).

### Demo A: ROI MAE on Five-Capture Canonical Sequence

| Method | ROI MAE mean ± std | Full-frame MAE mean ± std | Max error mean ± std |
|---|---|---|---|
| `fixed_alpha` | 0.32966 ± 0.08251 | 0.18033 ± 0.10549 | 0.60403 ± 0.00418 |
| `strong_heuristic` | 0.00657 ± 0.00247 | 0.00372 ± 0.00105 | 0.27507 ± 0.04350 |
| `dsfb_host_minimum` | 0.04522 ± 0.00683 | 0.02275 ± 0.00529 | 0.29093 ± 0.04583 |
| **`dsfb_plus_strong_heuristic`** | **0.00501 ± 0.00178** | **0.00305 ± 0.00092** | **0.25144 ± 0.04483** |

Lower is better. Mean and std are within-sequence across the five captures of one ordered shot; they describe repeatability within that shot, not cross-scene generalization.

**Key findings:**
- `dsfb_plus_strong_heuristic` is the strongest current result — a 23.7% relative reduction in ROI MAE over `strong_heuristic`
- Pure DSFB (`dsfb_host_minimum`) is `heuristic_favorable` on all five captures
- The strong hybrid demonstrates that DSFB structural supervision improves strong temporal heuristics

### Trust Temporal Trajectory

Across the five-capture sequence: onset at `frame_0001`, peak ROI error at `frame_0002`, recovery side at `frame_0005`.

```
Mean trust:        0.787 → 0.352 → 0.493
Intervention rate: 0.213 → 0.648 → 0.507
```

### Technology Readiness Level

Current status: **TRL 3.5** — deterministic crate executes on real Unreal-native captures, produces reproducible trust and intervention artifacts, and beats strong heuristic with the DSFB+hybrid method on the frozen five-capture benchmark. Pure DSFB remains heuristic-favorable; multi-scene generalization is untested.

---

## GPU Timing

RTX 4080 SUPER / Vulkan / wgpu. Imported-buffer kernel (not engine-integrated):

| Resolution | GPU dispatch mean (ms) |
|---|---|
| 256×144 | 1.04 |
| 1920×1080 | 17.59 |
| 3840×2160 | 67.72 |

Fast-path proxy (reduced deployment, `src/fast_path.rs`):

| Resolution | GPU dispatch mean (ms) |
|---|---|
| 1920×1080 | 1.44 |
| 3840×2160 | 5.74 |

These timings do not include engine-side concurrent rendering load.

---

## Canonical Evidence

The canonical evidence package is:

```
generated/canonical_2026_q1/sample_capture_contract_sequence_canonical/
```

Five real Unreal-native captures (`frame_0001` through `frame_0005`) from one ordered shot. Regenerate with:

```bash
WGPU_BACKEND=vulkan cargo run --release -- run-unreal-native \
  --manifest examples/unreal_native_capture_manifest.json \
  --output generated/unreal_native_runs
```

The run directory includes: `summary.json`, `metrics.csv`, `metrics_summary.json`, `canonical_metric_sheet.md`, `aggregation_summary.md`, `comparison_summary.md`, `failure_modes.md`, `provenance.json`, per-frame trust/alpha/intervention/residual maps, trust histogram, trust-vs-error curve, trust-conditioned error map, trust temporal trajectory, boardroom panel, executive evidence sheet, PDF bundle, ZIP bundle.

---

## Citation

```
de Beer, R. (2026). DSFB and Computer Graphics: Deterministic Structural Semiotics -
A Narrow Wedge for Structural Supervision of Trust-Regulated Temporal Reuse -
A Deterministic Observer Layer for Graphics Diagnostics, Auditability, and
Low-Risk Integration (v1.0). Zenodo. https://doi.org/10.5281/zenodo.19432403
```

BibTeX:

```bibtex
@misc{debeer2026dsfbgraphics,
  author    = {de Beer, R.},
  title     = {{DSFB and Computer Graphics: Deterministic Structural Semiotics ---
                A Narrow Wedge for Structural Supervision of Trust-Regulated Temporal Reuse ---
                A Deterministic Observer Layer for Graphics Diagnostics, Auditability, and
                Low-Risk Integration}},
  year      = {2026},
  version   = {v1.0},
  publisher = {Zenodo},
  doi       = {10.5281/zenodo.19432403},
  url       = {https://doi.org/10.5281/zenodo.19432403}
}
```

---

## IP Notice and Licensing

**Software license:** Apache 2.0. Applies to the Rust crate, source code, and all associated software artifacts as executable and distributable works.

**Theoretical framework:** proprietary Background IP of Invariant Forge LLC (Delaware LLC No. 10529072). The Apache 2.0 license does not constitute a license to the underlying theoretical framework, mathematical architecture, formal constructions, or supervisory methods described in the companion paper or in any paper in the DSFB series.

**Paper:** CC BY 4.0 applies to the text and figures of the companion paper as a written work. It does not constitute a license to the theoretical framework or methods described therein.

**Prior art deposits:** `10.5281/zenodo.15136609` (DSFB Structural Semiotics Engine, 2024) and `10.5281/zenodo.15136610` (DSFB TMTR Framework, 2024) predate this paper.

Commercial deployment, integration, or sublicensing of the framework requires a separate written license from Invariant Forge LLC. Licensing inquiries: licensing@invariantforge.net

---

## Reproducibility

- [CURRENT_STATUS.md](CURRENT_STATUS.md) — canonical benchmark numbers and current classification
- [SCENE_INDEX.md](SCENE_INDEX.md) — synthetic scenario descriptions
- [docs/EVIDENCE_WORKFLOW.md](docs/EVIDENCE_WORKFLOW.md) — evidence generation workflow
- [docs/FAILURE_MODES.md](docs/FAILURE_MODES.md) — known failure modes and mitigations
- [docs/REPRODUCIBILITY.md](docs/REPRODUCIBILITY.md) — full reproducibility guide
- [docs/DATASET_SCHEMA.md](docs/DATASET_SCHEMA.md) — dataset manifest schema
- [docs/UNREAL_CAPTURE_GUIDE.md](docs/UNREAL_CAPTURE_GUIDE.md) — Unreal Engine capture guide
- [colab/dsfb_computer_graphics_demo.ipynb](colab/dsfb_computer_graphics_demo.ipynb) — interactive demo notebook
- [colab/dsfb_unreal_native_evidence.ipynb](colab/dsfb_unreal_native_evidence.ipynb) — Unreal-native evidence notebook
