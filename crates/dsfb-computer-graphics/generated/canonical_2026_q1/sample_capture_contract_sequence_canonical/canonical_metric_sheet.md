# Canonical Metric Sheet

ROI is defined as pixels where baseline error exceeds 15% of local contrast. The mask is computed once from the baseline and held fixed across all methods. DSFB does not influence ROI selection.

Strong baseline: `strong_heuristic` (named strong heuristic clamp). Canonical baseline: `fixed_alpha`.

| Capture set | Metric | Baseline | Strong heuristic | DSFB | DSFB + heuristic | Winner |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| frame_0001 | ROI MAE | 0.28734 | 0.00844 | 0.05302 | 0.00635 | DSFB + heuristic |
| frame_0001 | Full-frame MAE | 0.11153 | 0.00413 | 0.02148 | 0.00333 | DSFB + heuristic |
| frame_0001 | Max error | 0.60685 | 0.30008 | 0.30050 | 0.30008 | Strong heuristic |
| frame_0001 | ROI coverage | 38.41% | 38.41% | 38.41% | 38.41% | fixed ROI mask |
| frame_0002 | ROI MAE | 0.45358 | 0.00265 | 0.03903 | 0.00203 | DSFB + heuristic |
| frame_0002 | Full-frame MAE | 0.34025 | 0.00222 | 0.02962 | 0.00175 | DSFB + heuristic |
| frame_0002 | Max error | 0.60852 | 0.20031 | 0.29211 | 0.19184 | DSFB + heuristic |
| frame_0002 | ROI coverage | 74.92% | 74.92% | 74.92% | 74.92% | fixed ROI mask |
| frame_0003 | ROI MAE | 0.28668 | 0.00860 | 0.05388 | 0.00615 | DSFB + heuristic |
| frame_0003 | Full-frame MAE | 0.07257 | 0.00296 | 0.01444 | 0.00235 | DSFB + heuristic |
| frame_0003 | Max error | 0.60450 | 0.31922 | 0.25549 | 0.24858 | DSFB + heuristic |
| frame_0003 | ROI coverage | 24.97% | 24.97% | 24.97% | 24.97% | fixed ROI mask |
| frame_0004 | ROI MAE | 0.22578 | 0.00852 | 0.04196 | 0.00662 | DSFB + heuristic |
| frame_0004 | Full-frame MAE | 0.10623 | 0.00528 | 0.02105 | 0.00440 | DSFB + heuristic |
| frame_0004 | Max error | 0.59634 | 0.25255 | 0.23677 | 0.21349 | DSFB + heuristic |
| frame_0004 | ROI coverage | 46.31% | 46.31% | 46.31% | 46.31% | fixed ROI mask |
| frame_0005 | ROI MAE | 0.39490 | 0.00466 | 0.03824 | 0.00388 | DSFB + heuristic |
| frame_0005 | Full-frame MAE | 0.27106 | 0.00398 | 0.02714 | 0.00345 | DSFB + heuristic |
| frame_0005 | Max error | 0.60392 | 0.30322 | 0.36979 | 0.30322 | Strong heuristic |
| frame_0005 | ROI coverage | 68.39% | 68.39% | 68.39% | 68.39% | fixed ROI mask |
