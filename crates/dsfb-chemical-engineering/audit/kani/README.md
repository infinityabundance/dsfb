# Kani — bounded model-checking of grammar soundness

[Kani](https://model-checking.github.io/kani/) is an open-source **bounded model checker for Rust**: for each
`#[kani::proof]` harness it explores *every* execution over a symbolic (non-enumerated) input domain and proves
the absence of panics, arithmetic overflow, out-of-bounds access, and any explicit `assert!` violation — not on a
sampled set of inputs, but on the whole bounded domain at once via CBMC + a SAT solver.

The harnesses live in `crates/dsfb-chemical-engineering-edge/src/kani_proofs.rs` (a `#[cfg(kani)]` module that is
**never part of a normal build** — it compiles only under the `kani` cfg the model checker sets). They pin down the
core per-sample DSFB grammar: the residual-triple → `AdmissibilityEnvelope` → `GrammarClassifier` path.

Run with: `cargo install --locked kani-verifier && cargo kani setup` (one-time), then
`cargo kani -p dsfb-chemical-engineering-edge`. Full CBMC trace: `kani.txt`; condensed verdict: `kani-summary.txt`.

## Result — 6 of 6 harnesses verified, 0 failures

Tool: `cargo-kani 0.67.0` · CBMC 6.8.0 · CaDiCaL 2.0.0 · bit-precise IEEE-754 float reasoning (16 object bits).

| Harness | Claim | Symbolic domain | Verdict |
|---|---|---|---|
| `proof_origin_triple_is_interior` | envelope monotonicity — origin triple is always interior | fixed origin | **SUCCESSFUL** (0/142) |
| `proof_nonfinite_is_sensor_fault` | NaN/±∞ ⇒ `SensorFault` | non-finite `r` | **SUCCESSFUL** (0/191) |
| `proof_determinism_two_runs_match` | identical input/params ⇒ identical state | finite `r`, `|r|<1e6` | **SUCCESSFUL** (0/191) |
| `proof_interior_finite_is_not_sensor_fault` | interior finite triple ≠ `SensorFault` (Wave-1 B3 soundness) | `|r|<1.0, |δ|<0.05, |σ|<0.05` | **SUCCESSFUL** (0/191) |
| `proof_beyond_bound_is_not_interior` | residual beyond `r_max` is never interior (envelope bounds) | finite `r>4.0` | **SUCCESSFUL** (0/142) |
| `proof_classify_is_total_on_finite` | `classify` total on finite input — no panic/overflow, terminates, returns a state (grammar totality) | `|r|,|δ|,|σ|<1e6` | **SUCCESSFUL** (0/190) |

**1047 verification conditions** (overflow / pointer-bounds / explicit-assert instances) generated across the six
harnesses, **all discharged**; total CBMC solve time ≈ 0.75 s. The per-harness `(0/N)` is "0 of N property checks
failed." See `kani-summary.txt` for the annotated breakdown and `kani.txt` for the raw CBMC proof log.

## What it does NOT certify

- **Bounded magnitude, not unbounded reals.** Kani reasons over the *bit-precise IEEE-754 `f64`* domain, and the
  symbolic inputs are bounded (`|·| < 1e6`, etc.) to keep the model finite. This proves the grammar logic over a
  finite-precision, bounded window — it is **not** a proof over the unbounded mathematical reals/integers. That
  unbounded layer is the job of the **Lean 4** proof (`formal/lean/DsfbGrammar.lean`) and the **Coq/Rocq**
  cross-check (`formal/coq/DsfbGrammar.v`); see also `breadth_surface.toml` claim `FORMAL-01`.
- **Model level, not the shipped binary.** Kani verifies the Rust harness as compiled to its GOTO model. The link
  from this model to the exact behaviour of the released `edge`/`cuda` binaries on real data remains the
  **empirical verify-replay determinism gate** (byte-exact replay), not a machine proof.
- **The per-sample grammar, not the whole pipeline.** These harnesses cover the residual → envelope → classifier
  step. Quorum fusion, episode compression, the heuristics bank, and the court record are covered elsewhere
  (Lean/Coq for the fusion/compression invariants; the test suite + the verify-replay gate for the rest). The
  `cuda` GPU FFI path is out of scope here (Kani interprets no foreign/GPU code — that surface is covered by the
  CPU-reference equivalence tests and the Miri UB run, `audit/miri/`).
- **No floating-point accuracy claim.** Proving "no panic / correct branch" is distinct from bounding rounding
  error; the numerical-stability story is the fixed-point `core` sibling and the replay digests, not Kani.

In short: Kani is the *bounded, exhaustive-over-its-domain* tier of a three-prover stack (Kani + Lean 4 + Coq),
complemented by Miri (UB), clippy, 295 workspace tests, and the verify-replay gate. It is strong evidence of
grammar soundness and totality over the bounded input domain — not a complete correctness or compliance certificate.
