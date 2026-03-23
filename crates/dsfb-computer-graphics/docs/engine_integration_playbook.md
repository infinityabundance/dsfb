# Engine Integration Playbook

Use this crate as a handoff target for one frame pair plus history.

## Minimal Export Steps

1. Export current color and reprojected history at identical resolution.
2. Export motion vectors in pixel units pointing to the previous frame.
3. Export current and reprojected depth.
4. Export current and reprojected normals in a consistent view-space basis.
5. Write a manifest that matches `examples/external_capture_manifest.json`.
6. Run `import-external` and inspect the generated trust, alpha, and intervention outputs.

## First External Evaluation

- Start with one instability-heavy frame pair and one neutral frame pair.
- Compare DSFB host-minimum against fixed alpha and a strong heuristic.
- Check whether intervention localizes to the expected instability region without causing excessive non-ROI penalty.

## What Still Requires External Validation

- real engine motion-vector correctness
- real renderer buffer conventions
- GPU performance under engine scheduling
- behavior on production image content
