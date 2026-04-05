# Scene: High-Contrast / Edge-Dominated Regions

## Description
This scene represents a specific temporal regime observed in Unreal Engine captures. It is constructed as a view over existing artifact bundles and does not introduce new data.

## Regime Characteristics
Rapid intensity variation across edges: the scene contains regions with high local contrast — bright light sources against dark backgrounds, or sharp material transitions — where any sub-pixel displacement of the reprojection sample produces a disproportionately large colour error relative to low-contrast regions.

## Typical Artifact
Flicker / edge instability — alternating over- and under-accumulation at high-contrast edges, visible as a thin, temporally varying bright or dark fringe along edges that should be stable.

## Expected DSFB Behavior (Observed, Not Claimed)
- Residual structure: structured residual transitions tightly localized to high-contrast edge loci; amplitude scales with local contrast magnitude
- Drift behavior: low average drift away from edges; elevated drift confined to edge-adjacent pixels
- Slew behavior: oscillatory rather than directional; frame-to-frame sign alternation possible at highest-contrast edges
- Trust response: persistent partial trust reduction at edge loci; amplitude sustained across frames as long as contrast remains high

## Source Mapping
This scene references existing outputs from:

- unreal_native_sample_manifest_smoke_*
- materialized_external/frame_*

No new captures are introduced.

## Notes
This grouping is interpretive and intended to aid analysis. It does not imply statistical separation or independent dataset construction.
