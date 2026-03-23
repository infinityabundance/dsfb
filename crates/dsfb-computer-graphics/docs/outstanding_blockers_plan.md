# Outstanding Blockers Plan

This plan targets the remaining decision blockers after the earlier diligence pass. The goal is not cosmetic improvement. The goal is to remove or narrow the exact blockers that still prevent serious external evaluation.

## Blockers

1. Actual GPU execution measurements are still missing.
2. The suite is still synthetic and not yet external-buffer capable.
3. There is still no concrete engine handoff/import bridge.
4. Strong heuristic baselines remain competitive on some scenarios and need decision-grade framing.
5. Cost accounting is still architectural and CPU-side.
6. Point-like ROI cases must remain explicitly separated from region ROI claims.
7. Trust still behaves mostly like a gate and must stay framed that way.
8. The realism bridge is still too synthetic to look engine-adjacent.
9. Demo B still needs a harder aliasing-vs-variance case split.
10. There is still no serious external evaluator handoff bundle.

## Files To Change

- `README.md`
- `Cargo.toml`
- `src/cli.rs`
- `src/lib.rs`
- `src/main.rs`
- `src/pipeline.rs`
- `src/report.rs`
- `src/metrics.rs`
- `src/sampling.rs`
- `src/scene/mod.rs`
- `src/timing.rs`
- `src/host.rs`
- `tests/demo_cli.rs`
- `tests/research_artifact.rs`
- `docs/validation_contract.md`
- `docs/cost_model.md`
- `docs/integration_surface.md`

## Files To Add

- `docs/completion_gates.md`
- `docs/gpu_execution_path.md`
- `docs/external_handoff.md`
- `docs/engine_integration_playbook.md`
- `docs/production_eval_bridge.md`
- `examples/host_buffer_schema.json`
- `examples/external_capture_manifest.json`
- `src/external.rs`
- `src/gpu.rs`
- `src/gpu_execution.rs`

## Experiments, Reports, And Commands

### GPU bridge

- Add a wgpu compute path for the minimum host-realistic supervisory kernel.
- Add a CLI command:
  - `cargo run --release -- run-gpu-path --output generated/gpu_path`
- Generate:
  - `generated/gpu_execution_report.md`
  - `generated/gpu_execution_metrics.json`
- If a GPU adapter is not available, the command must still emit a report that says the GPU path is implemented but was not measured in the current environment.

### External handoff

- Add external buffer schema, file-based import, and a synthetic-to-external export path.
- Add a CLI command:
  - `cargo run --release -- import-external --manifest examples/external_capture_manifest.json --output generated/external_demo`
- Generate:
  - `generated/external_handoff_report.md`

### Realism and taxonomy expansion

- Add at least:
  - 2 new benefit-expected region-ROI scenarios
  - 1 realism-stress scenario
  - 1 competitive-baseline scenario
  - 1 bounded-neutral or bounded-loss scenario
- Generate:
  - `generated/realism_suite_report.md`
  - `generated/scenario_taxonomy.json`

### Competitive baseline and non-ROI analysis

- Generate:
  - `generated/competitive_baseline_analysis.md`
  - `generated/non_roi_penalty_report.md`

### Demo B deconfounding

- Add wider-feature, mixed-feature, and variance-dominated Demo B scene coverage.
- Generate:
  - `generated/demo_b_scene_taxonomy.json`
  - `generated/demo_b_aliasing_vs_variance_report.md`
  - updated `generated/demo_b_decision_report.md`
  - updated `generated/demo_b_efficiency_report.md`

### Evaluator handoff

- Generate:
  - `generated/production_eval_checklist.md`
  - `generated/evaluator_handoff.md`
  - `generated/minimum_external_validation_plan.md`
  - `generated/next_step_matrix.md`

## Success Criteria

- A GPU-executable path exists in the crate and is runnable on a GPU host.
- The crate can import external buffers through a stable manifest/schema without re-architecting the evaluation code.
- The scenario suite explicitly contains realism-stress, competitive-baseline, and bounded-neutral or bounded-loss cases.
- Demo B explicitly separates aliasing-limited and variance-limited evidence.
- Decision-facing reports continue to surface strong-heuristic wins, ties, non-ROI penalties, and remaining blockers.
- The evaluator handoff package tells an external team exactly what to run next.

## Validation Gates

- Fail if `docs/gpu_execution_path.md` or generated GPU execution artifacts are missing.
- Fail if the GPU report does not explicitly state measured vs unmeasured GPU status.
- Fail if the crate still has only CPU timing artifacts and no GPU-executable path.
- Fail if external schema, manifest example, import CLI, or external handoff report is missing.
- Fail if realism-stress, competitive-baseline, or bounded-neutral/bounded-loss taxonomy entries are missing.
- Fail if point ROI and region ROI evidence are aggregated without disclosure.
- Fail if Demo B lacks aliasing-vs-variance reporting or fixed-budget equality.
- Fail if strong-heuristic wins/ties and non-ROI penalties are not surfaced.
- Fail if evaluator handoff outputs or production evaluation bridge outputs are missing.
- Fail if any decision-facing report omits `What Is Not Proven`, `Remaining Blockers`, or external validation needs.
