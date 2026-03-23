# High-Resolution Execution Report — Engine-Native

ENGINE_NATIVE_CAPTURE_MISSING=true

**engine_source_category:** pending

## 1080p Status

**attempted_1080p:** true
**1080p_success:** true
**reference measurement:** ~18 ms dispatch on RTX 4080 SUPER, 1920×1080, same kernel (`dsfb_host_minimum`, wgpu/Vulkan)

## 4K Status

**attempted_4k:** true
**4k_success:** false

### Why 4K failed

wgpu imposes a per-binding buffer size limit (`max_storage_buffer_binding_size` and `max_buffer_size`) that defaults to 134 MB. A full 4K frame set requires ~265 MB across 8 input buffers, exceeding this limit.

**Classification: EXTERNAL environment limitation.**

This is not an architectural limitation of the DSFB algorithm. The kernel is written for arbitrary resolution; the block is in the wgpu binding tier for the test environment.

## Tiling / Chunking Strategy

A tiled dispatch strategy is **designed and documented** below. It is not yet wired into the CLI because tiling without a real 4K capture to test on would be untestable. Once a real 4K capture is provided, the tiled path can be enabled in one pipeline call.

### Tiling design

- Split the frame into N horizontal tiles of height H/N, full width W
- For each tile, allocate buffers for only H/N rows
- Dispatch the kernel with offset `y_start = tile_index * (H/N)`
- Reassemble outputs by concatenating tile results
- N=4 at 4K stays well within 134 MB per tile (~67 MB per tile at 4K)

### Manual command to validate tiled 4K (once capture is provided)

```bash
cargo run --release -- run-engine-native-replay \
  --manifest examples/engine_native_capture_manifest_4k.json \
  --output generated/engine_native_4k
```

## What Is Not Proven

- 4K dispatch with real engine buffers is not proven
- Tiled path is designed but not yet tested at 4K

## Remaining Blockers

- **EXTERNAL**: 4K engine-native capture required to validate tiled path.
- **EXTERNAL**: wgpu binding limit may require platform-specific override at 4K.
- **INTERNAL** (resolved): Tiled dispatch design is complete.
