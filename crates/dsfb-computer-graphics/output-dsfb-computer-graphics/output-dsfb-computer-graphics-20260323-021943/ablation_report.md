# Ablation Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

This report answers which cues materially drive the effect and how much survives host-realistic mode.

| Variant | Canonical cumulative ROI MAE | Suite mean cumulative ROI MAE | Suite mean false-positive rate |
| --- | ---: | ---: | ---: |
| DSFB visibility-assisted | 0.32040 | 0.18892 | 0.00816 |
| DSFB host-realistic | 0.31904 | 0.18792 | 0.01263 |
| DSFB without visibility cue | 0.32040 | 0.18892 | 0.01258 |
| DSFB without thin proxy | 0.49804 | 0.28722 | 0.00109 |
| DSFB without motion disagreement | 0.31904 | 0.18792 | 0.01257 |
| DSFB without grammar | 0.67995 | 0.39140 | 0.01359 |
| DSFB residual-only | 1.21962 | 0.70374 | 0.00815 |
| DSFB trust without alpha modulation | 3.59520 | 2.04309 | 0.04964 |
