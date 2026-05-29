# dsfb-chemical-engineering-core

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-chemical-engineering/notebooks/dsfb_chemical_engineering_colab.ipynb)

[![dsfb-gray](https://img.shields.io/badge/dsfb--gray-69.5%25-yellowgreen)](../../audit/dsfb-gray/core/) [![unsafe](https://img.shields.io/badge/unsafe-forbidden-brightgreen)](../../audit/cargo-geiger/README.md) [![Miri](https://img.shields.io/badge/Miri-no%20UB-brightgreen)](../../audit/miri/README.md) [![audit](https://img.shields.io/badge/audit-suite-blue)](../../audit/)

*Verification: this `no_std`, no-heap, `#![forbid(unsafe_code)]` crate runs clean under **Miri** (8/8 tests, no undefined behaviour — see [`audit/miri/`](../../audit/miri/README.md)).*

Embedded grammar (no_std, no-heap, fixed-point): the residual triple + ring buffer + admissibility envelope + grammar state machine in scaled integers.

#![forbid(unsafe_code)] (no_std, no heap, panic=abort). Advisory, read-only; not a controller or safety function. See [`../../SAFETY.md`](../../SAFETY.md), [`../../SECURITY.md`](../../SECURITY.md), and the audit posture in [`../../audit/`](../../audit/).

## Citation

If you use DSFB-Chemical-Engineering, please cite:

> de Beer, R. (2026). *DSFB-Chemical-Engineering: Read-Only Residual Semiotics for Chemometrics-Augmented Fault Detection and Diagnosis in Chemical Engineering, with a Deterministic, Byte-Exact, CUDA-Accelerated Forensic Evidence Court* (1.0). Zenodo. <https://doi.org/10.5281/zenodo.20443279>

See [`CITATION.cff`](../../CITATION.cff) for the machine-readable record.
