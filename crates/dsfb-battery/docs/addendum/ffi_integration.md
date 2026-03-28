# FFI Integration Note

Status: Additive FFI support only. No claim is made of deployment qualification.

The crate already exposes a `staticlib` target and a narrow C ABI in:

- `src/ffi.rs`
- `include/dsfb_battery_ffi.h`

Addendum-specific helper surface:

| Symbol | Purpose |
|---|---|
| `dsfb_battery_default_config` | returns the default DSFB configuration |
| `dsfb_battery_evaluate_grammar_state` | evaluates the grammar state code |
| `dsfb_battery_evaluate_step_status` | returns state, tri-state color code, and reason-code code for a single advisory step |
| `dsfb_battery_run_capacity_summary` | batch summary helper over a capacity sequence |

Color-code mapping:

- `0 = Green`
- `1 = Yellow`
- `2 = Red`

Reason-code mapping:

- `-1 = none`
- `0 = SustainedCapacityFade`
- `5 = AcceleratingFadeKnee`
- other numeric values follow the Rust enum ordering exposed in `src/ffi.rs`

Build example:

```bash
cargo build --release --lib
cc -Iinclude ffi/dsfb_battery_addendum_example.c target/release/libdsfb_battery.a -lm -o ffi_example
```

The example source is at `ffi/dsfb_battery_addendum_example.c`.
