//! Typed heuristic-bank loading and normalization helpers.

use std::path::Path;

use anyhow::{anyhow, Context, Result};

use super::bank_builtin::builtin_heuristic_bank_entries;
use crate::engine::bank::{HeuristicBankMetadata, HeuristicBankRegistry};
use crate::io::schema::HEURISTIC_BANK_SCHEMA_VERSION;

pub(crate) fn load_builtin_registry() -> HeuristicBankRegistry {
    HeuristicBankRegistry {
        metadata: HeuristicBankMetadata {
            schema_version: HEURISTIC_BANK_SCHEMA_VERSION.to_string(),
            bank_version: "heuristic-bank/v3".to_string(),
            note: "Built-in conservative typed heuristic bank for deterministic structural semiotics retrieval.".to_string(),
        },
        entries: builtin_heuristic_bank_entries(),
    }
}

pub(crate) fn load_external_registry_json(path: &Path) -> Result<HeuristicBankRegistry> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read external heuristic bank {}", path.display()))?;
    let registry: HeuristicBankRegistry = serde_json::from_str(&source)
        .with_context(|| format!("failed to parse heuristic bank JSON {}", path.display()))?;
    Ok(normalize_registry(registry))
}

pub(crate) fn ensure_supported_bank_schema(schema_version: &str) -> Result<()> {
    if schema_version != HEURISTIC_BANK_SCHEMA_VERSION {
        return Err(anyhow!(
            "heuristic bank schema version mismatch: expected `{}`, got `{}`",
            HEURISTIC_BANK_SCHEMA_VERSION,
            schema_version
        ));
    }
    Ok(())
}

pub(crate) fn normalize_registry(mut registry: HeuristicBankRegistry) -> HeuristicBankRegistry {
    registry.entries.sort_by(|left, right| {
        left.heuristic_id
            .cmp(&right.heuristic_id)
            .then_with(|| left.retrieval_priority.cmp(&right.retrieval_priority))
    });
    for entry in &mut registry.entries {
        entry.compatible_with.sort();
        entry.incompatible_with.sort();
        entry.directional_incompatibility_exceptions.sort();
        entry.regime_tags.sort();
    }
    registry
}
