# dsfb-robotics

[![DSFB Gray Audit: 96.2% strong assurance posture](https://img.shields.io/badge/DSFB%20Gray%20Audit-96.2%25-brightgreen)](./audit/dsfb_robotics_scan.txt)
[![Miri: clean](https://img.shields.io/badge/Miri-clean-brightgreen)](./audit/miri/MIRI_AUDIT.md)
[![Kani: 6 harnesses](https://img.shields.io/badge/Kani-6%20harnesses-brightgreen)](./audit/kani/KANI_AUDIT.md)
[![Tests: 191 passing](https://img.shields.io/badge/tests-191%20passing-brightgreen)](./tests/)
[![paper-lock: reproduces 20/20 headline rows](https://img.shields.io/badge/paper--lock-reproduces%2020%2F20%20headline%20rows-brightgreen)](./audit/checksums.txt)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-robotics/colab/dsfb_robotics_reproduce.ipynb)

**DSFB Structural Semiotics Engine for Robotics Health Monitoring** — a deterministic, `no_std` + `no_alloc` + zero-`unsafe` observer layer that reads the residual streams existing robot control and prognostics pipelines already compute, and structures them into a human-readable grammar of typed episodes.

> **Status:** Version 1.0 (April 2026). The full crate ships: core DSFB pipeline (grammar FSM, envelope, engine, kinematics / balancing helpers), 20 dataset adapters, `paper-lock` binary, figure pipeline, Colab notebook, audit stack, and 81-page companion paper. Per-revision history in [`CHANGELOG.md`](CHANGELOG.md); architectural notes in [`ARCHITECTURE.md`](ARCHITECTURE.md).

---

## What this crate is

A read-only observer that takes residuals already produced by incumbent robotics pipelines — joint torque identification residuals, inverse-dynamics residuals, whole-body MPC contact-force residuals, centroidal-momentum observer residuals, bearing envelope-spectrum residuals, health-index trajectories — and emits a three-state grammar (`Admissible` / `Boundary` / `Violation`) with typed episode records and provenance-tagged audit trails.

The upstream pipelines **keep running unchanged**. Removing DSFB changes nothing about the robot's control or safety behaviour.

## What this crate is **not**

DSFB is **not** a competitor to any existing robotics method. It does not:

- Classify bearing faults or identify root cause
- Provide calibrated Pd/Pfa, F1, ROC-AUC, or confusion matrices
- Detect faults earlier than threshold alarms, CUSUM, EWMA, or RMS monitors
- Predict remaining useful life (RUL)
- Replace inverse-dynamics identification, Kalman/Luenberger observers, whole-body controllers, MPC, or any incumbent observer
- Guarantee hard real-time latency under any specific controller platform
- Provide ISO 10218-1/-2:2025, IEC 61508, or ISO 13849 certification

**Existing methods continue to outperform DSFB at their own tasks.** DSFB's role is orthogonal: it recovers *structure* from the residuals those methods discard.

## Architectural contract

| Guarantee | Enforcement |
|---|---|
| Observer-only (no upstream mutation) | Public API takes `&[f64]`; compile-time lifetime rules |
| `#![no_std]` | Crate root attribute; core links against no std runtime |
| `no_alloc` in core | Canonical signature `observe(residuals: &[f64], out: &mut [Episode]) -> usize` |
| Zero `unsafe` | `#![forbid(unsafe_code)]` at crate root |
| Deterministic | Pure-function core; identical ordered inputs → identical episodes |
| Bounded output | `observe` writes at most `out.len()` episodes |

## Canonical API

```rust
use dsfb_robotics::{Episode, observe};

let residuals: &[f64] = &[0.01, 0.02, 0.05, 0.12, 0.21];
let mut out = [Episode::empty(); 16];
let n = observe(residuals, &mut out);

for e in &out[..n] {
    // advisory only — no write-back, no upstream coupling
    let _ = (e.index, e.grammar, e.decision);
}
```

`Episode` fields are byte-identical to the canonical form in [`dsfb-semiconductor`](../dsfb-semiconductor) so downstream tooling consumes DSFB episodes uniformly across crates.

## Feature flags

| Feature | Description |
|---|---|
| *(none)* | Core engine: `no_std` + `no_alloc` + zero unsafe |
| `alloc` | Heap-backed convenience wrappers (e.g. `Vec<Episode>` return) |
| `std` | Host-side tooling (pipeline, I/O, output modules) |
| `serde` | JSON artefact serialization (requires `std`) |
| `paper_lock` | Deterministic headline-metric enforcement for the companion paper |
| `real_figures` | Real-dataset figure bank for the companion paper (requires `std`) |
| `experimental` | Exploratory extensions excluded from the paper-lock metric set |

## Dataset evaluation (companion paper, twenty real-world datasets)

The companion paper at `paper/dsfb_robotics.tex` evaluates DSFB on **twenty public real-world datasets** across three families. Every dataset is a physical-hardware recording under a permissive licence (Apache-2.0 / MIT / CC-BY-4.0 / CC-BY-SA-4.0 / BSD-3-Clause / academic-fair-use). Zero synthetic or simulated data is admitted.

| # | Family | Dataset | Provenance |
|---|---|---|---|
| 1 | PHM | CWRU Bearing | Case Western Reserve University Bearing Data Center |
| 2 | PHM | NASA / IMS Run-to-Failure | Lee et al. 2007, NASA Prognostics Data Repository |
| 3 | PHM | FEMTO-ST PRONOSTIA | Nectoux et al. 2012, IEEE PHM 2012 Challenge |
| 4 | Kinematics | KUKA LWR-IV+ (Simionato 7R) | Sapienza DIAG repository |
| 5 | Kinematics | Franka Emika Panda | Gaz et al. 2019, IEEE RA-L 4(4):4147–4154 |
| 6 | Kinematics | 7-DoF Panda DLR-class | Giacomuzzo et al. 2024, Zenodo 12516500 |
| 7 | Kinematics | UR10 pick-and-place | Polydoros et al. 2015, IEEE/RSJ IROS |
| 8 | Kinematics | DROID 100-episode slice | Khazatsky et al. 2024, Stanford / TRI |
| 9 | Kinematics | Open X-Embodiment NYU-ROT | RT-X 2024, Open X-Embodiment Collaboration |
| 10 | Kinematics | ALOHA bimanual static coffee | Zhao et al. 2023, RSS |
| 11 | Kinematics | ALOHA static tape | Zhao 2023, HuggingFace LeRobot |
| 12 | Kinematics | ALOHA static screw-driver | Zhao 2023, HuggingFace LeRobot |
| 13 | Kinematics | ALOHA static ping-pong | Zhao 2023, HuggingFace LeRobot |
| 14 | Kinematics | Mobile ALOHA wipe-wine | Fu, Zhao, Finn 2024, Stanford |
| 15 | Kinematics | SO-ARM100 pick-and-place | The Robot Studio + HuggingFace LeRobot 2024 |
| 16 | Balancing | MIT Mini-Cheetah / Cheetah 3 | Katz et al. 2019, IEEE ICRA; UMich-CURLY |
| 17 | Balancing | ergoCub push-recovery | Romualdi, Viceconte et al. 2024, IEEE Humanoids |
| 18 | Balancing | ergoCub Sorrentino balancing-torque | Sorrentino et al. 2025, IEEE RAL; ami-iit |
| 19 | Balancing | ANYmal-C GrandTour outdoor locomotion | ETH-Zürich Legged Robotics 2024 |
| 20 | Balancing | Unitree G1 humanoid teleoperation | `Makolon0321/unitree_g1_block_stack`, HuggingFace 2024–2025 |

Per-dataset provenance, SHA-256 checksums, and fetch instructions live at [`data/processed/PROCESSED_MANIFEST.json`](data/processed/PROCESSED_MANIFEST.json). All twenty processed-residual CSVs ship in-tree; raw upstream-source data is fetched on demand by [`scripts/preprocess_datasets.py`](scripts/preprocess_datasets.py).

## Dataset honesty disclosure

- **Real data only.** No synthetic data is mixed with real results anywhere in this crate or its paper. The only micro-fixtures used in unit tests are clearly illustrative arrays of ≤10 values (e.g. `[0.1, 0.2, 0.5, 1.2, 2.1]`).
- **No simulation frameworks.** MuJoCo, Isaac, Gazebo, RaiSim, Drake, Webots, and PyBullet are not used anywhere in this crate.
- **Paper-lock fallback policy.** When invoked without the required real dataset at the documented path, `paper-lock <slug>` exits with a clear error pointing to the relevant oracle-protocol doc under [`docs/`](docs/). It never silently substitutes a synthetic fixture.

## Reproducibility

End-to-end reproduction recipe lives in [`REPRODUCE.md`](REPRODUCE.md). One-command form:

```bash
cargo run --release --bin paper-lock --features std,paper_lock -- <slug>
```

A Colab notebook at [`colab/dsfb_robotics_reproduce.ipynb`](colab/) bundles the in-tree processed CSVs and reproduces every paper figure end-to-end. Free-tier Colab Run-All budget: ~30 min cold (LTO build dominates); ~4 min on a developer laptop with cached `target/`.

## Audit

The crate ships a layered audit stack — every artefact is reproducible from the committed source and documented inline.

| Audit | Status | Artefact |
|---|---|---|
| **DSFB Gray** assurance scan | 96.2 % strong assurance posture | [`audit/dsfb_robotics_scan.txt`](audit/dsfb_robotics_scan.txt) (plain text), [`.sarif.json`](audit/dsfb_robotics_scan.sarif.json), [`.dsse.json`](audit/dsfb_robotics_scan.dsse.json) (DSSE attestation, unsigned), [`.intoto.json`](audit/dsfb_robotics_scan.intoto.json) (in-toto provenance) |
| **Miri** undefined-behaviour audit | clean across 3 alias models (stacked borrows, tree borrows, no_std core) | [`audit/miri/MIRI_AUDIT.md`](audit/miri/MIRI_AUDIT.md) |
| **Kani** model-checking | 6 harnesses, all green | [`audit/kani/KANI_AUDIT.md`](audit/kani/KANI_AUDIT.md) |
| **Cargo-fuzz** | 1 M iterations × 2 targets ([`engine_roundtrip`](fuzz/fuzz_targets/engine_roundtrip.rs), [`grammar_fsm`](fuzz/fuzz_targets/grammar_fsm.rs)) | [`fuzz/RUN_LOG.md`](fuzz/RUN_LOG.md) |
| **Concurrency / Loom** | observer-non-mutation under thread interleavings | [`tests/concurrency_observer.rs`](tests/concurrency_observer.rs) |
| **Long-running stability** | 990 k-sample concatenated stream; no drift, no counter saturation | [`tests/long_running_stability.rs`](tests/long_running_stability.rs) |
| **Property tests (proptest)** | grammar invariants + orthogonality (the "no outperforms" claim) under shrinking | [`tests/proptest_invariants.rs`](tests/proptest_invariants.rs), [`tests/proptest_orthogonality.rs`](tests/proptest_orthogonality.rs) |
| **JSON Schema validation** | mechanical drift check between [`paper/paper_lock_schema.json`](paper/paper_lock_schema.json) and the production binary's JSON output | [`tests/schema_validation.rs`](tests/schema_validation.rs) |
| **Checksum regression (CI)** | per-dataset `paper-lock` JSON SHA-256s + processed-CSV SHA-256s pinned in [`audit/checksums.txt`](audit/checksums.txt) | [`.github/workflows/reproduce.yml`](.github/workflows/reproduce.yml) |
| **Bootstrap confidence intervals** | 1 000-replicate stationary-block bootstrap (Politis-Romano 1994) per dataset | [`audit/bootstrap/`](audit/bootstrap/) |
| **Effect size** | Cohen's d ≈ 0.852 on V-rate axis (zero-V vs non-zero-V cluster) | [`scripts/effect_size.py`](scripts/effect_size.py) → [`audit/effect_size/cluster_assignments.csv`](audit/effect_size/cluster_assignments.csv) |
| **Sensitivity grid** | 300-cell sweep over (W, K, β, δ_s) on `panda_gaz` | [`audit/sensitivity/`](audit/sensitivity/) |
| **Ablation study** | drift / slew / hysteresis disabled per cell | [`audit/ablation/`](audit/ablation/) |
| **Throughput tails** | per-dataset Criterion p50/p95/p99/max tables | [`audit/throughput/per_dataset_tails.csv`](audit/throughput/per_dataset_tails.csv) |
| **Pre-registration** | first-revision parameter freeze | git tag `paper-lock-protocol-frozen-v1` |

The [`audit/README.md`](audit/README.md) indexes every artefact with reproduction commands.

## Licensing

- **Reference implementation:** Apache-2.0 (see [`LICENSE`](LICENSE)).
- **Theoretical framework and supervisory methods:** proprietary Background IP of Invariant Forge LLC; commercial deployment requires a separate written licence. See [`NOTICE`](NOTICE).
- **Datasets:** each dataset retains its upstream licence; see [`data/processed/PROCESSED_MANIFEST.json`](data/processed/PROCESSED_MANIFEST.json).

Licensing enquiries: `licensing@invariantforge.net`

## Companion paper

`paper/dsfb_robotics.tex` — 81-page LaTeX specification of the DSFB framework applied to robotics health monitoring (Version 1.0, April 2026). The paper includes the augmentation-thesis hero figure (§1.4), the twenty-dataset evaluation (§10) with bootstrap CIs / sensitivity grid / ablation / Cohen's d effect size, the worked example on the Gaz 2019 Panda dataset (§11), an explicit Non-Claims table, a failure-modes section, a limitations section with the 50-point engineering-criticisms subsection, the Falsifiability statement, the Non-Intrusion Manifest appendix, and the Motif Gallery. Every empirical number in §10 is reproducible from this crate under `paper-lock`.

## Citation

If you use this crate or reference its companion paper, please cite:

> de Beer, R. (2026). *DSFB Structural Semiotics Engine for Robotics Health Monitoring: A Deterministic Augmentation Layer for Typed Residual Interpretation of Joint Degradation, Actuator Drift, and Kinematic Anomalies in Safety-Critical Robotic Systems* (v1.0). Zenodo. https://doi.org/10.5281/zenodo.19778382

BibTeX:

```bibtex
@software{debeer_2026_dsfb_robotics,
  author    = {de Beer, Riaan},
  title     = {DSFB Structural Semiotics Engine for Robotics Health Monitoring:
               A Deterministic Augmentation Layer for Typed Residual Interpretation
               of Joint Degradation, Actuator Drift, and Kinematic Anomalies in
               Safety-Critical Robotic Systems},
  version   = {v1.0},
  year      = {2026},
  publisher = {Zenodo},
  doi       = {10.5281/zenodo.19778382},
  url       = {https://doi.org/10.5281/zenodo.19778382}
}
```

Machine-readable metadata: see [`CITATION.cff`](CITATION.cff). Formal bibliographic entries are maintained in the companion paper's bibliography.

## Authorship & co-authorship policy

This crate and its paper are authored by Riaan de Beer (Invariant Forge LLC). 
