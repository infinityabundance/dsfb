# `dsfb-rf` Audit Folder

Static source-visible audit artefacts produced by **dsfb-gray**, DSFB's
locked-rubric code-quality and review-readiness scanner.

[![DSFB Gray Audit: 91.4% strong assurance posture](https://img.shields.io/badge/DSFB%20Gray%20Audit-91.4%25-brightgreen)](./dsfb_rf_scan.txt)

Scanner: [dsfb-gray](https://crates.io/crates/dsfb-gray) · Docs: <https://docs.rs/dsfb-gray>

---

## Posture

This folder is **not a certification**. It is a structured guideline for
improvement and internal review readiness. DSFB does not certify compliance
with IEC, ISO, RTCA, MIL, NIST, or any other standard. The report is a
source-visible structural audit of the shipped crate — evidence for
reviewers, not evidence for a certifier.

---

## Headline Result

| Crate | Version | Scanner version | Overall |
|-------|---------|-----------------|---------|
| `dsfb-rf` | 1.0.0 | `dsfb-assurance-score-v1` | **91.4 %** (strong assurance posture) |

Source SHA-256 (scanned tree): `5b79b4fe21a0335afb3fce3be20134d10dc8471ff9635d4f308cc63301de383f`
Scan generated: `2026-04-22T20:20:49Z`

### Section breakdown

| Section                       | Score  | Weight | Points |
|-------------------------------|--------|--------|--------|
| Safety Surface                | 100.0% |  15.0  |  15.0  |
| Verification Evidence         | 100.0% |  15.0  |  15.0  |
| Build / Tooling Complexity    | 100.0% |  10.0  |  10.0  |
| Lifecycle / Governance        | 100.0% |  10.0  |  10.0  |
| NASA/JPL Power of Ten         |  70.0% |  25.0  |  17.5  |
| Advanced Structural Checks    |  95.7% |  25.0  |  23.9  |
| **Overall**                   | **91.4 %** | 100.0 | **91.4** |

### Advisory broad subscores

| Subscore                     | Score% |
|------------------------------|--------|
| Correctness                  |  83.3  |
| Maintainability              |  94.4  |
| Concurrency / Async          |  75.0  |
| Resource Discipline          |  58.3  |
| Verification / Reviewability |  94.4  |
| Assurance / Provenance       |  94.3  |

---

## What's in this folder

| File | Purpose |
|------|---------|
| [`dsfb_rf_scan.txt`](./dsfb_rf_scan.txt) | Human-readable canonical report — the primary artefact |
| [`dsfb_rf_scan.sarif.json`](./dsfb_rf_scan.sarif.json) | SARIF 2.1.0 findings (viewable in GitHub, VS Code, etc.) |
| [`dsfb_rf_scan.intoto.json`](./dsfb_rf_scan.intoto.json) | in-toto statement wrapping the report (predicateType: `dsfb-gray/scan-report/v1`) |
| [`dsfb_rf_scan.dsse.json`](./dsfb_rf_scan.dsse.json) | Unsigned DSSE envelope around the in-toto statement (sign with `DSFB_SCAN_SIGNING_KEY`) |

---

## What the scanner checks

Scoring method: weighted checkpoint scoring.

- **Safety (15 %)** — unsafe surface, panic sites, FFI boundary count, `forbid(unsafe_code)`, SAFETY justifications.
- **Verification (15 %)** — tests present, property testing, fuzzing, concurrency exploration, regression fixtures.
- **Build / Tooling (10 %)** — pinned toolchain, lockfile presence, clippy/deny configuration, MSRV, CI visibility.
- **Lifecycle (10 %)** — LICENSE, CHANGELOG, SECURITY policy, CODEOWNERS, deprecation stance, release cadence signals.
- **NASA/JPL Power of Ten (25 %)** — P10-1…P10-10 as Rust-flavoured checks.
- **Advanced Structural (25 %)** — 23 motif checks (dynamic loading, global shared state, iterator unboundedness, serialization-growth motif, flow-control motif, allocation-growth motif, clock drift, etc.).

Fairness rule: each checkpoint contributes once, so large crates are not
penalised for having more code. Informational signals (motif match counts,
hotspot counts, capability flags) are reported but excluded from the score
denominator.

---

## Open findings (why we are not at 100 %)

Each item below is an honest, source-visible structural signal — not a
runtime defect. The scanner cannot mechanically prove absence in every case,
so some items remain `indeterminate` even where the underlying pattern is
intentional.

### 1. P10-3 — heap allocation during load path (`not applied`)

`hdf5_loader.rs` uses `Vec::with_capacity(...)` and `format!(...)` on the
dataset ingestion / error-formatting path. The core observer (`engine.rs`,
`grammar.rs`, `dsa.rs`, `envelope.rs`, `sign.rs`, `pipeline.rs`) is
array-backed and allocation-free. The scanner cannot distinguish
initialization-only allocation from steady-state allocation; the flag
remains `not applied` on that ambiguity.

*Context for reviewers.* Core runtime is `#![no_std]` with
`extern crate alloc` for the `Vec` return type on a single deserialisation
helper. Bare-metal targets (`thumbv7em-none-eabihf`,
`riscv32imac-unknown-none-elf`) build cleanly with `--no-default-features`
and never pull the HDF5 path.

### 2. P10-5 — assertion-density average below 2/function (`not applied`)

The crate has 489 non-test functions. The scanner's raw-regex
assertion-density estimator reports 0.05 per function; the rule requires
an average of ≥ 2.0. Hitting the threshold would require adding roughly
~950 debug assertions — nearly every flagged function is pure, total, and
type-checked at compile time, so blanket assertion injection would be
rubric-gaming rather than genuine invariant strengthening. The production
hot path (`observe`, grammar/DSA/envelope/sign) is covered by 360
unit + integration tests. We accept this checkpoint as not-applied in
exchange for avoiding cosmetic assertion spam.

### 3. P10-7 — return-value propagation (`indeterminate`)

No unchecked-return motifs were observed, but the scanner cannot
mechanically prove full return-value propagation across 68 files; it
therefore remains indeterminate. No open work item.

### 4. P10-8 — conditional-compilation fork count (`indeterminate`)

9 review-relevant `cfg` sites (alloc / serde / paper_lock / hdf5_loader /
std / experimental feature gates). Above the "≤4 sites → Applied"
threshold, below the "≥13 sites or any macro_rules! → NotApplied"
threshold. All gates are documented in the top-level `lib.rs` feature
header. Collapsing them further would mean removing public feature
surface, which would be a functional regression.

### 5. PLUGIN-LOAD — `libloading` in Cargo.lock (`elevated`)

Transitive dependency via `hdf5` → `libloading` for optional runtime
loading of the HDF5 shared library. Not reachable under default-features
builds; only present when the `hdf5_loader` feature is enabled. The flag
is retained because the dependency is technically present in the lockfile.

### 6. H-SERDE-01, H-GRPC-01, H-ALLOC-01, H-CLOCK-01 (matched motifs)

Heuristic motifs — **matched, not elevated**. These are source-visible
patterns that surface adjacent to `hdf5_loader.rs` (the one allocation
site) and output/timestamp helpers. They are structural reviewer prompts,
not defects. See the remediation guide inside
[`dsfb_rf_scan.txt`](./dsfb_rf_scan.txt) for the text the scanner emits
in each case.

---

## Reproducing this scan

Pin the scanner from crates.io:

```bash
cargo install dsfb-gray
dsfb-scan-crate --out-dir ./audit crates/dsfb-rf
```

Or from the dsfb monorepo:

```bash
cargo +nightly build --release --manifest-path crates/dsfb-gray/Cargo.toml
./target/release/dsfb-scan-crate --out-dir /tmp/scan crates/dsfb-rf
```

The scan is deterministic given the same source tree. The SHA-256 at the
top of the report pins the exact input.

---

## Reading the SARIF output

```bash
# GitHub code-scanning upload (optional)
gh api -X POST /repos/:owner/:repo/code-scanning/sarifs \
  -f commit_sha="$GIT_SHA" -f ref=refs/heads/main \
  -f sarif="$(gzip -c audit/dsfb_rf_scan.sarif.json | base64 -w0)"
```

VS Code users: install "SARIF Viewer" and open
`audit/dsfb_rf_scan.sarif.json` for an interactive findings pane wired to
source locations.

---

## Paper reference

See paper Appendix K — *dsfb-gray Audit Report Summary* — for the academic
write-up of this scan and its role in the crate's review posture.

---

## Non-certification statement

DSFB does not certify compliance with IEC 61508, ISO 26262, RTCA DO-178C,
MIL-STD-882E, NIST 800-53, or any other standard. This folder carries a
structured self-audit produced by a locked-rubric static scanner; the
score is a broad improvement target, not a compliance artefact.
