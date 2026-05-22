//! S1.2 — `DetectorRegistryV2` collection type.
//!
//! Wraps the deterministic
//! [`crate::generator::generate_s1_2_specs`] output with
//! `registry_hash_v2` provenance + counts that distinguish the
//! four panel-required population kinds:
//!
//! - **literature primitives** (the corpus seed, 54).
//! - **parameterized detector specs** (the registry output, 162
//!   at S1.2 = 54 × 3-point grid).
//! - **active detectors** (the subset the activation planner
//!   would mark active; at S1.2 all generated specs are
//!   `ImplementationKind::ScalarCpu` so the activation planner
//!   is NOT defined here; the "active" count is reported as 0
//!   until S1.3+ lands an activation planner).
//! - **admitted episodes** (always 0 at S1.2 — no GPU execution
//!   in this commit).

extern crate alloc;
use alloc::vec::Vec;

use crate::generator::generate_s1_2_specs;
use crate::hash::compute_registry_hash_v2;
use crate::DetectorSpec;

/// Panel-locked four-tier count of detector populations.
#[derive(Debug, Clone, Copy)]
pub struct RegistryCounts {
    /// Distinct literature primitives in the corpus (the seed
    /// record count). 54 at T.9 / T.10.
    pub literature_primitives: usize,
    /// Parameterised detector specs in the registry (S1.2
    /// generator output). 162 at S1.2 = 54 × 3 grid points.
    pub parameterized_specs: usize,
    /// Detectors the activation planner would mark active for a
    /// concrete task. At S1.2 the planner does not exist; the
    /// count is 0.
    pub active_detectors: usize,
    /// Bank-admitted episodes from a concrete run. Always 0 at
    /// S1.2 — no GPU execution in this commit.
    pub admitted_episodes: usize,
}

/// One full S1.2 detector registry.
#[derive(Debug, Clone)]
pub struct DetectorRegistryV2 {
    /// Generated specs in canonical order
    /// (`record.canonical_id` ascending, then `parameterization_id`
    /// ascending).
    pub specs: Vec<DetectorSpec>,
    /// 32-byte SHA-256 commitment over the canonical-byte
    /// projection of the registry. Two builds produce the same
    /// value.
    pub registry_hash_v2: [u8; 32],
    /// Four-tier population counts.
    pub counts: RegistryCounts,
    /// The `corpus_hash_v1` the registry is bound to.
    pub source_corpus_hash: [u8; 32],
}

impl DetectorRegistryV2 {
    /// Build the canonical S1.2 detector registry from the live
    /// corpus. Deterministic across two calls.
    #[must_use]
    pub fn build() -> Self {
        let specs = generate_s1_2_specs();
        let registry_hash_v2 = compute_registry_hash_v2(&specs);
        // Every spec carries the same `source_corpus_hash`; we
        // pull it from spec[0] (the generator guarantees the
        // list is non-empty as long as the corpus seed is
        // non-empty).
        let source_corpus_hash = if let Some(first) = specs.first() {
            first.source_corpus_hash
        } else {
            [0u8; 32]
        };
        let counts = RegistryCounts {
            literature_primitives: dsfb_gpu_atlas_corpus::seed::SEED.len(),
            parameterized_specs: specs.len(),
            // No activation planner at S1.2; no admitted
            // episodes either.
            active_detectors: 0,
            admitted_episodes: 0,
        };
        Self {
            specs,
            registry_hash_v2,
            counts,
            source_corpus_hash,
        }
    }
}
