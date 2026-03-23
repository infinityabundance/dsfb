# Blocker Check

Point-like ROI disclosure: thin_reveal=1 px, diagonal_reveal=1 px. These cases are no longer buried inside region-ROI aggregate claims.

## Removed

- Point-like ROI evidence is now labeled explicitly instead of being mixed into aggregate claims.
- Trust rank correlation is no longer used as a headline calibration claim when degenerate.
- Motion disagreement is not hidden in the minimum host-realistic path.
- Hazard weights are centralized and sensitivity-vetted.
- Demo B includes mixed-width region cases and equal-budget efficiency curves.

## Partially Removed

- GPU timing is addressed by a CPU proxy timing path and hardware-model estimates, but actual GPU measurements are still missing: `false`.
- Trust behavior is now described honestly, but broad calibration claims still remain blocked by limited scene diversity.

## Remaining

- The scenario suite is still synthetic and does not prove production-scene generalization.
- The strong heuristic baseline remains competitive on some cases, so the crate supports evaluation diligence rather than universal win claims.
- Cost accounting is architectural and CPU-side within the crate; it is not a measured GPU benchmark.
- Point-like ROI scenarios remain mechanically useful but statistically weak, so aggregate claims must stay separated from region-ROI evidence.
- Actual GPU execution measurements.

## What Is Not Proven

- This file does not claim all diligence blockers are removed.

## Remaining Blockers

- synthetic-only scope
- no production-scene engine integration
