# Unreal-Native Data Layout

This directory is reserved for real Unreal Engine capture bundles consumed by `run-unreal-native`.

Expected layout:

```text
data/unreal_native/
  sample_capture/
    frame_0001/
      current_color.json
      previous_color.json
      motion_vectors.json
      current_depth.json
      previous_depth.json
      current_normals.json
      previous_normals.json
      roi_mask.json
      disocclusion_mask.json
      metadata.json
      scene_state.json
      capture_commands.txt
      raw/
        current_color.png
        previous_color.png
        current_depth.png
        previous_depth.png
        current_normals.png
        previous_normals.png
```

Notes:

- These files must come from Unreal Engine. Synthetic files must not be stored here and must not be mislabeled as `unreal_native`.
- The crate-local sample export uses real Unreal SceneCapture PNG exports for color, depth visualization, and normal visualization.
- The checked-in sample then materializes deterministic replay JSON buffers from those raw exports plus recorded Unreal camera/object metadata.
- `motion_vectors.json` and the replay normal fields are metadata-derived for this minimal sample and are labeled as such in the metadata notes and capture log.
- The canonical manifest is [`examples/unreal_native_capture_manifest.json`](/home/one/dsfb/crates/dsfb-computer-graphics/examples/unreal_native_capture_manifest.json).
- The crate-local Unreal project scaffold is under [`unreal/DSFBTemporalCapture`](/home/one/dsfb/crates/dsfb-computer-graphics/unreal/DSFBTemporalCapture).
