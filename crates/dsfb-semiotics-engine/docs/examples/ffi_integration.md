# FFI Integration Example

The crate ships a nested FFI crate at `ffi/` for legacy-host experimentation.

## Build And Test

```bash
cargo test --manifest-path crates/dsfb-semiotics-engine/Cargo.toml -p dsfb-semiotics-engine-ffi
```

The checked-in header is:

- `ffi/include/dsfb_semiotics_engine.h`

The minimal examples are:

- `ffi/examples/minimal_ffi.c`
- `ffi/examples/minimal_ffi.cpp`

## What The ABI Exposes

The C ABI is intentionally small:

- create engine handle
- destroy engine handle
- push residual sample
- query current status
- reset engine

The queried status includes:

- bounded history size
- residual, drift, and slew norms
- grammar state
- grammar reason
- semantic disposition

This surface is intended for bounded online replay and simple interoperability experiments. It is not a field-validation or certification statement.
