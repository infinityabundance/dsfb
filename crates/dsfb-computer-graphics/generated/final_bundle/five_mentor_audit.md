# Five Mentor Audit

Demo A: On the canonical scenario, host-realistic DSFB reduced cumulative ROI MAE from 2.84366 for fixed alpha to 0.34793.

Demo B: On the canonical sampling scenario, imported trust reduced ROI MAE from 0.17226 for uniform allocation to 0.03184 under the same total budget.

Point-like ROI disclosure: thin_reveal=1 px, diagonal_reveal=1 px. Region-sized scenarios are reported separately for decision-facing aggregate claims.

## SBIR / Toyon

Passes: multi-scenario host-realistic evidence, explicit blockers, and fail-loud validation.

Still blocks: synthetic-only scope and no fielded integration.

Timing note: Only CPU proxy timing is available.

Readiness: ready for evaluation.

## NVIDIA

Passes: timing path exists, minimum path is explicit, motion extension is isolated.

Still blocks: no measured GPU execution; strong heuristic still competitive on some scenarios.

Timing note: Only CPU proxy timing is available.

Readiness: ready for evaluation.

## AMD / Intel

Passes: buffer, traffic, and scaling surfaces are explicit.

Still blocks: no hardware cache/bandwidth measurements.

Timing note: Only CPU proxy timing is available.

Readiness: ready for evaluation.

## Academic

Passes: honest ROI disclosure, ablations, trust diagnostics, and sensitivity sweeps.

Still blocks: synthetic breadth and no external benchmark corpus.

Timing note: Only CPU proxy timing is available.

Readiness: ready for evaluation.

## Licensing / Strategy

Passes: decision-facing reports show what passes, what ties, and what still blocks.

Still blocks: no engine case study or customer validation.

Timing note: Only CPU proxy timing is available.

Readiness: ready for evaluation.

## What Is Not Proven

- This audit does not claim funding close, licensing close, or deployment readiness.

## Remaining Blockers

- real GPU measurements
- external engine validation
