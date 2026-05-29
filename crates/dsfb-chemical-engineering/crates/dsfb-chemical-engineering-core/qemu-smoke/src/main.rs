//! Bare-metal smoke test: run the `no_std` DSFB core on an emulated Cortex-M3 (QEMU `lm3s6965evb`).
//!
//! Feeds a fixed scaled-residual sequence through [`DsfbCore`] and prints the grammar-token sequence plus a
//! deterministic checksum over semihosting, then exits QEMU. This demonstrates the core *executing on an MCU*
//! with statically-bounded memory and pure integer arithmetic. Honest scope: a smoke run, **not** a
//! real-time / WCET certification.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};
use dsfb_chemical_engineering_core::{DsfbCore, FixedEnvelope, SCALE};
use panic_halt as _;

#[entry]
fn main() -> ! {
    // Symmetric envelope for a zero-centred residual of scale k = 3, with a 10% grazing band.
    let env = FixedEnvelope::symmetric(3 * SCALE, SCALE / 10);
    let mut core = DsfbCore::<8>::new(env);

    // A fixed scenario in milli-units (×1000): quiet → sustained drift at +2.5 → a +8 spike → recovery.
    let seq_milli: [i64; 10] = [100, -100, 2500, 2500, 2500, 8000, 0, -100, 0, 50];

    let _ = hprintln!("DSFB-core QEMU smoke (Cortex-M3, no_std, fixed-point):");
    let mut checksum: u64 = 0;
    for (i, &m) in seq_milli.iter().enumerate() {
        let r = m * (SCALE / 1000); // milli-units → scaled fixed-point
        let (state, _reason) = core.step(r, true);
        let tok = state.token();
        let _ = hprintln!("  step {} r_scaled={} -> {}", i, r, tok);
        for b in tok.bytes() {
            checksum = checksum.wrapping_mul(31).wrapping_add(b as u64);
        }
    }
    let _ = hprintln!("token-checksum={}", checksum);
    let _ = hprintln!("OK");

    debug::exit(debug::EXIT_SUCCESS);
    loop {}
}
