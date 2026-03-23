# Realism Bridge Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Region-ROI evidence, realism-stress probes, competitive-baseline cases, and bounded-neutral controls now carry the main empirical load instead of leaving the story concentrated in point-ROI stress tests.

- Point-ROI scenarios: `2`
- Region-ROI scenarios: `6`
- Realism-stress scenarios: `2`
- Strong-heuristic-competitive scenarios: `2`
- Bounded-neutral or bounded-loss disclosures: `2`

| Scenario | Support | Tags | ROI pixels | Host vs fixed ROI gain | Host vs strong ROI gain | Bounded note |
| --- | --- | --- | ---: | ---: | ---: | --- |
| thin_reveal | PointLikeRoi | point_roi | 1 | 2.49573 | 0.23014 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| fast_pan | RegionRoi | competitive_baseline, region_roi | 644 | 1.64723 | -0.04395 | Strong heuristic remains better on this scenario; the report surfaces that rather than hiding it. |
| diagonal_reveal | PointLikeRoi | point_roi | 1 | 1.82760 | 0.41377 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| reveal_band | RegionRoi | region_roi | 156 | 1.83719 | 0.06868 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| motion_bias_band | RegionRoi | realism_stress, region_roi | 714 | 2.52368 | 0.33513 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| layered_slats | RegionRoi | region_roi | 162 | 2.05428 | 0.11961 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| noisy_reprojection | RegionRoi | realism_stress, region_roi | 1658 | 2.97736 | 0.74017 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| heuristic_friendly_pan | RegionRoi | competitive_baseline, region_roi | 576 | 2.07274 | 0.11158 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| contrast_pulse | NegativeControl | bounded_neutral_or_loss | 1872 | -0.00000 | -0.00000 | This is the honesty scenario: aggressive trust collapse is not expected to help, so false-positive response and non-ROI stability are the main evaluation criteria. |
| stability_holdout | NegativeControl | bounded_neutral_or_loss | 1008 | -0.00000 | -0.00000 | This is the honesty scenario: aggressive trust collapse is not expected to help, so false-positive response and non-ROI stability are the main evaluation criteria. |

## What Is Proven

- The crate now exposes a broader synthetic realism bridge with explicit external-handoff relevance instead of a narrow point-ROI-only story.

## What Is Not Proven

- These scenarios remain synthetic and do not replace external engine captures or production image content.

## Remaining Blockers

- The realism bridge still needs external replay on real engine buffers before it can be treated as production-adjacent evidence.
