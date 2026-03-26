# Unreal-Native Failure Modes

This file is first-class evidence for where the Unreal-native replay path should remain bounded or advisory.

## Structural Limits

- Residual-only evidence weakens when the host output already tracks the current frame closely.
- Missing ROI or disocclusion masks force the run to derive overlays from the DSFB response, which is useful for triage but not a substitute for exported engine annotations.
- Transparency, particles, UI, post effects, and specular-only motion can violate the view-space normal and monotonic-depth assumptions.
- If motion vectors are noisy or encoded in a convention that does not match the manifest, the run fails rather than silently downgrading.
- Where a host heuristic already performs strongly, DSFB should be interpreted as a bounded monitor or advisory layer.

## Export-Specific Notes

- Dataset `ue57_dsfb_temporal_capture_sample` uses motion_vector_convention = `pixel_offset_to_prev` and history_source = `previous_frame_export_plus_metadata_motion_reprojection`.

## Scaling Limits

- Scaling measurement kind: `measured_gpu`. Coverage status: `partial`.
