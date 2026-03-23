# Ablation Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

This report answers which cues materially drive the effect and how much survives host-realistic mode.

| Variant | Canonical cumulative ROI MAE | Suite mean cumulative ROI MAE | Suite mean false-positive rate |
| --- | ---: | ---: | ---: |
| DSFB visibility-assisted | 0.16026 | 0.19921 | 0.00910 |
| DSFB host-realistic minimum | 0.34793 | 0.23780 | 0.01154 |
| DSFB gated reference | 0.31904 | 0.31149 | 0.02927 |
| DSFB motion-augmented | 0.22180 | 0.19947 | 0.01194 |
| DSFB without visibility cue | 0.65707 | 0.59248 | 0.01086 |
| DSFB without thin proxy | 0.72393 | 0.43526 | 0.00816 |
| DSFB without motion disagreement | 0.34793 | 0.23780 | 0.01154 |
| DSFB without grammar | 0.73057 | 0.57297 | 0.01132 |
| DSFB residual-only | 1.09552 | 1.02857 | 0.02385 |
| DSFB trust without alpha modulation | 3.59520 | 2.38577 | 0.09259 |
