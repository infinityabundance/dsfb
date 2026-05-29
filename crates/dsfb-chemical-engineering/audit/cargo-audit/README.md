# cargo-audit — RustSec advisory scan

`cargo audit` checks `Cargo.lock` against the [RustSec advisory database](https://github.com/RustSec/advisory-db)
for dependencies with known security vulnerabilities.

## Result
- Advisory DB: **1098** advisories loaded.
- Scanned: **42** crate dependencies (the full workspace lock).
- Verdict: **clean — no known vulnerabilities reported** (`vulnerabilities.found = false`, exit 0). See
  `cargo-audit.txt` / `cargo-audit.json`.

The dependency surface is deliberately small (serde, serde_json, toml, csv, thiserror + their transitive deps);
the `core`/`atlas`/`corpus` crates are dependency-free `no_std`. The count rose 41 → 42 when the new
`dsfb-densor-runtime` member joined the workspace lock — it pulls only `serde` + `sha2`, both already present, plus
one additional transitive crate; the scan stays clean.

## What it does NOT certify
A clean advisory scan means no *known, published* RustSec advisory matches a locked dependency at scan time. It
is not a proof of absence of vulnerabilities, and it does not cover the project's own (first-party) code — that
is the job of the dsfb-gray assurance scan, cargo-geiger (unsafe), Miri (UB), clippy, and the test/proof suite.
