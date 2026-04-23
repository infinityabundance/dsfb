# `dsfb-rf` — SBIR Readiness

*Companion to `paper/dsfb_rf_sbir_trl_v1.tex`. Single source of truth for
TRL posture, Phase I risk-burn-down milestones, hardware roster, and
open-topic mapping.*

This document is a **posture and planning artefact**; it is not a
certification, not a contract instrument, and does not claim
any readiness outside the evidence attached in `audit/`,
`paper/dsfb_rf_v2.tex`, and the reproducible Colab notebook.
DSFB is positioned as a **structural observer** that augments incumbent
RF chains by reading residuals they already compute and usually
discard; it is not a detection benchmark, classifier, or replacement
for upstream producers.

---

## 1.  Current TRL Self-Assessment

Mirrored from `paper/dsfb_rf_v2.tex` Table~III (TRL breakdown). One-line
rationale per row; evidence is pinned to concrete artefacts.

| Component | TRL | Rationale | Evidence |
|-----------|-----|-----------|----------|
| Grammar / DSA / Envelope engine | 5 | 6 Kani proofs + 360 unit tests + bare-metal builds (Cortex-M4F, RISC-V 32) | `src/kani_proofs.rs`, `tests/`, `.github/workflows/quality.yml` |
| `q16_16` fixed-point resync | 5 | Kani `fixed_point_resync_drift` + `q16_16_quantize_panic` proofs lock the ulp bound | `src/kani_proofs.rs` |
| Amplitude-template Wasserstein-2 residual | 4 | Validated on the local RadioML 2018.01a slice; schema-preserving | `examples/radioml_hdf5.rs`, `paper/dsfb_rf_v2.tex` §VII.G |
| Real-dataset observer pipeline | 4 | 80+1 real figures reproduced across 8 slices; `generate_figures_real` golden path | `examples/generate_figures_real.rs` |
| ORACLE B210 calibration path | 3 | One SigMF slice loaded and analysed; hardware-in-loop not yet run | `data/slices/oracle_slice.sigmf-*` |
| Multi-emitter corroboration | 3 | `CorroborationAccumulator` library support + paper lemma; no hardware exhibit | `src/dsa.rs`, `paper/dsfb_rf_v2.tex` §V.B |
| SBIR packaging | 3 | Companion tech report + this document; no Phase I award record | `paper/dsfb_rf_sbir_trl_v1.tex` |

TRL definitions follow DoD scale (TRL 1 = principle; TRL 9 = mission
proven); the table is **current-state only**, not a proposal.

---

## 2.  Phase I Risk-Burn-Down Matrix — L1..L22 × Deliverables

Each row maps one limitation declared in
`paper/dsfb_rf_v2.tex` §XI (L1..L22) to a Phase I deliverable that
would burn down the risk. Budget columns are expense categories, not
dollar amounts; those belong in the proposal narrative.

| L-tag | Risk | Phase I Deliverable | Verification Evidence | Budget Line |
|-------|------|---------------------|----------------------|-------------|
| L1 | No P_d / P_fa claim | Calibrated per-target detection curves on B210 hardware-in-loop | `audit/hil_b210_roc.csv` + `paper/addendum.tex` | Personnel + hardware time |
| L2 | No universal detectability | Per-residual detectability-bound sweep across 3 producer classes (PLL, AGC, channel-eq) | `audit/detectability_sweep.csv` | Personnel |
| L3 | Block-B128 fixed | B $\in$ \{32, 64, 128, 256\} sensitivity scan on RadioML 2018.01a GOLD | Table V extension in Phase I report | Personnel |
| L4 | Single-dataset recall denominator | Secondary full-dataset eval on ORACLE or POWDER | Updated Table IV | Personnel + compute |
| L5 | Sorted-amplitude Wasserstein-2 only | Add change-rate and power-spectral residual variants | `examples/residual_variants.rs` | Personnel |
| L6 | Healthy-window heuristic | Bayesian-change-point baseline for the healthy-window boundary | `src/envelope_priors.rs` | Personnel |
| L7 | Fixed Stage III parameters | Per-residual hyperparameter calibration record | `audit/calibration_log.csv` | Personnel |
| L8 | No adversarial robustness | Phase I smart-jammer probe on B210 under simulated spoof | `audit/adversarial_b210.md` | Hardware time |
| L9 | No real-time budget proof beyond QEMU | Hardware-in-loop latency on Cortex-M4F reference board (STM32F4 Discovery) | `audit/m4f_latency_hil.csv` | Hardware + personnel |
| L10 | No MIL-STD-461G / DO-178C | Pre-certification gap analysis with DO-178C levels | `audit/do178c_gap.md` | Certification SME |
| L11 | No power-envelope budget | Energy-per-sample on Cortex-M4F @ 72 MHz | `audit/energy_m4f.csv` | Hardware + personnel |
| L12 | No multi-emitter HIL | Two-B210 multi-emitter exhibit with corroboration lemma applied | `audit/two_emitter.md` + `paper/addendum.tex` | Hardware + personnel |
| L13 | No ITAR / EAR clearance | Export-control review against Phase I scope | `audit/itar_review.md` | Legal SME |
| L14 | No cross-platform binary reproducibility | `cargo build --release` bit-identity on 3 host triples | `audit/bin_sha_matrix.csv` | Personnel |
| L15 | No long-horizon drift study | 72-hour continuous capture + envelope-drift audit | `audit/long_horizon.csv` | Hardware time |
| L16 | Multi-emitter scaling beyond M=2 | M $\in$ \{2, 4, 8\} corroboration-accumulator scaling under shared manifold | `audit/m_scaling.csv` | Personnel + compute |
| L17 | No field-EMI characterisation | Outdoor ISM-band capture with measured noise floor | `audit/field_isn.md` | Hardware + travel |
| L18 | Deferred W-sweep on real datasets | `wpred_sweep` at W $\in$ \{3, 5, 7\} on RadioML 2018.01a GOLD | Table V final row-fill | Compute |
| L19 | No operator-study UX data | Operator-in-loop study on ≥ 3 participants reviewing DSFB episodes | `audit/operator_ux.md` | Personnel + participants |
| L20 | No upstream-integration reference design | Reference integration with `gr-osmocom` block | `gnuradio_dsfb/` sibling crate | Personnel |
| L21 | No secondary formal-proof surface | Kani proofs for `CorroborationAccumulator` + `PermutationEntropyEstimator` | `src/kani_proofs.rs` addenda | Personnel |
| L22 | No plugin-load audit beyond dependency graph | Deny-list gate in `cargo-deny` against runtime `libloading` exposure | `deny.toml` + CI gate | Personnel |

Phase II would carry hardware milestones (X310 + VITA 49.2 at TRL 5,
CMOSS VPX at TRL 6) and a second-operator integration study. That
scope is deliberately out of this document to preserve the single-POC
posture.

---

## 3.  Named Hardware Roster

| Hardware | Purpose | Current Status |
|----------|---------|----------------|
| USRP B210 | Single-emitter ORACLE fingerprinting + Phase I HIL baseline | Owned (in-house) |
| USRP X310 + VITA 49.2 | Multi-channel corroboration (L12) + TRL 5 exhibit | Needs acquisition |
| NI mmWave (28 GHz) | Optional Phase II mmWave residual extension | Partner-hosted candidate |
| CMOSS VPX chassis | TRL 6 platform-integration target | Proposal-stage |
| STM32F407 Discovery (Cortex-M4F) | Bare-metal latency + energy reference board | Owned (in-house) |
| SiFive HiFive1 Rev B (RISC-V 32) | Bare-metal cross-ISA latency baseline | Owned (in-house) |
| x86-64 workstation (8-core, 32 GB) | Full-dataset amplitude-domain reproductions | Owned (in-house) |

"Owned" means reproducible from today; "Needs acquisition" means the
milestone matrix gates on its procurement.

---

## 4.  Open SBIR Topic Map

Cross-indexed with §3 of the companion tech report
(`paper/dsfb_rf_sbir_trl_v1.tex`). Single edit point: update this table
and mirror the edits into the companion report.

| Topic ID | Agency | DSFB Capability | L-tag | TRL Gate |
|----------|--------|-----------------|-------|----------|
| AFWERX spectrum-awareness (recurring) | AFWERX | Grammar-structured residual view over existing SDR stacks | L12, L17, L19 | TRL 4 → 5 |
| ONR EW residual-structure (open topic) | ONR | Structural episode compression on EW receiver residuals | L1, L5, L8 | TRL 4 → 5 |
| Army CCDC cognitive-radio monitoring | Army CCDC | Per-residual observer augmentation for cognitive-radio stacks | L11, L15, L20 | TRL 4 → 5 |

This list is **candidate topics**; it is not a claim of award, pre-award,
or government sponsorship.

---

## 5.  Non-Certification Statement

This document and its companion tech report do **not** constitute:

1. A DoD certification under DO-178C, DO-254, MIL-STD-461G, or any
   equivalent standard.
2. A representation that DSFB is cleared under ITAR or EAR — the
   L13 item above is the open risk disposition.
3. A guarantee of P_d / P_fa performance on any specific threat
   class or target hardware — see L1 and L2.
4. A statement of SBIR award, subaward, or pre-selection.

The single source of public evidence remains
`crates/dsfb-rf/audit/` (static assurance findings),
`paper/dsfb_rf_v2.tex` (reproducible claims with L1..L22
limitations), and the locally-reproducible Colab notebook.
Questions: `partnerships@invariantforge.net`.
