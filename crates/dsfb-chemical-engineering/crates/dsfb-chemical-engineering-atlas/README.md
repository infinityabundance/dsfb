# dsfb-chemical-engineering-atlas

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-chemical-engineering/notebooks/dsfb_chemical_engineering_colab.ipynb)

[![dsfb-gray](https://img.shields.io/badge/dsfb--gray-76.1%25-green)](../../audit/dsfb-gray/atlas/) [![unsafe](https://img.shields.io/badge/unsafe-forbidden-brightgreen)](../../audit/cargo-geiger/README.md) [![Miri](https://img.shields.io/badge/Miri-no%20UB-brightgreen)](../../audit/miri/README.md) [![audit](https://img.shields.io/badge/audit-suite-blue)](../../audit/)

*Verification: this `no_std`, `#![forbid(unsafe_code)]` crate runs clean under **Miri** (7/7 tests, no undefined behaviour — see [`audit/miri/`](../../audit/miri/README.md)).*

The **authority** crate for DSFB-Chemical-Engineering. It defines *what chemometric detectors and
process heuristics are allowed to mean* — it does **not** execute anything.

DSFB-Chemical-Engineering separates execution from authority:

| crate | role |
|-------|------|
| `dsfb-chemical-engineering-edge` | execution over process residuals (drift/slew/envelope grammar, detectors, fused episodes, reports) |
| `dsfb-chemical-engineering-cuda` | CUDA acceleration + forensic court (byte-exact, replayable case files) |
| **`dsfb-chemical-engineering-atlas`** | **chemometric detector records + H1–H6 process-heuristic records + validation gates + atlas hashes** |

## What it contains
- `ChemometricDetectorRecordV1` + `DETECTOR_RECORDS` — the 18 *executed* chemometric detectors as
  curated records (primitive family, decision functional, positive/negative witness, fusion axes,
  confuser profile, source refs). The wider prior-art surface stays catalogued in the edge corpus TOML.
- `ChemicalProcessHeuristicRecordV1` + `HEURISTIC_RECORDS` — H1–H6, the process-residual heuristic
  *records*: residual motif, explicit drift/slew/admissibility conditions (as inspectable strings),
  candidate label, and documented false-positive/false-negative modes. The executable predicates stay
  in `edge`; this crate carries only the string schema, so it has no dependency on any execution type.
- `validate()` — deterministic gates (id uniqueness, source-ref presence, H1–H6 presence, FP/FN-mode
  presence, primitive-family spread).
- `hashes` — `detector_atlas_hash_v1`, `chemical_process_heuristics_hash_v1`,
  `challenge_docket_hash_v1`, and the composite `atlas_hash_v1` (pinned by a frozen test).

## Detector selection — no DSFB-compatibility pre-screen
The atlas entries (both the 18 executed and the wider catalogued surface in the edge corpus TOML) were drawn
from the published chemometrics / fault-detection-and-diagnosis literature — classical MSPC (PCA-T², SPE/Q,
EWMA, CUSUM, Shewhart), dynamic/temporal, nonlinear/distributional, and process-structure families — **without
pre-screening any candidate for whether it "works well" under the DSFB grammar.** Which records are *executed*
versus left *catalogued* is determined **only** by the availability of a suitable public dataset and engineering
time, **not** by detector–DSFB compatibility: catalogued-then-promoted detectors (e.g. the Phase-C
`mewma`/`dpca`/`mosum`/`mmd` promotion, 14→18) entered execution on the same terms as the originals. This is
stated explicitly so the executed subset cannot be read as a cherry-pick: DSFB is a read-only layer over whatever
residuals these standard detectors emit, and it reports honestly where they (and therefore it) are blind.

## Properties
- **No execution.** No detector runs here; no residuals are computed.
- **No dependency on `edge`/`cuda`.** `edge` depends on this crate and proves, via a subset gate, that
  every detector and heuristic it executes is catalogued here.
- **`no_std`-capable** (`--no-default-features`); `serde` (Serialize-only) and `strict-validation` are
  opt-in features. `#![forbid(unsafe_code)]`.
- **Deterministic.** Every hash is a pure function of the `const` record tables.

## Non-claims
These records assert no detector-accuracy superiority. Every process-heuristic label is a *candidate*
hypothesis for operator review with documented failure modes, never a proven root cause.

## Citation

If you use DSFB-Chemical-Engineering, please cite:

> de Beer, R. (2026). *DSFB-Chemical-Engineering: Read-Only Residual Semiotics for Chemometrics-Augmented Fault Detection and Diagnosis in Chemical Engineering, with a Deterministic, Byte-Exact, CUDA-Accelerated Forensic Evidence Court* (1.0). Zenodo. <https://doi.org/10.5281/zenodo.20443279>

See [`CITATION.cff`](../../CITATION.cff) for the machine-readable record.
