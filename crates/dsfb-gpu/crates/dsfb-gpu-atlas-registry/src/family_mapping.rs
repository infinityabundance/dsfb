//! S1.2 — corpus `PrimitiveFamily` → registry `DetectorFamily`
//! mapping.
//!
//! The corpus crate's [`dsfb_gpu_atlas_corpus::types::PrimitiveFamily`]
//! has 18 broad family categories (ScalarThreshold, WindowStatistic,
//! SequentialRecurrence, …). The registry's
//! [`crate::DetectorFamily`] has 43 panel-recommended named
//! variants (Shewhart, Ewma, Cusum, …, EvmAnomaly). They are
//! semantically related but not 1:1; this mapping picks **one
//! representative registry family** per corpus family so the S1.2
//! generator can mint specs from every seed record without
//! requiring a hand-tuned per-record table.
//!
//! The mapping is hand-pinned, **panel-locked**, and one-to-one
//! at the granularity of corpus `PrimitiveFamily`. The S1.2
//! `registry_hash_v2` depends on this mapping byte-for-byte, so
//! changing it changes the receipt hash.

use crate::DetectorFamily;
use dsfb_gpu_atlas_corpus::types::PrimitiveFamily;

/// Map a corpus `PrimitiveFamily` to its representative registry
/// `DetectorFamily`. The choices reflect the closest named
/// detector in the registry seed; e.g. `WindowStatistic` → `Ewma`
/// because EWMA is the canonical window-statistic representative.
///
/// **Stability**: the mapping is panel-locked. Changing it
/// changes `registry_hash_v2` on every spec generated from the
/// affected corpus records.
#[must_use]
pub const fn corpus_to_registry_family(pf: PrimitiveFamily) -> DetectorFamily {
    match pf {
        PrimitiveFamily::ScalarThreshold => DetectorFamily::RobustZMad,
        PrimitiveFamily::WindowStatistic => DetectorFamily::Ewma,
        PrimitiveFamily::SequentialRecurrence => DetectorFamily::Cusum,
        PrimitiveFamily::DistributionDistance => DetectorFamily::KolmogorovSmirnov,
        PrimitiveFamily::RankStatistic => DetectorFamily::MannKendall,
        PrimitiveFamily::Spectral => DetectorFamily::FftBandEnergy,
        PrimitiveFamily::Wavelet => DetectorFamily::WaveletEnergy,
        // Graph families + negative-witness primitives do not yet
        // have a named registry seed at S1.2; all three map to
        // AutocorrelationBreak as the closest "structural
        // temporal anti-pattern" representative. S1.2-followups
        // may introduce GraphLocal / GraphGlobal / Confuser
        // registry families.
        PrimitiveFamily::GraphLocal
        | PrimitiveFamily::GraphGlobal
        | PrimitiveFamily::NegativeWitness => DetectorFamily::AutocorrelationBreak,
        PrimitiveFamily::TabularConstraint => DetectorFamily::FunctionalDependencyViolation,
        PrimitiveFamily::CategoricalHistogram => DetectorFamily::CardinalityDrift,
        PrimitiveFamily::Missingness => DetectorFamily::MissingnessSpike,
        PrimitiveFamily::ResidualObserver => DetectorFamily::ResidualEnvelopeExit,
        PrimitiveFamily::ProjectionResidual => DetectorFamily::PcaSpeQ,
        PrimitiveFamily::MultivariateHypothesis => DetectorFamily::HotellingT2,
        // InformationTheory primitives are represented by their
        // closest distribution-distance cousin; the registry
        // seed does not yet declare an information-theory family
        // explicitly.
        PrimitiveFamily::InformationTheory => DetectorFamily::JensenShannon,
        PrimitiveFamily::OperabilityDiagnostic => DetectorFamily::ValveHunting,
        PrimitiveFamily::DebugObservability => DetectorFamily::LatencyRamp,
    }
}
