# dsfb-robotics

[![DSFB Gray Audit: 96.2% strong assurance posture](https://img.shields.io/badge/DSFB%20Gray%20Audit-96.2%25-brightgreen)](./audit/dsfb_robotics_scan.txt)
[![Miri: clean](https://img.shields.io/badge/Miri-clean-brightgreen)](./audit/miri/MIRI_AUDIT.md)
[![Kani: 6 harnesses](https://img.shields.io/badge/Kani-6%20harnesses-brightgreen)](./audit/kani/KANI_AUDIT.md)
[![Tests: 191 passing](https://img.shields.io/badge/tests-191%20passing-brightgreen)](./tests/)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-robotics/colab/dsfb_robotics_reproduce.ipynb)

**DSFB Structural Semiotics Engine for Robotics Health Monitoring** — a deterministic, `no_std` + `no_alloc` + zero-`unsafe` observer layer that reads the residual streams existing robot control and prognostics pipelines already compute, and structures them into a human-readable grammar of typed episodes.

> **Status:** Phase 2 in progress (v0.1.0). The core DSFB pipeline (grammar FSM, envelope, engine, shared kinematics / balancing helpers) is live; the ten dataset adapters, paper-lock binary, figures, Colab notebook, and audit stack land across Phases 3–9. Phase roadmap and per-phase acceptance criteria are documented in [`CHANGELOG.md`](CHANGELOG.md) and [`ARCHITECTURE.md`](ARCHITECTURE.md).

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

## Dataset evaluation (companion paper, ten real-world datasets)

The companion paper at `paper/dsfb_robotics.tex` evaluates DSFB on **ten public real-world datasets** across three families. No synthetic data is used; every headline number comes from measurements on the datasets below under their published licences.

| # | Dataset | Family | Provenance | Residual DSFB structures |
|---|---|---|---|---|
| 1 | CWRU Bearing | PHM | Case Western Reserve University Bearing Data Center | BPFI envelope-spectrum deviation |
| 2 | IMS Run-to-Failure | PHM | NASA Prognostics Data Repository (Lee et al. 2007) | Health-index trajectory |
| 3 | C-MAPSS FD001/FD003 | Regime drift | NASA PHM08 (Saxena et al. 2008) | Multi-regime structural residual |
| 4 | KUKA LWR | **Kinematics** | Jubien–Gautier–Janot 2014 | Inverse-dynamics identification residual (link-side) |
| 5 | FEMTO-ST PRONOSTIA | PHM | IEEE PHM 2012 Challenge (Nectoux et al. 2012) | Vibration-HI trajectory |
| 6 | Franka Panda | **Kinematics** | Gaz, Cognetti, Oliva, Robuffo Giordano, De Luca, RA-L 2019 (DOI 10.1109/LRA.2019.2931248) | Inverse-dynamics identification residual (motor-side) |
| 7 | DLR Rollin' Justin / LWR-III | **Kinematics** | DLR Institute of Robotics & Mechatronics (Albu-Schäffer et al.) | Inverse-dynamics identification residual (link-side) |
| 8 | UR10 | **Kinematics** | Kufieta 2014 (NTNU); Kebria et al. 2016 | Inverse-dynamics identification residual (motor-side) |
| 9 | MIT Cheetah 3 / Mini-Cheetah | **Balancing** | Katz–Di Carlo–Kim, ICRA 2019 (MIT Biomimetics; MIT licence) | MPC contact-force residual + centroidal-momentum observer residual |
| 10 | IIT iCub push-recovery | **Balancing** | Nori, Traversaro et al. (IIT iCub Facility) | Contact-wrench residual + centroidal-momentum tracking residual |

See [`data/slices/SLICE_MANIFEST.json`](data/slices/) for per-dataset provenance, SHA-256 checksums, licences, and fetch instructions. Redistributable slices ship in-tree; datasets under data-use agreements (DLR Justin, iCub) ship as manifest-only pointers with a hard-error fallback in `paper-lock`.

## Dataset honesty disclosure

- **Real data only.** No synthetic data is mixed with real results anywhere in this crate or its paper. The only micro-fixtures used in unit tests are clearly illustrative arrays of ≤10 values (e.g. `[0.1, 0.2, 0.5, 1.2, 2.1]`).
- **No simulation frameworks.** MuJoCo, Isaac, Gazebo, RaiSim, Drake, Webots, and PyBullet are not used anywhere in this crate.
- **C-MAPSS provenance note.** NASA's C-MAPSS is a published benchmark generated by the MAPSS simulation code; we include it because the companion paper commits to the five-dataset spec, and we frame it strictly as a *cross-domain structural analogue* to multi-regime drift, not as a primary real-world claim. See `docs/cmapss_oracle_protocol.md` for the full framing.
- **Paper-lock fallback policy.** When invoked without the required real dataset at the documented path, `paper-lock <dataset>` exits with a clear error pointing to the relevant oracle-protocol doc. It never silently substitutes a synthetic fixture.

## Reproducibility

End-to-end reproduction recipe lives in [`REPRODUCE.md`](REPRODUCE.md). One-command form (Phase 4+):

```bash
cargo run --release --bin paper-lock --features std,paper_lock -- <dataset>
```

A Colab notebook at [`colab/dsfb_robotics_reproduce.ipynb`](colab/) (Phase 5) bundles the in-tree slices and reproduces all paper figures end-to-end in under five minutes on free-tier Colab.

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
- **Datasets:** each dataset retains its upstream licence; see [`data/slices/SLICE_MANIFEST.json`](data/slices/).

Licensing enquiries: `licensing@invariantforge.net`

## Companion paper

`paper/dsfb_robotics.tex` — 1185-line LaTeX specification of the DSFB framework applied to robotics health monitoring. The paper includes an explicit Non-Claims table (§11), a failure-modes section, a limitations section with reviewer-objection responses, and provenance for every dataset. Empirical results in §10 are populated from runs of this crate under `paper-lock`.

## Citation

See [`CITATION.cff`](CITATION.cff). Formal bibliographic entries are maintained in the companion paper's bibliography.

## Authorship & co-authorship policy

This crate and its paper are authored by Riaan de Beer (Invariant Forge LLC). 
