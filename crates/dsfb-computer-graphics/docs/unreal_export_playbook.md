# Unreal Export Playbook

This legacy filename now points to the strict Unreal-native docs:

- [`UNREAL_CAPTURE_GUIDE.md`](/home/one/dsfb/crates/dsfb-computer-graphics/docs/UNREAL_CAPTURE_GUIDE.md)
- [`DATASET_SCHEMA.md`](/home/one/dsfb/crates/dsfb-computer-graphics/docs/DATASET_SCHEMA.md)

The canonical command is:

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
