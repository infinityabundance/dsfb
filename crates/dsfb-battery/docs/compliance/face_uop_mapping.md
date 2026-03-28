# FACE UoP Mapping

Status: Partial

This file is a crate-local mapping artifact only. It does not claim FACE conformance certification.

Relevant crate components:
- `src/lib.rs`
- `src/detection.rs`
- `src/ffi.rs`
- `include/dsfb_battery_ffi.h`

Traceability matrix:

| FACE-oriented concern | Crate component | Evidence | Status | Notes |
|---|---|---|---|---|
| Portable algorithm boundary | `src/detection.rs`, `src/math.rs`, `src/types.rs` | Deterministic residual, drift, slew, and grammar-state logic | Partial | Core boundary is separable, but no formal FACE packaging metadata is emitted |
| Stable interface surface | `src/ffi.rs`, `include/dsfb_battery_ffi.h` | Narrow C ABI wrapper | Partial | Conservative integration surface exists, but no FACE IDL or transport profile is provided |
| Transport independence | `src/audit.rs::InterfaceContract.protocol_independent` | Audit contract explicitly states protocol independence | Partial | Mapping exists at artifact level only |
| Unit-of-portability style modularity | `src/lib.rs` exports and `std`/core separation | Core/host boundaries are explicit | Partial | No FACE profile verification is claimed |
