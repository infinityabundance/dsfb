# dsfb-chemical-engineering (Python bindings)

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-chemical-engineering/notebooks/dsfb_chemical_engineering_colab.ipynb)

[![dsfb-gray](https://img.shields.io/badge/dsfb--gray-73.4%25-green)](../../audit/dsfb-gray/py/) [![audit](https://img.shields.io/badge/audit-suite-blue)](../../audit/)

Thin [pyo3](https://pyo3.rs) bindings exposing a focused, file-free subset of the read-only DSFB courts to
Python. Standalone workspace (excluded from the host Rust build); the wheel is built + published with
[maturin](https://maturin.rs) — **publishing is USER-ONLY**.

## Surface
```python
import dsfb_chemical_engineering as dsfb

dsfb.version()                                 # -> "0.1.0"
dsfb.classify_unit_pair("degC", "K")           # -> "affine_offset_hazard"
dsfb.classify_unit_pair("bar", "Pa")           # -> "scale_mismatch"
dsfb.grade_readiness(n_tags=40, n_rows=5000, baseline_present=True, license_cleared=True,
                     missingness=0.01, unit_coverage=1.0, has_controllers=True)
# -> {"verdict": "ReadyWithCaveats", "n_caveat": ..., "n_critical_missing": 0, "court_hash": "..."}
```
The heavyweight pipeline (`analyze`, `casefile`) stays in the Rust binary / container; these bindings are the
cheap-to-marshal courts (unit consistency, data readiness) for use from Python notebooks.

## Build
```bash
pip install maturin
cd crates/dsfb-chemical-engineering-py
maturin develop --release          # build + install into the active venv
# or: maturin build --release      # produce an abi3 wheel (CPython 3.9+) in target/wheels/
```
The crate also builds with plain `cargo build --release` (it produces the cdylib). On a very new CPython
(e.g. 3.14, ahead of pyo3's declared support) set `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1` for the abi3
forward-compatible build.

## Bounded
Same non-claims as the underlying objects: advisory, read-only, deterministic, hash-sealed; no control or
safety-instrumented-function authority. Publishing the wheel (PyPI / `maturin publish`) is a USER-ONLY step
(see `docs/release_checklist.md`).

## Citation

If you use DSFB-Chemical-Engineering, please cite:

> de Beer, R. (2026). *DSFB-Chemical-Engineering: Read-Only Residual Semiotics for Chemometrics-Augmented Fault Detection and Diagnosis in Chemical Engineering, with a Deterministic, Byte-Exact, CUDA-Accelerated Forensic Evidence Court* (1.0). Zenodo. <https://doi.org/10.5281/zenodo.20443279>

See [`CITATION.cff`](../../CITATION.cff) for the machine-readable record.
