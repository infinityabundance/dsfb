# Kani proof harnesses for `dsfb-database`

This directory contains formal-method proof harnesses verified with the
[Kani Rust Verifier](https://github.com/model-checking/kani). The proofs
target the *pure*, data-free pieces of the motif grammar — the parts
where unbounded symbolic reasoning is tractable and where a bug would
silently corrupt every downstream episode.

## Running

Install kani (`cargo install --locked kani-verifier && cargo kani setup`)
and run from the crate root:

```bash
cargo kani --manifest-path crates/dsfb-database/Cargo.toml \
          --harness envelope_classify_total \
          --harness envelope_boundary_dominates \
          --harness envelope_stable_below_both \
          --harness envelope_threshold_monotonicity
```

Each harness is a bounded model-checking proof; the default unwind
bounds are sufficient because the targets are loop-free.

## Files

- `kani_envelope.rs` — total-function, dominance, and monotonicity
  properties of `grammar::envelope::classify`.
- `kani_motif_state.rs` — transition-soundness properties of
  `grammar::motifs::MotifState::advance` (no Stable → Recovering direct
  transition; dwell is enforced before an episode emits).

## Scope

Kani does *not* replace the unit tests, property tests, or fingerprint
locks — it complements them. The proofs cover *logical* correctness of
the envelope classifier and the motif state machine on **all**
finite-float inputs; the other harnesses cover integration correctness
on real datasets.

Proof harnesses are gated with `#[cfg(kani)]` so ordinary `cargo build`
and `cargo test` ignore them entirely.
