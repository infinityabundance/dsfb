# Security policy

## Supported versions

`dsfb-robotics` is currently in its scaffold phase (`0.1.x`). Security
fixes are applied to the latest `0.x` release. Once `1.0.0` ships, the
latest `1.x` minor and the previous minor will receive security fixes
for 12 months.

| Version | Supported |
|---|---|
| `0.1.x` | Yes (current) |
| `< 0.1` | No |

## Reporting a vulnerability

Please report security issues privately to `security@invariantforge.net`.

- Expect an acknowledgement within 72 hours.
- We will share a draft fix timeline within 7 working days.
- Credit in the changelog is provided unless you request otherwise.

Do **not** open public GitHub issues for security-sensitive reports.

## Scope

The following are considered in scope for security reports:

- Memory-safety violations in the crate (must not be possible; the crate
  is `#![forbid(unsafe_code)]`, but please report any construction that
  circumvents this).
- Logic errors in the `observe()` API that cause it to write past its
  `&mut [Episode]` output buffer (must not be possible by construction;
  the function is bounded).
- Cargo-deny licence-allowlist violations in transitive dependencies.
- Incorrect SHA-256 checksums in `data/slices/SLICE_MANIFEST.json`
  that point to tampered dataset slices.
- Any path by which a malicious upstream residual stream could cause
  DSFB to modify upstream state (must not be possible; the public API
  takes `&[f64]`, not `&mut [f64]`, and the crate is observer-only).

The following are **out of scope**:

- Denial-of-service via unbounded input sizes — the `observe()`
  signature bounds output to `out.len()`, so the caller controls cost.
- Upstream dataset providers' security posture (CWRU, NASA, IEEE PHM,
  MIT Biomimetics, IIT, etc.) — please report those upstream.
- LaTeX-build reproducibility issues — those are tracked as regular
  bugs, not security issues.

## Non-intrusion guarantee

The DSFB observer contract is cryptographic-strength in type:

```rust
pub fn observe(residuals: &[f64], out: &mut [Episode]) -> usize;
```

The `&[f64]` type binds the borrow-checker to prove at compile time
that the function cannot mutate upstream residual state. Any change to
this signature is a breaking, major-version change and will be
documented in `CHANGELOG.md` alongside a formal threat-model review.

## Supply-chain posture

- All direct dependencies are optional and feature-gated (`serde`,
  `serde_json`, `csv`). The default build has **zero** runtime
  dependencies.
- `cargo-deny` is enforced in CI; see `deny.toml`.
- No `build.rs` in the crate (eliminates build-time code execution).
- No proc-macro crate dependencies in the default build.
