# REPRODUCIBILITY

Reproducibility is tied to the run directory produced by `run-unreal-native`.

Each run writes:

- `run_manifest.json`
- `provenance.json`
- `summary.json`
- `metrics.csv`
- `metrics_summary.json`
- `materialized_unreal_external_manifest.json`

`provenance.json` records:

- dataset id
- schema version
- manifest path
- materialized manifest path
- run directory
- git commit
- CLI arguments
- epoch timestamp

Reproduction command:

```bash
/home/one/Unreal/UE_5.7.2/Engine/Binaries/Linux/UnrealEditor \
  crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/DSFBTemporalCapture.uproject \
  -ExecutePythonScript=crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/Scripts/export_unreal_native_capture.py \
  -stdout -FullStdOutLogOutput

python3 crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/Scripts/build_unreal_native_dataset.py

cd crates/dsfb-computer-graphics
cargo run --release -- run-unreal-native \
  --manifest examples/unreal_native_capture_manifest.json \
  --output generated/unreal_native_runs
```

The run is deterministic with respect to the imported buffers and the current crate implementation. If the Unreal export changes, the evidence should be regenerated rather than hand-edited.

The checked-in reference run produced from the current sample contract is:

- [`generated/unreal_native_runs/sample_capture_contract`](/home/one/dsfb/crates/dsfb-computer-graphics/generated/unreal_native_runs/sample_capture_contract)
