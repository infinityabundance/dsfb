//! Real-time contract helpers for the bounded live path.
//!
//! These helpers do not claim certified WCET or whole-crate zero-allocation behavior. They
//! expose machine-checkable summaries of the documented bounded live-path assumptions and the
//! current allocation audit.

use std::mem::size_of;

use serde::{Deserialize, Serialize};

use crate::engine::settings::EngineSettings;
use crate::engine::types::ResidualSample;

use super::OnlineStructuralEngine;

/// Stable schema identifier for machine-readable real-time contract summaries.
pub const REAL_TIME_CONTRACT_SUMMARY_SCHEMA_VERSION: &str = "dsfb-semiotics-real-time-contract/v1";

/// Machine-readable memory-budget estimate for one bounded live-path configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnlineMemoryBudgetEstimate {
    pub numeric_mode: String,
    pub channel_count: usize,
    pub history_buffer_capacity: usize,
    pub engine_handle_stack_bytes: usize,
    pub ring_slot_bytes: usize,
    pub retained_value_heap_bytes: usize,
    pub channel_name_storage_bytes: usize,
    pub estimated_total_bounded_bytes: usize,
    pub note: String,
}

/// Allocation finding for the bounded live path.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnlineAllocationAuditFinding {
    pub phase: String,
    pub symbol: String,
    pub allocation_behavior: String,
    pub note: String,
}

/// Machine-readable summary used by the real-time contract documentation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RealTimeContractSummary {
    pub schema_version: String,
    pub numeric_mode: String,
    pub default_history_buffer_capacity: usize,
    pub covered_symbols: Vec<String>,
    pub documented_memory_profiles: Vec<OnlineMemoryBudgetEstimate>,
    pub no_heap_alloc_after_init_verified: bool,
    pub allocation_audit_findings: Vec<OnlineAllocationAuditFinding>,
    pub no_panic_policy: String,
    pub no_nan_policy: String,
    pub timing_report_json_path: String,
    pub note: String,
}

/// Returns an estimated bounded-memory budget for the live residual window.
///
/// The estimate covers the online engine handle, ring slots, retained residual-value storage, and
/// channel-name storage. It intentionally excludes the heuristic bank and retrieval index because
/// those are initialization-time assets whose size is independent of per-sample growth.
#[must_use]
pub fn estimate_online_memory_budget(
    channel_count: usize,
    history_buffer_capacity: usize,
) -> OnlineMemoryBudgetEstimate {
    let engine_handle_stack_bytes = size_of::<OnlineStructuralEngine>();
    let ring_slot_bytes = size_of::<Option<ResidualSample>>() * history_buffer_capacity;
    let retained_value_heap_bytes = size_of::<f64>() * channel_count * history_buffer_capacity;
    let channel_name_storage_bytes = size_of::<String>() * channel_count;
    let estimated_total_bounded_bytes = engine_handle_stack_bytes
        + ring_slot_bytes
        + retained_value_heap_bytes
        + channel_name_storage_bytes;

    OnlineMemoryBudgetEstimate {
        numeric_mode: super::numeric_mode_label().to_string(),
        channel_count,
        history_buffer_capacity,
        engine_handle_stack_bytes,
        ring_slot_bytes,
        retained_value_heap_bytes,
        channel_name_storage_bytes,
        estimated_total_bounded_bytes,
        note: "Estimate covers the bounded online residual window and handle-local storage only. Bank-registry and retrieval-index heaps are initialization-time assets and are excluded from this growth budget.".to_string(),
    }
}

/// Returns the current documented allocation audit for the bounded live path.
#[must_use]
pub fn online_path_allocation_audit() -> Vec<OnlineAllocationAuditFinding> {
    vec![
        OnlineAllocationAuditFinding {
            phase: "initialization".to_string(),
            symbol: "OnlineStructuralEngine::new".to_string(),
            allocation_behavior: "allowed".to_string(),
            note: "Ring-buffer slots, channel names, builtin/external bank registry, and retrieval index are allocated at initialization time.".to_string(),
        },
        OnlineAllocationAuditFinding {
            phase: "per-sample".to_string(),
            symbol: "OnlineStructuralEngine::push_residual_sample".to_string(),
            allocation_behavior: "present".to_string(),
            note: "The hot path still materializes bounded Vec-backed residual, drift, slew, sign, and status structures per step. These allocations are bounded by channel_count and history_buffer_capacity, but they are not yet eliminated.".to_string(),
        },
        OnlineAllocationAuditFinding {
            phase: "optional offline accumulation".to_string(),
            symbol: "OnlineStructuralEngine::offline_history".to_string(),
            allocation_behavior: "opt-in".to_string(),
            note: "Offline accumulation is disabled by default. When enabled, it grows for export and forensics and is explicitly outside the bounded live-state contract.".to_string(),
        },
        OnlineAllocationAuditFinding {
            phase: "status/query wrappers".to_string(),
            symbol: "LiveEngineStatus / FFI copy helpers".to_string(),
            allocation_behavior: "present".to_string(),
            note: "Owned status strings and selected heuristic ID vectors remain allocation-bearing at the interface boundary.".to_string(),
        },
    ]
}

/// Builds the machine-readable real-time contract summary for the default documented profiles.
#[must_use]
pub fn build_real_time_contract_summary() -> RealTimeContractSummary {
    let settings = EngineSettings::default();
    RealTimeContractSummary {
        schema_version: REAL_TIME_CONTRACT_SUMMARY_SCHEMA_VERSION.to_string(),
        numeric_mode: super::numeric_mode_label().to_string(),
        default_history_buffer_capacity: settings.online.history_buffer_capacity,
        covered_symbols: vec![
            "OnlineStructuralEngine::new".to_string(),
            "OnlineStructuralEngine::push_residual_sample".to_string(),
            "OnlineStructuralEngine::push_residual_sample_batch".to_string(),
            "ffi::dsfb_semiotics_engine_push_sample".to_string(),
            "ffi::dsfb_semiotics_engine_push_sample_batch".to_string(),
        ],
        documented_memory_profiles: vec![
            estimate_online_memory_budget(1, 32),
            estimate_online_memory_budget(1, settings.online.history_buffer_capacity),
            estimate_online_memory_budget(3, settings.online.history_buffer_capacity),
            estimate_online_memory_budget(3, 128),
        ],
        no_heap_alloc_after_init_verified: false,
        allocation_audit_findings: online_path_allocation_audit(),
        no_panic_policy: "The bounded hot path is audited to avoid explicit unwrap/expect/panic! in non-test online-step code. Invalid inputs return Result errors instead of panicking. This is an engineering policy and source audit, not a formal proof.".to_string(),
        no_nan_policy: "Non-finite time or residual inputs are rejected at the live ingress boundary. Externally visible live status values are checked for finiteness before they are returned; non-finite internal values are converted into structured errors rather than emitted.".to_string(),
        timing_report_json_path: "docs/timing_determinism_report.json".to_string(),
        note: "This summary describes the current bounded live-path contract and explicit gaps. It does not claim certified WCET, whole-crate zero-allocation runtime, or platform certification.".to_string(),
    }
}
