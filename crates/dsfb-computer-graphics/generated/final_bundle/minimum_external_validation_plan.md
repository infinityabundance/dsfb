# Minimum External Validation Plan

1. Export one frame pair with current color, reprojected history, motion, depth, and normals.
2. Run `import-external` on that manifest.
3. Run `run-gpu-path` on the same machine.
4. Compare strong heuristic, fixed alpha, and DSFB host-realistic results.
5. Record ROI behavior, non-ROI penalty, and GPU timing.

## What Is Not Proven

- This plan does not imply the result will be positive.

## Remaining Blockers

- actual external captures still need to be exported
