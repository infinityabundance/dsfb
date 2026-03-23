# dsfb-computer-graphics

`dsfb-computer-graphics` is a self-contained Rust artifact for evaluating DSFB-style supervision in temporal reuse and fixed-budget sampling. The crate is designed to be decision-clean rather than cosmetically polished: it exposes point-ROI caveats, mixed outcomes, gate-like trust behavior, CPU-only timing limits, and validation failures instead of hiding them.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## What This Crate Proves

- A host-realistic minimum supervisory path exists and produces a real effect without relying on privileged visibility hints.
- Point-like ROI evidence and region-ROI evidence are separated explicitly in reports and validation.
- The current trust signal is described honestly as gate-like / weakly graded rather than advertised as smoothly calibrated.
- Motion disagreement is no longer hidden inside the minimum path; it is treated as an optional extension and reported separately.
- Demo B no longer depends only on the original thin sub-pixel line case.
- Hazard weights are centralized and sensitivity-vetted inside this crate.
- A hardware-facing timing path exists as a CPU-only proxy with analytical traffic and op estimates.

## What This Crate Does Not Prove

- It does not prove production-scene generalization.
- It does not prove measured GPU wins on real hardware.
- It does not prove deployment readiness or engine integration completeness.
- It does not prove globally calibrated trust or globally optimal parameter settings.

## Core Evidence Shape

### Demo A

Temporal reuse study with:

- fixed alpha
- residual threshold
- neighborhood clamp
- depth/normal rejection
- reactive-mask-style baseline
- strong heuristic baseline
- DSFB visibility-assisted reference
- DSFB host-realistic minimum
- DSFB gated reference
- DSFB motion-augmented extension
- DSFB ablations

Scenario suite:

- `thin_reveal`
- `fast_pan`
- `diagonal_reveal`
- `reveal_band`
- `motion_bias_band`
- `contrast_pulse`
- `stability_holdout`

### Demo B

Fixed-budget allocation study with:

- uniform
- edge-guided
- residual-guided
- contrast-guided
- variance-guided
- combined heuristic
- native trust
- imported trust
- hybrid trust + variance

## Recommended Commands

Use release mode for serious artifact generation:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-all --output generated/final_bundle
```

Run one scenario through the full bundle:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-scenario reveal_band --output generated/single_scenario
```

Run the canonical ablation slice:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-ablations --output generated/ablations
```

Generate the timing path only:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-timing --output generated/timing_only
```

Generate the resolution study only:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-resolution-scaling --output generated/scaling_only
```

Generate the sensitivity sweep only:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-sensitivity --output generated/sensitivity_only
```

Generate Demo B only:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-demo-b --output generated/demo_b_only
```

Generate the Demo B efficiency package:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-demo-b-efficiency --output generated/demo_b_efficiency_only
```

Export the small operator-facing summary:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- export-minimal-report --output generated/minimal
```

Validate an artifact directory:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- validate --output generated/final_bundle
```

## Output Contract

`run-all` writes the full bundle under the chosen output directory, including:

- `report.md`
- `reviewer_summary.md`
- `five_mentor_audit.md`
- `check_signing_blockers.md`
- `trust_diagnostics.md`
- `trust_diagnostics.json`
- `timing_report.md`
- `timing_metrics.json`
- `resolution_scaling_report.md`
- `resolution_scaling_metrics.json`
- `parameter_sensitivity_report.md`
- `parameter_sensitivity_metrics.json`
- `demo_b_decision_report.md`
- `demo_b_efficiency_report.md`
- `demo_b_metrics.json`
- `metrics.json`
- `figures/*.svg`
- `demo_b/*`

The validator fails if required files are missing, if point-ROI disclosure disappears, if degenerate trust rank correlation is presented as a headline claim, or if the timing report stops declaring that it is a CPU-only proxy.

## Key Current Readout

From the current default release bundle:

- Canonical `thin_reveal` ROI size is `1` pixel.
- `diagonal_reveal` ROI size is also `1` pixel.
- `reveal_band` and `motion_bias_band` provide the larger region-ROI evidence path.
- The current host-realistic trust behavior is reported as near-binary / gate-like.
- Actual GPU timing is not measured; the timing path is explicitly labeled `cpu_only_proxy`.

## Minimum Viable Integration Surface

The minimum host-realistic path consumes:

- current color
- reprojected history
- motion vectors
- current and reprojected depth
- current and reprojected normals

It produces:

- trust
- intervention
- alpha
- debug proxy fields

The minimum path no longer includes motion disagreement by default. That cue is kept as an optional extension and is reported separately.

For details:

- `docs/integration_surface.md`
- `docs/cost_model.md`
- `docs/gpu_path.md`
- `docs/validation_contract.md`

## DSFB Integration into Temporal Reuse

Baseline resolve:

```text
C_t(u) = alpha * C_t_current(u) + (1 - alpha) * C_{t-1}_reproj(u)
```

Supervised resolve:

```text
C_t(u) = alpha_t(u) * C_t_current(u) + (1 - alpha_t(u)) * C_{t-1}_reproj(u)
alpha_t(u) = alpha_min + (alpha_max - alpha_min) * (1 - T_t(u))
```

The current crate demonstrates supervisory modulation of temporal reuse, not a replacement renderer.

## GPU Implementation Considerations

The crate now exposes a timing path, but the report states explicitly that it is a CPU-only proxy in the current environment. The timing bundle still provides:

- per-stage timing
- op and traffic estimates
- minimum, host-realistic, and research/debug timing modes
- a higher-resolution selected-scenario proxy

For details, see `docs/gpu_path.md`.

## Mission and Transition Relevance

The artifact is useful when reviewers need:

- a replayable temporal-reuse failure story
- explicit blocker disclosure
- a path from synthetic evidence to engine-adjacent evaluation
- a diligence package that says where DSFB is neutral or worse

It remains a synthetic evaluation artifact rather than a fielded mission system.

## Product Framing and Integration Surfaces

The current strongest honest framing is:

- evaluation-ready for serious internal review
- not yet backed by real GPU measurements
- not yet backed by external engine validation
- suitable for diligence conversations because the weak points are surfaced, not hidden

## Scope Boundary

This crate is intentionally self-contained. All code, docs, metrics, reports, and generated outputs needed for the artifact live under `crates/dsfb-computer-graphics`.
