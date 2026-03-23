# Trust Mode Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

The current host-realistic implementation behaves as a near-binary gate rather than a smoothly calibrated continuous supervisor.

## Operating Mode Counts

- `NearBinaryGate`: `10` host-realistic scenarios

| Region-ROI scenario | Occupied bins | Effective levels | Entropy (bits) | Discreteness | Mode | Correlation note |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| fast_pan | 5 | 1 | 0.283 | 0.915 | NearBinaryGate | degenerate, not decision-facing |
| reveal_band | 2 | 1 | 0.082 | 0.975 | NearBinaryGate | degenerate, not decision-facing |
| motion_bias_band | 8 | 2 | 0.725 | 0.782 | NearBinaryGate | degenerate, not decision-facing |
| layered_slats | 2 | 1 | 0.087 | 0.974 | NearBinaryGate | degenerate, not decision-facing |
| noisy_reprojection | 10 | 2 | 0.789 | 0.763 | NearBinaryGate | degenerate, not decision-facing |
| heuristic_friendly_pan | 3 | 1 | 0.232 | 0.930 | NearBinaryGate | degenerate, not decision-facing |

## What Is Proven

- The trust signal is now described according to its actual operating mode instead of being overstated as smoothly calibrated.

## What Is Not Proven

- This report does not claim externally validated probabilistic calibration.
- A gate-like trust mode can still be useful externally, but this report does not turn it into a continuous confidence guarantee.

## Remaining Blockers

- Real external replay traces are still needed before the trust operating mode can be generalized beyond this synthetic suite.
