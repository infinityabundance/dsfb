# REPRODUCIBILITY

Reproducibility is tied to the run directory produced by `run-unreal-native`.

Current canonical status:

- [`../CURRENT_STATUS.md`](/home/one/dsfb/crates/dsfb-computer-graphics/CURRENT_STATUS.md)

Each run writes:

- `run_manifest.json`
- `provenance.json`
- `summary.json`
- `metrics.csv`
- `metrics_summary.json`
- `canonical_metric_sheet.md`
- `aggregation_summary.md`
- `materialized_unreal_external_manifest.json`
- `figures/trust_histogram.svg`
- `figures/trust_vs_error.svg`
- `figures/trust_conditioned_error_map.png`
- `per_frame/<label>/roi_mask.json`

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
WGPU_BACKEND=vulkan cargo run --release -- run-unreal-native \
  --manifest examples/unreal_native_capture_manifest.json \
  --output generated/unreal_native_runs
```

The run is deterministic with respect to the imported buffers and the current crate implementation. If the Unreal export changes, the evidence should be regenerated rather than hand-edited.

The checked-in reference run produced from the current sample contract is:

- [`generated/canonical_2026_q1/sample_capture_contract_sequence_canonical`](/home/one/dsfb/crates/dsfb-computer-graphics/generated/canonical_2026_q1/sample_capture_contract_sequence_canonical)

Important current limit:

- The current canonical sample uses exported `reference_color` on all 5 real captures, but that reference remains a higher-resolution Unreal export proxy rather than a path-traced or high-spp ground truth.
- The current GPU and scaling numbers were generated with `WGPU_BACKEND=vulkan`; they measure the imported-buffer compute path and do not replace in-engine profiling.
