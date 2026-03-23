# Evaluator Handoff

Run first:
- `cargo run --release -- run-all --output generated/final_bundle`
- `cargo run --release -- validate-final --output generated/final_bundle`
- `cargo run --release -- run-gpu-path --output generated/gpu_path`
- `cargo run --release -- run-external-replay --manifest examples/external_capture_manifest.json --output generated/external_demo`

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

## What Is Not Proven

- This handoff does not claim the current crate has already passed external evaluation.

## Remaining Blockers

- external engine captures
- GPU profiling on imported captures
