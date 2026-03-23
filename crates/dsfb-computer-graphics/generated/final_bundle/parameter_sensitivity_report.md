# Parameter Sensitivity Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Baseline mode: DSFB host-realistic minimum.

| Parameter | Mode | Value | Benefit wins vs fixed | Zero-ghost benefit scenarios | Canonical ROI MAE | Region mean ROI MAE | Motion-bias ROI MAE | Neutral non-ROI MAE | Robust corridor | Robustness |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| depth_weight | host_realistic | 0.500 | 3 | 1 | 0.52515 | 0.47196 | 0.65085 | 0.00000 | no | fragile |
| thin_weight | host_realistic | 0.500 | 3 | 2 | 0.50719 | 0.39140 | 0.55323 | 0.00000 | no | moderately_sensitive |
| grammar_weight | host_realistic | 0.500 | 3 | 1 | 0.54897 | 0.47643 | 0.65966 | 0.00000 | no | fragile |
| residual_threshold_scale | host_realistic | 0.500 | 3 | 2 | 0.27589 | 0.26552 | 0.36627 | 0.00000 | yes | robust |
| motion_weight | motion_augmented | 0.500 | 3 | 2 | 0.27910 | 0.27881 | 0.42081 | 0.00000 | yes | robust |
| depth_weight | host_realistic | 0.750 | 3 | 2 | 0.44261 | 0.37655 | 0.52756 | 0.00000 | no | moderately_sensitive |
| thin_weight | host_realistic | 0.750 | 3 | 2 | 0.43247 | 0.34350 | 0.48498 | 0.00000 | yes | robust |
| grammar_weight | host_realistic | 0.750 | 3 | 2 | 0.44723 | 0.37829 | 0.53154 | 0.00000 | no | moderately_sensitive |
| residual_threshold_scale | host_realistic | 0.750 | 3 | 2 | 0.32191 | 0.27633 | 0.37851 | 0.00000 | yes | robust |
| motion_weight | motion_augmented | 0.750 | 3 | 2 | 0.24878 | 0.27299 | 0.41998 | 0.00000 | yes | robust |
| depth_weight | host_realistic | 1.000 | 3 | 2 | 0.34793 | 0.29959 | 0.42329 | 0.00000 | yes | robust |
| thin_weight | host_realistic | 1.000 | 3 | 2 | 0.34793 | 0.29959 | 0.42329 | 0.00000 | yes | robust |
| grammar_weight | host_realistic | 1.000 | 3 | 2 | 0.34793 | 0.29959 | 0.42329 | 0.00000 | yes | robust |
| residual_threshold_scale | host_realistic | 1.000 | 3 | 2 | 0.34793 | 0.29959 | 0.42329 | 0.00000 | yes | robust |
| motion_weight | motion_augmented | 1.000 | 3 | 2 | 0.22180 | 0.27074 | 0.41942 | 0.00000 | yes | robust |
| depth_weight | host_realistic | 1.250 | 3 | 2 | 0.24858 | 0.24192 | 0.33961 | 0.00000 | yes | robust |
| thin_weight | host_realistic | 1.250 | 3 | 2 | 0.27327 | 0.26045 | 0.36823 | 0.00000 | yes | robust |
| grammar_weight | host_realistic | 1.250 | 3 | 2 | 0.24945 | 0.24060 | 0.33670 | 0.00000 | yes | robust |
| residual_threshold_scale | host_realistic | 1.250 | 3 | 2 | 0.35259 | 0.36058 | 0.54447 | 0.00000 | no | moderately_sensitive |
| motion_weight | motion_augmented | 1.250 | 3 | 2 | 0.19852 | 0.27056 | 0.41908 | 0.00000 | yes | robust |
| depth_weight | host_realistic | 1.500 | 3 | 2 | 0.17901 | 0.19934 | 0.27635 | 0.00000 | yes | robust |
| thin_weight | host_realistic | 1.500 | 3 | 2 | 0.21060 | 0.22652 | 0.32006 | 0.00000 | yes | robust |
| grammar_weight | host_realistic | 1.500 | 3 | 2 | 0.18005 | 0.19713 | 0.27159 | 0.00000 | yes | robust |
| residual_threshold_scale | host_realistic | 1.500 | 3 | 2 | 0.35469 | 0.44252 | 0.70169 | 0.00000 | no | fragile |
| motion_weight | motion_augmented | 1.500 | 3 | 2 | 0.17925 | 0.27043 | 0.41881 | 0.00000 | yes | robust |
| alpha_min | host_realistic | 0.040 | 3 | 2 | 0.42932 | 0.33381 | 0.44674 | 0.00000 | no | fragile |
| alpha_min | host_realistic | 0.080 | 3 | 2 | 0.34793 | 0.29959 | 0.42329 | 0.00000 | yes | robust |
| alpha_min | host_realistic | 0.120 | 3 | 2 | 0.28400 | 0.27159 | 0.40115 | 0.00000 | yes | robust |
| alpha_max | host_realistic | 0.840 | 3 | 0 | 0.58360 | 0.59348 | 0.71649 | 0.00000 | no | fragile |
| alpha_max | host_realistic | 0.960 | 3 | 2 | 0.34793 | 0.29959 | 0.42329 | 0.00000 | yes | robust |
| alpha_max | host_realistic | 0.990 | 3 | 2 | 0.25140 | 0.22586 | 0.35481 | 0.00000 | yes | robust |

- These sweeps are one-at-a-time sensitivity checks around the centralized hand-set parameterization. They are intended to show robustness corridors, not to overclaim a global optimum.
- The motion-weight sweep uses the optional motion-augmented profile because the minimum host-realistic path no longer includes motion disagreement by default.

- `robust` means the main benefit cases remain intact with bounded motion-bias and neutral-scene degradation.
- `moderately_sensitive` means the conclusion survives, but with narrower safety margin.
- `fragile` means the headline behavior or neutral-scene bound degrades materially.

## What Is Not Proven

- These sweeps do not claim global optimality or statistically complete calibration.

## Remaining Blockers

- Parameters are now centralized and sensitivity-vetted, but they are still hand-set rather than trained on an external benchmark.
