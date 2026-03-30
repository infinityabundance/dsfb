# CURRENT_STATUS

This crate's current canonical evidence path is the strict Unreal-native replay path.

Canonical package target:

- [`generated/canonical_2026_q1/sample_capture_contract_sequence_canonical`](/home/one/dsfb/crates/dsfb-computer-graphics/generated/canonical_2026_q1/sample_capture_contract_sequence_canonical)

Benchmark contract:

- ROI is defined as pixels where baseline error exceeds 15% of local contrast. The mask is computed once from the baseline and held fixed across all methods. DSFB does not influence ROI selection.
- Canonical baseline: `fixed_alpha`
- Canonical strong baseline: `strong_heuristic` (strong heuristic clamp)
- Canonical DSFB method: `dsfb_host_minimum`
- Canonical hybrid: `dsfb_plus_strong_heuristic`
- Canonical trust artifacts: `trust_histogram.svg`, `trust_vs_error.svg`, `trust_conditioned_error_map.png`, and `trust_temporal_trajectory.svg`

Current evidence posture:

- DSFB improves strong temporal heuristics via structural supervision.
- DSFB alone does not outperform strong heuristic baselines in the current evaluation.
- The ROI definition captures approximately 50% of the frame under the fixed baseline-relative threshold, making the metric closer to a global structural error measure than a sparse artifact mask.
- The current canonical run contains 5 real Unreal-native captures (`frame_0001` through `frame_0005`) from one unchanged-code ordered shot. ROI MAE mean +- std is `0.32966 +- 0.08251` for `fixed_alpha`, `0.00657 +- 0.00247` for `strong_heuristic`, `0.04522 +- 0.00683` for `dsfb_host_minimum`, and `0.00501 +- 0.00178` for `dsfb_plus_strong_heuristic`.
- Each capture includes exported `reference_color`; metric source is `real_reference` against a higher-resolution Unreal export proxy rather than a path-traced or high-spp ground truth.
- Pure DSFB remains `heuristic_favorable` on all 5 captures. The hybrid wins ROI MAE and full-frame MAE across the 5-capture sequence and wins or ties max error on every capture.
- Demo B remains an allocation proxy, but it is DSFB-helpful on all 5 captures. Mean ROI error is `0.23822` for imported trust, `0.26347` for the combined heuristic, and `0.30026` for uniform allocation.
- ROI coverage mean +- std is `50.60% +- 18.61%`.
- A temporal trust trajectory is now present for the checked-in real sequence: onset `frame_0001`, peak ROI `frame_0002`, recovery-side `frame_0005`; mean trust `0.78657 -> 0.35245 -> 0.49284`; intervention rate `0.21345 -> 0.64758 -> 0.50715`.
- Imported-buffer compute-path timings were measured on `NVIDIA GeForce RTX 4080 SUPER` / `Vulkan`: `1.0391 ms` at `256x144`, `17.5886 ms` at `1920x1080`, and `67.7201 ms` at `3840x2160`. These timings do not replace in-engine profiling.
- The crate keeps heuristic-favorable cases visible instead of filtering them out.

Canonical boardroom table:

| Surface | `fixed_alpha` | `strong_heuristic` | `dsfb_host_minimum` | `dsfb_plus_strong_heuristic` | Current reading |
| --- | ---: | ---: | ---: | ---: | --- |
| Capture count | 5 | 5 | 5 | 5 | one unchanged-code ordered shot |
| Demo A classification | n/a | wins over pure DSFB | `heuristic_favorable` on all 5 | current hero result | hybrid is the best current Demo A result |
| ROI MAE mean +- std | 0.32966 +- 0.08251 | 0.00657 +- 0.00247 | 0.04522 +- 0.00683 | 0.00501 +- 0.00178 | hybrid wins |
| Full-frame MAE mean +- std | 0.18033 +- 0.10549 | 0.00372 +- 0.00105 | 0.02275 +- 0.00529 | 0.00305 +- 0.00092 | hybrid wins |
| Max error mean +- std | 0.60403 +- 0.00418 | 0.27507 +- 0.04350 | 0.29093 +- 0.04583 | 0.25144 +- 0.04483 | hybrid wins / ties |
| ROI coverage mean +- std | 50.60% +- 18.61% | 50.60% +- 18.61% | 50.60% +- 18.61% | 50.60% +- 18.61% | fixed mask, closer to global structural error than sparse ROI |
| Demo B mean ROI error | n/a | combined heuristic = 0.26347 | imported trust = 0.23822 | n/a | positive but bounded gain vs uniform = 0.30026 |

Remaining bounded blockers:

- The 5 real captures come from one shot, so broader scene/regime distribution is still incomplete.
- The reference is a higher-resolution Unreal export proxy, not a path-traced or high-spp ground truth.
- Demo B is not yet a live renderer budget benchmark.
- Imported-buffer GPU timings do not prove final engine-side integration cost.

Supersession:

- This supersedes older single-capture proxy, synthetic-only, llvmpipe-only, and pre-Unreal-native diligence bundles.
- Historical bundles remain in `generated/`, but they are not the current canonical truth path.
