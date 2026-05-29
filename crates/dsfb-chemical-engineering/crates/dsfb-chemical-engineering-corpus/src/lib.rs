//! `dsfb-chemical-engineering-corpus` — the chemical-engineering **soft-sensor data corpus**.
//!
//! A provenance-bound, deduplicated, deterministic catalogue of *public* soft-sensor datasets — where
//! cheap, ubiquitous sensors (temperature, pressure, flow, level, current) infer a hard-to-measure
//! target (composition, quality, emissions) — on which **deterministic (densorial / tekmeric)**
//! inference can be demonstrated, in contrast to the probabilistic estimators that dominate soft
//! sensing. It mirrors the DSFB-GPU-Atlas canonicalisation-court discipline: `&'static const` records
//! (byte-identical builds), a mandatory [`types::SourceRef`] per entry, deterministic validation
//! gates, and a `corpus_hash_v1`.
//!
//! ## The deterministic-soft-sensor thesis
//! Soft sensors today are almost entirely probabilistic (PLS, neural nets, SVR/GPR, Bayesian
//! latent-variable models). Even the "deterministic" ones (OLS, PLS) are deterministic only in
//! computation — least squares is Gaussian maximum likelihood, optimal only under noise assumptions —
//! which is why they are noise-fragile. DSFB is deterministic with **no** probability model,
//! likelihood, loss, or distributional assumption: residual densor -> deterministic witness court ->
//! replayable case file. This corpus catalogues the public datasets on which that posture is exercised.
//!
//! ## Data provenance and IP boundary
//! **No dataset bytes are vendored or redistributed.** Each record cites the third-party source
//! (refinery / mining / wastewater / power benchmarks; UCI, Kaggle, Zenodo, CRAN, Mendeley) with its
//! URL and licence, and flags access (open / Kaggle / gated / code-generates-data). No ownership of any
//! dataset is claimed. The prior-art / novelty is the **DSFB and Densor (Deterministic Endoduction
//! Tekmeric Inference)** deterministic inference-from-residuals-and-noise and the heuristics-bank
//! *technology*, not the public data it is demonstrated on.
//!
//! ## Non-claims
//! Cataloguing a dataset asserts no predictive-accuracy superiority and no validated result on it; the
//! corpus is a sourced, deduplicated catalogue, and `implementation_status` honestly marks which
//! datasets a deterministic witness has actually been run on (`Executed`) versus catalogued only.

#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod hashes;
pub mod hashing;
pub mod seed;
pub mod types;
pub mod validation;

pub use hashes::corpus_hash_v1;
pub use seed::SOFT_SENSOR_DATASETS;
pub use types::{
    AccessConfidence, AccessStatus, ImplementationStatus, LicenseConfidence, ProcessDomain,
    RedistributionPolicy, SoftSensorDatasetRecordV1, SourceAuthorityKind, SourceRef, TargetKind,
};
pub use validation::{census, validate, ClassificationCensus, CorpusValidationReport};
