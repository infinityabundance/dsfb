# DSFB-RF Non-Intrusion Contract

**Invariant Forge LLC — Prior Art under 35 U.S.C. § 102**

## Formal Contract

The DSFB-RF engine implements the mapping O: R → E where R is the IQ residual
norm sequence (scalar f32 copies from upstream) and E is the advisory episode set
{Silent, Watch, Review, Escalate}*.

No element of E has a type that can be assigned to any upstream RF data structure.

## Type-System Enforcement

```rust
pub fn observe(&mut self, residual_norm: f32, ctx: PlatformContext) -> ObserveResult
```

- `residual_norm: f32` — a copied scalar, not a reference into upstream data
- `ctx: PlatformContext` — a copied struct
- `#![forbid(unsafe_code)]` in lib.rs — compiler-enforced, every build

## Operational Contract

Remove DSFB. The upstream receiver behaves identically to the pre-DSFB state.
No calibration, reconfiguration, or restart required.

*licensing@invariantforge.net*
