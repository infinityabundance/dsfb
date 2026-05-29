//! Navigation layer (panel "Immediate" item): the loader + canonical seal for `breadth_surface.toml`, a
//! top-level, machine-readable index of every major **verifiable claim → governing artifact → reproduction
//! command → evidence tier**. It turns the project's deliberate density into *organized* density: one place a
//! reader (or an auditor) can scan the whole claim surface and, for any claim, run exactly one command to get
//! its sealed artifact.
//!
//! It asserts **no new capability** — it is a curated index over what the courts/tests/manifests already
//! prove. Its honesty mechanism is the self-check court (`tests/breadth_surface_court.rs`): every tier must
//! resolve to a real [`ClaimStrength`], every CLI reproduction must name a real subcommand, every governing
//! artifact must exist on disk (or be a known frozen-hash name), and the category counts must equal what
//! `atlas::validate()` + the committed manifests actually report — so the index can never silently drift from
//! reality. The parsed index is hash-sealed ([`BreadthSurfaceV1::canonical_hash`]) so it joins the
//! governed-artifact discipline like `atlas_hash_v1` / the bundle roots.

use serde::Deserialize;

use crate::claim_strength::ClaimStrength;
use crate::hashing::CanonicalHasher;

/// Independently-known category counts the self-check court cross-validates against reality.
#[derive(Debug, Clone, Deserialize)]
pub struct ExpectedCounts {
    pub executed_detectors: usize,
    pub executed_fault_signatures: usize,
    pub fault_signatures: usize,
    pub heuristics: usize,
    pub datasets: usize,
    pub golden_replays: usize,
    pub balance_witnesses: usize,
    pub figures: usize,
}

/// One indexed claim: a stable statement with its tier, governing artifact, and reproduction command.
#[derive(Debug, Clone, Deserialize)]
pub struct Claim {
    /// Stable, unique, category-prefixed id (e.g. `AUTH-ATLAS-01`).
    pub id: String,
    /// One of the fixed category labels (see `CATEGORIES` in the court).
    pub category: String,
    /// The claim, in one sentence.
    pub statement: String,
    /// Evidence tier — MUST equal a [`ClaimStrength`] tag (`SealedFact` / `EvidenceInterpretation` /
    /// `SpeculativeImplication` / `NonClaim`).
    pub tier: String,
    /// The artifact that governs the claim: a repo-relative path, or a frozen-hash name.
    pub governing_artifact: String,
    /// `file` | `test` | `manifest` | `figure_manifest` | `script` | `frozen_hash`.
    pub artifact_kind: String,
    /// The exact command that reproduces the claim's evidence.
    pub reproduce: String,
    /// `cli_subcommand` | `cargo_test` | `script` | `external`.
    pub reproduce_kind: String,
    /// The claim's bounded non-claim (required non-empty for any Tier-3 `SpeculativeImplication`).
    #[serde(default)]
    pub non_claim_boundary: String,
}

/// The parsed `breadth_surface.toml` (schema v1).
#[derive(Debug, Clone, Deserialize)]
pub struct BreadthSurfaceV1 {
    pub schema: String,
    pub expected_counts: ExpectedCounts,
    #[serde(rename = "claim", default)]
    pub claims: Vec<Claim>,
}

impl BreadthSurfaceV1 {
    /// Load `breadth_surface.toml` from the repository root.
    pub fn load(repo_root: &std::path::Path) -> Result<Self, String> {
        let p = repo_root.join("breadth_surface.toml");
        let s = std::fs::read_to_string(&p).map_err(|e| format!("read {}: {e}", p.display()))?;
        toml::from_str(&s).map_err(|e| format!("parse breadth_surface.toml: {e}"))
    }

    /// Resolve a tier tag to its [`ClaimStrength`] (the inverse of [`ClaimStrength::tag`]); `None` if unknown.
    pub fn tier_of(tag: &str) -> Option<ClaimStrength> {
        [
            ClaimStrength::SealedFact,
            ClaimStrength::EvidenceInterpretation,
            ClaimStrength::SpeculativeImplication,
            ClaimStrength::NonClaim,
        ]
        .into_iter()
        .find(|c| c.tag() == tag)
    }

    /// Canonical digest over the parsed index in a fixed field order. Hashing the parsed *content* (not the
    /// raw file bytes) means comment/whitespace edits don't churn the seal, but any claim / tier / count
    /// change does — so every additive breadth widening is a single, reviewable, intentional re-freeze.
    pub fn canonical_hash(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", self.schema.as_bytes());
        let c = &self.expected_counts;
        for (k, v) in [
            ("executed_detectors", c.executed_detectors),
            ("executed_fault_signatures", c.executed_fault_signatures),
            ("fault_signatures", c.fault_signatures),
            ("heuristics", c.heuristics),
            ("datasets", c.datasets),
            ("golden_replays", c.golden_replays),
            ("balance_witnesses", c.balance_witnesses),
            ("figures", c.figures),
        ] {
            h.field("count_key", k.as_bytes());
            h.u64("count_val", v as u64);
        }
        for cl in &self.claims {
            h.field("id", cl.id.as_bytes());
            h.field("category", cl.category.as_bytes());
            h.field("statement", cl.statement.as_bytes());
            h.field("tier", cl.tier.as_bytes());
            h.field("governing_artifact", cl.governing_artifact.as_bytes());
            h.field("artifact_kind", cl.artifact_kind.as_bytes());
            h.field("reproduce", cl.reproduce.as_bytes());
            h.field("reproduce_kind", cl.reproduce_kind.as_bytes());
            h.field("non_claim_boundary", cl.non_claim_boundary.as_bytes());
        }
        h.finalize_hex()
    }

    /// For a `cli_subcommand` reproduction `"dsfb-chem-edge <sub> ..."`, the subcommand token (`<sub>`).
    pub fn cli_subcommand(reproduce: &str) -> Option<&str> {
        let mut it = reproduce.split_whitespace();
        let _bin = it.next()?; // "dsfb-chem-edge"
        it.next()
    }
}
