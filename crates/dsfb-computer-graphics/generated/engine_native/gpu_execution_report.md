# GPU Execution Report — Engine-Native Capture

ENGINE_NATIVE_CAPTURE_MISSING=true

**engine_source_category:** pending

**Measurement classification:** pending — no capture provided
**Actual GPU timing measured:** false

**actual_engine_native_data:** false

**kernel:** dsfb_host_minimum
**shader_language:** WGSL
**backend:** Vulkan (wgpu 0.19)

## GPU Execution: PENDING

No real engine-native capture was provided. GPU timing on engine-native data cannot be measured.

### Manual command to measure GPU on real capture

After providing a real capture, run:
```bash
cargo run --release -- run-engine-native-replay \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```

Expected output: `generated/engine_native/gpu_execution_report.md`

Expected fields:
- `measured_gpu: true`
- `actual_engine_native_data: true`
- `adapter:` <GPU name>
- `total_ms:` <dispatch time>

### Reference: DAVIS/Sintel measurements (same kernel, comparable resolution)

| Dataset | Resolution | dispatch_ms | adapter |
|---------|-----------|-------------|---------|
| DAVIS 2017 | 854×480 | ~4 ms | RTX 4080 SUPER |
| MPI Sintel | 1024×436 | ~4 ms | RTX 4080 SUPER |
| 1080p (synthetic) | 1920×1080 | ~18 ms | RTX 4080 SUPER |

## CPU vs GPU Parity

Measured on DAVIS and Sintel captures (same kernel path):

- Mean absolute trust delta (CPU vs GPU): < 1e-4
- Mean absolute alpha delta: < 1e-4
- Numerically equivalent within float precision

Readback is used for parity validation only. In production integration, readback is not required.

## What Is Not Proven

- GPU timing on real engine-native data is pending the capture
- 4K engine-native dispatch is limited by binding size (see high_res_execution_report.md)

## Remaining Blockers

- **EXTERNAL**: Real engine-native capture required for GPU timing.
- **EXTERNAL**: 4K dispatch requires tiling (see high_res_execution_report.md).
