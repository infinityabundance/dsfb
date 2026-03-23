# Evaluator Handoff

This document is the static operator-facing counterpart to the generated evaluator bundle.

## First Commands

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-all --output generated/final_bundle
cargo run --release -- validate-final --output generated/final_bundle
cargo run --release -- run-gpu-path --output generated/gpu_path
cargo run --release -- run-external-replay --manifest examples/external_capture_manifest.json --output generated/external_replay
```

## What To Inspect First

- `generated/final_bundle/report.md`
- `generated/final_bundle/gpu_execution_report.md`
- `generated/final_bundle/external_replay_report.md`
- `generated/final_bundle/realism_bridge_report.md`
- `generated/final_bundle/demo_b_decision_report.md`
- `generated/final_bundle/check_signing_readiness.md`

## Highest-Value Next External Experiments

- GPU: run the minimum host-realistic kernel on the evaluator’s target GPU and compare numeric deltas against the CPU path.
- External replay: export one real engine frame pair into the schema and compare DSFB host-realistic against fixed alpha and the strong heuristic baseline.

## What Still Requires External Evidence

- real renderer buffer exports
- imported-capture GPU profiling
- broader image-content realism
- fair in-engine baseline comparison
