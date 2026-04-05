# Scene: Disocclusion / History Invalidation

## Description
This scene represents a specific temporal regime observed in Unreal Engine captures. It is constructed as a view over existing artifact bundles and does not introduce new data.

## Regime Characteristics
Newly visible regions lacking valid history: a foreground object moves or the camera shifts to expose background pixels that were previously occluded and have no valid accumulation history from the prior frame. The temporal reuse pipeline receives a reprojected sample that is geometrically mismatched.

## Typical Artifact
Smear / mismatch — the previously occluded region inherits colour from the wrong surface, producing a visible smear or colour mismatch that persists for one or more frames until the history decays.

## Expected DSFB Behavior (Observed, Not Claimed)
- Residual structure: abrupt, spatially coherent residual patch at the disocclusion boundary; structure follows the occlusion contour
- Drift behavior: low before disocclusion event; spike at the transition frame
- Slew behavior: elevated slew at transition; asymmetric — onset is sharp, recovery is gradual
- Trust response: abrupt trust reduction at the exposed boundary; recovery rate depends on subsequent frame consistency

## Source Mapping
This scene references existing outputs from:

- unreal_native_sample_manifest_smoke_*
- materialized_external/frame_*

No new captures are introduced.

## Notes
This grouping is interpretive and intended to aid analysis. It does not imply statistical separation or independent dataset construction.
