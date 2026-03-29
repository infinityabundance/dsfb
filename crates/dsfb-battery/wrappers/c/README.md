# C Wrapper Example

This directory contains a minimal static-link example for the existing `dsfb-battery` C ABI.

Files:

- `dsfb_battery_summary_example.c`

Build sketch:

```bash
cargo build --release --lib
cc -Iinclude wrappers/c/dsfb_battery_summary_example.c \
  target/release/libdsfb_battery.a \
  -lm \
  -o dsfb_battery_summary_example
```

The example remains advisory-only and uses the existing narrow ABI from `include/dsfb_battery_ffi.h`.
