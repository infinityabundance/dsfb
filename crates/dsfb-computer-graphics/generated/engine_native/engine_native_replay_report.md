# Engine-Native Replay Report

ENGINE_NATIVE_CAPTURE_MISSING=true

**engine_source_category:** pending

**external-capable =** true

**pipeline:** same external replay path as DAVIS/Sintel validation

**DSFB mode:** host_minimum + host_realistic (same as external replay)

**GPU kernel:** dsfb_host_minimum (same as synthetic and DAVIS/Sintel)


## Replay Status: PENDING

No real engine-native capture was provided. This report is a pending placeholder.

### Manual command to replay after capture is provided

```bash
cargo run --release -- run-engine-native-replay \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```

## What Is Not Proven

- Renderer-integrated sampling is not proven (proxy allocation only)
- Ground-truth reference comparison requires explicit renderer export

## Remaining Blockers

- **EXTERNAL**: No real engine capture has been provided.
- **EXTERNAL**: Ground-truth reference frames require renderer export.
- **INTERNAL** (resolved): Same pipeline used — no special-case path.
