# MPI Sintel Mapping Report

## Why This Dataset

MPI Sintel is a standard, instantly recognizable renderer-origin motion benchmark with optical flow and official depth. It grounds the external validation package in motion-rich, renderer-like data serious graphics reviewers already know.

## DSFB Mode

- DSFB mode: `host_minimum_with_native_final_pass_color_native_depth_and_flow_derived_current_grid_history`
- Demo A metric mode: `clean_vs_final_proxy_when_reference_pass_is_present`
- Demo B mode: `fixed_budget_allocation_proxy_with_derived_motion_boundary_roi`
- reference strategy: `current clean pass as explicit proxy reference for final-pass inputs`

## Buffer Mapping

| Field | Quality | Source | Disclosure |
| --- | --- | --- | --- |
| current_color | native | official Sintel final-pass frame | native renderer-origin input frame |
| history_color | derived-high-confidence | previous final-pass frame reprojected onto the current frame | derived from native adjacent frames plus inverted official flow |
| motion_vectors | derived-high-confidence | current-grid backward flow derived from official forward flow | native flow exists, but the current-grid backward field is derived for this schema |
| current_depth | native | official Sintel depth archive | native depth |
| history_depth | derived-high-confidence | previous native depth reprojected onto the current frame | derived from native depth plus inverted official flow |
| current_normals | derived-high-confidence | depth-gradient normals from native depth | derived from the official depth field |
| history_normals | derived-high-confidence | previous depth-gradient normals reprojected onto the current frame | derived from native depth plus inverted official flow |
| roi_mask | derived-low-confidence | motion-boundary / depth-discontinuity support mask | no native ROI exists in Sintel, so ROI support is explicit derived logic |
| reference | derived-low-confidence | current clean pass used as a proxy reference for final-pass input | clean-vs-final is a renderer-like proxy, not temporal ground truth |

## Native Buffers

- `current_color`
- `optical_flow_forward`
- `current_depth`

## Derived Buffers

- `history_color`
- `motion_vectors`
- `history_depth`
- `current_normals`
- `history_normals`
- `roi_mask`
- `reference_proxy`

## Unsupported Buffers

- `renderer_ground_truth`

## Prepared Captures

| Label | Sequence | Frame | ROI kind | Case tags |
| --- | --- | ---: | --- | --- |
| ambush_5_mixed_frame_0047 | ambush_5 | 47 | derived_mixed_roi | realism_stress_case, mixed_regime_candidate, region_roi_case, high_motion_case |
| ambush_5_point_frame_0047 | ambush_5 | 47 | derived_point_roi | realism_stress_case, point_roi_case, high_motion_case |
| ambush_5_region_frame_0047 | ambush_5 | 47 | derived_region_roi | realism_stress_case, larger_roi_case, region_roi_case, high_motion_case |
| market_6_mixed_frame_0008 | market_6 | 8 | derived_mixed_roi | realism_stress_case, mixed_regime_candidate, region_roi_case, high_motion_case |
| market_6_region_frame_0008 | market_6 | 8 | derived_region_roi | realism_stress_case, larger_roi_case, region_roi_case, high_motion_case |


## Notes

- Sintel mapping report: /home/one/dsfb/crates/dsfb-computer-graphics/generated/sintel_mapping_report.md
- Sintel summary JSON: /home/one/dsfb/crates/dsfb-computer-graphics/generated/sintel_mapping_summary.json
- Depth is required here; if the official depth archive cannot be downloaded, preparation fails loudly.
