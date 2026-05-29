# dsfb-densor-runtime

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-chemical-engineering/notebooks/dsfb_chemical_engineering_colab.ipynb)

[![dsfb-gray](https://img.shields.io/badge/dsfb--gray-70.2%25-yellowgreen)](../../audit/dsfb-gray/densor-runtime/) [![unsafe](https://img.shields.io/badge/unsafe-forbidden-brightgreen)](../../audit/cargo-geiger/README.md)

A **thin, deterministic execution-substrate skeleton** for DSFB densor pipelines. It standardises one disciplined
spine so future DSFB work need not re-implement it:

```text
load manifest  ->  validate authority hashes  ->  execute stages  ->  seal evidence  ->  emit receipt
```

*Riaan de Beer — Invariant Forge LLC.* `#![forbid(unsafe_code)]`.

## What it is
- **`Densor`** — a sealed, self-verifying evidence object (`densor_id`, `densor_kind`, `evidence_hash`, `verify`).
- **`RuntimeStage<I, O>`** — one deterministic transform; declares the frozen `AuthorityHash`es it was built
  against; `execute` returns a `StageReceipt<O>` binding input/output hashes + authorities to the output.
- **`DensorManifest`** — the frozen declaration of densors + the authority allow-list a run may cite (`validate`).
- **`Runtime`** — the spine: validates the manifest, **gates** each stage (refusing any stage with no authority, or
  citing an authority the manifest did not freeze), and seals the ordered stage receipts into a…
- **`RuntimeReceiptV1`** — a tamper-evident `receipt_hash` over the pipeline id, the frozen authorities, and every
  stage summary (the substrate analogue of the chemical court record's `bundle_root`).
- **`RuntimeIndex`** — a read-only summary of a sealed run (pipeline id · stage ids · authorities · receipt hash).

Sealing reuses the DSFB `CanonicalHasher` discipline (`seal.rs`, a self-contained copy so the crate depends on no
chemical crate). SHA-256 via `sha2`.

## Discipline (deliberately small + strict)
- no network, no hidden downloads, no filesystem side effects in the core;
- no probabilistic behaviour — every output is a pure function of the inputs (determinism is tested);
- no authority mutation during execution — authorities are a frozen allow-list;
- **no claim without an authority hash** — a stage that declares none is refused (tested).

## What it does NOT do (non-claim)
This crate is a **mechanism, not an authority**. It carries **no chemical-engineering content and makes no
cross-domain claims**: the meaning of any pipeline lives entirely in the authorities a run cites and in the domain
crate that defines the stages (e.g. `dsfb-chemical-engineering-edge`). The runtime only attests *which stages ran
over which densors against which frozen authorities, sealing to which digest* — it asserts nothing about whether a
result is correct, optimal, or meaningful. It is the deterministic spine; the chemical artifact remains the domain
authority.

## Run the tests
```fish
cargo test -p dsfb-densor-runtime
```
The test module is a complete worked example: a two-stage residual pipeline (scale → count-beyond-band), each stage
bound to one frozen authority, covering determinism, tamper-detection, and both authority-gate refusals.

## Citation

If you use DSFB-Chemical-Engineering, please cite:

> de Beer, R. (2026). *DSFB-Chemical-Engineering: Read-Only Residual Semiotics for Chemometrics-Augmented Fault Detection and Diagnosis in Chemical Engineering, with a Deterministic, Byte-Exact, CUDA-Accelerated Forensic Evidence Court* (1.0). Zenodo. <https://doi.org/10.5281/zenodo.20443279>

See [`CITATION.cff`](../../CITATION.cff) for the machine-readable record.
