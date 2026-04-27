# Security Policy — `dsfb-atlas`

## Supported versions

| Version | Supported          |
|---------|--------------------|
| 2.0.x   | :white_check_mark: |
| < 2.0   | not applicable     |

## Threat model

`dsfb-atlas` is a **pure-data-pipeline** crate:

- 0 `unsafe` blocks (`unsafe_code = forbid` enforced via `[lints.rust]`).
- 0 FFI imports.
- 0 network access (`std::net`, `tokio::net`, `reqwest`, `hyper`).
- 0 thread spawns (`std::thread::spawn`, `tokio::spawn`, `rayon::*`).
- 0 shell-out (`std::process::Command`).
- 0 `*-sys` crates in the dependency tree.

The threat surface is therefore the input YAML parser (delegated to
`serde_yaml`) and the LaTeX-emitting string buffers. Both are exercised
under `cargo-fuzz` (target `audit/fuzz/fuzz_targets/yaml_part.rs`) and
the SHA-256 dedup invariant is proved in Kani
(`audit/AUDIT.md` §3, harness `dedup_collision_iff_repeated_body`).

## Reporting a vulnerability

Email <licensing@invariantforge.net> with a CVSS v3.1 estimate and a
proof-of-concept where applicable. We aim to acknowledge within 5
business days and to ship a coordinated disclosure within 90 days.

## Audit posture

The full audit posture (dsfb-gray, Miri, Kani, cargo-fuzz) lives at
[`audit/AUDIT.md`](./audit/AUDIT.md). The latest dsfb-gray scan report
lives at
`audit/reports/dsfb-gray-runs/<latest>/dsfb_atlas_scan.txt`.
