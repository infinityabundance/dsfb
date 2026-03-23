# Evaluator Handoff

Run first:
- `cargo run --release -- run-all --output generated/final_bundle`
- `cargo run --release -- validate-final --output generated/final_bundle`
- `cargo run --release -- run-gpu-path --output generated/gpu_path`
- `cargo run --release -- run-external-replay --manifest examples/external_capture_manifest.json --output generated/external_real`

Strongest current evidence:
- On the canonical scenario, host-realistic DSFB reduced cumulative ROI MAE from 2.84366 for fixed alpha to 0.34793.
- On the canonical sampling scenario, imported trust reduced ROI MAE from 0.17226 for uniform allocation to 0.03184 under the same total budget.

Weakest current evidence:
- no real external engine capture has been validated
- strong heuristic remains competitive on some scenarios

Single highest-value next GPU experiment:
- Run the measured `wgpu` minimum kernel on the target evaluator GPU and compare numeric deltas against the CPU reference on one region-ROI case and one realism-stress case.

Single highest-value next external replay experiment:
- Export one real frame pair from an engine into the external schema, replay it through DSFB host-realistic, and compare fixed alpha, strong heuristic, and DSFB on the same capture.

Engine-side baselines to keep:
- fixed alpha
- strong heuristic
- imported-trust or native-trust sampling policy where relevant

## External Validation

- exact command: `cargo run --release -- run-external-replay --manifest <manifest> --output generated/external_real`
- required data format: `current_color`, `reprojected_history`, `motion_vectors`, `current_depth`, `reprojected_depth`, `current_normals`, `reprojected_normals`, plus optional `mask`, `ground_truth`, and `variance`.
- expected outputs: `external_validation_report.md`, `gpu_external_report.md`, `gpu_external_metrics.json`, `demo_a_external_report.md`, `demo_b_external_report.md`, `demo_b_external_metrics.json`, `scaling_report.md`, `scaling_metrics.json`, `memory_bandwidth_report.md`, `integration_scaling_report.md`, and `figures/`.
- success looks like: the imported capture runs through the DSFB host-minimum path, GPU status is explicit, ROI vs non-ROI is separated, fixed-budget Demo B compares DSFB against stronger heuristics, and the scaling package says whether 1080p/4K, readback, and async insertion are viable.
- failure looks like: malformed schema, missing required buffers, no measured-vs-unmeasured GPU disclosure, no 1080p attempt or unavailable classification, budget mismatch, or reports that hide proxy-vs-real metric status.
- interpretation: ties against strong heuristics mean DSFB is behaving like a targeted supervisory overlay rather than a blanket replacement; losses plus higher non-ROI penalty should trigger engine-side tuning before any broader claim.

## What Is Not Proven

- This handoff does not claim the current crate has already passed external evaluation.

## Remaining Blockers

- external engine captures
- GPU profiling on imported captures

## Engine-Native Validation

### Status
ENGINE_NATIVE_CAPTURE_MISSING=true — infrastructure is complete; real capture not yet provided.

### What is in place
- `examples/engine_native_capture_manifest.json` — manifest template (update engine_type + paths)
- `examples/engine_native_buffer_schema.json` — complete buffer format spec
- `docs/engine_capture_schema.md` — schema and conventions
- `docs/unreal_export_playbook.md` — exact Unreal Engine export steps
- `docs/unity_export_playbook.md` — exact Unity export steps
- `docs/custom_renderer_export_playbook.md` — exact custom renderer steps
- `generated/engine_native/` — all pending placeholder reports (will be overwritten with real data)
- `generated/mixed_regime_confirmation_report.md` — internal mixed-regime confirmed (aliasing + variance co-active)
- `generated/manual_engine_native_commands.md` — exact manual steps

### Exact manual export steps
See `docs/unreal_export_playbook.md` for the precise UE5 sequence. Minimum required:
1. Export `current_color.exr` (pre-TAA linear HDR)
2. Export `history_color.exr` (TAA history input or previous frame)
3. Export `motion_vectors.exr` (pixel offset convention — see playbook for NDC conversion)
4. Export `current_depth.exr` (linear depth, larger = further)
5. Export `current_normals.exr` (view-space unit vectors)
6. Write `metadata.json` with frame_index, dimensions, real_external_data=true

### Exact import command
```bash
cargo run --release -- import-engine-native \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```

### Exact replay command (GPU + Demo A + Demo B — same pipeline as DAVIS/Sintel)
```bash
cargo run --release -- run-engine-native-replay \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```

### Exact validation command
```bash
# Strict (requires real capture — fails until capture is provided):
cargo run --release -- validate-final --output generated/final_bundle

# Permissive (passes even without real capture):
cargo run --release -- validate-final --output generated/final_bundle --allow-pending-engine-native
```

### What success looks like
- `ENGINE_NATIVE_CAPTURE_MISSING=false` in all engine_native reports
- `measured_gpu: true` with real adapter name and timing in `gpu_execution_report.md`
- `actual_engine_native_data: true` in GPU metrics JSON
- Demo A and Demo B reports with actual computed metrics (not TBD)
- `validate-final --output generated/final_bundle` passes without `--allow-pending-engine-native`

### What failure looks like
- `ENGINE_NATIVE_CAPTURE_MISSING=true` (no real capture provided — most common)
- Buffer shape mismatch (all buffers must have same width×height)
- Non-linear depth (reversed-Z not converted)
- World-space normals (not converted to view space)
- Post-tonemapped color (must be pre-TAA linear HDR)
