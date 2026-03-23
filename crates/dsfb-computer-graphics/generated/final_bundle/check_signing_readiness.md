# Check Signing Readiness

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

| Axis | Status | Evidence |
| --- | --- | --- |
| Internal artifact completeness | ready for diligence | GPU path present=`true`, external replay present=`true`, region-ROI scenarios=`6` |
| Immediate sign-off | blocked pending external evidence | external validation=`false`, measured GPU timing=`true` |
| External replay | blocked pending external evidence | source kind=`synthetic_compat` |

## What Is Proven

- The remaining blockers are now dominated by external validation needs rather than missing in-repo mechanisms.

## What Is Not Proven

- This report does not claim immediate sign-off without external replay evidence and broader engine-side measurement.

## Remaining Blockers

- Real external captures and imported-capture GPU profiling still gate immediate external sign-off.

## Engine-Native Validation Status (Updated)

| Axis | Status | Classification |
|------|--------|---------------|
| Engine-native infrastructure | complete | **INTERNAL** (resolved) |
| Engine-native schema + manifest | complete | **INTERNAL** (resolved) |
| Engine-native import/replay CLI | complete | **INTERNAL** (resolved) |
| Mixed-regime confirmation (internal) | confirmed | **INTERNAL** (resolved) |
| Validation gates (engine-native) | gated + strict | **INTERNAL** (resolved) |
| Real engine capture provided | **MISSING** | **EXTERNAL** blocker |
| GPU timing on engine-native data | **PENDING** | **EXTERNAL** blocker |
| Demo A/B on engine-native data | **PENDING** | **EXTERNAL** blocker |
| Mixed-regime on engine-native data | **PENDING** | **EXTERNAL** blocker |
| Renderer-integrated sampling | **PENDING** | **EXTERNAL** blocker |
| 4K dispatch (tiling needed) | **PENDING** | **EXTERNAL** env limitation |

## Immediate Sign-Off Classification

**Immediate sign-off is still blocked.**

Blocking reason: no real engine-native capture has been provided.

All internal infrastructure is complete and validated. The single remaining internal-solvable
blocker is resolved. All open gates are now strictly external.

### What would close immediate sign-off
1. Provide one real engine-native capture (see `docs/unreal_export_playbook.md`)
2. Run: `cargo run --release -- run-engine-native-replay --manifest examples/engine_native_capture_manifest.json --output generated/engine_native`
3. Confirm `ENGINE_NATIVE_CAPTURE_MISSING=false` in all reports
4. Run: `cargo run --release -- validate-final --output generated/final_bundle`

That single step closes ENGINE_NATIVE_CAPTURE_MISSING, produces measured GPU timing on real
engine buffers, generates Demo A/B on real data, and makes strict validate-final pass.
