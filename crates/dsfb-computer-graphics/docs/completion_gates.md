# Completion Gates

This crate is only considered complete when `cargo run --release -- validate --output <bundle>` passes.

## Timing And GPU Bridge

- `docs/gpu_execution_path.md` must exist.
- `generated/gpu_execution_report.md` and `generated/gpu_execution_metrics.json` must exist.
- The GPU execution report must explicitly say whether actual GPU timing was measured in the current environment.
- CPU proxy timing must not be presented as equivalent to measured GPU performance.

## External Handoff

- `docs/external_handoff.md`, `docs/engine_integration_playbook.md`, and `docs/production_eval_bridge.md` must exist.
- `examples/host_buffer_schema.json` and `examples/external_capture_manifest.json` must exist.
- The CLI must support `import-external`.
- The generated bundle must contain `external_handoff_report.md` and `external_demo/resolved_external_capture_manifest.json`.

## Scenario Taxonomy And Realism

- The realism-expanded suite must include point ROI, region ROI, realism-stress, competitive-baseline, and bounded-neutral or bounded-loss disclosures.
- `generated/scenario_taxonomy.json` and `generated/realism_suite_report.md` must exist.
- Point-like ROI and region-ROI evidence must remain separated in reporting and validation.

## Demo B

- `generated/demo_b_aliasing_vs_variance_report.md` and `generated/demo_b_scene_taxonomy.json` must exist.
- Fixed-budget equality must hold across policies.
- The scene taxonomy must include aliasing-limited, variance-limited, and mixed or edge-trap cases.

## Reporting Honesty

Every decision-facing report must include:

- what is proven
- what is not proven
- remaining blockers
- external validation needs

Unsupported language such as `production-ready`, `universal replacement`, or implied measured GPU performance without measurement is disallowed.

## Reproducibility Surface

The CLI must support:

- `run-all`
- `validate`
- `run-gpu-path`
- `import-external`
- `run-realism-suite`
- `run-demo-b-efficiency`
- `export-evaluator-handoff`
