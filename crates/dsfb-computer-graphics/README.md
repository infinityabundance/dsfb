# dsfb-computer-graphics

`dsfb-computer-graphics` is a self-contained Rust artifact for evaluating DSFB-style supervision in temporal reuse and fixed-budget sampling. The crate is designed to be decision-clean rather than cosmetically polished: it exposes point-ROI caveats, mixed outcomes, gate-like trust behavior, external-validation gaps, and validation failures instead of hiding them.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## What This Crate Proves

- A host-realistic minimum supervisory path exists and produces a real effect without relying on privileged visibility hints.
- Point-like ROI evidence and region-ROI evidence are separated explicitly in reports and validation.
- The current trust signal is described honestly as gate-like / weakly graded rather than advertised as smoothly calibrated.
- Motion disagreement is no longer hidden inside the minimum path; it is treated as an optional extension and reported separately.
- Demo B no longer depends only on the original thin sub-pixel line case.
- Hazard weights are centralized and sensitivity-vetted inside this crate.
- A GPU-executable minimum kernel now exists alongside the CPU proxy timing path.
- A stable external buffer import path exists for engine handoff without re-architecting the crate.

## What This Crate Does Not Prove

- It does not prove production-scene generalization.
- It does not prove universal engine integration success or externally validated production behavior.
- It does not prove globally calibrated trust or globally optimal parameter settings.

## Strongest Current Evidence

- Host-realistic DSFB still shows a real supervisory effect on explicit region-ROI cases rather than only point-ROI stress tests.
- The crate contains a real GPU-executable minimum kernel with measured-vs-unmeasured disclosure.
- The crate contains a real external replay path that can ingest a stable manifest without re-architecting the evaluator.
- Demo B now compares imported trust against edge/gradient, residual, contrast, variance, combined heuristic, native trust, and hybrid trust/variance policies.

## Biggest Remaining Blockers

- Real external engine captures are still required.
- Imported-capture GPU profiling is still required.
- The realism bridge is broader, but it is still synthetic.
- Strong heuristics still tie or win on some scenarios, so the correct framing remains targeted supervision rather than blanket replacement.

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
- `layered_slats`
- `noisy_reprojection`
- `heuristic_friendly_pan`
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

Generate the GPU execution path only:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-gpu-path --output generated/gpu_path
```

Import an external or synthetic-compat capture through the stable handoff schema:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- import-external --manifest examples/external_capture_manifest.json --output generated/external_demo
```

Run the same path through the evaluator-facing alias:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-external-replay --manifest examples/external_capture_manifest.json --output generated/external_replay
```

Generate the resolution study only:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-resolution-scaling --output generated/scaling_only
```

Generate the realism and taxonomy package only:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-realism-suite --output generated/realism_only
```

Evaluator-facing alias:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-realism-bridge --output generated/realism_bridge
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

Export the external evaluator handoff package:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- export-evaluator-handoff --output generated/evaluator_handoff
```

Validate an artifact directory:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- validate --output generated/final_bundle
```

Final-gate alias:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- validate-final --output generated/final_bundle
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
- `gpu_execution_report.md`
- `gpu_execution_metrics.json`
- `external_replay_report.md`
- `external_handoff_report.md`
- `realism_suite_report.md`
- `realism_bridge_report.md`
- `scenario_taxonomy.json`
- `trust_mode_report.md`
- `competitive_baseline_analysis.md`
- `non_roi_penalty_report.md`
- `product_positioning_report.md`
- `demo_b_decision_report.md`
- `demo_b_efficiency_report.md`
- `demo_b_competitive_baselines_report.md`
- `demo_b_aliasing_vs_variance_report.md`
- `demo_b_scene_taxonomy.json`
- `operating_band_report.md`
- `production_eval_checklist.md`
- `evaluator_handoff.md`
- `minimum_external_validation_plan.md`
- `next_step_matrix.md`
- `check_signing_readiness.md`
- `demo_b_metrics.json`
- `metrics.json`
- `figures/*.svg`
- `demo_b/*`

The validator fails if required files are missing, if point-ROI disclosure disappears, if degenerate trust rank correlation is presented as a headline claim, if external-validation needs disappear from decision-facing reports, or if the timing and GPU reports stop declaring measured vs unmeasured status.

## Key Current Readout

From the current default release bundle:

- Canonical `thin_reveal` ROI size is `1` pixel.
- `diagonal_reveal` ROI size is also `1` pixel.
- `reveal_band`, `layered_slats`, `motion_bias_band`, and `noisy_reprojection` provide the larger region-ROI evidence path.
- The current host-realistic trust behavior is reported as near-binary / gate-like.
- The CPU timing report remains explicitly labeled `cpu_only_proxy`.
- The GPU execution bundle reports measured-vs-unmeasured status separately; on hosts with a usable adapter it records actual GPU timings.

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
- `docs/gpu_execution_path.md`
- `docs/external_handoff.md`
- `docs/engine_integration_playbook.md`
- `docs/production_eval_bridge.md`
- `docs/validation_contract.md`
- `docs/completion_gates.md`

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

The crate now exposes both a CPU proxy timing path and a GPU execution path. The timing report remains CPU-side by design, while `run-gpu-path` records actual GPU execution when a usable adapter is present and explicitly reports when no measurement was possible. The timing bundle still provides:

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
