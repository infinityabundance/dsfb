# Aggregation Summary

ROI is defined as pixels where baseline error exceeds 15% of local contrast. The mask is computed once from the baseline and held fixed across all methods. DSFB does not influence ROI selection.

Real capture count in this run: `5`. Mean ± std claims require at least `3` real captures under unchanged code and parameters.

| Metric | Baseline mean ± std | Strong heuristic mean ± std | DSFB mean ± std | DSFB + heuristic mean ± std | Winner |
| --- | ---: | ---: | ---: | ---: | --- |
| ROI MAE | 0.32966 ± 0.08251 | 0.00657 ± 0.00247 | 0.04522 ± 0.00683 | 0.00501 ± 0.00178 | DSFB + heuristic |
