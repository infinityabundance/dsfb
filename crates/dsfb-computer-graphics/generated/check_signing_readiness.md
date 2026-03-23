# Check Signing Readiness

| Area | Status | Classification | Notes |
| --- | --- | --- | --- |
| DAVIS prep | ready | external | official DAVIS data mapped into the schema |
| Sintel prep | ready | external | official Sintel data mapped into the schema |
| DAVIS GPU | ready | external | measured_gpu=`true` |
| Sintel GPU | ready | external | measured_gpu=`true` |
| Taxonomy coverage | ready | external | aggregate_status=`complete_or_explicitly_missing` |

## Remaining Blockers

- [internal] mixed_regime_case coverage is partial for davis
- [internal] mixed_regime_case coverage is partial for sintel
- [internal] renderer-integrated sampling validation is still pending
