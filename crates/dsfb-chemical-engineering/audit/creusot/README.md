# Creusot — deductive verification of Rust (RUN to Coma IR; SMT step needs creusot's hermetic toolchain)

[Creusot](https://github.com/creusot-rs/creusot) translates Rust MIR to WhyML/Coma and discharges
**functional-correctness verification conditions** with Why3 + SMT. Annotate with `#[requires]`/`#[ensures]`; it
proves the contract for **all** inputs (unbounded — unlike Kani's bounded model checking).

## What was actually done (2026-05-27) — installed, harness written, real Coma IR generated
The full stack was installed in this sandbox and creusot was driven to a real artifact:
```fish
opam install -y why3 z3 why3find                    # provers/manager
cargo install --git https://github.com/creusot-rs/creusot cargo-creusot
rustup toolchain install nightly-2026-04-21 -c rustc-dev -c llvm-tools
# build creusot's custom rustc driver (the pinned-nightly part creusot-install failed to place):
cargo +nightly-2026-04-21 build --release -p creusot-rustc   # => target/release/creusot-rustc
# provers creusot pins, into its bin dir:
#   alt-ergo 2.6.2, z3 4.15.3, cvc4 1.8, cvc5 1.3.1 (downloaded from the upstream releases)
```
A `classify_axis` harness with the documented preconditions was written ([`harness_lib.rs`](harness_lib.rs)):
```rust
#[requires(lo@ < hi@)]
#[requires(0i64@ <= band_scaled@ && band_scaled@ < SCALE@)]
pub fn classify_axis(v: i64, lo: i64, hi: i64, band_scaled: i64) -> CoordClass { /* verbatim core port */ }
```

**Result: `cargo creusot --only coma` succeeds — creusot-rustc translated our `classify_axis` to Coma IR**
([`classify_axis.coma`](classify_axis.coma)). The IR is the real thing: it imports `creusot.int.Int128` (the
`i128`-promotion the overflow-safety rests on), carries the contract spans, the `t_CoordClass` type, the `SCALE`
constant, and `meta "compute_max_steps"`. This is the creusot-specific hard part working on our actual algorithm.

## The precise remaining boundary (the SMT prove step)
`cargo creusot` (full) refreshes `why3.conf` (all four provers detected ✓) but the **SMT prove step** fails: it
invokes creusot's pinned **`why3find` fork (git-eab37557)**, and the upstream `why3find 1.3.0` I have lacks its
`--no-autodetect-provers` flag. Building that fork in turn needs creusot's **`why3` fork (git-2c0f2992)** — its API
differs from opam why3 1.8.2 (`Pmodule.pmodule0`, `Term.t_loc`). I.e. creusot 0.12-dev requires its **own hermetic
opam switch** (forked why3 + why3find), which `creusot-install` builds — and that `opam switch create` step failed on
this CachyOS host. So: translation to Coma works on our code; discharging the VCs needs that hermetic switch.

## Where it attaches to DSFB
`core::classify_axis` — proving (unbounded) that the `i128`-promoted comparison is total/overflow-free and the
`Outside/Grazing/Interior` partition is exhaustive under the documented preconditions — the all-inputs form of the
85.6M-exec cargo-fuzz result. The Coma IR above is the input to that proof.

## What it does NOT certify
Creusot proves the **annotated contracts**, nothing more. The Coma IR is the verification *input*, not a discharged
proof — that last step awaits the hermetic toolchain. Floating-point and the CUDA path are out of scope.
