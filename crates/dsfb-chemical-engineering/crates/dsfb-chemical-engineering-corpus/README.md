# dsfb-chemical-engineering-corpus

[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-chemical-engineering/notebooks/dsfb_chemical_engineering_colab.ipynb)

[![dsfb-gray](https://img.shields.io/badge/dsfb--gray-78.6%25-green)](../../audit/dsfb-gray/corpus/) [![unsafe](https://img.shields.io/badge/unsafe-forbidden-brightgreen)](../../audit/cargo-geiger/README.md) [![Miri](https://img.shields.io/badge/Miri-no%20UB-brightgreen)](../../audit/miri/README.md) [![audit](https://img.shields.io/badge/audit-suite-blue)](../../audit/)

*Verification: this `no_std`, `#![forbid(unsafe_code)]` crate runs clean under **Miri** (7/7 tests, no undefined behaviour — see [`audit/miri/`](../../audit/miri/README.md)).*

The chemical-engineering **soft-sensor data corpus** for DSFB-Chemical-Engineering — a
provenance-bound, deduplicated, deterministic catalogue of *public* datasets where **cheap sensors
infer a hard-to-measure target**, on which deterministic (densorial / tekmeric) inference is exercised.

| crate | role |
|-------|------|
| `dsfb-chemical-engineering-edge` | execution over process residuals |
| `dsfb-chemical-engineering-cuda` | CUDA acceleration + forensic court |
| `dsfb-chemical-engineering-atlas` | chemometric detector + process-heuristic + fault-signature authority |
| **`dsfb-chemical-engineering-corpus`** | **soft-sensor dataset catalogue (cheap sensors → hard-to-measure target)** |

## What it contains
- `SoftSensorDatasetRecordV1` + `SOFT_SENSOR_DATASETS` — **20** public soft-sensor datasets: the canonical
  **SRU**, **Debutanizer**, **Tennessee Eastman**, **Mining flotation (% silica)**, **CCPP**, plus gas-turbine
  CO/NOx, steel energy, UCI/BSM1/N2O wastewater, pulp Kappa, PRONTO, SECOM, gas-sensor drift, IndPenSim,
  and the gated SWaT/WADI — each with cheap-sensor channels, the target, the input→target lag, access,
  licence, and a deterministic-inference note.
- `SourceRef` provenance on **every** record; `cheap_sensor` flags the spectroscopy-input sets
  (Tecator/Corn NIR) as *not* the cheap-sensor thesis.
- Four hash-sealed **provenance classification tiers** per record (`license_confidence`,
  `access_confidence`, `redistribution_policy`, `source_authority`) — honest disclosure, counted by
  `census()`; a gate asserts each axis partitions all 20 records.
- `validate()` gates (sourced, never-redistributed, canonical core present, domain spread) + `corpus_hash_v1`.

## The deterministic-soft-sensor thesis
Soft sensors today are almost entirely **probabilistic** (PLS, neural nets, SVR/GPR, Bayesian
latent-variable models). Even the "deterministic" ones (OLS, PLS) are deterministic only in
*computation* — least squares is Gaussian maximum likelihood, optimal only under noise assumptions,
which is why they are noise-fragile. DSFB is deterministic with **no** probability model, likelihood,
loss, or distributional assumption:

> residual **densor → deterministic witness court → replayable case file**

This corpus catalogues the public datasets on which that posture is demonstrated.

## Data provenance and IP boundary
**No dataset bytes are vendored or redistributed.** Each record cites its third-party source (URL +
licence) and flags access (open / Kaggle / gated / code-generates-data). **No ownership of any dataset
is claimed.** The prior-art / novelty is the **DSFB + Densor (Deterministic Endoduction Tekmeric
Inference)** deterministic inference-from-residuals-and-noise and heuristics-bank *technology*, not the
public data. `scripts/prep_softsensor.py` can seal a local dataset copy's provenance (SHA-256) without
redistributing it.

## Non-claims
Cataloguing a dataset asserts no predictive-accuracy superiority and no validated result on it.
`implementation_status` honestly marks `Executed` vs `Catalogued`. `#![forbid(unsafe_code)]`,
`no_std`-capable, deterministic, no network.

## Citation

If you use DSFB-Chemical-Engineering, please cite:

> de Beer, R. (2026). *DSFB-Chemical-Engineering: Read-Only Residual Semiotics for Chemometrics-Augmented Fault Detection and Diagnosis in Chemical Engineering, with a Deterministic, Byte-Exact, CUDA-Accelerated Forensic Evidence Court* (1.0). Zenodo. <https://doi.org/10.5281/zenodo.20443279>

See [`CITATION.cff`](../../CITATION.cff) for the machine-readable record.
