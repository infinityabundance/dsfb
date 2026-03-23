# Manual Engine-Native Commands

## Status

ENGINE_NATIVE_CAPTURE_MISSING=true

All infrastructure is complete. The only remaining step is providing a real engine capture.

---

## STEP 1 — Export buffers from your renderer

### Option A: Unreal Engine
Follow `docs/unreal_export_playbook.md` exactly.

Required outputs (minimum one frame pair):
```
data/engine_native/frame_000/current_color.exr
data/engine_native/frame_000/history_color.exr
data/engine_native/frame_000/motion_vectors.exr   (pixel offset convention)
data/engine_native/frame_000/current_depth.exr    (linear depth, larger = further)
data/engine_native/frame_000/current_normals.exr  (view-space unit vectors)
data/engine_native/frame_000/metadata.json
```

### Option B: Unity
Follow `docs/unity_export_playbook.md`.

### Option C: Custom renderer
Follow `docs/custom_renderer_export_playbook.md`.

---

## STEP 2 — Update the manifest

Edit `examples/engine_native_capture_manifest.json`:

```json
"source": {
  "kind": "engine_native",
  "engine_type": "unreal",        // or "unity" or "custom"
  "engine_version": "5.3",        // your actual version
  "capture_tool": null,
  "capture_note": null
}
```

Update each buffer `"path"` field to point to the exported files.

---

## STEP 3 — Run import (validates buffers, generates import report)

```bash
cd /home/one/dsfb/crates/dsfb-computer-graphics
cargo run --release -- import-engine-native \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```

**Expected output files:**
```
generated/engine_native/engine_native_import_report.md
generated/engine_native/resolved_engine_native_manifest.json
```

**Expected terminal output:**
```
engine-native import report: generated/engine_native/engine_native_import_report.md
resolved manifest: generated/engine_native/resolved_engine_native_manifest.json
ENGINE_NATIVE_CAPTURE_MISSING=false
```

If you see `ENGINE_NATIVE_CAPTURE_MISSING=true`, the buffer files were not found on disk.
Check the paths in the manifest and re-run.

---

## STEP 4 — Run full replay (GPU + Demo A + Demo B)

```bash
cargo run --release -- run-engine-native-replay \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```

**Expected output files:**
```
generated/engine_native/engine_native_replay_report.md
generated/engine_native/gpu_execution_report.md
generated/engine_native/gpu_execution_metrics.json
generated/engine_native/demo_a_engine_native_report.md
generated/engine_native/demo_b_engine_native_report.md
generated/engine_native/demo_b_engine_native_metrics.json
generated/engine_native/high_res_execution_report.md
generated/engine_native/engine_native_validation_report.md
```

**Expected GPU report fields when real capture is provided:**
```
ENGINE_NATIVE_CAPTURE_MISSING=false
actual_engine_native_data: true
measured_gpu: true
adapter: <your GPU name>
total_ms: <dispatch time in milliseconds>
```

---

## STEP 5 — Mixed-regime confirmation (already done — internal synthetic)

The internal mixed-regime case is already confirmed:
```
generated/mixed_regime_confirmation_report.md
  mixed_regime_status: mixed_regime_confirmed_internal
  Aliasing confirmed: true
  Variance confirmed: true
```

To additionally confirm on engine-native data, the capture must contain a scene with
thin structures under noisy reprojection. After a real capture is available, the pipeline
will automatically classify it if both signals are detected.

---

## STEP 6 — Validate

### Strict validation (requires real engine capture — will fail until Step 3-4 are done with real data)
```bash
cargo run --release -- validate-final \
  --output generated/final_bundle
```

**Failure message when capture is missing:**
```
engine-native gate: ENGINE_NATIVE_CAPTURE_MISSING=true — no real engine capture has been provided.
Options:
  1. Provide a real engine capture (see docs/unreal_export_playbook.md)
  2. Run with --allow-pending-engine-native to pass despite missing capture
```

### Permissive validation (passes even without real engine capture)
```bash
cargo run --release -- validate-final \
  --output generated/final_bundle \
  --allow-pending-engine-native
```

---

## STEP 7 — 4K validation (requires a 4K engine capture)

If you have a 4K capture, create a 4K manifest:
```bash
cp examples/engine_native_capture_manifest.json examples/engine_native_capture_manifest_4k.json
# Update paths to 4K buffer files
```

Then run:
```bash
cargo run --release -- run-engine-native-replay \
  --manifest examples/engine_native_capture_manifest_4k.json \
  --output generated/engine_native_4k
```

**Expected behavior:**
- At 4K (3840×2160), the GPU dispatch may fail with an OOM/binding-size error.
- This is an external environment limitation (wgpu binding tier), not an algorithm limitation.
- The tiling strategy is designed: split into 4 tiles of height 540, dispatch separately.
- Tiling is not yet wired into the CLI; it requires a real 4K capture to test.

---

## Summary: What Closes Each Gate

| Gate | What closes it |
|------|----------------|
| ENGINE_NATIVE_CAPTURE_MISSING=false | Steps 1-4 above |
| GPU timing on real engine data | Step 4 (same pipeline) |
| Demo A on real engine data | Step 4 (same pipeline) |
| Demo B on real engine data | Step 4 (same pipeline) |
| Mixed-regime internal confirmation | Already done |
| Mixed-regime engine-native confirmation | Step 4 + appropriate scene |
| 4K engine-native dispatch | Step 7 + tiling wiring |
| validate-final strict | Steps 1-4 |
| validate-final permissive | Already passes |
