//! # dsfb-chemical-engineering-wasm — the interactive Chemical Court "what-if" simulator
//!
//! A browser tool that **replays a Court Record's residual stream through the DSFB grammar under an
//! operator-amended admissibility envelope** — the Wave-7 "interactive Chemical Court simulator" item.
//! The residual stream is the *immutable evidence*: the operator drags the envelope half-width `k`, the
//! grazing-band fraction, and the drift window, and watches how the **same** evidence is reclassified
//! (which samples become `DriftAccum` / `SlewSpike` / `EnvViolation` / …, how many episodes form). It is
//! a HAZOP / training / forensic what-if instrument — *"if our admissibility band had been this tight,
//! how would the court have read this episode?"* — over evidence that never changes.
//!
//! ## Architecture (why it is built this way)
//! - It compiles to `wasm32-unknown-unknown` and depends ONLY on the dependency-free embedded
//!   [`dsfb_chemical_engineering_core`] crate (the same fixed-point grammar that runs on the Cortex-M3 and
//!   under QEMU). So the *exact* integer grammar an operator could deploy at the edge is the one replaying
//!   in their browser — no second implementation to keep honest.
//! - It uses **raw `extern "C"` exports + hand-written JS glue**, not `wasm-bindgen`: the whole tool is
//!   `cargo build --target wasm32-unknown-unknown --release` plus a static HTML page, with no build-tool or
//!   extra-dependency supply-chain surface — consistent with the auditability ethos of the rest of the stack.
//! - The numeric work is the pure, host-testable [`simulate_into`]; the `extern "C"` wrappers only marshal
//!   the fixed linear-memory buffers. `cargo test` gates the logic on the host even though the UI is the
//!   user's to exercise in a browser.
//!
//! ## What this is NOT (standing DSFB boundaries)
//! - **Not a controller or safety function.** Read-only, advisory; it classifies a residual stream and
//!   counts episodes — no actuation, no SIS / IEC-61511 authority.
//! - **Not bit-identical to the edge float pipeline.** It is the embedded fixed-point sibling grammar (the
//!   edge crate + its replay-hash gate remain the reference); treat the two as the same *grammar*,
//!   calibrated independently — not a cross-checked numeric pair.
//! - **Not an amendment to the sealed record.** What-if envelope changes are sandboxed in the browser; the
//!   evidence stream (and its digest, shown in the UI) is never mutated. The simulator cannot re-seal or
//!   alter a Court Record.
//! - **Not plant data.** The bundled `web/sample_residuals.json` is a clearly-labelled synthetic
//!   demonstrator; real plant data is never redistributed (recipes + digests only, per the project policy).

#![deny(unsafe_code)] // denied crate-wide; lifted only around the documented linear-memory marshalling
                      // below (deny, not forbid, precisely so that one audited block can opt in).

use dsfb_chemical_engineering_core::{DsfbCore, FixedEnvelope, GrammarState, SCALE};

/// Maximum samples the fixed shared buffers hold — one residual stream replayed per call.
pub const MAX_SAMPLES: usize = 8192;

/// Stable per-sample grammar token code (mirrors [`GrammarState`] order) for compact transfer to JS.
/// 0 NOM · 1 DA · 2 SS · 3 EV · 4 BG · 5 RC · 6 CP · 7 SF.
#[inline]
fn token_code(s: GrammarState) -> u8 {
    match s {
        GrammarState::Nominal => 0,
        GrammarState::DriftAccum => 1,
        GrammarState::SlewSpike => 2,
        GrammarState::EnvViolation => 3,
        GrammarState::BoundaryGrazing => 4,
        GrammarState::Recovery => 5,
        GrammarState::Compound => 6,
        GrammarState::SensorFault => 7,
    }
}

/// Summary of one what-if replay over a residual stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimSummary {
    /// Samples processed (`min(residuals.len(), out.len())`).
    pub n: u32,
    /// Number of maximal *episodes*: consecutive runs of non-nominal, non-recovery samples.
    pub episodes: u32,
    /// Samples whose grammar token is not `NOM`.
    pub non_nominal: u32,
    /// Samples in a hard envelope breach (`EnvViolation` / `Compound` / `SensorFault`).
    pub breaches: u32,
}

/// Scale an engineering-unit residual to the fixed-point `i64` the core consumes, rounding half away from
/// zero to match the `SCALE = 1e6` convention (no `std` float intrinsic needed — pure arithmetic).
#[inline]
fn scale(x: f64) -> i64 {
    let v = x * SCALE as f64;
    if v >= 0.0 {
        (v + 0.5) as i64
    } else {
        (v - 0.5) as i64
    }
}

/// Replay `residuals` through a window-`N` core under a symmetric envelope, writing one token code per
/// sample into `out` and accumulating the summary counts. Monomorphised per window size by [`simulate_into`].
fn run_core<const N: usize>(residuals: &[f64], k_scaled: i64, band_scaled: i64, out: &mut [u8]) -> SimSummary {
    let env = FixedEnvelope::symmetric(k_scaled, band_scaled);
    let mut core = DsfbCore::<N>::new(env);
    let (mut episodes, mut non_nominal, mut breaches) = (0u32, 0u32, 0u32);
    let mut in_run = false; // true while inside a maximal non-nominal/non-recovery episode
    let n = residuals.len().min(out.len());
    for (i, &r) in residuals.iter().enumerate().take(n) {
        let (state, _reason) = core.step(scale(r), true);
        out[i] = token_code(state);
        if !state.is_nominal() {
            non_nominal += 1;
        }
        if matches!(state, GrammarState::EnvViolation | GrammarState::Compound | GrammarState::SensorFault) {
            breaches += 1;
        }
        // An episode is a maximal run of "active" states (anything but Nominal and the one-shot Recovery
        // marker). Count a new episode on each rising edge into an active run.
        let active = !state.is_nominal() && state != GrammarState::Recovery;
        if active && !in_run {
            episodes += 1;
        }
        in_run = active;
    }
    SimSummary { n: n as u32, episodes, non_nominal, breaches }
}

/// Replay `residuals` under a symmetric admissibility envelope (half-width `k`, grazing-band fraction
/// `band` in `[0,1)`) at a drift `window` of samples, writing one grammar token code per sample into `out`
/// (truncated to `out.len()`). The window snaps to one of the core's monomorphised sizes {8, 16, 32}.
///
/// Pure and deterministic — this is the simulator's whole behaviour, gated by the host unit tests; the
/// `extern "C"` wrappers below only marshal it to/from the browser.
pub fn simulate_into(residuals: &[f64], k: f64, band: f64, window: usize, out: &mut [u8]) -> SimSummary {
    let k_scaled = scale(k).max(1); // a degenerate k<=0 envelope would flag everything; floor at 1 scaled unit
    let band_scaled = scale(band).clamp(0, SCALE - 1); // band is a fraction of the half-width: [0, 1)
    match window {
        0..=11 => run_core::<8>(residuals, k_scaled, band_scaled, out),
        12..=23 => run_core::<16>(residuals, k_scaled, band_scaled, out),
        _ => run_core::<32>(residuals, k_scaled, band_scaled, out),
    }
}

/// Raw `wasm32` exports + the shared linear-memory buffers (hand-written JS glue; no wasm-bindgen).
///
/// This is the only part of the crate that touches `unsafe` — the FFI symbol exports (`#[no_mangle]`) and
/// the raw-pointer view of the shared buffers — so it opts into `unsafe_code` while the rest of the crate
/// stays `deny(unsafe_code)`. The browser and the module share two fixed buffers in wasm linear memory: JS
/// writes the residual stream into `IN_BUF` (pointer from [`ffi::dsfb_sim_in_ptr`]), calls
/// [`ffi::dsfb_sim_run`], then reads the per-sample token codes from `OUT_BUF` (pointer from
/// [`ffi::dsfb_sim_out_ptr`]). wasm is single-threaded and JS drives this sequence synchronously, so the
/// `static mut` buffers are never aliased or re-entered.
pub mod ffi {
    #![allow(unsafe_code)]
    use super::{simulate_into, MAX_SAMPLES};

    static mut IN_BUF: [f64; MAX_SAMPLES] = [0.0; MAX_SAMPLES];
    static mut OUT_BUF: [u8; MAX_SAMPLES] = [0; MAX_SAMPLES];

    /// Pointer to the input residual buffer (JS writes up to [`dsfb_sim_max_samples`] `f64`s here).
    #[no_mangle]
    pub extern "C" fn dsfb_sim_in_ptr() -> *mut f64 {
        // `addr_of_mut!` (not `&mut`) avoids creating a reference to the static — no aliasing UB.
        core::ptr::addr_of_mut!(IN_BUF) as *mut f64
    }

    /// Pointer to the output token buffer (JS reads `n` `u8` token codes here after [`dsfb_sim_run`]).
    #[no_mangle]
    pub extern "C" fn dsfb_sim_out_ptr() -> *const u8 {
        core::ptr::addr_of!(OUT_BUF) as *const u8
    }

    /// Capacity of the shared buffers, in samples.
    #[no_mangle]
    pub extern "C" fn dsfb_sim_max_samples() -> u32 {
        MAX_SAMPLES as u32
    }

    /// Replay the first `n` residuals in `IN_BUF` under `(k, band, window)`, fill `OUT_BUF` with the
    /// per-sample grammar token codes, and return the episode count. JS derives the non-nominal / breach
    /// tallies from the token stream it reads back (so no wide return type / BigInt is needed on the JS side).
    #[no_mangle]
    pub extern "C" fn dsfb_sim_run(n: u32, k: f64, band: f64, window: u32) -> u32 {
        let n = (n as usize).min(MAX_SAMPLES);
        // SAFETY: single-threaded wasm with synchronous JS drive — JS writes IN_BUF, calls this once, then
        // reads OUT_BUF, with no reentrancy or concurrent access. The raw pointers address the two static
        // arrays and the slices are length `n <= MAX_SAMPLES`, so both are valid, aligned, non-overlapping.
        let (inp, out) = unsafe {
            (
                core::slice::from_raw_parts(core::ptr::addr_of!(IN_BUF) as *const f64, n),
                core::slice::from_raw_parts_mut(core::ptr::addr_of_mut!(OUT_BUF) as *mut u8, n),
            )
        };
        simulate_into(inp, k, band, window as usize, out).episodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Token codes for readable assertions.
    const NOM: u8 = 0;
    const EV: u8 = 3;

    /// A flat zero stream is entirely nominal: no episodes, no breaches, every token NOM.
    #[test]
    fn flat_stream_is_all_nominal() {
        let r = [0.0f64; 64];
        let mut out = [0u8; 64];
        let s = simulate_into(&r, 3.0, 0.1, 16, &mut out);
        assert_eq!(s.n, 64);
        assert_eq!(s.episodes, 0);
        assert_eq!(s.non_nominal, 0);
        assert_eq!(s.breaches, 0);
        assert!(out.iter().all(|&t| t == NOM));
    }

    /// A sustained step beyond the envelope half-width breaches (`EnvViolation`) and forms one episode.
    #[test]
    fn sustained_step_breaches_once() {
        let mut r = [0.0f64; 80];
        for v in r.iter_mut().skip(40) {
            *v = 5.0; // well beyond k = 3
        }
        let mut out = [0u8; 80];
        let s = simulate_into(&r, 3.0, 0.1, 16, &mut out);
        assert_eq!(s.episodes, 1, "one maximal active run after the step");
        assert!(s.breaches >= 30, "the post-step samples are hard EnvViolation breaches");
        assert_eq!(out[0], NOM);
        assert_eq!(out[79], EV, "still breaching at the end of the sustained step");
    }

    /// The what-if lever bites: a tighter envelope flags strictly more (or equal) than a looser one over the
    /// SAME immutable stream — the core property the browser tool demonstrates.
    #[test]
    fn tighter_envelope_flags_at_least_as_much() {
        // A ramp from 0 to ~4 so intermediate samples sit between a tight and a loose half-width.
        let r: Vec<f64> = (0..120).map(|i| i as f64 * 4.0 / 120.0).collect();
        let mut out = [0u8; 120];
        let tight = simulate_into(&r, 2.0, 0.1, 16, &mut out);
        let loose = simulate_into(&r, 3.5, 0.1, 16, &mut out);
        assert!(
            tight.non_nominal >= loose.non_nominal,
            "tighter k must not flag fewer samples (tight={} loose={})",
            tight.non_nominal,
            loose.non_nominal
        );
        assert!(tight.non_nominal > loose.non_nominal, "on a ramp the tighter envelope flags strictly more");
    }

    /// Determinism: identical inputs give byte-identical token streams and summaries across runs.
    #[test]
    fn replay_is_deterministic() {
        let r: Vec<f64> = (0..200).map(|i| ((i * 7 % 11) as f64 - 5.0) * 0.5).collect();
        let (mut a, mut b) = ([0u8; 200], [0u8; 200]);
        let sa = simulate_into(&r, 3.0, 0.1, 16, &mut a);
        let sb = simulate_into(&r, 3.0, 0.1, 16, &mut b);
        assert_eq!(sa, sb);
        assert_eq!(a, b);
    }

    /// The window selector reaches each monomorphised core size without panicking and stays in-bounds when
    /// `out` is shorter than the residual stream (truncation contract).
    #[test]
    fn window_selection_and_truncation() {
        let r = [1.0f64; 50];
        for w in [8usize, 16, 32, 100] {
            let mut out = [0u8; 20]; // shorter than r → must truncate to 20
            let s = simulate_into(&r, 3.0, 0.1, w, &mut out);
            assert_eq!(s.n, 20, "summary counts the truncated length");
        }
    }
}
