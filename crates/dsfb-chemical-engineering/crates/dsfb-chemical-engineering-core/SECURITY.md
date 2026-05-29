# Security — dsfb-chemical-engineering-core

Embedded grammar (no_std, no-heap, fixed-point): the residual triple + ring buffer + admissibility envelope + grammar state machine in scaled integers.

Unsafe posture: #![forbid(unsafe_code)] (no_std, no heap, panic=abort). This crate follows the workspace security policy — see the canonical
[`../../SECURITY.md`](../../SECURITY.md). Machine-readable audits (dsfb-gray, cargo-audit, cargo-geiger,
Miri, …) are in [`../../audit/`](../../audit/). Report vulnerabilities per the repo policy.
