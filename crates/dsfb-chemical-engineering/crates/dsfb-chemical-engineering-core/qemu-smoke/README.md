# DSFB-core bare-metal QEMU smoke run

Runs the `no_std`, no-heap, fixed-point [`dsfb-chemical-engineering-core`](..) on an **emulated Cortex-M3**
(QEMU `lm3s6965evb`) to demonstrate it executes on a real MCU target with statically-bounded memory and pure
integer arithmetic.

## Run it
Requires `qemu-system-arm` and the Rust target (`rustup target add thumbv7m-none-eabi`):

```bash
cd crates/dsfb-chemical-engineering-core/qemu-smoke
cargo run --release          # builds for thumbv7m-none-eabi and launches QEMU (runner in .cargo/config.toml)
```

It feeds a fixed scaled-residual sequence (quiet → drift → a +8 spike → recovery) through `DsfbCore::<8>` and
prints the grammar-token sequence + a deterministic checksum over **semihosting**, then exits QEMU.

## Expected output (deterministic)
```
DSFB-core QEMU smoke (Cortex-M3, no_std, fixed-point):
  step 0 r_scaled=100000 -> NOM
  step 1 r_scaled=-100000 -> NOM
  step 2 r_scaled=2500000 -> NOM
  step 3 r_scaled=2500000 -> NOM
  step 4 r_scaled=2500000 -> NOM
  step 5 r_scaled=8000000 -> EV
  step 6 r_scaled=0 -> CP
  step 7 r_scaled=-100000 -> DA
  step 8 r_scaled=0 -> DA
  step 9 r_scaled=50000 -> DA
token-checksum=15794140667410667713
OK
```
(`NOM` nominal · `EV` envelope-violation · `CP` compound drift+slew · `DA` drift-accumulation — the +8 spike at
step 5 breaches the raw-residual envelope; it then lingers in the 8-sample window, keeping the windowed-mean
drift elevated, exactly as the bounded-memory core is designed to behave.)

## Honest scope
This is a **smoke run** — proof the core runs on an emulated MCU with bounded memory and deterministic integer
arithmetic. It is **not** a real-time / worst-case-execution-time (WCET) certification, and the fixed-point
core is **not claimed bit-identical** to the edge crate's float pipeline (which remains the reference, with its
own replay-hash gate). This harness is a standalone workspace, excluded from the host build (it links
`cortex-m-rt` and only builds for `thumbv7m-none-eabi`).
