# Engine-Native Capture: Last-Mile Plan

## Audit: Current State

### What exists
- External replay path: `run_external_validation_bundle` handles any manifest with `source.kind = files | synthetic_compat`
- CLI: `import-external` / `run-external-replay` replays any `ExternalCaptureManifest`
- DAVIS 2017 and MPI Sintel replayed, GPU-executed, Demo A/B reported
- `validate_final_bundle` passes on `generated/final_bundle`
- All 10 tests pass

### What does NOT exist
1. `source.kind = engine_native` is not a recognized manifest source kind
2. No `examples/engine_native_capture_manifest.json`
3. No `examples/engine_native_buffer_schema.json`
4. No engine export playbooks (Unreal, Unity, custom renderer)
5. No `import-engine-native` CLI command
6. No `run-engine-native-replay` CLI command
7. No `generated/engine_native/` directory or any of its outputs
8. No `generated/mixed_regime_confirmation_report.md`
9. `validate_final_bundle` does not check for engine-native artifacts
10. 4K OOM issue not addressed with tiling/chunking

### Current validation gap for licensing/NVIDIA/SBIR
The crate is externally validated on DAVIS and Sintel (real video datasets) but not on any
**engine-native** temporal buffer capture — meaning buffers exported directly from a real-time
renderer's TAA/temporal pipeline. This is the final gap.

---

## Remaining Blockers

### Internal blockers (solvable in this repo)
1. `ExternalCaptureSource` enum lacks `EngineNative` variant
2. No `engine_native` CLI commands exist
3. `validate_final_bundle` does not gate on engine-native artifacts
4. No mixed-regime internal confirmation case computed
5. 4K high-res report not written
6. No engine export playbooks written

### External blockers (require action outside this repo)
1. **No real engine capture has been provided.** Until someone runs a renderer and exports
   the canonical buffer set, `ENGINE_NATIVE_CAPTURE_MISSING=true` in all engine-native reports.
2. **GPU timing on real engine buffers** is pending the same capture.
3. **True engine-native mixed-regime** requires a renderer capture that naturally contains
   both aliasing and variance pressure in the same ROI.

---

## Files to Change

| File | Change |
|------|--------|
| `src/external.rs` | Add `EngineNative` variant to `ExternalCaptureSource` |
| `src/lib.rs` | Add `pub mod engine_native; pub mod mixed_regime;` |
| `src/cli.rs` | Add `ImportEngineNative`, `RunEngineNativeReplay`, `ConfirmMixedRegime` commands; update `ValidateFinal` |
| `src/pipeline.rs` | Add `run_engine_native_import_pipeline`, `run_engine_native_replay_pipeline`, `confirm_mixed_regime_pipeline`, `validate_engine_native_gates`; update `validate_final_bundle` signature |

## New Files to Add

| File | Purpose |
|------|---------|
| `src/engine_native.rs` | Import, replay, and report generation for engine-native captures |
| `src/mixed_regime.rs` | Internal mixed-regime confirmation using synthetic scenario data |
| `examples/engine_native_capture_manifest.json` | Canonical engine-native manifest (pending capture) |
| `examples/engine_native_buffer_schema.json` | Engine-native buffer schema with engine-specific fields |
| `docs/engine_capture_schema.md` | Schema specification and conventions |
| `docs/unreal_export_playbook.md` | Exact Unreal Engine buffer export steps |
| `docs/unity_export_playbook.md` | Exact Unity buffer export steps |
| `docs/custom_renderer_export_playbook.md` | Generic custom renderer export steps |
| `generated/engine_native/*.md` | All engine-native reports (pending or real) |
| `generated/mixed_regime_confirmation_report.md` | Mixed-regime confirmation |
| `generated/manual_engine_native_commands.md` | Exact manual steps for real capture |

---

## Manual Commands (External Steps Required)

These steps cannot be automated — they require a real renderer:

### Step 1: Export buffers from Unreal Engine
See `docs/unreal_export_playbook.md` for exact steps.

### Step 2: Prepare the manifest
```bash
# Edit examples/engine_native_capture_manifest.json
# Replace "engine_type": "pending" with "engine_type": "unreal"
# Point buffer paths to your exported files
```

### Step 3: Run import
```bash
cd crates/dsfb-computer-graphics
cargo run --release -- import-engine-native \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```

### Step 4: Run full replay
```bash
cargo run --release -- run-engine-native-replay \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```

### Step 5: Validate
```bash
cargo run --release -- validate-final \
  --output generated/final_bundle
# (will fail until real engine capture is provided unless --allow-pending-engine-native is passed)
```

---

## Acceptance Criteria

1. `cargo build --release` succeeds
2. `cargo run --release -- import-engine-native --manifest examples/engine_native_capture_manifest.json --output generated/engine_native` generates all required report files
3. `cargo run --release -- confirm-mixed-regime --output generated` generates `generated/mixed_regime_confirmation_report.md`
4. `cargo run --release -- validate-final --output generated/final_bundle --allow-pending-engine-native` passes
5. `cargo run --release -- validate-final --output generated/final_bundle` FAILS with explicit message about missing engine-native capture
6. All 10 existing tests still pass

---

## Validation Gates

`validate_final_bundle` (without `--allow-pending-engine-native`) must fail if:
- `generated/engine_native/engine_native_import_report.md` is missing
- `generated/engine_native/engine_native_replay_report.md` is missing
- `generated/engine_native/gpu_execution_report.md` is missing
- `generated/engine_native/demo_a_engine_native_report.md` is missing
- `generated/engine_native/demo_b_engine_native_report.md` is missing
- `generated/engine_native/engine_native_validation_report.md` is missing
- `generated/engine_native/high_res_execution_report.md` is missing
- `generated/mixed_regime_confirmation_report.md` is missing
- `generated/manual_engine_native_commands.md` is missing
- Any of the above contains ENGINE_NATIVE_CAPTURE_MISSING=true (real capture required)
- `mixed_regime_confirmed` is asserted without evidence
