// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Versioned, read-only heuristics-bank helper.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use thiserror::Error;

pub const HEURISTICS_BANK_JSON: &str = "config/heuristics_bank_v1.json";
pub const HEURISTICS_BANK_SHA256: &str = "config/heuristics_bank_v1.sha256";

#[derive(Debug, Error)]
pub enum HeuristicsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("heuristics bank hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("heuristics bank missing required fields")]
    InvalidShape,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicsBankEntryRecord {
    pub entry_id: String,
    pub pattern: String,
    pub regime_scope: String,
    pub admissibility_assumptions: String,
    pub interpretation: String,
    pub uncertainty_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicsBankArtifact {
    pub artifact_type: String,
    pub schema_version: String,
    pub bank_version: String,
    pub frozen: bool,
    pub entries: Vec<HeuristicsBankEntryRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeuristicsBankVerification {
    pub artifact_type: String,
    pub bank_version: String,
    pub verified: bool,
    pub expected_sha256: String,
    pub actual_sha256: String,
}

pub fn load_heuristics_bank(crate_dir: &Path) -> Result<HeuristicsBankArtifact, HeuristicsError> {
    let path = crate_dir.join(HEURISTICS_BANK_JSON);
    let json = std::fs::read_to_string(path)?;
    let bank: HeuristicsBankArtifact = serde_json::from_str(&json)?;
    if bank.artifact_type.is_empty()
        || bank.schema_version.is_empty()
        || bank.bank_version.is_empty()
        || bank.entries.is_empty()
    {
        return Err(HeuristicsError::InvalidShape);
    }
    Ok(bank)
}

pub fn verify_heuristics_bank(
    crate_dir: &Path,
) -> Result<HeuristicsBankVerification, HeuristicsError> {
    let bank = load_heuristics_bank(crate_dir)?;
    let json_path = crate_dir.join(HEURISTICS_BANK_JSON);
    let hash_path = crate_dir.join(HEURISTICS_BANK_SHA256);

    let bytes = std::fs::read(json_path)?;
    let actual_sha256 = sha256_hex(&bytes);
    let expected_sha256 = std::fs::read_to_string(hash_path)?.trim().to_string();

    if expected_sha256 != actual_sha256 {
        return Err(HeuristicsError::HashMismatch {
            expected: expected_sha256,
            actual: actual_sha256,
        });
    }

    Ok(HeuristicsBankVerification {
        artifact_type: "dsfb_battery_heuristics_bank_verification".to_string(),
        bank_version: bank.bank_version,
        verified: true,
        expected_sha256: expected_sha256.clone(),
        actual_sha256,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heuristics_bank_verification_passes_for_tracked_bank() {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let verification = verify_heuristics_bank(crate_dir).unwrap();
        assert!(verification.verified);
        assert_eq!(verification.expected_sha256, verification.actual_sha256);
    }
}
