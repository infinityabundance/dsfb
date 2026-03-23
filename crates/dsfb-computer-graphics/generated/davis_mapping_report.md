# DAVIS 2017 Mapping Report

## Why This Dataset

DAVIS is a standard, immediately recognizable real-video benchmark with dense segmentation masks. It anchors the external replay path in real image content instead of only synthetic scenes.

## DSFB Mode

- DSFB mode: `host_minimum_with_native_color_and_roi_plus_derived_motion_depth_normal_proxies`
- Demo A metric mode: `proxy_only_without_renderer_ground_truth`
- Demo B mode: `fixed_budget_allocation_proxy_with_native_roi_masks`
- reference strategy: `no reference frame; current-vs-history proxy metrics only`

## Buffer Mapping

| Field | Quality | Source | Disclosure |
| --- | --- | --- | --- |
| current_color | native | official DAVIS RGB frame | native DAVIS image |
| history_color | native | adjacent DAVIS RGB frame reprojected to the current frame using the derived motion field | native frames; reprojection uses the derived motion proxy |
| motion_vectors | derived-low-confidence | deterministic block-matching optical-flow proxy | derived from adjacent DAVIS frames; not native optical flow |
| current_depth | derived-low-confidence | segmentation-guided relative-depth proxy | foreground/background relative-depth proxy only; not metric depth |
| history_depth | derived-low-confidence | previous-frame relative depth warped into current frame | derived from previous DAVIS mask/image and the motion proxy |
| current_normals | derived-low-confidence | depth-gradient normals | derived from the relative-depth proxy |
| history_normals | derived-low-confidence | previous depth-gradient normals warped into current frame | derived from the previous relative-depth proxy |
| roi_mask | native | official DAVIS segmentation annotation | native binary ROI support after unioning non-zero objects |
| reference | unavailable | not provided by DAVIS in this mapping | no renderer-quality temporal ground truth is available in the mapped path |

## Native Buffers

- `current_color`
- `history_color`
- `roi_mask`

## Derived Buffers

- `motion_vectors`
- `current_depth`
- `history_depth`
- `current_normals`
- `history_normals`

## Unsupported Buffers

- `ground_truth_reference`

## Prepared Captures

| Label | Sequence | Frame | ROI kind | Case tags |
| --- | --- | ---: | --- | --- |
| dance-twirl_frame_0079 | dance-twirl | 79 | native_region_roi | realism_stress_case, larger_roi_case, mixed_regime_candidate |
| soapbox_frame_0069 | soapbox | 69 | native_region_roi | realism_stress_case, larger_roi_case, mixed_regime_candidate |
| camel_frame_0020 | camel | 20 | native_region_roi | realism_stress_case, larger_roi_case, mixed_regime_candidate |


## Notes

- DAVIS mapping report: /home/one/dsfb/crates/dsfb-computer-graphics/generated/davis_mapping_report.md
- DAVIS summary JSON: /home/one/dsfb/crates/dsfb-computer-graphics/generated/davis_mapping_summary.json
- Derived fields are labeled derived-low-confidence in both the reports and the manifests.
- No derived-high-confidence fields exist for this dataset: DAVIS does not provide native optical flow, metric depth, or renderer-origin outputs, so all derived buffers are explicitly labeled derived-low-confidence rather than derived-high-confidence.
