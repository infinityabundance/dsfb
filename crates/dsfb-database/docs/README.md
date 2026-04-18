# `dsfb-database` documentation

This directory is a reviewer-oriented index into the crate's supporting
documentation. The canonical top-level documents are:

- [README.md](../README.md) — user-facing overview
- [CHANGELOG.md](../CHANGELOG.md) — release notes
- [ARCHITECTURE.md](../ARCHITECTURE.md) — module layout and data flow
- [SAFETY.md](../SAFETY.md) — safety invariants and review surface
- [SECURITY.md](../SECURITY.md) — security policy and threat model

Files under this directory:

- [`evaluation_model.md`](evaluation_model.md) — two-tier evaluation design
  (controlled TPC-DS vs. real-world observation) and what each tier claims.
- [`fingerprint_locks.md`](fingerprint_locks.md) — which tests hash which
  byte streams; what breaks a fingerprint vs. what does not.
- [`residual_channels.md`](residual_channels.md) — construction formulas
  for the five residual classes, with references to the source files.
- [`motif_grammar.md`](motif_grammar.md) — motif state machines, drift/slew
  thresholds, minimum-dwell rules.
