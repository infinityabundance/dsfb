//! The densor manifest — the frozen declaration of which densors + authorities a pipeline run is allowed to use.
//!
//! Before executing anything, the runtime loads this manifest and validates it: every entry must be identifiable,
//! ids must be unique, and the frozen authority set is the allow-list a stage's declared authorities are checked
//! against. The manifest is data (no behaviour); validation is pure.

use crate::authority::AuthorityHash;
use crate::densor::DensorKind;
use crate::errors::RuntimeError;
use serde::{Deserialize, Serialize};

/// One declared densor in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DensorEntry {
    pub id: String,
    pub kind: DensorKind,
    /// The expected 32-byte evidence hash of this densor (checked against the produced object on execution).
    pub evidence_hash: [u8; 32],
}

/// A pipeline's frozen manifest: the densors it carries + the authority allow-list its stages may cite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DensorManifest {
    pub pipeline_id: String,
    pub densors: Vec<DensorEntry>,
    /// The complete frozen set of authorities any stage in this run is permitted to have been built against.
    pub authorities: Vec<AuthorityHash>,
}

impl DensorManifest {
    /// Validate the manifest is well-formed: non-empty pipeline id, every densor identifiable, unique ids, and at
    /// least one frozen authority (a run with no authority anchor is not admissible). Pure; never mutates.
    pub fn validate(&self) -> Result<(), RuntimeError> {
        if self.pipeline_id.trim().is_empty() {
            return Err(RuntimeError::ManifestInvalid("empty pipeline_id".into()));
        }
        if self.authorities.is_empty() {
            return Err(RuntimeError::ManifestInvalid(
                "no frozen authorities (a run needs an authority anchor)".into(),
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        for d in &self.densors {
            if d.id.trim().is_empty() {
                return Err(RuntimeError::ManifestInvalid(
                    "a densor entry has an empty id".into(),
                ));
            }
            if !seen.insert(d.id.as_str()) {
                return Err(RuntimeError::ManifestInvalid(format!(
                    "duplicate densor id '{}'",
                    d.id
                )));
            }
        }
        Ok(())
    }

    /// True iff `authority` is in the manifest's frozen allow-list (matched by name AND digest).
    pub fn permits_authority(&self, authority: &AuthorityHash) -> bool {
        self.authorities
            .iter()
            .any(|a| a.name == authority.name && a.hash == authority.hash)
    }
}
