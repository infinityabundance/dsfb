# Demo A — Engine-Native Capture

ENGINE_NATIVE_CAPTURE_MISSING=true

**engine_source_category:** pending

**ROI source:** N/A (pending capture)
**non-ROI evaluation:** pending
**metric_source:** proxy temporal metrics (no renderer ground truth)

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Demo A: PENDING

No real engine-native capture was provided. Demo A evaluation cannot be run.

See `docs/unreal_export_playbook.md` or `docs/unity_export_playbook.md` for export steps.

### Expected Demo A output when capture is provided

| Method | Overall MAE | ROI MAE | Non-ROI MAE | Intervention rate |
|--------|------------|---------|-------------|-------------------|
| fixed_alpha_0.1 | TBD | TBD | TBD | TBD |
| strong_heuristic | TBD | TBD | TBD | TBD |
| DSFB host-minimum | TBD | TBD | TBD | TBD |

## ROI Disclosure

- If an `roi_mask` is not natively exported from the renderer, a derived mask is used.
- Derived masks are labeled `derived-low-confidence` in the import report.
- ROI vs non-ROI metrics are always separated regardless of mask source.

## Trust Mode Summary

- DSFB host-minimum: uses GPU kernel, same as DAVIS/Sintel path
- DSFB host-realistic: uses full profile with all signals
- Proxy metrics: no renderer ground truth is available unless explicitly exported

## What Is Not Proven

- Ground-truth comparison requires explicit renderer reference export
- Engine-native Demo A on real capture is pending
- ROI from native engine mask (most evaluators prefer this) not confirmed

## Remaining Blockers

- **EXTERNAL**: No real engine capture has been provided.
- **EXTERNAL**: Ground-truth reference requires explicit renderer export.
- **EXTERNAL**: Native ROI mask requires explicit renderer export.
