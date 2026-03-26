# FAILURE_MODES

This crate treats failure modes as first-class evidence, not cleanup notes.

## Signal Limits

- Weak residuals can make DSFB advisory rather than decisive.
- Strong host heuristics can tie or beat DSFB on some captures.
- The checked-in Unreal sample is intentionally retained even though its current Demo A classification is `heuristic_favorable`.
- Missing ROI or disocclusion exports reduce interpretability.
- Motion-vector convention errors are fatal because silent normalization mistakes would poison the evidence.
- The minimal crate-local Unreal sample currently uses metadata-derived motion vectors and replay normals because the editor-side Linux capture path did not expose numerically stable dense velocity and unit-normal exports.
- The minimal crate-local Unreal sample uses `monotonic_visualized_depth`, not overclaimed linear depth.

## Content Limits

- Transparency
- particles
- UI
- post effects
- specular-only motion
- poorly observed disocclusions

These regimes can violate the current assumptions about depth, normals, and reprojection.

## Deployment Limits

- GPU timing in the bundle is environment-specific.
- Demo B remains an allocation proxy unless a renderer-integrated sample budget path is exported.
- This crate does not prove universal renderer integration.

## Language Discipline

Allowed posture:

- supervisory layer
- trust / admissibility / intervention signals
- engine-native empirical replay
- evidence consistent with reduced temporal artifact risk
- promising insertion point for Phase I / pilot work

Not allowed posture:

- universal proof
- solved rendering
- renderer replacement
- production-ready without engine integration evidence
