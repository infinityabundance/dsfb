// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Helper output-path resolution for additive workflows.

use chrono::Local;
use std::path::{Path, PathBuf};

pub fn resolve_helper_output_dir(
    crate_dir: &Path,
    workflow_dir: &str,
    workflow_prefix: &str,
    explicit_output: Option<PathBuf>,
) -> PathBuf {
    explicit_output.unwrap_or_else(|| {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        crate_dir
            .join("outputs")
            .join(workflow_dir)
            .join(format!("{}_{}", workflow_prefix, timestamp))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_output_defaults_to_timestamped_nested_directory() {
        let crate_dir = Path::new("/tmp/dsfb-battery");
        let resolved = resolve_helper_output_dir(
            crate_dir,
            "multicell",
            "dsfb_battery_multicell",
            None,
        );

        assert_eq!(resolved.parent().unwrap().file_name().unwrap(), "multicell");
        let stem = resolved.file_name().unwrap().to_string_lossy();
        assert!(stem.starts_with("dsfb_battery_multicell_"));
        assert_eq!(stem.len(), "dsfb_battery_multicell_YYYYMMDD_HHMMSS".len());
    }

    #[test]
    fn helper_output_preserves_explicit_override() {
        let crate_dir = Path::new("/tmp/dsfb-battery");
        let explicit = PathBuf::from("/tmp/custom-output");
        let resolved = resolve_helper_output_dir(
            crate_dir,
            "ablation",
            "dsfb_battery_ablation",
            Some(explicit.clone()),
        );

        assert_eq!(resolved, explicit);
    }
}
