# Engine-Native Import Report

ENGINE_NATIVE_CAPTURE_MISSING=true

**engine_source_category:** pending

**manifest_path:** `examples/engine_native_capture_manifest.json`

## Buffer Status

| Buffer | Required | Present | Quality | Format |
|--------|----------|---------|---------|--------|
| current_color | yes | no | unavailable | - |
| history_color | yes | no | unavailable | - |
| motion_vectors | yes | no | unavailable | - |
| current_depth | yes | no | unavailable | - |
| history_depth | optional | no | unavailable | - |
| current_normals | yes | no | unavailable | - |
| history_normals | optional | no | unavailable | - |
| roi_mask | optional | no | unavailable | - |
| jitter | optional | no | unavailable | - |
| exposure | optional | no | unavailable | - |
| camera_matrices | optional | no | unavailable | - |
| history_validity_mask | optional | no | unavailable | - |

## Import Status: PENDING

No real engine-native capture has been provided.

To provide a capture, see `docs/unreal_export_playbook.md`, `docs/unity_export_playbook.md`, or `docs/custom_renderer_export_playbook.md`.

After exporting buffers, update `examples/engine_native_capture_manifest.json` with:
1. `source.engine_type` set to `unreal`, `unity`, or `custom`
2. Buffer paths pointing to the exported files

Then re-run:
```bash
cargo run --release -- import-engine-native \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```

## Validation Errors

- ENGINE_NATIVE_CAPTURE_MISSING: no real engine buffers provided

## What Is Not Proven

- Renderer-integrated sampling is not proven (proxy Demo B only)
- Mixed-regime confirmation on engine-native data is still pending
- Ground-truth renderer reference is not available unless explicitly exported

## Remaining Blockers

- **EXTERNAL**: No real engine capture has been provided. See playbooks.
- **EXTERNAL**: Ground-truth reference frames require renderer export.
- **EXTERNAL**: Mixed-regime confirmation on engine-native data requires an appropriate scene.
