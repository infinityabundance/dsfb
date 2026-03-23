# Reviewer Summary

This crate packages a deterministic temporal reuse study for DSFB supervision. Demo A uses a moving occluder, a disocclusion event, thin worst-case geometry, fixed-alpha TAA, a residual-threshold baseline, and a DSFB path that only changes the supervisory blend-control layer.

In this bounded synthetic setting, DSFB reduced ghost persistence duration from 12 to 0 frames relative to the fixed-alpha baseline.

Against the residual-threshold baseline, DSFB reduced ghost persistence duration from 4 to 0 frames.

DSFB plugs into temporal reuse through blend modulation rather than estimator replacement:

```text
C_t(u) = alpha_t(u) * C_t_current(u) + (1 - alpha_t(u)) * C_{t-1}_reproj(u)
alpha_t(u) = alpha_min + (alpha_max - alpha_min) * (1 - T_t(u))
```

Estimated systems footprint: local per-pixel or per-tile residual/proxy/trust operations, bounded temporal memory, and a linear-with-pixel-count supervisory pass that can be evaluated at reduced resolution.

Transition relevance: replayable supervisory evidence, visible failure-response timing, and an attachable middleware shape for temporal reuse, QA logging, and adaptive compute routing.

Commercial relevance: the current crate is not an SDK, but it demonstrates the product shape of a supervisory trust layer that could attach to engine temporal reuse, reconstruction, or traceability workflows.

Default run details: 18 frames at 160 x 96, reveal frame 6, trust-drop frame 6, residual-baseline response frame 6.
