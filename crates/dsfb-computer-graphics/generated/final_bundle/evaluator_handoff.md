# Evaluator Handoff

Run first:
- `cargo run --release -- run-all --output generated/final_bundle`
- `cargo run --release -- validate-final --output generated/final_bundle`
- `cargo run --release -- run-gpu-path --output generated/gpu_path`
- `cargo run --release -- run-external-replay --manifest examples/external_capture_manifest.json --output generated/external_real`

Strongest current evidence:
- On the canonical scenario, host-realistic DSFB reduced cumulative ROI MAE from 2.84366 for fixed alpha to 0.34793.
- On the canonical sampling scenario, imported trust reduced ROI MAE from 0.17226 for uniform allocation to 0.03184 under the same total budget.

Weakest current evidence:
- no real external engine capture has been validated
- strong heuristic remains competitive on some scenarios

Single highest-value next GPU experiment:
- Run the measured `wgpu` minimum kernel on the target evaluator GPU and compare numeric deltas against the CPU reference on one region-ROI case and one realism-stress case.

Single highest-value next external replay experiment:
- Export one real frame pair from an engine into the external schema, replay it through DSFB host-realistic, and compare fixed alpha, strong heuristic, and DSFB on the same capture.

Engine-side baselines to keep:
- fixed alpha
- strong heuristic
- imported-trust or native-trust sampling policy where relevant

## External Validation

- exact command: `cargo run --release -- run-external-replay --manifest <manifest> --output generated/external_real`
- required data format: `current_color`, `reprojected_history`, `motion_vectors`, `current_depth`, `reprojected_depth`, `current_normals`, `reprojected_normals`, plus optional `mask`, `ground_truth`, and `variance`.
- expected outputs: `external_validation_report.md`, `gpu_external_report.md`, `gpu_external_metrics.json`, `demo_a_external_report.md`, `demo_b_external_report.md`, `demo_b_external_metrics.json`, `scaling_report.md`, `scaling_metrics.json`, `memory_bandwidth_report.md`, `integration_scaling_report.md`, and `figures/`.
- success looks like: the imported capture runs through the DSFB host-minimum path, GPU status is explicit, ROI vs non-ROI is separated, fixed-budget Demo B compares DSFB against stronger heuristics, and the scaling package says whether 1080p/4K, readback, and async insertion are viable.
- failure looks like: malformed schema, missing required buffers, no measured-vs-unmeasured GPU disclosure, no 1080p attempt or unavailable classification, budget mismatch, or reports that hide proxy-vs-real metric status.
- interpretation: ties against strong heuristics mean DSFB is behaving like a targeted supervisory overlay rather than a blanket replacement; losses plus higher non-ROI penalty should trigger engine-side tuning before any broader claim.

## What Is Not Proven

- This handoff does not claim the current crate has already passed external evaluation.

## Remaining Blockers

- external engine captures
- GPU profiling on imported captures
