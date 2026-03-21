# FFI Integration Example

The crate ships a nested FFI crate at `ffi/` for legacy-host experimentation and bounded online
integration.

## Build Artifacts

Build the FFI crate in release mode:

```bash
cargo build --manifest-path crates/dsfb-semiotics-engine/Cargo.toml \
  -p dsfb-semiotics-engine-ffi \
  --release
```

The crate is configured to emit:

- `cdylib`
- `staticlib`
- `rlib`

under the usual Cargo target directory, for example:

- `target/release/libdsfb_semiotics_engine_ffi.so`
- `target/release/libdsfb_semiotics_engine_ffi.a`

The checked-in header is:

- `ffi/include/dsfb_semiotics_engine.h`

The minimal examples are:

- `ffi/examples/minimal_ffi.c`
- `ffi/examples/minimal_ffi.cpp`

## ABI Surface

The C ABI is intentionally small and code-oriented:

- create engine handle
- destroy engine handle
- push one residual sample
- query the current status snapshot
- query trust scalar directly
- copy current syntax / grammar / semantic labels into caller-owned buffers
- copy the last error string into a caller-owned buffer
- reset the engine

`DsfbCurrentStatus` is the numeric machine interface. It carries:

- bounded history size and current live occupancy
- residual, drift, and slew norms
- `syntax_code`
- `grammar_state`
- `grammar_reason`
- `semantic_disposition`
- `trust_scalar`

Human-readable strings are optional convenience helpers. The ABI keeps them out of the struct and
copies them into caller-owned buffers through dedicated functions. That keeps ownership rules
boring:

- the caller allocates the buffer
- DSFB writes a NUL-terminated string into it
- `DSFB_FFI_BUFFER_TOO_SMALL` means the string was truncated
- `dsfb_semiotics_engine_last_error_length()` reports the required buffer size for the last error

## Ownership And Calling Conventions

- handles are created by `dsfb_semiotics_engine_create(...)`
- handles are released by `dsfb_semiotics_engine_destroy(...)`
- null handles and null output pointers return numeric error codes rather than panicking
- the last error string is global to the FFI layer and can be copied after a failure
- the live engine uses bounded online history only; offline artifact accumulation is not required at the ABI boundary

## Stepwise Loop

The C and C++ examples both use the intended deployment pattern:

1. create the bounded engine handle
2. push one sample
3. query status codes plus human-readable labels
4. print or log the result
5. repeat

This is a candidate integration surface for downstream trust-aware gating or operator telemetry. It
is not a field-validation or certification statement.
