# Integration Scaling Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Pipeline Insertion

- The minimum kernel executes after history reprojection and after motion/depth/normal buffers are available, but before temporal resolve consumes per-pixel alpha or intervention.
- Alpha modulation is consumed by the temporal accumulation pass; trust and intervention are optional debug or allocator-driving side products.
- Production readback is not required. The current reports read buffers back only for validation and CPU/GPU delta checks.

## Async-Compute Feasibility

- Async execution is feasible if reprojected history, depth, and normals are already materialized and the downstream TAA resolve can wait on a GPU-side signal rather than CPU readback.
- The minimum kernel has no scattered history gather and no CPU dependency, so the main async-compute risk is overlap contention on memory bandwidth rather than synchronization correctness.
- Profiling still needs to confirm that the 3x3 current-color neighborhood reads do not stall other post or denoise passes when overlapped.

## Hazards / Barriers / Transitions

- Inputs should be transitioned to shader-read / storage-read state before dispatch.
- Trust, alpha, and intervention outputs should be transitioned from UAV/storage-write into the state required by the temporal resolve or any downstream debug visualization.
- A production integration should avoid CPU fences; only GPU barriers and queue synchronization should be required.

## Pipeline Compatibility

- The minimum kernel is compatible with tiled, deferred, and post-lighting temporal pipelines because it consumes already-aligned per-pixel buffers and writes only local trust/alpha/intervention fields.
- The current design remains compatible with tiled or asynchronous execution because it does not require CPU-side intervention in production.

## Scaling Interpretation

- native_imported 854x480: measured_gpu = `true`, ms/MPixel = 11.3826, approx_linear = unknown
- scaled_1080p 1920x1080: measured_gpu = `true`, ms/MPixel = 8.8530, approx_linear = false
- scaled_4k 3840x2160: measured_gpu = `false`, ms/MPixel = n/a, approx_linear = unknown

## What Is Not Proven

- This integration note is implementation-specific analysis, not a substitute for engine-side trace profiling.

## Remaining Blockers

- Async overlap, queue contention, and barrier cost still need confirmation inside a real renderer.
