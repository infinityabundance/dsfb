# dsfb-chemical-engineering-edge

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-chemical-engineering/notebooks/dsfb_chemical_engineering_colab.ipynb)

[![dsfb-gray](https://img.shields.io/badge/dsfb--gray-65.6%25-orange)](../../audit/dsfb-gray/edge/) [![unsafe](https://img.shields.io/badge/unsafe-forbidden-brightgreen)](../../audit/cargo-geiger/README.md) [![Kani](https://img.shields.io/badge/Kani-6%2F6%20harnesses-brightgreen)](../../audit/kani/README.md) [![cargo-fuzz](https://img.shields.io/badge/cargo--fuzz-225.2M%20execs%2C%200%20crashes-brightgreen)](../../audit/cargo-fuzz/README.md) [![audit](https://img.shields.io/badge/audit-suite-blue)](../../audit/)

Deterministic, read-only residual-semiotics layer for chemical-engineering / chemometric process
monitoring — **CPU / edge build, no GPU required**, `#![forbid(unsafe_code)]`.

*Riaan de Beer — Invariant Forge LLC — ORCID 0009-0006-1155-027X.*

## What it does

Ingests the residuals, scores, distances, reconstruction errors, and detector disagreements that
established chemometrics methods already emit, then applies **Drift–Slew Fusion Bootstrap** — drift,
slew, admissibility envelopes, deterministic quorum fusion, and a chemical heuristics bank — to
produce auditable structural episodes with deterministic replay. It replaces no estimator,
controller, historian, or alarm system, and writes to no upstream register.

## Pipeline

```
baseline → standardise → PCA (T²/SPE) → detector atlas → per-detector DSFB grammar
        → quorum fusion → heuristics bank → metrics → deterministic replay hash
```

## Detector atlas

A literature detector corpus (`corpus/chemometric_atlas.toml`, build-time SHA-256 validated) of 57
canonicalised detectors across four families (Classical MSPC, Dynamic/Temporal,
Nonlinear/Distributional, Process-structure). 18 are **executed** by the demo pipeline; the rest are
**catalogued** prior-art surface, honestly distinguished by an `implementation_status` field.

## Commands

```bash
cargo run --release -p dsfb-chemical-engineering-edge -- demo            # full demo → artifacts + zip
cargo run --release -p dsfb-chemical-engineering-edge -- analyze <name>  # one dataset
cargo run --release -p dsfb-chemical-engineering-edge -- atlas           # corpus validation summary
cargo run --release -p dsfb-chemical-engineering-edge -- verify-replay   # determinism check
```

`demo` discovers committed dataset slices under `data/slices/*.csv`; if none are present it runs a
built-in deterministic synthetic suite so the demo always works. Output is a timestamped
`output-dsfb-chemical-engineering/<stamp>/` containing per-dataset `detector_outputs.csv`,
`residual_streams.csv`, `dsfb_episodes.csv`, `heuristic_labels.csv`, `result.json`; run-level
`manifest.json`, `replay_hashes.json`, `metrics.csv`, `report.md`; figure traces; and a single
`artifact_bundle.zip`.

## Plant-reality &amp; evidence objects

The crate carries ~98 sealed `…V1` evidence types (each with its own SHA-256 seal + a `verify()` re-derivation
and an explicit non-claim). The plant-reality layer — what makes a residual witness read like senior process
engineering rather than an anonymous anomaly score — includes, among others:

- **`hazop_guidewords`** — `HazopGuidewordMappingV1`: HAZOP No/More/Less/Reverse/… guidewords mapped to residual analogues.
- **`basis_descriptor`** — `BasisDescriptorV1`: wet/dry/mass/mole quantity basis with a `comparable_with` guard.
- **`calibration_passport`** — `CalibrationModelPassportV1`: PAT/NIR calibration provenance (RMSEP, bias, leverage, Q-residual, instrument transfer) with an in/out-of-validation-range gate.
- **`ne107_adapter`** — `NamurNe107AdapterV1`: an *executable* DSFB-state → NAMUR NE 107 status adapter (test-pinned to the report's string mapping; every `Failure` is a witness-qualified *candidate*, never a device verdict).
- **`equipment_signatures`** — `EquipmentSignatureRecordV1` / `EquipmentSignatureBankV1`: an equipment-class bank (pump / heat-exchanger / reactor / column) with required/forbidden/supporting witnesses and an A–D burden tier.

Each is **additive, off the replay path, self-sealed (its own hash — not folded into `atlas_hash_v1`)**, and read-only.

## Honesty

Metrics include an explicit **unknown rate** — the fraction of fused episodes deliberately left
unlabelled. The correct failure mode is to emit *"unknown structural episode (evidence preserved)"*
rather than force a confident diagnosis. Synthetic data is always labelled as synthetic; it is never
presented as measured data.

## Formal verification &amp; fuzzing

`src/kani_proofs.rs` carries Kani harnesses for determinism, non-interference, and envelope
invariants (`cargo kani`). The `fuzz/` directory holds the **cargo-fuzz** companion — libFuzzer + ASan
targets that hammer the same grammar classifier (float and `no_std` i64 paths) and the unit-string lexer
over the full input domain (`cargo +nightly fuzz run <target>`); a 60-second-per-target campaign logged
**225.2M executions with 0 crashes** (see [`../../audit/cargo-fuzz/`](../../audit/cargo-fuzz/README.md)).
The bounded Kani proof and the unbounded-domain fuzz campaign are complementary, not redundant.

## Citation

If you use DSFB-Chemical-Engineering, please cite:

> de Beer, R. (2026). *DSFB-Chemical-Engineering: Read-Only Residual Semiotics for Chemometrics-Augmented Fault Detection and Diagnosis in Chemical Engineering, with a Deterministic, Byte-Exact, CUDA-Accelerated Forensic Evidence Court* (1.0). Zenodo. <https://doi.org/10.5281/zenodo.20443279>

See [`CITATION.cff`](../../CITATION.cff) for the machine-readable record.
