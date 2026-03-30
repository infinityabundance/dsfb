# Unreal-Native Comparison Summary

Dataset `ue57_dsfb_temporal_capture_sequence_sample` is labeled `unreal_native` and was executed through the strict Unreal-native replay path.

## Frozen Benchmark Contract

- ROI is defined as pixels where baseline error exceeds 15% of local contrast. The mask is computed once from the baseline and held fixed across all methods. DSFB does not influence ROI selection.
- Canonical baseline ladder: `fixed_alpha`, `strong_heuristic`, `dsfb_host_minimum`, `dsfb_plus_strong_heuristic`.
- Fixed capture count in this run: `5` real Unreal-native capture(s).
- Trust diagnostics generated for the canonical run: `figures/trust_histogram.svg`, `figures/trust_vs_error.svg`, `figures/trust_conditioned_error_map.png`, `figures/trust_temporal_trajectory.svg`.

## Current Result Posture

- DSFB improves strong temporal heuristics via structural supervision.
- DSFB alone does not outperform strong heuristic baselines in the current evaluation.
- The ROI definition captures approximately 50% of the frame under the fixed baseline-relative threshold, making the metric closer to a global structural error measure than a sparse artifact mask.
- Demo A ROI MAE mean ± std is 0.32966 ± 0.08251 for `fixed_alpha`, 0.00657 ± 0.00247 for `strong_heuristic`, 0.04522 ± 0.00683 for `dsfb_host_minimum`, and 0.00501 ± 0.00178 for `dsfb_plus_strong_heuristic`.
- ROI coverage mean ± std is 50.60% ± 18.61% across the fixed capture family.
- Trust trajectory facts for this run: onset `frame_0001`, peak ROI `frame_0002`, recovery-side `frame_0005`; mean trust = 0.78657 -> 0.35245 -> 0.49284; intervention rate = 0.21345 -> 0.64758 -> 0.50715.
- Demo B mean ROI error is 0.23822 for imported trust, 0.26347 for the combined heuristic, and 0.30026 for uniform allocation.

## Capture Classification

- `frame_0001` (DSFBTemporalCapture/minimal_temporal_sequence frame 1): `heuristic_favorable`. ROI pixels = 14159, ROI coverage = 38.41%. DSFB ROI MAE = 0.05302, DSFB + heuristic ROI MAE = 0.00635, strong heuristic ROI MAE = 0.00844, fixed alpha ROI MAE = 0.28734.
- `frame_0002` (DSFBTemporalCapture/minimal_temporal_sequence frame 2): `heuristic_favorable`. ROI pixels = 27619, ROI coverage = 74.92%. DSFB ROI MAE = 0.03903, DSFB + heuristic ROI MAE = 0.00203, strong heuristic ROI MAE = 0.00265, fixed alpha ROI MAE = 0.45358.
- `frame_0003` (DSFBTemporalCapture/minimal_temporal_sequence frame 3): `heuristic_favorable`. ROI pixels = 9205, ROI coverage = 24.97%. DSFB ROI MAE = 0.05388, DSFB + heuristic ROI MAE = 0.00615, strong heuristic ROI MAE = 0.00860, fixed alpha ROI MAE = 0.28668.
- `frame_0004` (DSFBTemporalCapture/minimal_temporal_sequence frame 4): `heuristic_favorable`. ROI pixels = 17073, ROI coverage = 46.31%. DSFB ROI MAE = 0.04196, DSFB + heuristic ROI MAE = 0.00662, strong heuristic ROI MAE = 0.00852, fixed alpha ROI MAE = 0.22578.
- `frame_0005` (DSFBTemporalCapture/minimal_temporal_sequence frame 5): `heuristic_favorable`. ROI pixels = 25211, ROI coverage = 68.39%. DSFB ROI MAE = 0.03824, DSFB + heuristic ROI MAE = 0.00388, strong heuristic ROI MAE = 0.00466, fixed alpha ROI MAE = 0.39490.

## Demo B Policy Posture

- `frame_0001`: DSFB-helpful allocation case. Imported trust ROI error = 0.19726, combined heuristic ROI error = 0.22313, uniform ROI error = 0.27572.
- `frame_0002`: DSFB-helpful allocation case. Imported trust ROI error = 0.35576, combined heuristic ROI error = 0.37702, uniform ROI error = 0.38515.
- `frame_0003`: DSFB-helpful allocation case. Imported trust ROI error = 0.16317, combined heuristic ROI error = 0.20053, uniform ROI error = 0.25888.
- `frame_0004`: DSFB-helpful allocation case. Imported trust ROI error = 0.17477, combined heuristic ROI error = 0.19330, uniform ROI error = 0.23506.
- `frame_0005`: DSFB-helpful allocation case. Imported trust ROI error = 0.30011, combined heuristic ROI error = 0.32338, uniform ROI error = 0.34649.

## Boundaries

- This is evidence consistent with reduced temporal artifact risk in bounded cases, not a claim of universal outperformance.
- Aggregated mean ± std claims are emitted in `aggregation_summary.md` because this run contains 5 unchanged-code real captures.
- Demo B remains an advisory allocation proxy unless a live renderer budget path is exported.
- The crate is acting as a supervisory trust / admissibility / intervention layer, not a renderer replacement.
