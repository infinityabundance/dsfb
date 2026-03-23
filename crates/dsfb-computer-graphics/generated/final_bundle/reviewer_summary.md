# Reviewer Summary

On the canonical scenario, host-realistic DSFB reduced cumulative ROI MAE from 2.84366 for fixed alpha to 0.34793.

On the canonical sampling scenario, imported trust reduced ROI MAE from 0.17226 for uniform allocation to 0.03184 under the same total budget.

Point-like ROI disclosure: thin_reveal=1 px, diagonal_reveal=1 px. These remain mechanically relevant but statistically weak and are not treated as region-scale aggregate evidence.

Trust conclusion: The current host-realistic implementation behaves as a near-binary gate rather than a smoothly calibrated continuous supervisor.

Timing conclusion: `cpu_only_proxy` with actual GPU timing measured = `false`.

What is still blocked:
- real GPU execution measurements
- broader external scene validation

What is now decision-clean:
- host-realistic minimum path is explicit
- point vs region ROI evidence is separated
- motion disagreement is optional rather than hidden in the minimum path
- Demo B includes region and mixed-width evidence
- weights are centralized and sensitivity-vetted

## What Is Not Proven

- no actual GPU timing in this environment
- no production-scene or engine deployment proof

## Remaining Blockers

- real GPU execution measurements
- broader external scene validation
