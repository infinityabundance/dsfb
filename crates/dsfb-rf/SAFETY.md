# Safety Policy — `dsfb-rf`

## Posture

- `#![forbid(unsafe_code)]` at the crate root.
- `#![no_std]` by default; `alloc` and `std` are additive feature flags.
- No `unsafe` blocks anywhere in the `src/` tree.
- No FFI boundary in the library. The optional `hdf5_loader` feature is a
  thin wrapper around `hdf5-metno` used only by `paper-lock` calibration
  tooling (not by `dsfb-rf` as a library consumer).
- No raw pointers, no function pointers, no interior-mutability primitives
  (`RefCell`, `Cell`, `Mutex`, `RwLock`, `OnceCell`) on the hot path.

## Panic & Exit-Time Discipline

The grammar FSM, DSA accumulator, admissibility envelope, Q16.16 fixed-point
math, and sign-tuple paths are panic-free by construction. Kani harnesses
(`src/kani_proofs.rs`, 6 proofs) formally verify:

1. Grammar panic-freedom under arbitrary `SignTuple` input.
2. Severity bound monotonicity.
3. Envelope / judgment consistency.
4. Decimation epoch count bound.
5. Fixed-point resync drift bound.
6. Q16.16 quantize panic-freedom.

The `AdmissibilityEnvelope` theorem (paper Thm 1) bounds the time to
structural exit on the admissible manifold; proof is in Appendix A of
`paper/dsfb_rf_v2.tex`.

## Known Sharp Edges

- `.unwrap()` / `.expect()` appear in demo binaries under `examples/` to
  assert calibration preconditions (healthy-window must be passed before
  `run_stage_iii`). These are outside the library surface and are removed
  by `cargo publish` from the shipped artefact. Library callers must
  propagate the same precondition themselves — it is documented in
  `docs/API_PRECONDITIONS.md`.
- The `paper-lock` binary (behind `paper_lock,hdf5_loader` feature) is a
  calibration harness, not a library entry point. It panics if the
  RadioML 2018.01a GOLD file is missing — this is intentional and scoped
  to single-operator use on local hardware.
- Fixed-point Q16.16 arithmetic uses `saturating_*` on intermediate
  multiplies but `wrapping_*` on the final narrowing step after the
  explicit bound check at `src/q_fixed.rs::quantize`.

## Verification Evidence

- **Formal (Kani):** 6 harnesses, 30-min timeout each, run in CI on
  reference hardware (gate scheduled for v1.0.2).
- **Property testing:** `tests/proptest_invariants.rs` covers
  sign-tuple / grammar-FSM / DSA-bound / envelope-monotonicity /
  Q16.16 round-trip invariants on randomised input.
- **Concurrency exploration:** `tests/concurrency_observer.rs`
  exercises `Send`/`Sync` bounds on engine state.
- **Fuzzing:** `fuzz/fuzz_targets/engine_roundtrip.rs` runs
  cargo-fuzz against the public observer entry point.
- **Cross-target timing:** `.github/workflows/qemu_timing.yml`
  proves per-sample latency bounds on Cortex-M4F, RV32, x86-64.

## Reporting

Report suspected safety-policy violations via the private channel in
`SECURITY.md`.
