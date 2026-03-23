# External Dataset Plan

## Why DAVIS

DAVIS 2017 is the default real-video external dataset because serious reviewers recognize it immediately as a standard video-object-segmentation benchmark with real image sequences and dense segmentation masks. For this crate it provides:

- real captured image sequences rather than synthetic internal scenes
- native per-frame segmentation masks that can be used honestly as ROI support
- natural motion, occlusion, disocclusion, blur, and foreground/background interaction that stress temporal reuse without pretending to be a renderer

## Why Sintel

MPI Sintel is the default renderer-like / motion-aware external dataset because it is widely recognized in rendering, motion-estimation, and temporal-reconstruction discussions. For this crate it provides:

- renderer-origin image sequences with clean and final passes
- native optical flow
- native depth where the official depth training archive is available
- motion-rich, boundary-rich, atmospheric, and high-displacement content that is legible to graphics reviewers

## Buffer Contract By Dataset

### DAVIS

Native:

- current color / adjacent-frame color from official DAVIS frames
- segmentation masks for ROI support

Derived or approximated:

- motion vectors: deterministic block-matching proxy derived from adjacent DAVIS frames
- current depth / history depth: segmentation-guided relative-depth proxy, explicitly non-metric
- current normals / history normals: derived from the depth proxy
- optional reference / ground truth: unavailable by default

Honest DSFB modes:

- host-minimum / host-realistic path with native color and ROI, plus explicitly labeled derived motion/depth/normal proxies
- Demo A on proxy metrics only unless a separate reference is supplied
- Demo B as fixed-budget allocation proxy only

### MPI Sintel

Native:

- current color / adjacent-frame color from official image passes
- optical flow from the official flow archive in the complete dataset
- depth from the official depth training archive when present

Derived or approximated:

- current-grid backward motion vectors: derived by inverting / splatting official forward flow
- history color / history depth / history normals reprojected onto the current frame using the derived backward flow
- normals: derived from native depth
- ROI / mask: derived motion-boundary-aware support when no native ROI exists
- optional reference: clean-pass proxy when final-pass inputs are used, explicitly labeled as a pass proxy rather than renderer ground truth

Honest DSFB modes:

- host-minimum / host-realistic path with native color, native flow-derived motion support, native depth when available, and derived normals
- Demo A with explicitly labeled clean-vs-final proxy metrics when used
- Demo B as fixed-budget allocation proxy only

## Required Reports, Figures, And Metrics

Preparation / mapping:

- `docs/dataset_mapping.md`
- `generated/dataset_preparation_report.md`
- `generated/davis_mapping_report.md`
- `generated/sintel_mapping_report.md`
- `examples/davis_external_manifest.json`
- `examples/sintel_external_manifest.json`

Per-dataset execution:

- `generated/external_davis/*`
- `generated/external_sintel/*`
- replay report and replay metrics
- GPU execution report and GPU execution metrics
- Demo A external report
- Demo B external report and metrics
- scaling report and scaling metrics
- memory / bandwidth report
- integration / async report
- trust map, intervention map, ROI overlay, before / after images

Cross-dataset decision package:

- `generated/external_validation_taxonomy.json`
- `generated/external_validation_report.md`
- `generated/evaluator_handoff.md`
- `generated/check_signing_readiness.md`

## Exact Validation Gates

Preparation fails if any of these are missing:

- DAVIS preparation path
- Sintel preparation path
- `docs/dataset_mapping.md`
- `generated/dataset_preparation_report.md`
- derived-vs-native disclosure in the mapping reports

Replay fails if any of these are missing:

- `examples/davis_external_manifest.json`
- `examples/sintel_external_manifest.json`
- `generated/external_davis/external_replay_report.md`
- `generated/external_sintel/external_replay_report.md`
- explicit ROI-source disclosure

GPU fails if any of these are missing:

- per-dataset GPU execution report
- per-dataset GPU execution metrics
- explicit `measured_gpu` disclosure
- CPU-vs-GPU parity deltas

Demo A / Demo B fail if any of these are missing:

- per-dataset external reports
- strong-heuristic comparisons
- fixed-budget equality disclosure
- aliasing / variance / mixed-regime disclosure
- proxy-vs-ground-truth disclosure

Scaling / integration fails if any of these are missing:

- 1080p attempt or explicit unavailability statement
- scaling report
- memory / bandwidth report
- integration / async report
- production readback statement
- async feasibility statement

Final decision fails if any of these are missing:

- realism-stress coverage or explicit missing classification
- larger-ROI coverage or explicit missing classification
- mixed-regime coverage or explicit missing classification
- `generated/external_validation_report.md`
- `generated/evaluator_handoff.md`
- `generated/check_signing_readiness.md`
