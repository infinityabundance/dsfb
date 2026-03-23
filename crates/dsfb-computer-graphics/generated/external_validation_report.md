# External Validation Report

## Why DAVIS And Sintel

- DAVIS: DAVIS is a standard, immediately recognizable real-video benchmark with dense segmentation masks. It anchors the external replay path in real image content instead of only synthetic scenes.
- MPI Sintel: MPI Sintel is a standard, instantly recognizable renderer-origin motion benchmark with optical flow and official depth. It grounds the external validation package in motion-rich, renderer-like data serious graphics reviewers already know.

## Dataset Contributions

- DAVIS contributes real captured video plus native segmentation masks.
- MPI Sintel contributes renderer-origin motion-rich sequences, optical flow, and official depth when available.

## Native Vs Derived Buffers

- DAVIS native buffers: current_color, history_color, roi_mask
- DAVIS derived buffers: motion_vectors, current_depth, history_depth, current_normals, history_normals
- DAVIS unsupported buffers: ground_truth_reference
- MPI Sintel native buffers: current_color, optical_flow_forward, current_depth
- MPI Sintel derived buffers: history_color, motion_vectors, history_depth, current_normals, history_normals, roi_mask, reference_proxy
- MPI Sintel unsupported buffers: renderer_ground_truth

## DSFB Modes Run

- DAVIS: `host_minimum_with_native_color_and_roi_plus_derived_motion_depth_normal_proxies`
- MPI Sintel: `host_minimum_with_native_final_pass_color_native_depth_and_flow_derived_current_grid_history`

## GPU Execution Summary

- DAVIS measured_gpu=`true`, actual_real_external_data=`true`
- MPI Sintel measured_gpu=`true`, actual_real_external_data=`true`

## Demo A External Results

- DAVIS uses proxy-only Demo A metrics because no renderer-quality reference exists in the mapped path.
- MPI Sintel uses a clean-vs-final pass proxy when available and labels it explicitly as proxy rather than renderer ground truth.

## Demo B External Results

- DAVIS captures evaluated: 3
  - dance-twirl_frame_0079 regime=`aliasing_limited` fixed_budget_equal=`true`
  - soapbox_frame_0069 regime=`variance_limited` fixed_budget_equal=`true`
- MPI Sintel captures evaluated: 5
  - ambush_5_mixed_frame_0047 regime=`variance_limited` fixed_budget_equal=`true`
  - ambush_5_point_frame_0047 regime=`variance_limited` fixed_budget_equal=`true`

## Scaling And Memory

- DAVIS attempted_1080p=`true` attempted_4k=`true`
- MPI Sintel attempted_1080p=`true` attempted_4k=`true`
- 1080p scaling is attempted on both datasets.
- 4K scaling is attempted when the GPU path can run on scaled buffers.
- Memory / bandwidth reports explicitly state that readback is used for validation, not required in production.

## Pipeline Insertion / Async

- Async feasibility is discussed per dataset in the integration reports.
- Production readback is explicitly classified as not required.
- Barrier / transition discussion remains implementation guidance rather than proof.

## Coverage Taxonomy

- davis: realism_stress_case=`covered`, larger_roi_case=`covered`, mixed_regime_case=`explicitly_missing`
- sintel: realism_stress_case=`covered`, larger_roi_case=`covered`, mixed_regime_case=`explicitly_missing`

## What Is Proven

- DAVIS and MPI Sintel are both integrated into the same DSFB external replay path.
- GPU execution is attempted on both dataset-mapped paths, with measured-vs-unmeasured status made explicit.
- Native-vs-derived buffer provenance is disclosed instead of hidden.

## What Is Not Proven

- This package does not prove production-engine integration.
- Demo B remains an allocation proxy rather than a live renderer sampling benchmark.
- DAVIS depth and normal support remain derived proxies, not native geometry buffers.

## Remaining Blockers

- mixed_regime_case coverage is partial for davis
- mixed_regime_case coverage is partial for sintel
- renderer-integrated sampling validation is still pending

## Next Highest-Value Experiment

- Export one engine-native temporal capture with true history, motion, depth, and normals, then run the same DAVIS/Sintel comparison stack on that capture to close the renderer-integration gap.
