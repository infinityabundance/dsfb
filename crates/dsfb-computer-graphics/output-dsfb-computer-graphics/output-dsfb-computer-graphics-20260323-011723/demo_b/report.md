# DSFB Computer Graphics Demo B Report

## Overview

Demo B is a bounded fixed-budget adaptive-sampling study on the canonical reveal frame. It uses the DSFB trust field from Demo A as a supervisory signal for sample redistribution rather than as a temporal blend controller.

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Sampling Surface

The estimator operates on a continuous version of the reveal frame with subpixel thin geometry, sharp foreground-object edges, and the same disocclusion event used by Demo A.

- Resolution: 160 x 96
- Reveal frame: 6
- Reference estimate: 64 spp per pixel

## Budget Fairness

The uniform baseline and the DSFB-guided allocation use the same total sample budget: 30720 samples.

The guided policy assigns a minimum of 1 spp per pixel, caps at 12 spp per pixel, and redistributes the remaining budget according to low-trust hazard weights.

## Metrics

- Uniform MAE: 0.00289
- Guided MAE: 0.00250
- Uniform RMSE: 0.01869
- Guided RMSE: 0.01504
- Uniform ROI MAE: 0.17348
- Guided ROI MAE: 0.03714
- Uniform ROI RMSE: 0.19957
- Guided ROI RMSE: 0.04356
- ROI mean guided spp: 12.00
- Trust ROI mean carried from Demo A: 0.1200

In this bounded study, the DSFB-guided allocation is intended to show how a trust field could steer fixed-budget sampling rather than prove an optimal adaptive-sampling policy.
