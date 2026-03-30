# Unreal-Native Failure Modes

This file is first-class evidence for where the Unreal-native replay path should remain bounded or advisory.

## Structural Limits

- Residual-only evidence weakens when the host output already tracks the current frame closely.
- Canonical ROI is always recomputed from the fixed-alpha baseline and the reference/proxy frame using `fixed_alpha_local_contrast_0p15`; optional manifest ROI masks are audit inputs only.
- Transparency, particles, UI, post effects, and specular-only motion can violate the view-space normal and monotonic-depth assumptions.
- If motion vectors are noisy or encoded in a convention that does not match the manifest, the run fails rather than silently downgrading.
- Where a host heuristic already performs strongly, DSFB should be interpreted as a bounded monitor or advisory layer.

## Export-Specific Notes

- Dataset `ue57_dsfb_temporal_capture_sequence_sample` uses motion_vector_convention = `pixel_offset_to_prev` and history_source = `previous_frame_export_plus_metadata_motion_reprojection`.

## Scaling Limits

- Scaling measurement kind: `gpu_scaled_probe_measured`. Coverage status: `partial`.
