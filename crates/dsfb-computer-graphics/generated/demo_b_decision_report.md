# Demo B Decision Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

On the canonical sampling scenario, imported trust reduced ROI MAE from 0.17226 for uniform allocation to 0.03184 under the same total budget.

## Thin-Structure Reveal

Thin-Structure Reveal: imported-trust ROI MAE 0.03184, combined-heuristic ROI MAE 0.04977, uniform ROI MAE 0.17226.

Imported trust remains competitive under equal budget on this scenario.

## Fast Lateral Reveal

Fast Lateral Reveal: imported-trust ROI MAE 0.04760, combined-heuristic ROI MAE 0.08558, uniform ROI MAE 0.11886.

Imported trust remains competitive under equal budget on this scenario.

## Diagonal Subpixel Reveal

Diagonal Subpixel Reveal: imported-trust ROI MAE 0.00770, combined-heuristic ROI MAE 0.01245, uniform ROI MAE 0.01607.

Imported trust remains competitive under equal budget on this scenario.

## Contrast Pulse Stress

Contrast Pulse Stress: imported-trust ROI MAE 0.00008, combined-heuristic ROI MAE 0.00008, uniform ROI MAE 0.00008.

Neutral case: guidance is not expected to produce a large win, so non-ROI penalties and concentration behavior matter more than raw ROI gain.

## Stability Holdout

Stability Holdout: imported-trust ROI MAE 0.00672, combined-heuristic ROI MAE 0.00363, uniform ROI MAE 0.00672.

Neutral case: guidance is not expected to produce a large win, so non-ROI penalties and concentration behavior matter more than raw ROI gain.

## What is not proven

- This study does not prove an optimal sampling controller.
- It does not prove that imported trust beats every cheap heuristic on every scene.
- It does not claim production renderer integration.
