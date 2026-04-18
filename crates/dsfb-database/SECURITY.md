# Security Policy

`dsfb-database` is a deterministic, read-only observer. It does not modify the
database under observation and does not open network sockets. The threat model
below describes the boundaries we treat as in-scope for security review.

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | yes       |

Only the latest 0.1.x release is actively supported. Prior pre-release tags are
historical.

## Threat Model

### In scope

- Arbitrary untrusted CSV / text / YAML input on the `run` and grammar paths.
  Parsers must not panic, must not allocate unboundedly on adversarial input,
  and must not leak secrets from the host environment into artefacts.
- Arbitrary untrusted `pg_stat_statements` dumps fed to the Postgres adapter.
- Arbitrary untrusted SQL text fed to the SQLShare-text adapter (skeleton
  tokenisation must terminate on pathological input).
- Emitted artefact integrity. Figures and CSVs written to `out/` must be a
  deterministic function of the input stream and the pinned configuration;
  nondeterministic or host-derived content leaking into artefacts is a bug.

### Out of scope

- The underlying DBMS security posture. We read telemetry that the operator
  has already chosen to expose; we do not validate DBMS authentication,
  authorisation, or audit configuration.
- Transitive-dependency CVEs that surface only through features we do not
  enable (tracked via `cargo audit`).
- Side-channel attacks on host-specific timing (the crate uses monotonic
  clocks for control paths and exposes wall-clock time only in labels).

## Reporting a Vulnerability

Email `security@invariantforge.net` with:

1. A minimal reproducing input.
2. The crate version (`CRATE_VERSION`) and Rust toolchain you observed it on.
3. The expected vs. observed behaviour.

Expect an acknowledgement within 5 business days. Coordinated disclosure
window is 90 days from acknowledgement; shorter on operator-impacting issues.

## Hardening Posture

- `#![forbid(unsafe_code)]` is declared at the crate root. Any `unsafe` block
  is a compile error and must be introduced behind a named review request.
- Panics are avoided on normal paths. Residual and grammar code uses
  `Result` propagation; `.expect(...)` is reserved for documented
  mathematical invariants with a comment naming the invariant.
- The scanner harness in `tests/deterministic_replay.rs` pins residual and
  episode streams by SHA-256, so silent drift in parser or grammar logic
  breaks the build.
- CI invokes `cargo clippy -D warnings`, `cargo test --release`, and the
  `dsfb-gray` audit on every PR.
