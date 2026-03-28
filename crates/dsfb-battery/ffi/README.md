# Addendum FFI Example

This directory contains additive FFI example material for the `dsfb-battery` crate.

Primary files:

- `dsfb_battery_addendum_example.c`

The actual ABI definitions remain in:

- `include/dsfb_battery_ffi.h`

Build sketch:

```bash
cargo build --release --lib
cc -Iinclude ffi/dsfb_battery_addendum_example.c target/release/libdsfb_battery.a -lm -o ffi_example
```

This example is engineering support material only.
