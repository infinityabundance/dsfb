# Scene: Thin Geometry / Subpixel Reveal

## Description
This scene represents a specific temporal regime observed in Unreal Engine captures. It is constructed as a view over existing artifact bundles and does not introduce new data.

## Regime Characteristics
Visibility emergence at subpixel scale: thin features (geometry threads, silhouette edges) that appear or disappear over consecutive frames at sub-pixel precision, stressing the temporal accumulation history with sparse, intermittent signal.

## Typical Artifact
Temporal instability / ghosting — accumulated history from a prior frame where the feature was invisible contaminates the accumulation when it reappears, producing faint haloing or persistence.

## Expected DSFB Behavior (Observed, Not Claimed)
- Residual structure: localized residual onset concentrated at the thin feature locus; surrounding pixels remain low-residual
- Drift behavior: low inter-frame drift baseline punctuated by a spike at reveal frames
- Slew behavior: modest slew; feature onset does not produce sustained directional shift
- Trust response: localized trust reduction at reveal onset; partial recovery once the feature stabilizes in subsequent frames

## Source Mapping
This scene references existing outputs from:

- unreal_native_sample_manifest_smoke_*
- materialized_external/frame_*

No new captures are introduced.

## Notes
This grouping is interpretive and intended to aid analysis. It does not imply statistical separation or independent dataset construction.
