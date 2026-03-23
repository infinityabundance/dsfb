# Ablation Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

This report answers which cues materially drive the effect and how much survives host-realistic mode.

| Variant | Canonical cumulative ROI MAE | Suite mean cumulative ROI MAE | Suite mean false-positive rate |
| --- | ---: | ---: | ---: |
| DSFB visibility-assisted | 0.16026 | 0.18370 | 0.00909 |
| DSFB host-realistic minimum | 0.34793 | 0.18891 | 0.01160 |
| DSFB gated reference | 0.31904 | 0.24558 | 0.02504 |
| DSFB motion-augmented | 0.22180 | 0.15012 | 0.01176 |
| DSFB without visibility cue | 0.65707 | 0.46916 | 0.00952 |
| DSFB without thin proxy | 0.72393 | 0.34419 | 0.00860 |
| DSFB without motion disagreement | 0.34793 | 0.18891 | 0.01160 |
| DSFB without grammar | 0.73057 | 0.45502 | 0.00994 |
| DSFB residual-only | 1.09552 | 0.81324 | 0.01797 |
| DSFB trust without alpha modulation | 3.59520 | 2.03804 | 0.07425 |
