# Final Blocker Removal Plan

This document replaces the earlier upgrade plan with a blocker-complete execution map for the current `dsfb-computer-graphics` crate.

Scope boundary:

- Only files inside `crates/dsfb-computer-graphics` may change.
- No workspace-root edits are permitted.
- If a stronger result would normally require engine integration or external benchmark data, this crate must state that limitation explicitly rather than imply it is solved.

## Current Audit Summary

The existing crate is already substantive:

- deterministic multi-scenario Demo A and Demo B flows exist
- host-realistic DSFB mode exists
- stronger heuristics and ablations exist
- fixed-budget equality exists in Demo B
- integration and cost documents exist

The current binding blockers remain real in the audited code and generated outputs:

1. No timing artifact or hardware-facing measurement path exists.
2. Point-like ROI scenarios are headline-visible but not analytically disclosed as point measurements.
3. `trust_error_rank_correlation` is still emitted even when trust occupies only one to three bins.
4. Trust behaves as a near-binary gate in the current implementation.
5. Motion disagreement has no effect in the current ablation suite.
6. Neighborhood clamp failure is shown numerically but not explained mechanically.
7. Demo B still leans on thin sub-pixel structure and aliasing-sensitive wins.
8. No credible resolution scaling artifact exists.
9. Hazard weights and thresholds remain scattered and hand-set.
10. Non-ROI penalty is surfaced numerically but not bounded or interpreted well enough.

## File-Level Implementation Map

### Existing files to change

- `Cargo.toml`
- `README.md`
- `docs/cost_model.md`
- `docs/gpu_implementation.md`
- `docs/integration_surface.md`
- `src/cli.rs`
- `src/config.rs`
- `src/cost.rs`
- `src/dsfb/mod.rs`
- `src/frame.rs`
- `src/host.rs`
- `src/lib.rs`
- `src/metrics.rs`
- `src/pipeline.rs`
- `src/plots.rs`
- `src/report.rs`
- `src/sampling.rs`
- `src/scene/mod.rs`
- `src/taa/mod.rs`
- `tests/colab_notebook.rs`
- `tests/demo_cli.rs`
- `tests/research_artifact.rs`

### New files to add

- `docs/gpu_path.md`
- `docs/validation_contract.md`
- `src/parameters.rs`
- `src/timing.rs`
- generated reports and figures emitted by code:
  - `generated/timing_report.md`
  - `generated/timing_metrics.json`
  - `generated/trust_diagnostics.md`
  - `generated/trust_diagnostics.json`
  - `generated/resolution_scaling_report.md`
  - `generated/resolution_scaling_metrics.json`
  - `generated/parameter_sensitivity_report.md`
  - `generated/parameter_sensitivity_metrics.json`
  - `generated/demo_b_efficiency_report.md`

## Blocker-by-Blocker Plan

### 1. Timing / hardware path

Files:

- `src/timing.rs`
- `src/cost.rs`
- `src/pipeline.rs`
- `src/report.rs`
- `src/cli.rs`
- `docs/gpu_path.md`

Work:

- add a reproducible timing command that measures crate-local CPU proxy timings for:
  - scalar reference path
  - streamlined minimum supervisory path
  - host-realistic path
  - full debug/reporting path
- emit explicit per-stage timing, operations per pixel, memory traffic estimate, and optimization levers
- add a minimal GPU-oriented compute scaffold description and pass decomposition inside the crate
- label the result explicitly as CPU-measured proxy unless actual GPU timing is available at run time

Acceptance:

- `generated/timing_report.md` and `generated/timing_metrics.json` exist
- timing report explicitly states one of:
  - `GPU-measured`
  - `GPU-estimated`
  - `CPU-only proxy`
- no unsupported production language appears

Fail gates:

- validation fails if timing artifacts are missing
- validation fails if timing classification text is absent
- validation fails if unsupported phrases such as `production-ready` appear

### 2. ROI disclosure / scenario / metric repair

Files:

- `src/scene/mod.rs`
- `src/metrics.rs`
- `src/report.rs`
- `src/plots.rs`
- `src/pipeline.rs`

Work:

- add ROI taxonomy metadata:
  - point-like ROI
  - region ROI
  - negative control
- add ROI size and area-fraction reporting everywhere
- add at least two new benefit-expected region-ROI scenarios with ROI > 15 pixels
- ensure at least one benefit-expected scenario has ROI > 50 pixels
- add imperfect-motion / noisy-reprojection scenarios with subpixel or biased motion behavior
- add resolution scaling study across small, medium, and high-resolution configurations
- separate aggregate reporting for point-like and region ROI scenarios

Acceptance:

- every decision-facing report lists ROI size and scenario category
- point-like ROI scenarios are labeled explicitly
- new region-ROI scenarios are present in both metrics and reports
- `generated/resolution_scaling_report.md` and `generated/resolution_scaling_metrics.json` exist

Fail gates:

- validation fails if ROI size is missing
- validation fails if point-like ROI scenarios are unlabeled
- validation fails if aggregate tables mix point-like and region ROI without disclosure

### 3. Trust calibration cleanup

Files:

- `src/host.rs`
- `src/parameters.rs`
- `src/metrics.rs`
- `src/plots.rs`
- `src/report.rs`
- `src/pipeline.rs`

Work:

- centralize trust mapping and hazard merge behavior
- add trust diagnostics:
  - histogram
  - occupied bins
  - entropy / discreteness score
  - calibration bins
  - effective trust-level count
  - operating-mode classification
- quarantine or remove degenerate rank-correlation as a headline metric
- optionally add a smoother graded mode if it improves interpretability without misrepresenting the method

Acceptance:

- `generated/trust_diagnostics.md` and `generated/trust_diagnostics.json` exist
- decision-facing reports state whether the current mode is gate-like or graded
- trust rank correlation is never used as primary calibration evidence when occupancy is degenerate

Fail gates:

- validation fails if trust diagnostics are missing
- validation fails if near-binary behavior is not disclosed when present
- validation fails if calibration quality is claimed without multi-bin support

### 4. Motion disagreement resolution

Files:

- `src/scene/mod.rs`
- `src/host.rs`
- `src/metrics.rs`
- `src/report.rs`
- `docs/integration_surface.md`
- `docs/cost_model.md`

Work:

- add at least one motion-stress scenario with vector bias, boundary mismatch, or motion inconsistency
- compare minimum path versus motion-augmented path
- either:
  - show where motion materially matters, or
  - demote motion from the minimum path and document it as optional

Acceptance:

- ablation report explains motion contribution with scenario evidence
- the minimum path does not include motion disagreement unless evidence justifies it

Fail gates:

- validation fails if the minimum path still includes motion disagreement while all motion evidence is null
- validation fails if motion is kept but not explained analytically

### 5. Neighborhood clamp explanation

Files:

- `src/metrics.rs`
- `src/report.rs`

Work:

- add mechanistic diagnostics for neighborhood clamp:
  - ROI trigger rate
  - clamp-distance statistics
  - cases where history stays inside the local neighborhood hull
- write explicit explanation for why the clamp misses thin reveal cases

Acceptance:

- main report and blocker report explain the failure mode instead of only listing the bad score

Fail gates:

- validation fails if neighborhood clamp is discussed only numerically and not mechanically

### 6. Demo B anti-confound strengthening

Files:

- `src/scene/mod.rs`
- `src/sampling.rs`
- `src/metrics.rs`
- `src/plots.rs`
- `src/report.rs`
- `src/pipeline.rs`

Work:

- add non-subpixel and mixed-width sampling scenes
- add at least one variance-driven or texture-driven scene where edge guidance is not enough
- add native-trust and hybrid guidance variants under strict fixed-budget equality
- expand budget-efficiency curves to multiple budgets
- separate aliasing, variance, and prioritization interpretations in reporting

Acceptance:

- `generated/demo_b_efficiency_report.md` and updated `generated/demo_b_metrics.json` exist
- Demo B decision reporting explicitly distinguishes aliasing recovery from allocation quality
- fixed-budget equality remains exact for every policy

Fail gates:

- validation fails if Demo B still only demonstrates thin sub-pixel evidence
- validation fails if fixed-budget equality is broken
- validation fails if aliasing-versus-allocation distinction is missing from decision-facing reports

### 7. Hazard parameter centralization and sensitivity

Files:

- `src/parameters.rs`
- `src/host.rs`
- `src/taa/mod.rs`
- `src/metrics.rs`
- `src/pipeline.rs`
- `src/report.rs`
- `src/plots.rs`

Work:

- move all DSFB hazard weights, trust mapping constants, smoothstep edges, grammar strengths, and alpha bounds into centralized parameter structs
- centralize baseline thresholds as well
- run systematic one-at-a-time sensitivity sweeps over key parameters
- classify parameters as robust, fragile, or neutral

Acceptance:

- `generated/parameter_sensitivity_report.md` and `generated/parameter_sensitivity_metrics.json` exist
- reports state whether parameters are hand-set, sensitivity-vetted, or calibrated
- magic constants are no longer scattered in code paths

Fail gates:

- validation fails if sensitivity artifacts are missing
- validation fails if reports still describe weights opaquely
- validation fails if key thresholds remain hard-coded in multiple logic sites

### 8. Reporting overhaul

Files:

- `src/report.rs`
- `src/pipeline.rs`
- `src/plots.rs`

Work:

- regenerate all decision-facing reports from the richer metrics
- add explicit sections for:
  - what is proven
  - what is not proven
  - remaining blockers
  - ROI disclosure
  - trust operating mode
  - motion relevance
  - neighborhood clamp mechanics
  - non-ROI penalties
  - timing path honesty

Acceptance:

- every required report exists
- every decision-facing report contains `What Is Not Proven`
- every decision-facing report contains `Remaining Blockers`

Fail gates:

- validation fails if any required report is missing
- validation fails if unsupported claims appear

### 9. Hard validation gates and CLI usability

Files:

- `src/cli.rs`
- `src/pipeline.rs`
- `tests/demo_cli.rs`
- `tests/research_artifact.rs`
- `tests/colab_notebook.rs`
- `README.md`
- `docs/validation_contract.md`

Work:

- add commands for:
  - `run-all`
  - `run-scenario`
  - `run-ablations`
  - `run-timing`
  - `run-resolution-scaling`
  - `run-sensitivity`
  - `run-demo-b`
  - `run-demo-b-efficiency`
  - `validate`
  - `export-minimal-report`
- make validation enforce artifact completeness, honesty sections, timing classification, ROI disclosure, and budget equality

Acceptance:

- one command generates the complete artifact bundle
- one command validates it
- single-purpose study commands also work

Fail gates:

- validation fails if required artifacts, figures, JSON files, or disclosures are missing
- tests fail if CLI commands regress

## Required Experiments and Outputs

### New or expanded scenario studies

- existing point-like scenarios retained and explicitly labeled:
  - `thin_reveal`
  - `diagonal_reveal`
- existing benefit region scenario retained:
  - `fast_pan`
- new region-ROI scenarios to add:
  - multi-segment reveal band
  - mixed-width reveal band
  - imperfect-motion / reprojection-bias reveal
- existing negative controls retained:
  - `contrast_pulse`
  - `stability_holdout`

### Generated outputs that must exist after `run-all`

- `generated/report.md`
- `generated/reviewer_summary.md`
- `generated/five_mentor_audit.md`
- `generated/check_signing_blockers.md`
- `generated/ablation_report.md`
- `generated/trust_diagnostics.md`
- `generated/trust_diagnostics.json`
- `generated/timing_report.md`
- `generated/timing_metrics.json`
- `generated/resolution_scaling_report.md`
- `generated/resolution_scaling_metrics.json`
- `generated/parameter_sensitivity_report.md`
- `generated/parameter_sensitivity_metrics.json`
- `generated/demo_b_decision_report.md`
- `generated/demo_b_efficiency_report.md`
- updated `generated/demo_b/metrics.json`

### Figures that must be emitted from code

- trust histogram
- trust calibration plot
- ROI taxonomy / ROI size figure
- timing breakdown figure
- resolution scaling figure
- parameter sensitivity figure
- motion relevance figure
- Demo B efficiency figure

## Execution Order

1. Write this plan and the validation contract.
2. Centralize parameters and expand scenario metadata.
3. Add region-ROI and imperfect-motion scenarios.
4. Implement timing pipeline and GPU-path documentation.
5. Add trust diagnostics and motion-resolution logic.
6. Strengthen Demo B against aliasing-only interpretation.
7. Add sensitivity and resolution studies.
8. Rewrite reports and figures from generated metrics.
9. Add hard validation gates and final CLI/README usability updates.
