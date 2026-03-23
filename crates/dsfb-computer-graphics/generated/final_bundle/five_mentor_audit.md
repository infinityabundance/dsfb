# Five Mentor Audit

Demo A: On the canonical scenario, host-realistic DSFB reduced cumulative ROI MAE from 2.84366 for fixed alpha to 0.34793.

Demo B: On the canonical sampling scenario, imported trust reduced ROI MAE from 0.17226 for uniform allocation to 0.03184 under the same total budget.

Point-like ROI disclosure: thin_reveal=1 px, diagonal_reveal=1 px. Region-sized scenarios are reported separately for decision-facing aggregate claims.

## SBIR / Toyon

Passes: multi-scenario host-realistic evidence, explicit blockers, fail-loud validation, and evaluator handoff package.

Still blocks: synthetic-only scope and no fielded integration.

Timing note: Measured GPU timing is available.

External handoff note: external-capable = `true`, externally validated = `false`.

Readiness: ready for evaluation.

## NVIDIA

Passes: GPU-executable minimum kernel exists, minimum path is explicit, and motion extension is isolated.

Still blocks: no measured engine-integrated GPU execution; strong heuristic still competitive on some scenarios.

Timing note: Measured GPU timing is available.

External handoff note: external-capable = `true`, externally validated = `false`.

Readiness: ready for evaluation.

## AMD / Intel

Passes: buffer, traffic, scaling, and external import surfaces are explicit.

Still blocks: no hardware cache/bandwidth measurements on real imported captures.

Timing note: Measured GPU timing is available.

External handoff note: external-capable = `true`, externally validated = `false`.

Readiness: ready for evaluation.

## Academic

Passes: honest ROI disclosure, ablations, trust diagnostics, sensitivity sweeps, and scenario taxonomy.

Still blocks: synthetic breadth and no external benchmark corpus.

Timing note: Measured GPU timing is available.

External handoff note: external-capable = `true`, externally validated = `false`.

Readiness: ready for evaluation.

## Licensing / Strategy

Passes: decision-facing reports show what passes, what ties, what external data is needed, and what to test next.

Still blocks: no engine case study or customer validation.

Timing note: Measured GPU timing is available.

External handoff note: external-capable = `true`, externally validated = `false`.

Readiness: ready for evaluation.

## What Is Not Proven

- This audit does not claim funding close, licensing close, or deployment readiness.

## Remaining Blockers

- external engine validation
