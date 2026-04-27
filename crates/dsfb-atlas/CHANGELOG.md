# Changelog — `dsfb-atlas`

All notable changes to this crate are recorded here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] — 2026-04-26

### Added
- 10,000-theorem LaTeX generator with SHA-256 proof-uniqueness verification.
- 4-tier empirical-anchor system (T1 dsfb-bank witness / T2 paperstack /
  T3 public dataset / T4 structural-only).
- Bank-id cross-validation against `crates/dsfb-bank/spec/*.yaml`.
- Coverage report and longtable theorem index emission.
- `audit/` folder containing dsfb-gray, Miri, Kani, cargo-fuzz scripts +
  reports + the cargo-fuzz YAML target.
- `CITATION.cff` referencing Zenodo DOI `10.5281/zenodo.19798649`.
- `tests/atlas_end_to_end.rs` integration test covering the byte-determinism
  and proof-uniqueness invariants.
- Function-level decomposition so each helper is ≤ 60 LOC (NASA/JPL P10-4).
- `[lints]` table with `unsafe_code = forbid` and `clippy::pedantic` warns.

### Internal
- `dedup` module gained a Kani proof harness and four unit tests.
- `generator` module decomposed from a 220-line entry point into ten
  single-responsibility helpers.

[2.0.0]: https://github.com/infinityabundance/dsfb/releases/tag/atlas-v2.0.0
