# Dataset Mapping

This document records the native-vs-derived mapping used to run the DSFB external replay path on DAVIS and MPI Sintel.

## DAVIS 2017

- manifest: `/home/one/dsfb/crates/dsfb-computer-graphics/examples/davis_external_manifest.json`
- DSFB mode: `host_minimum_with_native_color_and_roi_plus_derived_motion_depth_normal_proxies`
- derived-vs-native disclosure: all fields below are labeled as native, derived-high-confidence, derived-low-confidence, or unavailable

| Field | Quality | Source |
| --- | --- | --- |
| current_color | native | official DAVIS RGB frame |
| history_color | native | adjacent DAVIS RGB frame reprojected to the current frame using the derived motion field |
| motion_vectors | derived-low-confidence | deterministic block-matching optical-flow proxy |
| current_depth | derived-low-confidence | segmentation-guided relative-depth proxy |
| history_depth | derived-low-confidence | previous-frame relative depth warped into current frame |
| current_normals | derived-low-confidence | depth-gradient normals |
| history_normals | derived-low-confidence | previous depth-gradient normals warped into current frame |
| roi_mask | native | official DAVIS segmentation annotation |
| reference | unavailable | not provided by DAVIS in this mapping |

## MPI Sintel

- manifest: `/home/one/dsfb/crates/dsfb-computer-graphics/examples/sintel_external_manifest.json`
- DSFB mode: `host_minimum_with_native_final_pass_color_native_depth_and_flow_derived_current_grid_history`
- derived-vs-native disclosure: all fields below are labeled as native, derived-high-confidence, derived-low-confidence, or unavailable

| Field | Quality | Source |
| --- | --- | --- |
| current_color | native | official Sintel final-pass frame |
| history_color | derived-high-confidence | previous final-pass frame reprojected onto the current frame |
| motion_vectors | derived-high-confidence | current-grid backward flow derived from official forward flow |
| current_depth | native | official Sintel depth archive |
| history_depth | derived-high-confidence | previous native depth reprojected onto the current frame |
| current_normals | derived-high-confidence | depth-gradient normals from native depth |
| history_normals | derived-high-confidence | previous depth-gradient normals reprojected onto the current frame |
| roi_mask | derived-low-confidence | motion-boundary / depth-discontinuity support mask |
| reference | derived-low-confidence | current clean pass used as a proxy reference for final-pass input |

