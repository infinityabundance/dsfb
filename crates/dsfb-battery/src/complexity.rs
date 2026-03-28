// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Engineer-facing arithmetic-floor and complexity helper.

use crate::types::PipelineConfig;
use chrono::Utc;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ComplexityOperationEstimate {
    pub floating_point_add_sub: usize,
    pub floating_point_mul_div: usize,
    pub abs_ops: usize,
    pub comparisons: usize,
    pub integer_counter_updates: usize,
    pub window_reads: usize,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplexityMemoryFootprint {
    pub residual_window_samples: usize,
    pub drift_window_samples: usize,
    pub envelope_scalars: usize,
    pub persistence_counters: usize,
    pub config_scalars: usize,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplexityArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub output_contract: String,
    pub algorithmic_order_per_update: String,
    pub implementation_shape: String,
    pub operation_estimate: ComplexityOperationEstimate,
    pub memory_footprint: ComplexityMemoryFootprint,
    pub notes: Vec<String>,
}

pub fn estimate_dsfb_update_complexity(config: &PipelineConfig) -> ComplexityArtifact {
    ComplexityArtifact {
        artifact_type: "dsfb_battery_complexity_report".to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        output_contract: "engineer_facing_complexity_helper".to_string(),
        algorithmic_order_per_update: "O(1) arithmetic for the local update rules; O(N) over N samples for the current batch crate execution path.".to_string(),
        implementation_shape: "The current crate processes full capacity sequences in batch, but each residual/drift/slew and grammar-state update uses fixed-width windows and constant-size counters.".to_string(),
        operation_estimate: ComplexityOperationEstimate {
            floating_point_add_sub: 3,
            floating_point_mul_div: 2,
            abs_ops: 3,
            comparisons: 8,
            integer_counter_updates: 2,
            window_reads: 4,
            note: "Approximate warm-path count for residual, drift, slew, persistence, and grammar classification. This excludes CSV I/O, JSON export, and figure generation.".to_string(),
        },
        memory_footprint: ComplexityMemoryFootprint {
            residual_window_samples: config.drift_window + 1,
            drift_window_samples: config.drift_window + 1,
            envelope_scalars: 3,
            persistence_counters: 2,
            config_scalars: 8,
            note: "The streaming-state estimate covers the rolling residual/drift windows plus envelope and persistence state. The current crate also stores full vectors because the implementation is batch-oriented.".to_string(),
        },
        notes: vec![
            "The count is intentionally approximate and transparent rather than exact by instruction-level proof.".to_string(),
            "No embedded suitability claim is made beyond the arithmetic and state estimate reported here.".to_string(),
        ],
    }
}

pub fn write_complexity_report(
    artifact: &ComplexityArtifact,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut lines = Vec::new();
    lines.push("DSFB Battery Complexity Report".to_string());
    lines.push(format!("Generated at: {}", artifact.generated_at_utc));
    lines.push(format!(
        "Per-update order of growth: {}",
        artifact.algorithmic_order_per_update
    ));
    lines.push(format!(
        "Implementation shape: {}",
        artifact.implementation_shape
    ));
    lines.push("Operation estimate per warm update:".to_string());
    lines.push(format!(
        "- floating-point add/sub: {}",
        artifact.operation_estimate.floating_point_add_sub
    ));
    lines.push(format!(
        "- floating-point mul/div: {}",
        artifact.operation_estimate.floating_point_mul_div
    ));
    lines.push(format!(
        "- abs ops: {}",
        artifact.operation_estimate.abs_ops
    ));
    lines.push(format!(
        "- comparisons: {}",
        artifact.operation_estimate.comparisons
    ));
    lines.push(format!(
        "- integer counter updates: {}",
        artifact.operation_estimate.integer_counter_updates
    ));
    lines.push(format!(
        "- rolling window reads: {}",
        artifact.operation_estimate.window_reads
    ));
    lines.push(format!("  Note: {}", artifact.operation_estimate.note));
    lines.push("Memory/state footprint estimate:".to_string());
    lines.push(format!(
        "- residual window samples: {}",
        artifact.memory_footprint.residual_window_samples
    ));
    lines.push(format!(
        "- drift window samples: {}",
        artifact.memory_footprint.drift_window_samples
    ));
    lines.push(format!(
        "- envelope scalars: {}",
        artifact.memory_footprint.envelope_scalars
    ));
    lines.push(format!(
        "- persistence counters: {}",
        artifact.memory_footprint.persistence_counters
    ));
    lines.push(format!(
        "- config scalars: {}",
        artifact.memory_footprint.config_scalars
    ));
    lines.push(format!("  Note: {}", artifact.memory_footprint.note));
    for note in &artifact.notes {
        lines.push(format!("- {}", note));
    }

    std::fs::write(path, lines.join("\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complexity_report_mentions_batch_vs_update_shape() {
        let artifact = estimate_dsfb_update_complexity(&PipelineConfig::default());
        assert!(artifact.algorithmic_order_per_update.contains("O(1)"));
        assert!(artifact.implementation_shape.contains("batch"));
        assert_eq!(
            artifact.memory_footprint.residual_window_samples,
            PipelineConfig::default().drift_window + 1
        );
    }
}
