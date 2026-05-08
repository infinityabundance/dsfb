//! Bank-optimisation audit harness (Session 17, post-Phase-8).
//!
//! Read-only telemetry over the existing fusion + per-detector
//! pipeline. The audit harness consumes `DetectorOutput` slices
//! captured during a normal `run_fusion_evaluation` call and
//! produces:
//!
//! - per-detector selectivity index (fault-firing-rate /
//!   healthy-firing-rate),
//! - cross-fixture mean / stddev of selectivity per detector,
//! - per-axis (tier) discrimination ratios,
//! - confuser-pair utilisation evidence,
//! - leave-one-fixture-out cross-validation aggregates.
//!
//! The audit performs zero algorithmic mutation of the engine — every
//! input is already-computed `DetectorOutput` data; every output is a
//! report struct or rendered markdown. Bank curation
//! (`heuristics_bank.rs`) is untouched by the audit module itself;
//! the canonical-bank merge in Phase ζ.9 is gated by the
//! cross-validation harness in this module.
//!
//! All audit functions are deterministic (Theorem 9 preserved): given
//! the same input slice they produce the same numbers, in the same
//! BTreeMap iteration order.
//!
//! Standing discipline (Sessions 1-16): no synthetic data, no
//! Cargo-deps, no mutation of the no_std core. The audit module is
//! gated behind `#[cfg(feature = "std")]`.

#![cfg(feature = "std")]

pub mod detector_firing;
pub mod axis_discrimination;
pub mod confuser_audit;
pub mod loo_cv;
pub mod motif_refinement;
pub mod calibrated_weights;
pub mod bootstrap;

pub use detector_firing::{
    compute_detector_selectivity_per_fixture,
    aggregate_detector_audit,
    render_detector_selectivity_md,
    DetectorSelectivity,
    CrossFixtureDetectorEntry,
    CrossFixtureDetectorReport,
};

pub use axis_discrimination::{
    compute_axis_discrimination,
    render_axis_discrimination_md,
    AxisDiscriminationEntry,
    AxisDiscriminationReport,
};

pub use confuser_audit::{
    audit_confuser_pairs,
    render_confuser_audit_md,
    ConfuserPairAuditEntry,
    ConfuserAuditReport,
};

pub use loo_cv::{
    run_loo_cv,
    aggregate_loo_cv,
    aggregate_kfold_cv,
    render_loo_cv_baseline_md,
    render_kfold_cv_md,
    refinement_passes_gate,
    LooCvFixtureRecord,
    LooCvAggregate,
    KFoldCvAggregate,
    KFoldFoldRecord,
    RefinementGateVerdict,
};

pub use motif_refinement::{
    build_motif_refinement,
    build_motif_refinement_from_observations,
    render_motif_refinement_md,
    EpisodeMotifObservation,
    MotifRefinementEntry,
    MotifRefinementReport,
    AffinityDivergence,
};

pub use calibrated_weights::{
    canonical_calibrated_weight_overrides,
    TOP_DETECTORS_BY_SELECTIVITY,
};

pub use bootstrap::{
    bootstrap_ci,
    bootstrap_ci_with_seed,
    render_bootstrap_md,
    BootstrapAggregate,
    BootstrapCi,
    DEFAULT_BOOTSTRAP_ITERATIONS,
    DEFAULT_BOOTSTRAP_SEED,
};
