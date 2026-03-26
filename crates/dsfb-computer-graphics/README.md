# dsfb-computer-graphics

`dsfb-computer-graphics` is a Rust crate for evaluating DSFB as a deterministic supervisory layer over temporal graphics pipelines. The canonical proof path is now the strict Unreal-native replay path: real Unreal-exported frame buffers and metadata are ingested, validated, replayed through the existing DSFB temporal supervision core, and packaged into a decision-grade evidence bundle.

DSFB in this crate is not a renderer replacement. The posture is narrower and more useful:

- trust estimation over temporal reuse inputs
- admissibility / regime gating
- intervention and alpha signals for downstream resolve logic
- replayable evidence, provenance, and audit artifacts

The intended insertion point is temporal anti-aliasing / temporal reuse supervision, with a sober extension path toward adaptive sampling, simulation integrity monitoring, and certification-style replay.

## Canonical Path

If you want to regenerate the crate-local Unreal sample from the installed editor:

```bash
/home/one/Unreal/UE_5.7.2/Engine/Binaries/Linux/UnrealEditor \
  crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/DSFBTemporalCapture.uproject \
  -ExecutePythonScript=crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/Scripts/export_unreal_native_capture.py \
  -stdout -FullStdOutLogOutput

python3 crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/Scripts/build_unreal_native_dataset.py
```

Use the strict Unreal-native command when the input really came from Unreal Engine:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-unreal-native \
  --manifest examples/unreal_native_capture_manifest.json \
  --output generated/unreal_native_runs
```

What this command does:

- validates a strict `dsfb_unreal_native_v1` manifest
- refuses synthetic, pending, proxy-labeled, or mis-provenanced input
- materializes the Unreal capture into the crate’s stable replay contract
- runs the DSFB replay bundle on the imported capture
- writes a timestamped Unreal-native run directory
- generates `summary.json`, `metrics.csv`, `metrics_summary.json`, `comparison_summary.md`, `failure_modes.md`, `provenance.json`, per-frame maps, a boardroom panel, an executive evidence sheet, a PDF bundle, and a ZIP bundle

A checked-in evidence run for the canonical sample currently lives under:

- [`generated/unreal_native_runs/sample_capture_contract`](/home/one/dsfb/crates/dsfb-computer-graphics/generated/unreal_native_runs/sample_capture_contract)

That sample currently lands as a `heuristic_favorable` Demo A case, which is kept on purpose as an honesty check rather than filtered out.

## What Counts As Unreal-Native

The strict path accepts only manifests labeled:

- `schema_version = "dsfb_unreal_native_v1"`
- `dataset_kind = "unreal_native"`
- `provenance_label = "unreal_native"`
- `engine.engine_name = "unreal_engine"`
- `engine.real_engine_capture = true`

Per capture, the contract requires:

- `current_color`
- `previous_color`
- `motion_vectors`
- `current_depth`
- `previous_depth`
- `current_normals`
- `previous_normals`
- `metadata`

Optional but strongly recommended:

- `host_output`
- `history_color`, `history_depth`, `history_normals` if the engine exposes them directly
- `roi_mask`
- `disocclusion_mask`
- `reference_color`

The strict path does not silently synthesize missing required buffers. If a required file is absent or malformed, the run fails.

The crate-local sample currently retains the following raw Unreal exports under [`data/unreal_native/sample_capture/frame_0001/raw`](/home/one/dsfb/crates/dsfb-computer-graphics/data/unreal_native/sample_capture/frame_0001/raw):

- final-color SceneCapture PNGs for `current_color` and `previous_color`
- `SceneDepth` visualization PNGs for `current_depth` and `previous_depth`
- `WorldNormal` visualization PNGs for `current_normals` and `previous_normals`

The checked-in replay dataset materializes from those raw exports and the recorded Unreal camera/object metadata:

- `current_color.json` and `previous_color.json` are linearized from the raw color PNGs
- `current_depth.json` and `previous_depth.json` are decoded from the raw depth visualization PNGs and labeled `monotonic_visualized_depth`
- `current_normals.json` and `previous_normals.json` are metadata-derived unit normals for this minimal sample
- `motion_vectors.json` is a metadata-derived dense pixel-offset field for this minimal sample

That means the checked-in sample manifest labels:

- `normal_space = "world_space_unit"`
- `depth_convention = "monotonic_visualized_depth"`
- `motion_vector_convention = "pixel_offset_to_prev"`

## Dataset Contract

Canonical manifest:

- [`examples/unreal_native_capture_manifest.json`](/home/one/dsfb/crates/dsfb-computer-graphics/examples/unreal_native_capture_manifest.json)

Canonical data root:

- [`data/unreal_native`](/home/one/dsfb/crates/dsfb-computer-graphics/data/unreal_native)

Canonical schema and guide:

- [`docs/DATASET_SCHEMA.md`](/home/one/dsfb/crates/dsfb-computer-graphics/docs/DATASET_SCHEMA.md)
- [`docs/UNREAL_CAPTURE_GUIDE.md`](/home/one/dsfb/crates/dsfb-computer-graphics/docs/UNREAL_CAPTURE_GUIDE.md)

The manifest supports either:

- direct history buffers exported from Unreal, or
- previous-frame exports plus motion-vector reprojection performed in the crate

That second case is still engine-native because the inputs are real Unreal exports. It is explicitly labeled as such and is not presented as synthetic equivalence.

## Unreal Project Scaffold

The crate-local Unreal scaffold lives under:

- [`unreal/DSFBTemporalCapture`](/home/one/dsfb/crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture)

Key files:

- [`unreal/DSFBTemporalCapture/DSFBTemporalCapture.uproject`](/home/one/dsfb/crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/DSFBTemporalCapture.uproject)
- [`unreal/DSFBTemporalCapture/Scripts/export_unreal_native_capture.py`](/home/one/dsfb/crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/Scripts/export_unreal_native_capture.py)
- [`unreal/DSFBTemporalCapture/Scripts/build_unreal_native_dataset.py`](/home/one/dsfb/crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/Scripts/build_unreal_native_dataset.py)
- [`unreal/DSFBTemporalCapture/README.md`](/home/one/dsfb/crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/README.md)

This project assumes Unreal Engine is already installed on the machine. The crate stores only the project/export files, not the engine itself.

## Output Bundle

Each `run-unreal-native` execution writes a dedicated run directory under the chosen output root. The bundle includes:

- `summary.json`
- `metrics.csv`
- `metrics_summary.json`
- `comparison_summary.md`
- `failure_modes.md`
- `provenance.json`
- `run_manifest.json`
- `materialized_unreal_external_manifest.json`
- `gpu_execution_report.md`
- `demo_a_external_report.md`
- `demo_b_external_report.md`
- `external_validation_report.md`
- `per_frame/<label>/trust_map.png`
- `per_frame/<label>/alpha_map.png`
- `per_frame/<label>/intervention_map.png`
- `per_frame/<label>/residual_map.png`
- `per_frame/<label>/instability_overlay.png`
- `per_frame/<label>/boardroom_panel_<label>.png`
- `executive_evidence_sheet.png`
- `artifacts_bundle.pdf`
- `artifacts_bundle.zip`
- `notebook_manifest.json`

The PDF and ZIP are generated automatically by the crate-local bundle builder:

- [`colab/build_unreal_native_bundle.py`](/home/one/dsfb/crates/dsfb-computer-graphics/colab/build_unreal_native_bundle.py)

## Notebook

The Unreal-native Colab / notebook entry point is:

- [`colab/dsfb_unreal_native_evidence.ipynb`](/home/one/dsfb/crates/dsfb-computer-graphics/colab/dsfb_unreal_native_evidence.ipynb)

It is designed to:

- explain what real Unreal-native input means
- refuse synthetic relabeling
- run the strict Unreal-native CLI
- display the executive sheet and primary panel inline
- expose PDF and ZIP downloads

## Secondary Paths

The crate still contains synthetic and generic external replay workflows. They remain useful, but they are secondary support only:

- `run-all`, `run-demo-a`, `run-demo-b`, and the internal realism bridge are synthetic or semi-synthetic
- `run-external-replay` is a generic file-based replay path
- `run-unreal-native` is the canonical Unreal proof path

Those paths are not equivalent and are not labeled as equivalent.

## Commercial Framing

The credible claim from this crate is bounded:

- DSFB can be inserted as a supervisory layer over a temporal reuse path
- the Unreal-native replay path produces evidence consistent with reduced temporal artifact risk in some regimes
- results depend on observability, exported buffers, and regime specification
- strong heuristics can remain competitive or win on some captures
- the checked-in Unreal sample is intentionally retained even though its Demo A classification is `heuristic_favorable`

The crate does not claim:

- universal outperformance
- solved rendering
- renderer replacement
- production readiness without engine-side integration proof

## Reproducibility Docs

- [`docs/EVIDENCE_WORKFLOW.md`](/home/one/dsfb/crates/dsfb-computer-graphics/docs/EVIDENCE_WORKFLOW.md)
- [`docs/FAILURE_MODES.md`](/home/one/dsfb/crates/dsfb-computer-graphics/docs/FAILURE_MODES.md)
- [`docs/REPRODUCIBILITY.md`](/home/one/dsfb/crates/dsfb-computer-graphics/docs/REPRODUCIBILITY.md)
