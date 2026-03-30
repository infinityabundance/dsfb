# CURRENT_CANONICAL_RUN

Use this run as the current canonical package:

- [`sample_capture_contract_sequence_canonical`](/home/one/dsfb/crates/dsfb-computer-graphics/generated/canonical_2026_q1/sample_capture_contract_sequence_canonical)

Do not treat these sibling directories as canonical:

- `sample_capture_contract`
- `sample_capture_contract_debug`
- `sample_capture_contract_canonical`
- `sample_capture_contract_sequence_canonical_pre_note_fix`
- `sample_capture_contract_sequence_canonical_pre_truth_gate_fix`
- `sample_capture_contract_sequence_canonical_pre_hw_gpu`
- `sample_capture_contract_sequence_canonical_v2`

Reason:

- The current canonical package is the 5-capture strict Unreal-native sequence run with exported `reference_color`, the canonical DSFB + heuristic hybrid ladder, temporal trust trajectory artifacts, and hardware-backed `NVIDIA GeForce RTX 4080 SUPER` / `Vulkan` imported-buffer GPU scaling.
- DSFB improves strong temporal heuristics via structural supervision.
- DSFB alone does not outperform strong heuristic baselines in the current evaluation.
- The ROI definition captures approximately 50% of the frame under the fixed baseline-relative threshold, making the metric closer to a global structural error measure than a sparse artifact mask.
- Pure DSFB is `heuristic_favorable` on all 5 Demo A captures; `dsfb_plus_strong_heuristic` is the current hero result with ROI MAE mean +- std `0.00501 +- 0.00178` versus `0.00657 +- 0.00247` for `strong_heuristic`.
- Trust trajectory facts for the current canonical run are onset `frame_0001`, peak ROI `frame_0002`, recovery-side `frame_0005`, mean trust `0.78657 -> 0.35245 -> 0.49284`, and intervention rate `0.21345 -> 0.64758 -> 0.50715`.
