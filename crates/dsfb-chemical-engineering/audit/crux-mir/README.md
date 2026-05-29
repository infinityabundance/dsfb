# crux-mir — symbolic execution of Rust MIR (BUILT + RUN; `Overall status: Valid`)

[`crux-mir`](https://github.com/GaloisInc/crucible/tree/master/crux-mir) (Galois) symbolically executes Rust **MIR**
on the [Crucible](https://github.com/GaloisInc/crucible) engine + `what4` + SMT, discharging `#[crux::test]` symbolic
tests. It explores all paths over a symbolic domain via a **different engine** than Kani (CBMC) — so running both is
**cross-engine corroboration**: agreement between two independent symbolic executors is stronger than either alone.

## What was actually done (2026-05-27) — built from source + ran, real verdict
The full Haskell toolchain + crux-mir were built in this sandbox (a substantial build):
```fish
ghcup install ghc 9.6.7; ghcup install cabal               # GHC + cabal
git clone --recurse-submodules https://github.com/GaloisInc/crucible
cd crucible; cabal update; cabal build exe:crux-mir         # builds crucible + what4 + crux-mir
# mir-json (emits the MIR JSON crux-mir consumes), for crux-mir's pinned nightly-2025-09-14:
rustup toolchain install nightly-2025-09-14 -c rustc-dev,rust-src
cd dependencies/mir-json; cargo +nightly-2025-09-14 install --path . --locked
mir-json-translate-libs                                     # -> rlibs/ (translated std, 58 libs)
export CRUX_RUST_LIBRARY_PATH=$PWD/rlibs
cargo crux-test -- --solver z3                              # run, using z3 as the SMT backend
```
Installed: crux-mir **0.12.0.0.99** (Crux 0.9.0.0.99), mir-json, GHC 9.6.7.

**Result on the DSFB grammar-core invariant harness** ([`harness_lib.rs`](harness_lib.rs), `run_verdict.txt`):
```
[Crux] Goal status:  Total: 4   Proved: 4   Disproved: 0   Incomplete: 0   Unknown: 0
[Crux] Overall status: Valid.
```
crux-mir symbolically executed two `#[crux::test]`s over **all** `i64` symbolic inputs and **proved all 4 goals
Valid**: (1) the documented `lo < hi` precondition implies a strictly-positive axis width, and (2) a valid grazing
band's complement stays in `(0, SCALE]`. This is the **second-engine (Crucible) corroboration** of the Kani proofs.

## Honest scope of this result
- The **full `classify_axis`** harness (the `i128`-promoted nonlinear comparison) is also symbolically *executable*
  here — crux-mir compiled it via mir-json and invoked z3 — but its verification conditions are **nonlinear
  128-bit-bitvector multiplications**, which **time out** in any SMT solver (z3 included): BV nonlinear `mul` is hard
  regardless of value bounds (it's the bit-width, not the range, that defeats the solver). So the *linear* core
  invariants are proven Valid here; the nonlinear i128 overflow-safety is covered instead by the bounded Kani proof +
  the 85.6M-exec cargo-fuzz campaign (and is the target for the Creusot deductive proof). This is an honest
  solver-capacity boundary, not a crux-mir or DSFB defect.
- Like Kani, crux-mir is **bounded symbolic execution** — loop/recursion bounds and solver timeouts apply (a timeout
  is *unknown*, not *disproven*). It verifies the MIR model, not the compiled binary or the CUDA path.

## What it does NOT certify
A `Valid` verdict proves the discharged goals over the symbolic domain explored; it is not a complete-correctness or
compliance certificate. Its value here is **engine diversity** (a second independent prover agreeing with Kani).
