//! Schema-validated JSON signature format for DSFB heuristics and motifs.
//!
//! # Digital Twin Fragments
//! A `.dsfb` signature file is a portable, schema-validated JSON document
//! that encodes exactly what a particular failure mode (e.g., "Target
//! Depletion" or "RF Matching Drift") looks like in DSFB semiotic space.
//!
//! Tool vendors can ship a signature file that references:
//! * The motif names that constitute the failure signature.
//! * The grammar state sequence expected during the failure episode.
//! * The physical sensors (by dimension tag) that carry the drift signal.
//! * The recommended operator action and escalation policy.
//!
//! The DSFB engine can then load these signatures at runtime and extend
//! the heuristics bank without recompilation.
//!
//! # Schema Version
//! Every signature file must declare a `schema_version` field.  The current
//! schema version is `"1.0"`.  The engine rejects files with unknown
//! schema versions at load time.

use crate::error::DsfbSemiconductorError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The schema version string for the current signature format.
pub const SIGNATURE_SCHEMA_VERSION: &str = "1.0";

// ─── Motif Signature ──────────────────────────────────────────────────────────

/// A named, serialisable motif entry suitable for embedding in a `.dsfb`
/// signature file.
///
/// # Schema Compatibility
/// This struct is the canonical serialisation target; do not add non-optional
/// fields without bumping `schema_version`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DsfbMotifSignature {
    /// Unique motif identifier — must match a value in
    /// [`ALLOWED_MOTIFS`](crate::syntax::ALLOWED_MOTIFS) or be a custom
    /// extension prefixed with `"custom_"`.
    pub motif_id: String,
    /// Human-readable description for operator-facing displays.
    pub description: String,
    /// Sequence of expected motif labels in temporal order (time → right).
    pub motif_sequence: Vec<String>,
    /// Grammar states that must be active concurrently with this motif.
    pub required_grammar_states: Vec<String>,
    /// Physical sensor dimension tags (e.g., `"sccm"`, `"milli_torr"`) that
    /// are expected to carry the signal.  [`None`] means "any dimension".
    pub expected_dimensions: Option<Vec<String>>,
    /// Minimum number of consecutive runs the motif must persist before
    /// this signature is considered matched.
    pub minimum_persistence_runs: usize,
}

// ─── Heuristics Bank Entry ────────────────────────────────────────────────────

/// A single entry in a serialisable, schema-validated heuristics bank.
///
/// This is the portable unit of knowledge: a tool vendor can ship a JSON
/// array of these entries as a `.dsfb` signature file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DsfbHeuristicSignature {
    /// Unique identifier for this heuristic.
    pub heuristic_id: String,
    /// Human-readable name (e.g., `"Target Depletion"`).
    pub name: String,
    /// Engineering description — appears in operator dashboards and audit
    /// trails.
    pub description: String,
    /// Motif signatures that compose this heuristic.
    pub motif_signatures: Vec<DsfbMotifSignature>,
    /// Recommended operator action on match: `"Monitor"`, `"Watch"`,
    /// `"Review"`, or `"Escalate"`.
    pub action: String,
    /// Escalation policy if the action is not resolved within the
    /// `escalation_timeout_runs` window.
    pub escalation_policy: String,
    /// Number of runs after initial match before auto-escalation.
    pub escalation_timeout_runs: usize,
    /// Whether this heuristic requires corroboration from ≥2 sensors.
    pub requires_corroboration: bool,
    /// Attribution / provenance: who authored this signature.
    pub author: Option<String>,
    /// Semantic label emitted in the traceability manifest on match.
    pub semantic_label: String,
    /// Known limitations for this heuristic.
    pub known_limitations: Option<String>,
}

// ─── Signature File ───────────────────────────────────────────────────────────

/// A complete `.dsfb` signature file: a schema-versioned bundle of heuristic
/// signatures ready for runtime loading.
///
/// # Example
/// ```
/// use dsfb_semiconductor::signature::{DsfbSignatureFile, SIGNATURE_SCHEMA_VERSION};
///
/// let file = DsfbSignatureFile {
///     schema_version: SIGNATURE_SCHEMA_VERSION.into(),
///     tool_class: "ICP Etch".into(),
///     vendor: Some("Example Semiconductor Equipment Inc.".into()),
///     heuristics: vec![],
/// };
///
/// let json = serde_json::to_string_pretty(&file).unwrap();
/// assert!(json.contains("schema_version"));
/// assert!(json.contains("1.0"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DsfbSignatureFile {
    /// Schema version — must be `"1.0"` for this crate version.
    pub schema_version: String,
    /// Broad class of tool this signature targets (e.g., `"ICP Etch"`,
    /// `"PECVD"`, `"CMP"`).
    pub tool_class: String,
    /// Optional vendor attribution.
    pub vendor: Option<String>,
    /// The heuristic entries in this signature file.
    pub heuristics: Vec<DsfbHeuristicSignature>,
}

impl DsfbSignatureFile {
    /// Validate schema version and structural constraints.
    ///
    /// Returns `Err` with a descriptive message if validation fails.
    pub fn validate(&self) -> Result<(), DsfbSemiconductorError> {
        if self.schema_version != SIGNATURE_SCHEMA_VERSION {
            return Err(DsfbSemiconductorError::Config(format!(
                "unsupported signature schema version '{}'; expected '{}'",
                self.schema_version, SIGNATURE_SCHEMA_VERSION
            )));
        }

        if self.tool_class.trim().is_empty() {
            return Err(DsfbSemiconductorError::Config(
                "tool_class must not be empty".into(),
            ));
        }

        for h in &self.heuristics {
            if h.heuristic_id.trim().is_empty() {
                return Err(DsfbSemiconductorError::Config(
                    "heuristic_id must not be empty".into(),
                ));
            }
            let valid_actions = ["Monitor", "Watch", "Review", "Escalate"];
            if !valid_actions.contains(&h.action.as_str()) {
                return Err(DsfbSemiconductorError::Config(format!(
                    "heuristic '{}': action '{}' is not in {:?}",
                    h.heuristic_id, h.action, valid_actions
                )));
            }
            for motif in &h.motif_signatures {
                if motif.minimum_persistence_runs == 0 {
                    return Err(DsfbSemiconductorError::Config(format!(
                        "heuristic '{}' motif '{}': minimum_persistence_runs must be > 0",
                        h.heuristic_id, motif.motif_id
                    )));
                }
            }
        }

        Ok(())
    }

    /// Load and validate a signature file from disk.
    ///
    /// # Errors
    /// Returns [`DsfbSemiconductorError`] on I/O failure, JSON parse failure,
    /// or schema validation failure.
    pub fn load(path: &Path) -> Result<Self, DsfbSemiconductorError> {
        let content = std::fs::read_to_string(path)
            .map_err(DsfbSemiconductorError::Io)?;
        let file: Self = serde_json::from_str(&content)
            .map_err(|e| DsfbSemiconductorError::Config(format!("JSON parse error: {e}")))?;
        file.validate()?;
        Ok(file)
    }

    /// Serialise to a pretty-printed JSON string.
    pub fn to_json_pretty(&self) -> Result<String, DsfbSemiconductorError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| DsfbSemiconductorError::Config(format!("JSON serialise error: {e}")))
    }

    /// Write to a file path.
    pub fn write(&self, path: &Path) -> Result<(), DsfbSemiconductorError> {
        let json = self.to_json_pretty()?;
        std::fs::write(path, json)
            .map_err(DsfbSemiconductorError::Io)
    }

    /// Return a reference signature for the "Target Depletion" failure mode.
    /// This can be shipped as an example `.dsfb` file to tool vendors.
    pub fn example_target_depletion() -> Self {
        Self {
            schema_version: SIGNATURE_SCHEMA_VERSION.into(),
            tool_class: "ICP Etch".into(),
            vendor: Some("reference_dsfb_v1".into()),
            heuristics: vec![DsfbHeuristicSignature {
                heuristic_id: "target_depletion_v1".into(),
                name: "Target Depletion (Sputter Source)".into(),
                description: concat!(
                    "Slow, monotonic drift of the gas-flow residual toward the ",
                    "admissibility boundary, co-occurring with a matching pressure ",
                    "drift in the opposite direction.  Signature of consumable ",
                    "target erosion in sputter-based etch chambers."
                )
                .into(),
                motif_signatures: vec![DsfbMotifSignature {
                    motif_id: "slow_drift_precursor".into(),
                    description: "Monotonic positive drift approaching ρ".into(),
                    motif_sequence: vec![
                        "slow_drift_precursor".into(),
                        "boundary_grazing".into(),
                        "persistent_instability".into(),
                    ],
                    required_grammar_states: vec![
                        "SustainedDrift".into(),
                        "BoundaryGrazing".into(),
                        "PersistentViolation".into(),
                    ],
                    expected_dimensions: Some(vec!["sccm".into(), "milli_torr".into()]),
                    minimum_persistence_runs: 5,
                }],
                action: "Review".into(),
                escalation_policy: "Escalate if motif persists for > 25 runs without recovery".into(),
                escalation_timeout_runs: 25,
                requires_corroboration: true,
                author: Some("DSFB Reference Library v1.0".into()),
                semantic_label: "target_depletion".into(),
                known_limitations: Some(concat!(
                    "False positives possible when gas composition changes due to ",
                    "recipe parameter sweep; gate with ProcessContext.recipe_step.",
                ).into()),
            }],
        }
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_signature_is_valid() {
        let sig = DsfbSignatureFile::example_target_depletion();
        sig.validate().expect("example signature should be valid");
    }

    #[test]
    fn wrong_schema_version_is_rejected() {
        let mut sig = DsfbSignatureFile::example_target_depletion();
        sig.schema_version = "0.99".into();
        assert!(sig.validate().is_err());
    }

    #[test]
    fn invalid_action_is_rejected() {
        let mut sig = DsfbSignatureFile::example_target_depletion();
        sig.heuristics[0].action = "Alert".into(); // not in allowed set
        assert!(sig.validate().is_err());
    }

    #[test]
    fn empty_tool_class_is_rejected() {
        let mut sig = DsfbSignatureFile::example_target_depletion();
        sig.tool_class = "  ".into();
        assert!(sig.validate().is_err());
    }

    #[test]
    fn zero_persistence_is_rejected() {
        let mut sig = DsfbSignatureFile::example_target_depletion();
        sig.heuristics[0].motif_signatures[0].minimum_persistence_runs = 0;
        assert!(sig.validate().is_err());
    }

    #[test]
    fn round_trip_json_serialisation() {
        let sig = DsfbSignatureFile::example_target_depletion();
        let json = sig.to_json_pretty().unwrap();
        let parsed: DsfbSignatureFile = serde_json::from_str(&json).unwrap();
        assert_eq!(sig, parsed);
    }

    #[test]
    fn write_and_load_via_tempfile() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.dsfb");
        let sig = DsfbSignatureFile::example_target_depletion();
        sig.write(&path).unwrap();
        let loaded = DsfbSignatureFile::load(&path).unwrap();
        assert_eq!(sig, loaded);
    }
}
