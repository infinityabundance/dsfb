# DSFB Non-Intrusion Manifest

**Read-only audit-readiness declaration for safety-officer review prior to deployment.**

This manifest is **not** a certification document. No certification body currently certifies a "deterministic semiotic engine" for safety-critical loops; pursuing IEC 61508 / ISO 13849 evidence is a Phase III commercialisation activity. This is the operator-facing artefact that supports an internal safety case. It is designed to be signed off by a Safety Officer in under twenty minutes.

---

## What `dsfb-robotics` reads

A single `&[f64]` residual slice per `observe(...)` call. The slice is consumed read-only:

- No upstream callback is registered.
- No controller state is queried.
- No observer / Kalman state is touched.
- No shared memory is mapped.

The framework consumes the residual stream existing observers already produce as a by-product, never the controller's hot path.

## What `dsfb-robotics` writes

- Its own JSON aggregate output (`paper-lock <slug>` stdout, schema-validated against `paper/paper_lock_schema.json`).
- An operator-facing review CSV when invoked with `--emit-review-csv`.

The framework writes to **no** upstream file, **no** controller register, **no** safety chain, **no** watchdog timer.

## Build-time enforcement

| Mechanism | Source |
|---|---|
| `#![forbid(unsafe_code)]` | `src/lib.rs` |
| `#![no_std]` core | `src/lib.rs` |
| `no_alloc` core feature | `Cargo.toml` |
| `lto = true` | `Cargo.toml` `[profile.release]` |
| `codegen-units = 1` | `Cargo.toml` `[profile.release]` |
| `panic = "abort"` | `Cargo.toml` `[profile.release]` |
| Pinned Rust 1.85.1 | `rust-toolchain.toml` |

## Test-time enforcement

| Audit | Status | Evidence path |
|---|---|---|
| Test suite | 191 tests, 0 failures | `cargo test --features std,paper_lock` |
| Checksum regression CI | byte-identical paper-lock JSON across re-runs | `.github/workflows/reproduce.yml` |
| Miri × 3 alias models | clean (stacked-borrows, tree-borrows, no-std-core) | `audit/miri/MIRI_AUDIT.md` |
| Kani | 6 harnesses, all green | `audit/kani/KANI_AUDIT.md` |
| Loom interleavings | observer-non-mutation under thread interleavings | `tests/concurrency_observer.rs` |
| cargo-fuzz | 1 M iterations × 2 targets, 0 crashes | `fuzz/RUN_LOG.md` |
| DSFB-gray assurance | 96.2 % strong assurance posture | `audit/dsfb_robotics_scan.txt` |
| JSON Schema validation | mechanical drift check | `tests/schema_validation.rs` |

## Runtime guarantees

| Quantity | Observed bound | Source |
|---|---|---|
| End-to-end cost per residual sample | ≤ 59.5 ns/sample (worst-case across slate) | `audit/throughput/per_dataset_tails.csv` |
| Median per-sample cost across slate | ~32–35 ns/sample | same |
| Worst-case excursion above p99 | ≤ 2.5 ns/sample | same |

Reported as a **measured Criterion-sample bound**, not a formally bounded WCET. Static-WCET via instrumented `cargo asm` audit is a v1.1 hardening target (paper §Future Work).

## What DSFB MUST NOT touch (architectural commitments)

DSFB is, by design, incapable of writing to any of the following. Any deployment wiring that connects a DSFB output to one of these is **outside the framework's design** and breaks the audit trail:

- The Emergency Stop chain (E-Stop).
- The Safety-Rated Monitored Stop (SMS) circuit.
- Controller torque commands or any actuator-facing register.
- Observer Kalman-filter state.
- Watchdog timers.
- Hardware safety interlocks.

See `docs/DEPLOYMENT_ANTIPATTERNS.md` anti-pattern 5 for the full deployment-time discipline.

## Per-deployment sign-off block

```
DEPLOYMENT
==========
Deployment date:        ____________________
Robot platform:         ____________________
Site / cell:            ____________________
Operator-of-record:     ____________________

PAPER-LOCK BINARY
=================
Binary path:            target/release/paper-lock
SHA-256 (record):       ____________________
Build host:             ____________________
Toolchain:              rustc 1.85.1 (per rust-toolchain.toml)

CALIBRATION
===========
Calibration window:     first 20 % of finite samples
Calibrated ρ:           ____________________
Canonical params:       W=8, K=4, β=0.5, δ_s=0.05
                        (frozen at git tag paper-lock-protocol-frozen-v1)

ARCHITECTURAL COMMITMENTS REVIEWED
==================================
[ ] Read-only contract acknowledged
[ ] No connection to E-Stop, SMS, torque, observer state, or watchdog
[ ] Output piped to human-review channel only (operator dashboard / triage queue)
[ ] Anti-pattern guide reviewed (docs/DEPLOYMENT_ANTIPATTERNS.md)
[ ] Failure-mode analysis attached (paper §13 Failure Modes)

SIGNATURES
==========
Safety Officer:         ____________________   Date: ____________
System Integrator:      ____________________   Date: ____________
```

## How to compute the binary SHA-256

```
cd crates/dsfb-robotics
cargo build --release --bin paper-lock --features std,paper_lock
sha256sum target/release/paper-lock
```

Record the output in the sign-off block above. The binary is bit-identical across architectures in the committed CI matrix; any cross-architecture divergence is itself a falsifier of the determinism claim and should be reported (see paper §Falsifiability).

## What this manifest does not cover

- Site-specific safety analysis (operator's responsibility).
- Upstream observer / controller integrity (separate audit).
- Network / cyber-physical attack on the residual stream (the residual is consumed read-only; spoofing happens upstream of DSFB and is a controller-integrity concern, not a DSFB attack vector).
- ISO 10218-2 / IEC 61508 / ISO 13849 certification (none claimed).

The manifest is not a substitute for safety review; it is a one-page substrate that gives safety review something concrete to work against.
