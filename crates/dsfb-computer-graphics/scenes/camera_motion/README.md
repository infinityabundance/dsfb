# Scene: Camera Motion / Reprojection Instability

## Description
This scene represents a specific temporal regime observed in Unreal Engine captures. It is constructed as a view over existing artifact bundles and does not introduce new data.

## Regime Characteristics
Motion-driven reprojection error: camera translation or rotation produces non-trivial per-pixel motion vectors. The reprojection mapping displaces history samples by more than one pixel, increasing the probability of misalignment when sub-pixel geometry or texture detail is involved.

## Typical Artifact
Temporal instability — imperfect reprojection introduces a consistent but small per-pixel error that accumulates over frames, producing a soft instability that does not resolve without additional blending weight reduction.

## Expected DSFB Behavior (Observed, Not Claimed)
- Residual structure: broad, low-amplitude residual field correlated with motion magnitude; not as spatially localized as thin-geometry onset
- Drift behavior: sustained non-zero drift throughout the motion interval; magnitude proportional to reprojection error
- Slew behavior: smooth, directional slew aligned with camera motion direction; duration matches the motion duration
- Trust response: moderate, spatially diffuse trust reduction across the motion field; recovers once motion stops or slows

## Source Mapping
This scene references existing outputs from:

- unreal_native_sample_manifest_smoke_*
- materialized_external/frame_*

No new captures are introduced.

## Notes
This grouping is interpretive and intended to aid analysis. It does not imply statistical separation or independent dataset construction.
