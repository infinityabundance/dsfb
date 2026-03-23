# Demo B Decision Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

On the canonical sampling scenario, imported trust reduced ROI MAE from 0.17226 for uniform allocation to 0.03184 under the same total budget.

This report explicitly separates aliasing recovery on point-like thin features from allocation quality on mixed-width, variance-limited, and edge-trap region cases under fixed-budget equality.

## Thin-Structure Reveal

Taxonomy: `aliasing_limited`. Sampling taxonomy: `coverage-dominated point reveal`. Support category: `PointLikeRoi`.

Thin-Structure Reveal: imported-trust ROI MAE 0.03184, combined-heuristic ROI MAE 0.04977, uniform ROI MAE 0.17226.

Imported trust remains competitive under equal budget on this scenario.

## Fast Lateral Reveal

Taxonomy: `mixed`. Sampling taxonomy: `thin-band reveal with textured background`. Support category: `RegionRoi`.

Fast Lateral Reveal: imported-trust ROI MAE 0.00220, combined-heuristic ROI MAE 0.00603, uniform ROI MAE 0.00804.

Imported trust remains competitive under equal budget on this scenario.

## Diagonal Subpixel Reveal

Taxonomy: `aliasing_limited`. Sampling taxonomy: `subpixel diagonal coverage case`. Support category: `PointLikeRoi`.

Diagonal Subpixel Reveal: imported-trust ROI MAE 0.00770, combined-heuristic ROI MAE 0.01245, uniform ROI MAE 0.01607.

Imported trust remains competitive under equal budget on this scenario.

## Textured Reveal Band

Taxonomy: `mixed`. Sampling taxonomy: `mixed-width reveal band with aliasing and texture`. Support category: `RegionRoi`.

Textured Reveal Band: imported-trust ROI MAE 0.00343, combined-heuristic ROI MAE 0.00972, uniform ROI MAE 0.01459.

Imported trust remains competitive under equal budget on this scenario.

## Motion-Bias Reveal Band

Taxonomy: `mixed`. Sampling taxonomy: `motion-mismatch reveal band`. Support category: `RegionRoi`.

Motion-Bias Reveal Band: imported-trust ROI MAE 0.00974, combined-heuristic ROI MAE 0.01716, uniform ROI MAE 0.03229.

Imported trust remains competitive under equal budget on this scenario.

## Layered Slat Reveal

Taxonomy: `mixed`. Sampling taxonomy: `layered slat reveal with mixed stable and unstable zones`. Support category: `RegionRoi`.

Layered Slat Reveal: imported-trust ROI MAE 0.00353, combined-heuristic ROI MAE 0.00881, uniform ROI MAE 0.01985.

Imported trust remains competitive under equal budget on this scenario.

## Noisy Reprojection Reveal

Taxonomy: `variance_limited`. Sampling taxonomy: `realism-stress reveal with noisy reprojection`. Support category: `RegionRoi`.

Noisy Reprojection Reveal: imported-trust ROI MAE 0.01056, combined-heuristic ROI MAE 0.02067, uniform ROI MAE 0.03290.

Imported trust remains competitive under equal budget on this scenario.

## Heuristic-Friendly Pan

Taxonomy: `edge_trap`. Sampling taxonomy: `competitive baseline reveal`. Support category: `RegionRoi`.

Heuristic-Friendly Pan: imported-trust ROI MAE 0.00252, combined-heuristic ROI MAE 0.00692, uniform ROI MAE 0.01030.

Imported trust remains competitive under equal budget on this scenario.

## Contrast Pulse Stress

Taxonomy: `variance_limited`. Sampling taxonomy: `negative control`. Support category: `NegativeControl`.

Contrast Pulse Stress: imported-trust ROI MAE 0.00008, combined-heuristic ROI MAE 0.00008, uniform ROI MAE 0.00008.

Neutral case: guidance is not expected to produce a large win, so non-ROI penalties and concentration behavior matter more than raw ROI gain.

## Stability Holdout

Taxonomy: `variance_limited`. Sampling taxonomy: `negative control`. Support category: `NegativeControl`.

Stability Holdout: imported-trust ROI MAE 0.00672, combined-heuristic ROI MAE 0.00363, uniform ROI MAE 0.00672.

Neutral case: guidance is not expected to produce a large win, so non-ROI penalties and concentration behavior matter more than raw ROI gain.

## What is not proven

- This study does not prove an optimal sampling controller.
- It does not prove that imported trust beats every cheap heuristic on every scene.
- It does not claim production renderer integration.
- External validation is still required before extending these conclusions to real renderer sample allocation.

## Remaining Blockers

- Demo B still lacks real-engine shading complexity and measured rendering hardware runs.
- External handoff for imported supervision exists, but no external sample-allocation capture is included.
