# Validation Contract

This document defines what `cargo run --release -- validate --output <dir>` must check for a completed bundle.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Required Generated Reports

- `report.md`
- `reviewer_summary.md`
- `five_mentor_audit.md`
- `check_signing_blockers.md`
- `trust_diagnostics.md`
- `timing_report.md`
- `resolution_scaling_report.md`
- `parameter_sensitivity_report.md`
- `demo_b_decision_report.md`
- `demo_b_efficiency_report.md`

Each decision-facing report must contain:

- a `What Is Not Proven` section
- a `Remaining Blockers` section

## Required JSON Artifacts

- `metrics.json`
- `trust_diagnostics.json`
- `timing_metrics.json`
- `resolution_scaling_metrics.json`
- `parameter_sensitivity_metrics.json`
- `demo_b_metrics.json`

## Required Figures

- `fig_system_diagram.svg`
- `fig_trust_map.svg`
- `fig_before_after.svg`
- `fig_trust_vs_error.svg`
- `fig_intervention_alpha.svg`
- `fig_ablation.svg`
- `fig_roi_nonroi_error.svg`
- `fig_leaderboard.svg`
- `fig_scenario_mosaic.svg`
- `fig_trust_histogram.svg`
- `fig_roi_taxonomy.svg`
- `fig_parameter_sensitivity.svg`
- `fig_resolution_scaling.svg`
- `fig_motion_relevance.svg`
- Demo B figures

## Honesty Gates

Validation must fail if:

- point-like ROI disclosure is missing from the main report
- the timing report does not state whether timing is GPU-measured or CPU-only proxy
- degenerate trust rank correlation is allowed to appear as headline evidence
- unsupported phrases such as `production-ready` or `state-of-the-art` appear in the main report

## Behavioral Gates

Validation must fail if:

- host-realistic DSFB does not beat fixed alpha on the canonical scenario
- the suite loses all neutral or mixed outcomes
- point-like and region-ROI scenario groups are not both surfaced
- Demo B breaks fixed-budget equality

## Reproducibility Commands

Full bundle:

```bash
cargo run --release -- run-all --output generated/final_bundle
```

Validation:

```bash
cargo run --release -- validate --output generated/final_bundle
```

Timing only:

```bash
cargo run --release -- run-timing --output generated/timing_only
```

Resolution scaling only:

```bash
cargo run --release -- run-resolution-scaling --output generated/scaling_only
```

Sensitivity only:

```bash
cargo run --release -- run-sensitivity --output generated/sensitivity_only
```

Demo B efficiency only:

```bash
cargo run --release -- run-demo-b-efficiency --output generated/demo_b_efficiency_only
```
