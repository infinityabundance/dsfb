# dsfb-gray assurance audit — DSFB-Chemical-Engineering

`dsfb-gray` (the maintainer's own deterministic Rust crate auditor; `dsfb-assurance-score-v1`) was run on every
crate and the workspace root. Per its own README: **"this score is a broad improvement and review-readiness
target. It is not a compliance certification."** We report the real numbers — not a gamed figure.

## Scores (per-crate scan, `dsfb-scan-crate <crate>`)

| Crate | Baseline | After doc pass | After audit suite + kani/miri marking | Posture |
|---|---|---|---|---|
| `corpus` | 72.2% | 76.1% | **78.6%** | developing but substantial |
| `atlas` | 68.5% | 72.3% | **76.1%** | developing but substantial |
| `py` | 69.5% | 73.4% | **73.4%** | developing but substantial |
| `dsfb-densor-runtime` | — | — | **70.2%** ‡ | developing but substantial |
| `core` | 64.4% | 69.5% | **69.5%** | mixed |
| `edge` | 56.3% | 60.1% | **65.6%** § | mixed |
| `wasm` | 59.6% | 64.7% | **64.7%** | mixed |
| `cuda` | 46.3% | 50.1% | **50.1%** | limited |
| workspace root | — | 50.7% | **47.0%** † | limited |

Artifacts per crate (text report + SARIF + in-toto + DSSE attestation) are in `audit/dsfb-gray/<crate>/`.

**Refresh note (2026-05-27).** After the edge crate grew by five plant-reality / evidence modules (HAZOP guidewords,
basis descriptor, calibration passport, NE 107 adapter, equipment-signature bank — 87 → **92** source files), every
crate was re-scanned. **All seven per-crate scores were unchanged** — `edge` held at **62.6 %** despite +5 files,
because the new modules are documented and bounded (dsfb-gray's fairness rule does not punish a crate for having more
clean code). The per-crate scores are the stable, canonical signal.

§ **Refresh note (2026-05-28).** After the P92–P97 batch, `edge` grew again (92 → **97** source files: the new
`artifact_index.rs` index generator plus the P93 EvidenceKind propagation edits) and its score **rose 62.6 % →
65.6 %** — a genuine move, not gaming: the added code is fully commented and bounded, so dsfb-gray's fairness rule
credited it (the same mechanism that held the score flat on the prior +5-file growth now rewarded a larger,
better-documented increment). ‡ The batch also added a **new sixth workspace member, `dsfb-densor-runtime`** (11
source files, a thin deterministic execution substrate carrying **no chemical / no cross-domain claims**), which
scans at **70.2 %** on first audit. It scores like the `no_std` authority crates for the same honest reason — it is
small, `#![forbid(unsafe_code)]` (0 unsafe sites), and clean. One caveat is **reported, not worked around**: the
skeleton's functions show ~0 assertion density (it is a trait/seal/receipt scaffold, not yet an asserting hot path),
which the rubric flags; we leave it honest rather than pad assertions to chase the checkpoint. The other six per-crate
scores are unchanged.

§ **Refresh note (2026-05-28, P98–P102 batch).** `edge` grew again (97 → **99** source files: the new
`index_court.rs` — `ArtifactIndexCourtV1`/`verify-index` — plus the feature-gated `densor_demo.rs`) and its score
**held at 65.6 %**: the new modules are fully commented and bounded, so the fairness rule neither rewarded nor
punished the increment. The other scores are unchanged. (P102's operator-report columns + the governed A1
evidence re-freeze touched data/golden hashes, not the dsfb-gray surface.)

† The **workspace-root aggregate is scan-scope-sensitive and is not a stable code-quality number** — it walks whatever
`.rs` files exist under the scanned tree at run time, including build-script-generated sources under `target/`. The
earliest 50.7 % run walked **1225** files; the 2026-05-27 clean run walked **135** (first-party-dominated) and scored
**44.0 %**; the 2026-05-28 clean run walks **153** (the new `dsfb-densor-runtime` member plus the edge growth) and
scores **47.0 %**. Each figure scans a *different file set*, so the movement is a **scope difference, not a code
regression** — the aggregate drifts with the source tree by construction. We report the current reproducible number
honestly and direct readers to the **per-crate scores** as the meaningful measure.

## Formal-method & UB evidence is now marked (kani + miri)
With the Kani and Miri audits in place, a re-scan now **marks the real verification evidence** per crate:
- **`edge` — Kani:** the scan detects **32 formal-method signals** from `src/kani_proofs.rs` (`#[kani::proof]` ×6)
  plus the `cfg(kani)` build wiring and the README Kani/fuzz badges; this is the crate's formal-verification surface
  (the 6/6 bounded grammar-soundness harnesses — `audit/kani/`).
- **`core` / `atlas` / `corpus` — Miri:** these `no_std`, `#![forbid(unsafe_code)]` crates are now run clean
  under **Miri** (8/8 + 7/7 + 7/7 unit tests, **no UB** — `audit/miri/`), and the scan marks `miri` as a
  static-analyzer signal in each (Power-of-Ten Rule 10 evidence).

The audit-suite + governance completion raised `edge` (+2.5), `atlas` (+3.8), and `corpus` (+2.5) over the
doc-pass baseline through **genuine documentation only**. The Miri marking added an analyzer signal but did **not**
move the score: that Power-of-Ten checkpoint stays *indeterminate* because it also wants warnings-as-errors
(`#![deny(warnings)]` / `-D warnings`), which we **deliberately do not force** — that is a known build-fragility
anti-pattern, not an improvement, so we report the honest indeterminate rather than game it.

## What the improvement was (honest)
The only change made to raise scores was adding **genuine OSS-hygiene documentation** that a public prior-art
release should have anyway and that dsfb-gray's lifecycle/governance rubric credits by file presence: per-crate
`README.md` (added the one missing, `core`), `SECURITY.md`, `SAFETY.md`, `CHANGELOG.md`, `ARCHITECTURE.md`, a
`docs/` pointer, plus repo-level `SECURITY.md` / `SAFETY.md` / `CONTRIBUTING.md` / `CHANGELOG.md` /
`CODE_OF_CONDUCT.md`. Governance jumped ~23% → ~62% per crate. **No code was changed to chase the score, and
no filler was manufactured purely to game checkpoints.**

## Why the scores are moderate (50–76%), and why that is the honest result
dsfb-gray's rubric is weighted toward NASA/JPL Power-of-Ten + structural discipline (25% + 25%) tuned for
small, no-allocation, heavily-asserted, safety-critical crates. A real 30K-LOC multi-crate project scores
moderately, for *legitimate* reasons we do not hide:
- **`edge`/`cuda` score lowest** — `edge` is a 23K-LOC `std` CPU tool that allocates freely (567 heap-motif
  sites), has 26 functions over 60 lines, and low assertion density; `cuda` carries the GPU FFI `unsafe`
  boundary. These are normal, intentional traits of an execution/acceleration crate, not defects.
- **`atlas`/`corpus`/`core`/`py` score highest** — the `no_std` authority + embedded crates are clean, bounded,
  and `#![forbid(unsafe_code)]`, which is exactly what the rubric rewards. The assurance differential
  (authority/embedded > execution/GPU) is itself a truthful signal.
- **Two honest scanner limitations** (reported, not worked around): (1) the per-crate scan does not resolve
  Cargo **workspace inheritance**, so `license` / `repository` / `homepage` / `rust-version` set at
  `[workspace.package]` read as "not declared" for member crates — we did NOT de-inherit them into each crate
  just to satisfy the scanner; (2) repo-root governance/CI/LICENSE are not seen by a per-crate scan, so the
  workspace-root scan (which aggregates all 30K LOC) scores *lower*, not higher, than the cleaner individual
  authority crates.

## Non-claim
This is a structural, source-visible assurance score — a review-readiness target, **not** a certification of
security, correctness, or runtime behaviour. The complementary machine audits (cargo-audit, cargo-geiger,
Miri, …) are in the sibling folders under `audit/`.
