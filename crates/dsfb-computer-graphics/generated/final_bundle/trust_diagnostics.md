# Trust Diagnostics

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

The current host-realistic implementation behaves as a near-binary gate rather than a smoothly calibrated continuous supervisor.

| Scenario | Run | ROI pixels | Occupied bins | Entropy (bits) | Mode | Correlation note |
| --- | --- | ---: | ---: | ---: | --- | --- |
| thin_reveal | dsfb_host_realistic | 1 | 2 | 0.028 | Some(NearBinaryGate) | degenerate, not decision-facing |
| thin_reveal | dsfb_host_gated_reference | 1 | 4 | 0.086 | Some(NearBinaryGate) | degenerate, not decision-facing |
| thin_reveal | dsfb_motion_augmented | 1 | 3 | 0.158 | Some(NearBinaryGate) | degenerate, not decision-facing |
| fast_pan | dsfb_host_realistic | 644 | 5 | 0.283 | Some(NearBinaryGate) | degenerate, not decision-facing |
| fast_pan | dsfb_host_gated_reference | 644 | 4 | 0.423 | Some(NearBinaryGate) | degenerate, not decision-facing |
| fast_pan | dsfb_motion_augmented | 644 | 6 | 0.399 | Some(NearBinaryGate) | degenerate, not decision-facing |
| diagonal_reveal | dsfb_host_realistic | 1 | 2 | 0.027 | Some(NearBinaryGate) | degenerate, not decision-facing |
| diagonal_reveal | dsfb_host_gated_reference | 1 | 3 | 0.060 | Some(NearBinaryGate) | degenerate, not decision-facing |
| diagonal_reveal | dsfb_motion_augmented | 1 | 3 | 0.136 | Some(NearBinaryGate) | degenerate, not decision-facing |
| reveal_band | dsfb_host_realistic | 156 | 2 | 0.082 | Some(NearBinaryGate) | degenerate, not decision-facing |
| reveal_band | dsfb_host_gated_reference | 156 | 4 | 0.406 | Some(NearBinaryGate) | degenerate, not decision-facing |
| reveal_band | dsfb_motion_augmented | 156 | 4 | 0.213 | Some(NearBinaryGate) | degenerate, not decision-facing |
| motion_bias_band | dsfb_host_realistic | 714 | 8 | 0.725 | Some(NearBinaryGate) | degenerate, not decision-facing |
| motion_bias_band | dsfb_host_gated_reference | 714 | 7 | 0.797 | Some(NearBinaryGate) | degenerate, not decision-facing |
| motion_bias_band | dsfb_motion_augmented | 714 | 10 | 0.831 | Some(NearBinaryGate) | degenerate, not decision-facing |
| layered_slats | dsfb_host_realistic | 162 | 2 | 0.087 | Some(NearBinaryGate) | degenerate, not decision-facing |
| layered_slats | dsfb_host_gated_reference | 162 | 4 | 0.409 | Some(NearBinaryGate) | degenerate, not decision-facing |
| layered_slats | dsfb_motion_augmented | 162 | 4 | 0.230 | Some(NearBinaryGate) | degenerate, not decision-facing |
| noisy_reprojection | dsfb_host_realistic | 1658 | 10 | 0.789 | Some(NearBinaryGate) | degenerate, not decision-facing |
| noisy_reprojection | dsfb_host_gated_reference | 1658 | 8 | 1.040 | Some(WeaklyGraded) | non-degenerate |
| noisy_reprojection | dsfb_motion_augmented | 1658 | 10 | 0.859 | Some(NearBinaryGate) | degenerate, not decision-facing |
| heuristic_friendly_pan | dsfb_host_realistic | 576 | 3 | 0.232 | Some(NearBinaryGate) | degenerate, not decision-facing |
| heuristic_friendly_pan | dsfb_host_gated_reference | 576 | 4 | 0.570 | Some(NearBinaryGate) | degenerate, not decision-facing |
| heuristic_friendly_pan | dsfb_motion_augmented | 576 | 5 | 0.363 | Some(NearBinaryGate) | degenerate, not decision-facing |
| contrast_pulse | dsfb_host_realistic | 1872 | 1 | -0.000 | Some(NearBinaryGate) | degenerate, not decision-facing |
| contrast_pulse | dsfb_host_gated_reference | 1872 | 1 | -0.000 | Some(NearBinaryGate) | degenerate, not decision-facing |
| contrast_pulse | dsfb_motion_augmented | 1872 | 1 | -0.000 | Some(NearBinaryGate) | degenerate, not decision-facing |
| stability_holdout | dsfb_host_realistic | 1008 | 1 | -0.000 | Some(NearBinaryGate) | degenerate, not decision-facing |
| stability_holdout | dsfb_host_gated_reference | 1008 | 1 | -0.000 | Some(NearBinaryGate) | degenerate, not decision-facing |
| stability_holdout | dsfb_motion_augmented | 1008 | 1 | -0.000 | Some(NearBinaryGate) | degenerate, not decision-facing |

## What Is Not Proven

- These diagnostics do not prove probabilistic calibration in the statistical sense.
- Point-ROI scenarios remain weak evidence for smooth trust calibration even when they are mechanically useful.

## Remaining Blockers

- The current trust signal still needs broader region-scale evidence and real-engine traces before it can be called broadly calibrated.
