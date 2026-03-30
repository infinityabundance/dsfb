# EVIDENCE_WORKFLOW

## 1. Prepare Unreal Capture

Current canonical status:

- [`../CURRENT_STATUS.md`](/home/one/dsfb/crates/dsfb-computer-graphics/CURRENT_STATUS.md)

Use the crate-local Unreal project scaffold:

- [`unreal/DSFBTemporalCapture`](/home/one/dsfb/crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture)

Export the raw Unreal sample:

```bash
/home/one/Unreal/UE_5.7.2/Engine/Binaries/Linux/UnrealEditor \
  crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/DSFBTemporalCapture.uproject \
  -ExecutePythonScript=crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/Scripts/export_unreal_native_capture.py \
  -stdout -FullStdOutLogOutput
```

Materialize the strict replay dataset:

```bash
python3 crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/Scripts/build_unreal_native_dataset.py
```

The dataset is written into:

- [`data/unreal_native`](/home/one/dsfb/crates/dsfb-computer-graphics/data/unreal_native)

The canonical manifest is already wired to that sample:

- [`examples/unreal_native_capture_manifest.json`](/home/one/dsfb/crates/dsfb-computer-graphics/examples/unreal_native_capture_manifest.json)

If you only want to replay the checked-in sample, you can skip the two export steps above.

## 2. Run Strict Unreal-Native Replay

```bash
cd crates/dsfb-computer-graphics
WGPU_BACKEND=vulkan cargo run --release -- run-unreal-native \
  --manifest examples/unreal_native_capture_manifest.json \
  --output generated/unreal_native_runs
```

The command creates a dedicated run directory and produces:

- the materialized external replay manifest
- the replay and validation reports
- per-frame trust / alpha / intervention / residual maps
- comparison and failure-mode summaries
- executive sheet, PDF, ZIP, and notebook manifest

## 3. Review the Decision Artifacts

Primary files:

- `summary.json`
- `metrics_summary.json`
- `canonical_metric_sheet.md`
- `aggregation_summary.md`
- `comparison_summary.md`
- `failure_modes.md`
- `executive_evidence_sheet.png`
- `artifacts_bundle.pdf`
- `figures/trust_histogram.svg`
- `figures/trust_vs_error.svg`
- `figures/trust_temporal_trajectory.svg`

Current checked-in canonical run:

- [`generated/canonical_2026_q1/sample_capture_contract_sequence_canonical`](/home/one/dsfb/crates/dsfb-computer-graphics/generated/canonical_2026_q1/sample_capture_contract_sequence_canonical)

## 4. Use the Notebook If Needed

Notebook entry point:

- [`colab/dsfb_unreal_native_evidence.ipynb`](/home/one/dsfb/crates/dsfb-computer-graphics/colab/dsfb_unreal_native_evidence.ipynb)

It should be used to display an existing real Unreal-native run, not to relabel synthetic data.
