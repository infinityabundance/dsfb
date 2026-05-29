# Security — dsfb-chemical-engineering-cuda

GPU evidence factory + forensic court: on-GPU SHA-256 + Merkle evidence root, byte-exact to the CPU reference, digest-equivalence-gated.

Unsafe posture: unsafe confined to the CUDA FFI boundary; the CPU-reference path is safe. This crate follows the workspace security policy — see the canonical
[`../../SECURITY.md`](../../SECURITY.md). Machine-readable audits (dsfb-gray, cargo-audit, cargo-geiger,
Miri, …) are in [`../../audit/`](../../audit/). Report vulnerabilities per the repo policy.
