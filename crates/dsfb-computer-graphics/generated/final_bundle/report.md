# DSFB Computer Graphics Evaluation Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Scope

This artifact is a deterministic crate-local evaluation package for temporal-reuse supervision and fixed-budget sampling allocation. It is intended to clear diligence blockers honestly, not to imply production readiness.

## ROI Disclosure

| Scenario | Support category | ROI pixels | ROI fraction | Disclosure |
| --- | --- | ---: | ---: | --- |
| thin_reveal | PointLikeRoi | 1 | 0.00007 | Canonical reveal ROI collapses to a single disoccluded thin-structure pixel at the default resolution. It remains mechanically relevant but statistically weak and must be reported as point-like evidence. |
| fast_pan | RegionRoi | 644 | 0.04193 | The ROI is a small but regional disocclusion strip rather than a single point. It is still sparse and should not be mixed with large-band ROI results without disclosure. |
| diagonal_reveal | PointLikeRoi | 1 | 0.00007 | At default resolution the diagonal reveal also reduces to point-like support. It is useful for aliasing behavior, but not as a region-sized aggregate claim. |
| reveal_band | RegionRoi | 156 | 0.01016 | This scenario is intentionally region-sized so cumulative ROI metrics are not driven by a single pixel. |
| motion_bias_band | RegionRoi | 714 | 0.04648 | This is a region ROI with deliberately imperfect motion information. It is the main scenario used to decide whether motion disagreement belongs in the minimum path. |
| contrast_pulse | NegativeControl | 1872 | 0.12188 | This negative control uses a large ROI on purpose, but it is not a benefit-expected disocclusion case. |
| stability_holdout | NegativeControl | 1008 | 0.06562 | This is a negative-control background patch used to bound non-ROI damage and false-positive intervention. |

Point-like ROI scenarios are kept because they remain mechanically relevant, but they are not mixed with region-ROI evidence without explicit disclosure.

## Scenario Outcomes

| Scenario | Expectation | Host vs fixed ROI gain | Host vs strong ROI gain | Non-ROI penalty vs strong | Clamp trigger mean | Note |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| thin_reveal | BenefitExpected | 2.49573 | 0.23014 | 0.00028 | 0.00000 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| fast_pan | BenefitExpected | 1.64723 | -0.04395 | 0.00058 | 0.06582 | Strong heuristic remains better on this scenario; the report surfaces that rather than hiding it. |
| diagonal_reveal | BenefitExpected | 1.82760 | 0.41377 | -0.00005 | 0.00000 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| reveal_band | BenefitExpected | 1.83719 | 0.06868 | -0.00077 | 0.05128 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| motion_bias_band | BenefitExpected | 2.52368 | 0.33513 | -0.00174 | 0.12020 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| contrast_pulse | NeutralExpected | -0.00000 | -0.00000 | 0.00000 | 0.00000 | This is the honesty scenario: aggressive trust collapse is not expected to help, so false-positive response and non-ROI stability are the main evaluation criteria. |
| stability_holdout | NeutralExpected | -0.00000 | -0.00000 | 0.00000 | 0.00000 | This is the honesty scenario: aggressive trust collapse is not expected to help, so false-positive response and non-ROI stability are the main evaluation criteria. |

## Trust Diagnostics

The current host-realistic implementation behaves as a near-binary gate rather than a smoothly calibrated continuous supervisor.

Degenerate trust-error rank correlations are retained only as diagnostics and are not used here as decision-facing calibration evidence.

## Motion Disagreement Decision

The minimum host-realistic path excludes motion disagreement. On `motion_bias_band`, the optional motion-augmented path changed cumulative ROI MAE from 0.42329 to 0.41942. That makes motion disagreement an optional extension rather than a minimum-path requirement.

## Demo B Confound Handling

On the canonical sampling scenario, imported trust reduced ROI MAE from 0.17226 for uniform allocation to 0.03184 under the same total budget.

Demo B now includes mixed-width and textured region scenarios alongside the original thin-point case, and reports equal-budget curves at 1, 2, 4, and 8 mean spp. The goal is to separate aliasing recovery from structurally better allocation.

## Resolution Scaling

- The high-resolution tier is a selected-scenario scalable proxy rather than a full 1080p sweep. It is intended to demonstrate structural persistence beyond the toy default resolution without pretending to be a shipping-engine benchmark.
- The canonical thin_reveal point-ROI case is intentionally kept at the default resolution only. At higher resolutions its exact one-pixel disocclusion geometry becomes path-dependent and is not a stable scaling metric.
- Memory footprint numbers are analytical host-realistic buffer estimates from the crate cost model.

## Timing Path

Timing classification: `cpu_only_proxy`. Actual GPU timing measured: `false`.

## Cost Model

“The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation.”

“The framework is compatible with tiled and asynchronous GPU execution.”

| Mode | Buffers | Ops / px | Reads / px | Writes / px |
| --- | ---: | ---: | ---: | ---: |
| Minimal mode | 3 | 20 | 6 | 3 |
| Host-realistic mode | 8 | 60 | 22 | 9 |
| Full research/debug mode | 12 | 66 | 24 | 16 |

## Parameter Sensitivity

Centralized hazard weights are still hand-set, but they are now sensitivity-vetted. Robust corridor sweep points found: 22 of 31.

## What Is Proven

- The supervisory effect is real under a host-realistic minimum path, not only with privileged visibility hints.
- Point-like ROI evidence and region-ROI evidence are now reported separately.
- Motion disagreement is no longer treated as mandatory in the minimum path.
- Demo B no longer relies only on the original thin sub-pixel case.

## What Is Not Proven

- This artifact does not prove production-scene generalization.
- It does not prove measured GPU wins or production deployment performance.
- It does not prove globally calibrated trust or globally optimal parameter settings.

## Remaining Blockers

- The scenario suite is still synthetic and does not prove production-scene generalization.
- The strong heuristic baseline remains competitive on some cases, so the crate supports evaluation diligence rather than universal win claims.
- Cost accounting is architectural and CPU-side within the crate; it is not a measured GPU benchmark.
- Point-like ROI scenarios remain mechanically useful but statistically weak, so aggregate claims must stay separated from region-ROI evidence.
- Real GPU execution data remains outstanding.
- External engine traces and broader scene diversity remain future work.
