//! Atlas detector algebra + S1.2 registry generator (post-T.10).
//!
//! **Front-door identity (panel-locked)**: this crate generates
//! the `DetectorSpec` records the densorial / tekmeric
//! inference stack admits as deterministic witnesses. The
//! Atlas does not learn detectors; it generates them as a
//! cartesian expansion of a panel-locked parameter grid over
//! the T.10-frozen literature corpus, then binds every spec to
//! `corpus_hash_v1` so the registry is a court artifact, not a
//! detector zoo. The GPU side (the `dsfb-gpu-debug-cuda` CUDA
//! Evidence Factory) shades residual densors into canonical
//! evidence bytes; this registry tells the court which
//! detector specs that evidence is allowed to be admitted
//! against.
//!
//! **Panel-locked scope (S1.1 + S1.1.1 + S1.2)**:
//!
//! S1.1 defined the detector algebra type system. S1.1.1 added the
//! post-T.10 cross-field rule binding every spec to either a zero
//! `source_corpus_hash` (algebra-only fixture) or the canonical
//! `corpus_hash_v1` (registry-bound). **S1.2** is the disciplined
//! first registry-generation pass: walk the 54-record T.10-frozen
//! corpus seed across a panel-locked 3-point parameter grid,
//! emit 162 `DetectorSpec` records bound to `corpus_hash_v1`,
//! and compute `registry_hash_v2` over the canonical-byte
//! projection of the generated list. The crate does NOT:
//!
//! - Generate 2,000 detectors. S1.2 is panel-bounded to 150-300
//!   specs; the wider grid lands as S1.2.1+ after planner /
//!   dispatcher infrastructure matures.
//! - Emit `CaseFileV2` bodies (T.11 work).
//! - Execute Atlas family kernels on GPU.
//! - Wire NVRTC / JIT Mode B.
//! - Touch the Detector Delta Ledger.
//! - Land D512 / D1024 / D2000 ladder rungs.
//! - Make publication-grade claims about Atlas detector count.
//! - Activate detectors (no activation planner; S1.3 work).
//! - Admit episodes (no GPU execution; future work).
//!
//! **Panel-locked grammar**:
//!
//! ```text
//!   Detector = LiteraturePrimitive
//!            × ParameterGrid
//!            × Transform
//!            × Window
//!            × Comparator
//!            × Gate
//!            → DetectorSpec
//! ```
//!
//! The algebra is the deterministic spelling of a detector
//! identity. Every `DetectorSpec` carries a canonical name in the
//! panel-locked format:
//!
//! ```text
//!   {FAMILY}__{TRANSFORM}__W{WINDOW}__{STATISTIC}__{COMPARATOR}__P{PERSISTENCE}
//! ```
//!
//! Example: `ROBUST_Z_MAD__RESIDUAL__W64__MAD__TWO_SIDED__P3`.
//!
//! **Corpus binding (post-T.10 cross-field rule)**: every
//! `DetectorSpec` carries `corpus_binding_status` plus a
//! `source_corpus_hash: [u8; 32]`. The verifier admits
//! `HashFrozenT10` iff `source_corpus_hash != [0; 32]`, and
//! admits `PreHashT9InternalAudit` iff `source_corpus_hash ==
//! [0; 32]`. This honest cross-field gate prevents a spec from
//! claiming the corpus freeze without carrying its bytes, and
//! prevents a pre-hash spec from carrying a stale or fabricated
//! corpus hash. S1.2 additionally requires the hash equal the
//! live `compute_corpus_hash_v1()` and the `primitive_id` resolve
//! to a known canonical corpus record; see
//! [`verify::verify_registry_spec`].
//!
//! **The Atlas does not learn detector usefulness.** S1.1.1 ships
//! the grammar plate + the post-T.10 cross-field rule, not the
//! factory. The disciplined first registry pass (162 specs over
//! 54 corpus primitives × a 3-point grid) is the separate S1.2
//! commit; it builds on top of this verifier without changing it.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
// Tests assert with `.unwrap()` / `.expect()` for short, readable
// invariants; the workspace's pedantic lints would otherwise flag
// each occurrence.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod axis;
pub mod canonical_name;
pub mod comparator;
pub mod corpus_binding;
pub mod cost;
pub mod domain;
pub mod family;
pub mod family_mapping;
pub mod gate;
pub mod generator;
pub mod hash;
pub mod ids;
pub mod implementation;
pub mod numeric;
pub mod parameter;
pub mod registry;
pub mod spec;
pub mod statistic;
pub mod template;
pub mod transform;
pub mod verify;
pub mod window;

pub use axis::AxisBinding;
pub use canonical_name::CanonicalDetectorName;
pub use comparator::Comparator;
pub use corpus_binding::CorpusBindingStatus;
pub use cost::CostClass;
pub use domain::{DomainTag, DomainTagSet};
pub use family::DetectorFamily;
pub use family_mapping::corpus_to_registry_family;
pub use gate::Gate;
pub use generator::{generate_s1_2_specs, S1_2_GRID_POINTS_PER_RECORD};
pub use hash::{
    compute_registry_hash_v2, write_registry_hash_v2_material, REGISTRY_HASH_V2_DOMAIN,
    REGISTRY_HASH_V2_SCHEMA,
};
pub use ids::{DetectorId, FamilyId, ParameterizationId};
pub use implementation::ImplementationKind;
pub use numeric::NumericMode;
pub use parameter::DetectorParamSet;
pub use registry::{DetectorRegistryV2, RegistryCounts};
pub use spec::DetectorSpec;
pub use statistic::Statistic;
pub use template::DetectorTemplate;
pub use transform::Transform;
pub use verify::{
    verify_registry_spec, verify_spec, verify_template, VerifyError, VerifyErrorKind,
};
pub use window::WindowSpec;
