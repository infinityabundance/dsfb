# Final Completion Gates

This crate is only in its final diligence state when both commands pass:

```bash
cargo run --release -- run-all --output generated/final_bundle
cargo run --release -- validate-final --output generated/final_bundle
```

## GPU Path

- `docs/gpu_execution_path.md` must exist.
- `generated/gpu_execution_report.md` and `generated/gpu_execution_metrics.json` must exist.
- The GPU report must explicitly state whether actual GPU timing was measured in the current environment.
- A GPU-runnable path must exist in-repo even if the current environment cannot measure it.

## External Replay

- `docs/external_replay.md` and `docs/engine_integration_playbook.md` must exist.
- `examples/external_buffer_schema.json` and `examples/external_capture_manifest.json` must exist.
- The CLI must support `run-external-replay` and `replay-external` aliases.
- `generated/external_replay_report.md` must exist and must distinguish external-capable from externally validated.
- `generated/external_real/external_validation_report.md`, `generated/external_real/gpu_external_report.md`, `generated/external_real/demo_a_external_report.md`, and `generated/external_real/demo_b_external_report.md` must exist.
- `generated/external_real/scaling_report.md`, `generated/external_real/scaling_metrics.json`, `generated/external_real/memory_bandwidth_report.md`, and `generated/external_real/integration_scaling_report.md` must exist.
- The external scaling package must explicitly attempt 1080p and state whether 4K was measured or unavailable.
- The external reports must explicitly state whether readback is required in production and whether async-compute insertion is plausible.
- The external validation package must explicitly label coverage for `realism_stress_case`, `larger_roi_case`, and `mixed_regime_case`, with partial coverage called out when any class is missing.

## Realism Bridge

- `generated/realism_bridge_report.md` and `generated/scenario_taxonomy.json` must exist.
- Taxonomy must explicitly expose point ROI, region ROI, realism stress, strong heuristic competition, and bounded-neutral or bounded-loss cases.
- Region-ROI evidence must remain visible in headline evidence instead of collapsing into point-ROI aggregates.

## Trust Honesty

- `generated/trust_mode_report.md` must exist.
- Trust operating mode must be classified explicitly.
- Degenerate trust-rank correlation must not be used as primary calibration evidence.

## Motion Resolution

- Motion disagreement must either be justified on a probe scenario or remain demoted from the minimum path.
- `docs/integration_surface.md` and `docs/cost_model.md` must agree with the default path.

## Demo B

- `generated/demo_b_competitive_baselines_report.md`, `generated/demo_b_aliasing_vs_variance_report.md`, and `generated/demo_b_efficiency_report.md` must exist.
- Fixed-budget equality must hold across policies.
- Headline reporting must include at least one non-thin-feature case.

## Product Positioning And Robustness

- `generated/competitive_baseline_analysis.md`, `generated/non_roi_penalty_report.md`, `generated/product_positioning_report.md`, and `generated/operating_band_report.md` must exist.
- The bundle must explicitly surface strong-heuristic wins or ties and quantify non-ROI penalties.
- Operating bands must classify robust, moderately sensitive, and fragile regions.

## Evaluator Handoff

- `docs/evaluator_handoff.md`, `generated/evaluator_handoff.md`, `generated/minimum_external_validation_plan.md`, `generated/next_step_matrix.md`, and `generated/check_signing_readiness.md` must exist.
- The handoff must identify one highest-value next GPU experiment and one highest-value next external replay experiment.
- The handoff must distinguish what is proven in-crate from what still requires external evidence.
