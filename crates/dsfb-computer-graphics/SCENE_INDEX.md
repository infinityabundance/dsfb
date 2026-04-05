# Unreal Engine Scene Index

This repository includes multiple Unreal Engine captures processed through a consistent, deterministic DSFB pipeline. The captures are organized post hoc into representative temporal regimes to aid interpretation. These groupings do not reflect different algorithm configurations, but rather different input conditions observed in engine-native data.

The canonical five-frame sequence used in the paper is drawn from a single ordered capture. Additional captures included here are provided for qualitative inspection and consistency verification across regimes. No claim of exhaustive coverage or statistical generalization is made.

## Scenes

### Scene A — Thin Geometry / Subpixel Reveal
- Regime: visibility emergence at subpixel scale
- Typical artifact: temporal instability / ghosting
- DSFB signal: localized trust reduction, structured residual onset
- Source: unreal_native_sample_manifest_smoke_*

### Scene B — Disocclusion / History Invalidation
- Regime: newly visible regions lacking valid history
- Typical artifact: smear / mismatch
- DSFB signal: elevated slew, abrupt residual transition
- Source: unreal_native_sample_manifest_smoke_*

### Scene C — Camera Motion / Reprojection Instability
- Regime: motion-driven reprojection error
- Typical artifact: temporal instability
- DSFB signal: sustained drift variation
- Source: unreal_native_sample_manifest_smoke_*

### Scene D — High-Contrast / Edge-Dominated Regions
- Regime: rapid intensity variation across edges
- Typical artifact: flicker / edge instability
- DSFB signal: structured residual transitions
- Source: unreal_native_sample_manifest_smoke_*

NOTE:
All scenes reuse the same deterministic pipeline and are derived from engine-native captures. They are not independent datasets, but distinct regimes observed within available captures.
