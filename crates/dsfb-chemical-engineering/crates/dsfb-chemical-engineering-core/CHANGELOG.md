# Changelog — dsfb-chemical-engineering-core

Per-crate notes; the authoritative workspace ledger is [`../../CHANGELOG.md`](../../CHANGELOG.md) and
`PROJECT_PLAN.md`. Versioning is workspace-wide (`0.1.0`); publishing is maintainer-only.

## Unreleased
- Audited by the workspace `audit/` suite (dsfb-gray assurance score + Rust security/UB tools).
- Embedded grammar (no_std, no-heap, fixed-point): the residual triple + ring buffer + admissibility envelope + grammar state machine in scaled integers.
