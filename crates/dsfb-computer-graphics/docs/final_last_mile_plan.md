# Final Last Mile Plan

This plan targets the remaining internal artifact blockers after the earlier diligence passes. The objective is to leave only external-validation blockers wherever the current environment cannot honestly provide more.

## Remaining Blockers

1. The crate needs an explicit measured-vs-unmeasured GPU execution contract tied to a real GPU-runnable kernel path.
2. The external handoff path needs to become an explicit external replay path with schema validation, replay naming, and evaluator-facing commands.
3. The realism suite needs stronger taxonomy signaling so region-ROI, realism-stress, competitive-baseline, and bounded-neutral evidence are impossible to miss.
4. Trust still behaves mostly as a gate, so the bundle needs an explicit trust-mode report instead of relying only on generic diagnostics.
5. Motion disagreement is optional and must stay demoted cleanly in architecture, docs, cost story, and validation.
6. Demo B needs an explicit competitive-baseline report so it cannot be read as uniform-vs-trust only.
7. Competitive strong heuristics need direct product-positioning analysis rather than just disclosure.
8. Parameter robustness needs an explicit operating-band report that tells an external evaluator what to tune first and what to leave alone.
9. The evaluator handoff surface needs exact next GPU and next external replay experiments, plus a check-signing readiness summary.
10. The final validator must fail if any of those last-mile artifacts or disclosures are missing.

## Files To Change

- `README.md`
- `src/cli.rs`
- `src/lib.rs`
- `src/external.rs`
- `src/pipeline.rs`
- `src/report.rs`
- `tests/demo_cli.rs`
- `tests/research_artifact.rs`
- `docs/engine_integration_playbook.md`

## Files To Add

- `docs/final_completion_gates.md`
- `docs/external_replay.md`
- `docs/evaluator_handoff.md`
- `examples/external_buffer_schema.json`

## Generated Artifacts To Add

- `generated/external_replay_report.md`
- `generated/realism_bridge_report.md`
- `generated/trust_mode_report.md`
- `generated/demo_b_competitive_baselines_report.md`
- `generated/product_positioning_report.md`
- `generated/operating_band_report.md`
- `generated/check_signing_readiness.md`

## Commands

- `cargo run --release -- run-all --output generated/final_bundle`
- `cargo run --release -- validate --output generated/final_bundle`
- `cargo run --release -- validate-final --output generated/final_bundle`
- `cargo run --release -- run-gpu-path --output generated/gpu_path`
- `cargo run --release -- run-external-replay --manifest examples/external_capture_manifest.json --output generated/external_replay`
- `cargo run --release -- run-realism-bridge --output generated/realism_bridge`
- `cargo run --release -- run-demo-b-efficiency --output generated/demo_b_efficiency`
- `cargo run --release -- export-evaluator-handoff --output generated/evaluator_handoff`

## Acceptance Criteria

- The repo contains a real GPU-executable kernel path and the report explicitly states whether GPU timing was measured in the current environment.
- The repo contains a real external replay path with schema examples, manifest examples, layout validation, and replay outputs.
- Region-ROI cases, realism-stress cases, competitive-baseline cases, and bounded-neutral cases are explicit in taxonomy output and remain separated from point-ROI headline evidence.
- Trust operating mode is reported directly as gate-like, weakly graded, or strongly graded, with region-ROI evidence included.
- Motion disagreement remains optional unless new evidence justifies more; minimum-path docs and cost language must agree.
- Demo B reports explicitly compare against gradient/contrast/variance-style heuristics and separate aliasing-limited from variance-limited evidence.
- Product-positioning reports explicitly surface strong-heuristic wins, ties, non-ROI penalties, and the correct DSFB framing.
- Operating-band reporting classifies parameters as robust, moderately sensitive, or fragile and identifies what an external evaluator should tune first.
- The evaluator handoff bundle identifies one highest-value next GPU experiment and one highest-value next external replay experiment.
- Validation fails if any of the required artifacts or disclosures are missing.

## Validation Gates

- Fail if `docs/gpu_execution_path.md`, `generated/gpu_execution_report.md`, or `generated/gpu_execution_metrics.json` are missing.
- Fail if GPU reports do not explicitly classify themselves as measured GPU or unmeasured GPU path.
- Fail if `docs/external_replay.md`, `examples/external_buffer_schema.json`, `examples/external_capture_manifest.json`, replay CLI aliases, or `generated/external_replay_report.md` are missing.
- Fail if imported buffers do not validate consistent dimensions and layouts.
- Fail if `generated/realism_bridge_report.md` or `generated/scenario_taxonomy.json` are missing, or if taxonomy omits realism-stress, strong-heuristic-competitive, and bounded-neutral coverage.
- Fail if `generated/trust_mode_report.md` is missing or if trust mode is not explicitly classified.
- Fail if the minimum path still implies motion disagreement is required.
- Fail if `generated/demo_b_competitive_baselines_report.md` or `generated/demo_b_aliasing_vs_variance_report.md` are missing, or if fixed-budget equality breaks.
- Fail if `generated/product_positioning_report.md` or `generated/operating_band_report.md` are missing.
- Fail if `generated/check_signing_readiness.md` is missing or does not classify readiness vs external blocking.
- Fail if decision-facing reports omit `What Is Not Proven`, `Remaining Blockers`, or external-validation needs.
