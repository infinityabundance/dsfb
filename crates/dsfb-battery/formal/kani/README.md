# Kani Proof Harnesses

These harnesses cover a small, honest subset of the DSFB core logic. They are provided as addendum scaffolds and do not claim whole-crate formal verification.

Covered areas:

- persistence counter update rule
- grammar-state result remains within the declared enum
- repeated threshold evaluation is deterministic for the same bounded inputs

Expected command if Kani is available:

```bash
cargo kani --manifest-path crates/dsfb-battery/Cargo.toml --lib --no-default-features --features alloc
```

The harness source is included through `src/lib.rs` only when `cfg(kani)` is set, so normal cargo builds are unaffected.
