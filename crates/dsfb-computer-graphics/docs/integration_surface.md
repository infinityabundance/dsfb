# Integration Surface

This document describes the attachable host-style interface implemented inside this crate.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Purpose

The goal is to show a concrete supervisory attachment point rather than a vague confidence story. The crate expresses the method as typed buffers and explicit outputs that fit a temporal pipeline.

## Minimum Host-Realistic Path

Current decision-facing minimum path:

- current color
- reprojected history color
- motion vectors for reprojection
- current depth
- reprojected depth
- current normals
- reprojected normals

Outputs:

- trust
- intervention
- alpha
- debug proxy fields
- structural state labels

This is the path represented by `dsfb_host_realistic`.

## Important Current Clarifications

### Trust behavior

The current implementation behaves more like a high-sensitivity gate than a smoothly calibrated continuous supervisor. The trust field remains useful as a supervisory signal, but the crate does not describe it as broadly calibrated.

### Motion disagreement

Motion disagreement is no longer treated as part of the minimum path. It remains available as `dsfb_motion_augmented` and is reported as an optional extension for motion-biased scenarios.

That change exists because the present suite does not justify claiming that motion disagreement is always necessary.

### Synthetic visibility

Synthetic visibility is retained only as a research/debug comparison path. It is not presented as a deployable host input.

## Temporal Reuse Attachment

Baseline resolve:

```text
C_t(u) = alpha * C_t_current(u) + (1 - alpha) * C_{t-1}_reproj(u)
```

Supervised resolve:

```text
C_t(u) = alpha_t(u) * C_t_current(u) + (1 - alpha_t(u)) * C_{t-1}_reproj(u)
alpha_t(u) = alpha_min + (alpha_max - alpha_min) * (1 - T_t(u))
```

The underlying estimator is unchanged. The supervisory layer changes how aggressively the pipeline falls back to current-frame replacement.

## Sampling Attachment

Demo B uses the same evidence family in a different role:

- imported temporal trust
- native trust-style single-frame allocation
- variance and heuristic alternatives

That makes the crate useful for evaluating both temporal supervision and budget routing without pretending that the two are already a production-integrated system.

## Pass Decomposition

Plausible pass order:

1. reproject history, depth, and optional normals
2. compute local disagreement proxies
3. classify simplified structural state
4. combine hazards into trust and intervention
5. derive alpha
6. apply temporal resolve
7. optionally export trust / intervention / logging surfaces

## What This Document Proves

- the host-facing inputs are explicit
- the minimum path is concrete
- the optional motion and synthetic-visibility paths are separated from the minimum path
- trust, intervention, and alpha are real crate outputs

## What This Document Does Not Prove

- engine integration is already complete
- the current interface is optimal for every renderer
- the pass ordering has been tuned on hardware
