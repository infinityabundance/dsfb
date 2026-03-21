# MOSA / SOSA Compatibility Note

This crate does **not** claim formal MOSA or SOSA certification or compliance.

What it does claim, conservatively:

- a narrow C ABI boundary via [`ffi/include/dsfb_semiotics_engine.h`](../ffi/include/dsfb_semiotics_engine.h)
- an opaque-handle design (`EngineHandle *`) for low coupling across component boundaries
- `#[repr(C)]` enums and structs on the exported status surface in the FFI crate
- additive wrapper layers, such as [`ffi/include/dsfb.hpp`](../ffi/include/dsfb.hpp), that do not
  change the underlying component boundary

Why that is compatible in spirit with modular architectures:

- the deterministic engine logic can sit behind a stable software-component interface
- callers do not need to know Rust internals or memory layout beyond the checked-in header
- the bounded online path and artifact path remain separable

What is **not** being claimed:

- formal MOSA conformance
- formal SOSA conformance
- procurement or certification status

The practical point is architectural compatibility and low-coupling interface design, not a false
claim of formal standard compliance.
