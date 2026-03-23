# Blocker Check

## Removed

- Host-realistic DSFB mode exists and is reported separately from visibility-assisted mode.
- Stronger baselines are present and scored across multiple scenarios.
- A bounded neutral scenario is included to expose false positives.
- Demo B enforces fixed-budget fairness across multiple policies.

## Partially Removed

- Strong heuristic baselines are now explicit, but they remain competitive on some scenarios.
- Cost confidence is better because buffers and stages are explicit, but hardware validation remains undone.

## Remaining

- The scenario suite is still synthetic and does not prove production-scene generalization.
- The strong heuristic baseline remains competitive on some cases, so the crate supports evaluation diligence rather than universal win claims.
- Cost accounting is architectural and CPU-side within the crate; it is not a measured GPU benchmark.
