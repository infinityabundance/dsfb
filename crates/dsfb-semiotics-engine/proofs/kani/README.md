# Kani Proof Scope

These harnesses cover selected bounded properties only:

- `TrustScalar::new` always clamps into `[0,1]`
- `TrustScalar::new` does not emit `NaN`
- the exported grammar-reason enum remains a closed typed set

They do not prove the full DSFB engine.

Run locally when `cargo-kani` is installed:

```bash
cargo kani --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --harness proof_trust_scalar_in_unit_interval
cargo kani --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --harness proof_trust_scalar_not_nan
cargo kani --manifest-path crates/dsfb-semiotics-engine/Cargo.toml --harness proof_grammar_reason_enum_is_closed
```
