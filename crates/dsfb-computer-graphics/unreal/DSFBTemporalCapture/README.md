# DSFB Temporal Capture Unreal Project

This is the crate-local Unreal project scaffold for producing a strict `unreal_native` capture bundle for [`run-unreal-native`](/home/one/dsfb/crates/dsfb-computer-graphics/src/cli.rs).

What this project is for:

- create a minimal Unreal scene inside the crate
- export real Unreal current and previous frame images plus scene metadata
- materialize the strict replay buffers expected by the Rust replay path
- keep the Unreal-side setup next to the crate instead of treating it as out-of-band tribal knowledge

Expected commands from the repo root:

```bash
/home/one/Unreal/UE_5.7.2/Engine/Binaries/Linux/UnrealEditor \
  crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/DSFBTemporalCapture.uproject \
  -ExecutePythonScript=crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/Scripts/export_unreal_native_capture.py \
  -stdout -FullStdOutLogOutput

python3 crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture/Scripts/build_unreal_native_dataset.py
```

Notes:

- `export_unreal_native_capture.py` writes the raw Unreal PNG exports and `scene_state.json`.
- `build_unreal_native_dataset.py` converts those real Unreal exports into the strict JSON replay buffers consumed by `run-unreal-native`.
- The scripts are crate-local export helpers, not a renderer plugin replacement.
- The capture contract expects real Unreal outputs under [`data/unreal_native`](/home/one/dsfb/crates/dsfb-computer-graphics/data/unreal_native).
- For the checked-in minimal sample, motion vectors and replay normals are metadata-derived because that editor-side Linux path did not expose a numerically stable dense velocity export or unit-normal field.
- If Unreal-side buffer dumping changes across engine versions, update the script and re-export; do not relabel old or synthetic data as `unreal_native`.
