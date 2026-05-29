# Security policy — DSFB-Chemical-Engineering

## Posture
DSFB-Chemical-Engineering is a **read-only, advisory** evidence layer. It emits no control signal, writes to
no process register, and has **no control or safety-instrumented-function (SIS/SIF) authority**. Removing it
restores the pre-deployment baseline exactly. The `edge`, `atlas`, `corpus`, and `core` crates set
`#![forbid(unsafe_code)]`; `unsafe` is confined to the CUDA FFI boundary (`cuda`) and one audited
linear-memory marshalling block in `wasm` (which sets `#![deny(unsafe_code)]` and lifts it only there).

## Reporting a vulnerability
This is a prior-art / defensive-publication artifact. Report suspected vulnerabilities by opening a private
security advisory on the GitHub repository (`https://github.com/infinityabundance/dsfb/tree/main/crates/dsfb-chemical-engineering`) or by emailing the
maintainer (`riaan@invariantforge.net`). Please include a reproduction and the affected crate/commit.

## Supply chain & reproducibility
Builds are dependency-light; determinism is fixed-point (`SCALE = 1e6`) with `--fmad=false`, giving byte-exact
replay (`verify-replay` 6/6) and 20/20 frozen Court Record bundle roots. The `audit/` directory holds the
machine-readable security/assurance reports (dsfb-gray, cargo-audit, cargo-geiger, Miri, …). The
`release-scrub --archive-dir` court fails a release archive that leaks private backups.
