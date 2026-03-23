# Evaluator Handoff

## Standard External Datasets: DAVIS + Sintel

- prepare DAVIS: `cargo run --release -- prepare-davis --output data/external/davis`
- prepare Sintel: `cargo run --release -- prepare-sintel --output data/external/sintel`
- replay DAVIS: `cargo run --release -- run-external-replay --manifest examples/davis_external_manifest.json --output generated/external_davis`
- replay Sintel: `cargo run --release -- run-external-replay --manifest examples/sintel_external_manifest.json --output generated/external_sintel`
- validate everything: `cargo run --release -- validate-final --output generated`

Expected outputs:
- `external_davis/*` and `external_sintel/*` with replay, GPU, Demo A, Demo B, scaling, memory, and integration reports.
- `external_validation_taxonomy.json`.
- `external_validation_report.md`.
- `check_signing_readiness.md`.

Success looks like:
- both manifests load
- both dataset paths produce replay + GPU reports
- proxy-vs-native distinctions stay explicit
- fixed-budget Demo B remains equal across all policies

Failure looks like:
- dataset download blocked
- missing per-dataset report or manifest
- hidden derived buffers or missing disclosure

Interpretation rule:
- DAVIS and clean-vs-final Sintel comparisons may use proxies; read those as decision support, not renderer ground truth.
