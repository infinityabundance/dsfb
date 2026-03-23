# Evaluator Handoff

Run first:
- `cargo run --release -- run-all --output generated/final_bundle`
- `cargo run --release -- validate --output generated/final_bundle`
- `cargo run --release -- run-gpu-path --output generated/gpu_path`
- `cargo run --release -- import-external --manifest examples/external_capture_manifest.json --output generated/external_demo`

Strongest current evidence:
- On the canonical scenario, host-realistic DSFB reduced cumulative ROI MAE from 2.84366 for fixed alpha to 0.34793.
- On the canonical sampling scenario, imported trust reduced ROI MAE from 0.17226 for uniform allocation to 0.03184 under the same total budget.

Weakest current evidence:
- no real external engine capture has been validated
- strong heuristic remains competitive on some scenarios

Single highest-value next external experiment:
- Export one real frame pair from an engine into the external schema and run the GPU path on that imported capture.

## What Is Not Proven

- This handoff does not claim the current crate has already passed external evaluation.

## Remaining Blockers

- external engine captures
- GPU profiling on imported captures
