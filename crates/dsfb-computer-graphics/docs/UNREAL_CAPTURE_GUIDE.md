# UNREAL_CAPTURE_GUIDE

Purpose:

- produce a real Unreal-exported frame pair for `run-unreal-native`
- keep the export expectations explicit
- avoid fake completeness around internal TAA history access

## Project Location

- [`unreal/DSFBTemporalCapture`](/home/one/dsfb/crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture)

## Baseline Commands

From the repo root:

```bash
/home/one/Unreal/UE_5.7.2/Engine/Binaries/Linux/UnrealEditor \
  crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/DSFBTemporalCapture.uproject \
  -ExecutePythonScript=crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/Scripts/export_unreal_native_capture.py \
  -stdout -FullStdOutLogOutput

python3 crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/Scripts/build_unreal_native_dataset.py
```

The crate-local scripts now perform two explicit stages:

1. `export_unreal_native_capture.py`
   - builds a minimal scene scaffold
   - captures frame 0 and frame 1 raw Unreal PNG exports
   - writes `scene_state.json` and the raw images into `data/unreal_native/sample_capture/frame_0001/raw`
2. `build_unreal_native_dataset.py`
   - linearizes the raw color exports into `json_rgb_f32`
   - decodes the raw depth visualization into `json_scalar_f32`
   - derives motion vectors, replay normals, ROI mask, and disocclusion mask from the recorded Unreal metadata
   - writes the strict replay files under `data/unreal_native/sample_capture/frame_0001`

## Required Unreal Outputs

For each capture frame pair:

- current frame color export
- previous frame color export
- current depth export
- previous depth export
- current normals export
- previous normals export
- metadata JSON

Recommended:

- raw scene-state metadata sufficient to derive motion, ROI, and disocclusion for the minimal sample
- a direct host output if the engine-side path exposes one cleanly

## Export Expectations

- The checked-in minimal sample uses SceneCapture final-color PNG exports, then linearizes them into replay color buffers. That is why the manifest uses `tonemap = "scene_capture_png_linearized"` instead of claiming a pre-tonemap float export.
- Depth must be labeled honestly. The checked-in sample uses a real Unreal depth visualization export and labels it `monotonic_visualized_depth`.
- Normals must be labeled honestly. `view_space_unit` and `world_space_unit` are both supported. The checked-in sample labels `world_space_unit`.
- Motion vectors must be labeled honestly. This crate accepts `pixel_offset_to_prev` or `ndc_to_prev` and normalizes the latter into pixel offsets before replay. The checked-in sample uses metadata-derived `pixel_offset_to_prev`.

If Unreal exports motion in NDC, set `motion_vector_convention = "ndc_to_prev"` in the manifest and the crate will normalize it into pixel offsets.

## Known Reality

- Unreal does not always expose the internal TAA history buffer directly.
- Unreal editor-side Linux capture did not expose a stable dense float velocity export for this minimal sample.
- Unreal WorldNormal PNG export is retained as provenance in `raw/`, but the checked-in replay normal field is metadata-derived rather than overclaiming that visualization as a numerically stable unit-normal buffer.
- This crate therefore supports previous-frame exports plus motion-vector reprojection as a first-class Unreal-native path.
- That is still real engine-native input because the buffers and metadata came from Unreal. It is not presented as synthetic equivalence.

## After Export

Update or replace:

- [`examples/unreal_native_capture_manifest.json`](/home/one/dsfb/crates/dsfb-computer-graphics/examples/unreal_native_capture_manifest.json)

Then run:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-unreal-native \
  --manifest examples/unreal_native_capture_manifest.json \
  --output generated/unreal_native_runs
```

The checked-in sample can be replayed immediately without rerunning Unreal. Its existing evidence bundle lives under:

- [`generated/unreal_native_runs/sample_capture_contract`](/home/one/dsfb/crates/dsfb-computer-graphics/generated/unreal_native_runs/sample_capture_contract)

This command must fail if:

- any required file is missing
- any provenance field is not `unreal_native`
- any metadata field contradicts the manifest

That failure behavior is deliberate.
