//! Big-O accounting for each forensic step.
//!
//! References: `DSCD-08` and `DSCD-09` for finite path-length and finite causal
//! propagation bounds, plus `CORE-10` for deterministic compositional scaling.

use serde::Serialize;

/// Per-step complexity log entry.
#[derive(Clone, Debug, Serialize)]
pub struct StepComplexity {
    /// Step index in the trace.
    pub step_index: usize,
    /// Number of measurement channels.
    pub channel_count: usize,
    /// Total graph vertices including the fused root.
    pub vertex_count: usize,
    /// Total directed causal edges.
    pub edge_count: usize,
    /// Primitive operation estimate for the DSFB update.
    pub dsfb_ops: usize,
    /// Primitive operation estimate for graph construction.
    pub graph_ops: usize,
    /// Primitive operation estimate for the optional EKF baseline.
    pub ekf_ops: usize,
    /// Total primitive operation estimate.
    pub total_ops: usize,
    /// Estimated transient memory in scalar words.
    pub memory_words: usize,
    /// Symbolic per-step complexity for the DSFB update.
    pub dsfb_big_o: String,
    /// Symbolic per-step complexity for graph construction.
    pub graph_big_o: String,
    /// Symbolic per-step complexity for the EKF baseline.
    pub ekf_big_o: String,
    /// End-to-end symbolic per-step complexity.
    pub total_big_o: String,
}

/// Classify the audit complexity of one step.
///
/// References: `DSCD-08`, `DSCD-09`, and `CORE-10`.
pub fn classify_step_complexity(
    step_index: usize,
    channel_count: usize,
    edge_count: usize,
    baseline_enabled: bool,
) -> StepComplexity {
    let vertex_count = channel_count + 1;
    let dsfb_ops = 12 * channel_count + 18;
    let graph_ops = channel_count * channel_count + edge_count + 4 * channel_count + 12;
    let ekf_ops = if baseline_enabled {
        26 * channel_count + 48
    } else {
        0
    };
    let total_ops = dsfb_ops + graph_ops + ekf_ops;
    let memory_words = vertex_count * vertex_count + 10 * channel_count + 24;

    StepComplexity {
        step_index,
        channel_count,
        vertex_count,
        edge_count,
        dsfb_ops,
        graph_ops,
        ekf_ops,
        total_ops,
        memory_words,
        dsfb_big_o: "O(c)".to_string(),
        graph_big_o: "O(c^2)".to_string(),
        ekf_big_o: if baseline_enabled {
            "O(c)".to_string()
        } else {
            "O(1)".to_string()
        },
        total_big_o: "O(c^2)".to_string(),
    }
}
