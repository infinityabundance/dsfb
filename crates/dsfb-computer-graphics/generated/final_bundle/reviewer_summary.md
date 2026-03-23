# Reviewer Summary

On the canonical scenario, host-realistic DSFB reduced cumulative ROI MAE from 2.84366 for fixed alpha to 0.34793.

On the canonical sampling scenario, imported trust reduced ROI MAE from 0.17226 for uniform allocation to 0.03184 under the same total budget.

Point-like ROI disclosure: thin_reveal=1 px, diagonal_reveal=1 px. These remain mechanically relevant but statistically weak and are not treated as region-scale aggregate evidence.

Trust conclusion: The current host-realistic implementation behaves as a near-binary gate rather than a smoothly calibrated continuous supervisor.

Timing conclusion: `cpu_only_proxy` with actual GPU timing measured = `false`.

GPU bridge conclusion: `actual_gpu_timing_measured` with actual GPU timing measured = `true`.

External bridge conclusion: external-capable = `true`, externally validated = `false`.

What is still blocked:
- broader external scene validation
- engine-side GPU profiling on imported captures

What is now decision-clean:
- host-realistic minimum path is explicit
- point vs region ROI evidence is separated
- motion disagreement is optional rather than hidden in the minimum path
- Demo B includes region and mixed-width evidence
- weights are centralized and sensitivity-vetted
- a real GPU-executable kernel exists in the crate
- external buffers can be imported through a stable manifest

## What Is Not Proven

- no production-scene or engine deployment proof

## Remaining Blockers

- broader external scene validation
- engine-side GPU profiling on imported captures
