# Production Evaluation Bridge

This document is an evaluator handoff, not a production-readiness claim.

## Proven In This Crate

- A minimum host-realistic DSFB supervisory path exists.
- The crate can run the path on synthetic data and through an external-capable file schema.
- The crate can execute a GPU-oriented compute kernel when a usable adapter is present.

## Not Proven In This Crate

- production-scene generalization
- external engine validation
- production-scale GPU performance
- deployment readiness

## Requires External Validation

- real engine-exported buffers
- in-engine baseline comparisons
- measured GPU timings on target hardware
- evaluation under production resolution and production content

## Minimum External Evaluation

1. Export one frame pair with the required buffers.
2. Run `import-external`.
3. Run `run-gpu-path` on the target GPU host.
4. Compare DSFB against fixed alpha and the strongest local heuristic baseline.
5. Record ROI behavior, non-ROI penalty, and timing.
