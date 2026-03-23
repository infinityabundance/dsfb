# Ablation Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

This report answers which cues materially drive the effect and how much survives host-realistic mode.

| Variant | Canonical cumulative ROI MAE | Suite mean cumulative ROI MAE | Suite mean false-positive rate |
| --- | ---: | ---: | ---: |
| DSFB visibility-assisted | 0.32040 | 0.20801 | 0.00819 |
| DSFB host-realistic | 0.31904 | 0.20622 | 0.01268 |
| DSFB without visibility cue | 0.32040 | 0.20801 | 0.01262 |
| DSFB without thin proxy | 0.49804 | 0.31086 | 0.00109 |
| DSFB without motion disagreement | 0.31904 | 0.20622 | 0.01262 |
| DSFB without grammar | 0.67995 | 0.41330 | 0.01364 |
| DSFB residual-only | 0.15600 | 0.09769 | 0.00067 |
| DSFB trust without alpha modulation | 3.59520 | 2.25150 | 0.04947 |
