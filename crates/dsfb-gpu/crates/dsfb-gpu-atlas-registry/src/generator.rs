//! S1.2 — corpus-bound registry generator.
//!
//! Walks the live corpus
//! [`dsfb_gpu_atlas_corpus::seed::SEED`] (the 54 T.10-frozen
//! literature primitives) and emits a deterministic set of
//! [`crate::DetectorSpec`] records by cartesian-producting each
//! primitive against a small parameter grid.
//!
//! **Panel-locked scope (S1.2 is a disciplined first pass)**:
//!
//! The target is **150-300 detector specs**, not 2,000. The
//! generator is the machinery; future commits (S1.2.1+) widen
//! the grid. At S1.2 every primitive contributes exactly 3
//! parameter-grid points:
//!
//! - `(W32, persistence=2, comparator=High)`
//! - `(W64, persistence=3, comparator=TwoSided)`
//! - `(W128, persistence=5, comparator=TwoSided)`
//!
//! 54 primitives × 3 points = **162 generated specs**.
//!
//! **Provenance binding**: every generated spec carries:
//!
//! - `primitive_id = Some(record.canonical_id)` (the corpus
//!   canonical handle).
//! - `corpus_binding_status = HashFrozenT10`.
//! - `source_corpus_hash =
//!   compute_corpus_hash_v1().bytes` (the live T.10 hash).
//!
//! A registry-level verifier
//! ([`crate::verify::verify_registry_spec`]) rejects any spec
//! whose `source_corpus_hash` does not match the live corpus
//! hash. A registry generated against one corpus snapshot will
//! fail verification against any other snapshot.
//!
//! **Implementation kind**: every generated spec declares
//! [`crate::ImplementationKind::ScalarCpu`] regardless of the
//! corpus L-band. The five `L6_CpuGpuByteEquivalent` records
//! (canonical IDs 14, 15, 41, 42, 43 — the dsfb-gpu-debug-core
//! bank surface) DO have GPU implementations, but the S1.2
//! `DetectorSpec` declaration does NOT claim a GPU surface
//! because the registry generator has no GPU dispatch layer
//! yet. The honesty rule from S1.1 stands: `ImplementationKind`
//! is `ScalarCpu` unless explicitly upgraded by a future
//! commit that wires a family kernel.

extern crate alloc;
use alloc::vec::Vec;

use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::types::LiteratureDetector;
use dsfb_gpu_debug_core::sha256;

use crate::canonical_name::CanonicalDetectorName;
use crate::family_mapping::corpus_to_registry_family;
use crate::{
    AxisBinding, Comparator, CorpusBindingStatus, CostClass, DetectorId, DetectorParamSet,
    DetectorSpec, DomainTagSet, Gate, ImplementationKind, NumericMode, ParameterizationId,
    Statistic, Transform, WindowSpec,
};

/// One (window, persistence, comparator, gate) point in the
/// S1.2 grid. Three points per primitive, panel-locked.
#[derive(Debug, Clone, Copy)]
struct GridPoint {
    window: WindowSpec,
    persistence: u32,
    comparator: Comparator,
    gate: Gate,
}

const S1_2_GRID: &[GridPoint] = &[
    GridPoint {
        window: WindowSpec::W32,
        persistence: 2,
        comparator: Comparator::High,
        gate: Gate::Persistence,
    },
    GridPoint {
        window: WindowSpec::W64,
        persistence: 3,
        comparator: Comparator::TwoSided,
        gate: Gate::Persistence,
    },
    GridPoint {
        window: WindowSpec::W128,
        persistence: 5,
        comparator: Comparator::TwoSided,
        gate: Gate::Persistence,
    },
];

/// The number of parameter-grid points generated per corpus
/// record at S1.2. Pinned for the receipt and acceptance tests.
pub const S1_2_GRID_POINTS_PER_RECORD: usize = 3;

/// Pick a representative `Transform` for a corpus record. The
/// S1.2 grid uses a single transform per primitive (the
/// transform is more about the primitive's natural signal than
/// about the parameter sweep); subsequent commits may introduce
/// per-point transform variation.
const fn transform_for_record(_r: &LiteratureDetector) -> Transform {
    // Most literature primitives operate on residual / error
    // signals; pick `Residual` as the broad default. The
    // dsfb-gpu-debug bank primitives (LatencyRamp, ErrorBurst,
    // SlewShockRecovery, FanoutCascadeCandidate) can be
    // specialised in a follow-on commit.
    Transform::Residual
}

/// Pick a representative `Statistic` for a corpus record based
/// on its primitive family. The mapping mirrors the
/// `corpus_to_registry_family` mapping in spirit; both are
/// hand-pinned and panel-locked. Several primitive families
/// share the same representative statistic when the underlying
/// witness shape collapses to the same numeric form (e.g. both
/// `Spectral` and `Wavelet` reduce to `BandEnergy`); those arms
/// are merged via `|` patterns.
const fn statistic_for_record(r: &LiteratureDetector) -> Statistic {
    use dsfb_gpu_atlas_corpus::types::PrimitiveFamily;
    match r.primitive_family {
        PrimitiveFamily::ScalarThreshold => Statistic::Mad,
        PrimitiveFamily::WindowStatistic => Statistic::Mean,
        PrimitiveFamily::SequentialRecurrence | PrimitiveFamily::Missingness => Statistic::Sum,
        PrimitiveFamily::DistributionDistance | PrimitiveFamily::InformationTheory => {
            Statistic::Quantile
        }
        PrimitiveFamily::RankStatistic => Statistic::SignedRank,
        PrimitiveFamily::Spectral | PrimitiveFamily::Wavelet => Statistic::BandEnergy,
        PrimitiveFamily::GraphLocal
        | PrimitiveFamily::GraphGlobal
        | PrimitiveFamily::OperabilityDiagnostic => Statistic::Autocorrelation,
        PrimitiveFamily::TabularConstraint => Statistic::RunLength,
        PrimitiveFamily::CategoricalHistogram => Statistic::Iqr,
        PrimitiveFamily::ResidualObserver | PrimitiveFamily::DebugObservability => Statistic::Max,
        PrimitiveFamily::ProjectionResidual | PrimitiveFamily::MultivariateHypothesis => {
            Statistic::Variance
        }
        PrimitiveFamily::NegativeWitness => Statistic::Range,
    }
}

/// Compute a deterministic per-spec parameter hash. Hand-rolled
/// over the canonical bytes of the parameter tuple so the hash
/// does not depend on `Debug` output and is reproducible across
/// builds.
fn parameter_hash(record_canonical_id: u32, point_index: u32, point: GridPoint) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(64);
    buf.extend_from_slice(b"DSFB-GPU-ATLAS:S1.2-PARAMETER-HASH:v1\0");
    buf.extend_from_slice(&record_canonical_id.to_be_bytes());
    buf.extend_from_slice(&point_index.to_be_bytes());
    buf.extend_from_slice(&point.window.cells.to_be_bytes());
    buf.extend_from_slice(&point.persistence.to_be_bytes());
    buf.extend_from_slice(point.comparator.canonical_wire_name().as_bytes());
    buf.push(0);
    buf.extend_from_slice(point.gate.canonical_wire_name().as_bytes());
    buf.push(0);
    sha256(&buf)
}

/// Generate the S1.2 detector-spec list over the live corpus.
/// Two calls produce byte-identical output.
///
/// Each (corpus record, grid point) pair produces one spec.
/// Spec ids are assigned deterministically:
///
/// - `detector_id = record.canonical_id * 1000 + point_index`
/// - `parameterization_id = point_index`
///
/// (The factor of 1000 keeps detector ids comfortably separated
/// even if a future commit widens the grid to >1000 points per
/// record.)
#[must_use]
pub fn generate_s1_2_specs() -> Vec<DetectorSpec> {
    let corpus_hash = compute_corpus_hash_v1();
    let mut out: Vec<DetectorSpec> = Vec::with_capacity(SEED.len() * S1_2_GRID.len());
    for record in SEED {
        let registry_family = corpus_to_registry_family(record.primitive_family);
        let transform = transform_for_record(record);
        let statistic = statistic_for_record(record);
        for (point_index, point) in S1_2_GRID.iter().enumerate() {
            let params = DetectorParamSet::new(point.window.cells, point.persistence, 1 << 16, 0);
            let canonical_name = CanonicalDetectorName::build(
                registry_family,
                transform,
                statistic,
                point.comparator,
                params,
            );
            let detector_id =
                DetectorId(record.canonical_id.0 * 1000 + u32::try_from(point_index).unwrap_or(0));
            let parameterization_id = ParameterizationId(u32::try_from(point_index).unwrap_or(0));
            let parameter_hash =
                parameter_hash(record.canonical_id.0, parameterization_id.0, *point);

            // Inherit the corpus record's domain bits + fusion
            // axes byte-for-byte. The registry crate's bit
            // positions match the corpus crate's by construction
            // (pinned by an S1.1 test).
            let domain_tags = DomainTagSet::from_raw(record.origin_domains.0);
            let axis_binding = AxisBinding(record.fusion_axes.0);

            out.push(DetectorSpec {
                detector_id,
                parameterization_id,
                family: registry_family,
                transform,
                window: point.window,
                statistic,
                comparator: point.comparator,
                gate: point.gate,
                persistence_windows: point.persistence,
                axis_binding,
                domain_tags,
                cost_class: CostClass::Light,
                numeric_mode: NumericMode::Q16_16,
                // Honesty rule: ScalarCpu unless a future commit
                // wires a family kernel that genuinely runs on
                // GPU.
                implementation_kind: ImplementationKind::ScalarCpu,
                parameter_hash,
                primitive_id: Some(record.canonical_id),
                corpus_binding_status: CorpusBindingStatus::HashFrozenT10,
                source_corpus_hash: corpus_hash.bytes,
                canonical_name,
            });
        }
    }
    out
}
