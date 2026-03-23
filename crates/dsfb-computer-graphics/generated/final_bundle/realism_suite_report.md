# Realism Suite Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

| Scenario | Support | Tags | ROI pixels | ROI fraction | Host vs fixed ROI gain | Host vs strong ROI gain |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| thin_reveal | PointLikeRoi | point_roi | 1 | 0.00007 | 2.49573 | 0.23014 |
| fast_pan | RegionRoi | competitive_baseline, region_roi | 644 | 0.04193 | 1.64723 | -0.04395 |
| diagonal_reveal | PointLikeRoi | point_roi | 1 | 0.00007 | 1.82760 | 0.41377 |
| reveal_band | RegionRoi | region_roi | 156 | 0.01016 | 1.83719 | 0.06868 |
| motion_bias_band | RegionRoi | realism_stress, region_roi | 714 | 0.04648 | 2.52368 | 0.33513 |
| layered_slats | RegionRoi | region_roi | 162 | 0.01055 | 2.05428 | 0.11961 |
| noisy_reprojection | RegionRoi | realism_stress, region_roi | 1658 | 0.10794 | 2.97736 | 0.74017 |
| heuristic_friendly_pan | RegionRoi | competitive_baseline, region_roi | 576 | 0.03750 | 2.07274 | 0.11158 |
| contrast_pulse | NegativeControl | bounded_neutral_or_loss | 1872 | 0.12188 | -0.00000 | -0.00000 |
| stability_holdout | NegativeControl | bounded_neutral_or_loss | 1008 | 0.06562 | -0.00000 | -0.00000 |

## What Is Proven

- The suite now contains explicit realism-stress and competitive-baseline cases alongside point-ROI and region-ROI evidence.

## What Is Not Proven

- These scenarios are still synthetic and do not replace external renderer captures.

## Remaining Blockers

- Real production-scene generalization still requires external captures.
