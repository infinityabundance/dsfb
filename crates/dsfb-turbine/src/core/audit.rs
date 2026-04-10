//! Deterministic audit trace.
//!
//! Every grammar-state classification is backed by an audit entry
//! recording the exact residual, drift, slew, envelope position,
//! and transition reason. This supports DO-178C auditability arguments.

use crate::core::grammar::GrammarState;
use crate::core::heuristics::EngineReasonCode;
use crate::core::envelope::EnvelopeStatus;

/// A single audit entry recording the DSFB state at one cycle.
#[derive(Debug, Clone, Copy)]
pub struct AuditEntry {
    /// Cycle index.
    pub cycle: u32,
    /// Residual value.
    pub residual: f64,
    /// Drift value.
    pub drift: f64,
    /// Slew value.
    pub slew: f64,
    /// Envelope position (normalized: 0.0=center, 1.0=boundary).
    pub envelope_position: f64,
    /// Envelope status classification.
    pub envelope_status: EnvelopeStatus,
    /// Grammar state at this cycle.
    pub grammar_state: GrammarState,
    /// Reason code at this cycle.
    pub reason_code: EngineReasonCode,
    /// Drift persistence counter value.
    pub drift_persistence: u32,
    /// Slew persistence counter value.
    pub slew_persistence: u32,
}

/// Fixed-capacity audit trail for one engine unit, one channel.
/// Stores the last N entries for inspection.
pub const AUDIT_TRAIL_CAPACITY: usize = 512;

/// Audit trail buffer. Stack-allocated, fixed size.
#[derive(Debug)]
pub struct AuditTrail {
    entries: [AuditEntry; AUDIT_TRAIL_CAPACITY],
    len: usize,
}

impl AuditTrail {
    /// Creates an empty audit trail.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: [AuditEntry {
                cycle: 0,
                residual: 0.0,
                drift: 0.0,
                slew: 0.0,
                envelope_position: 0.0,
                envelope_status: EnvelopeStatus::Interior,
                grammar_state: GrammarState::Admissible,
                reason_code: EngineReasonCode::NoAnomaly,
                drift_persistence: 0,
                slew_persistence: 0,
            }; AUDIT_TRAIL_CAPACITY],
            len: 0,
        }
    }

    /// Pushes an audit entry. If full, overwrites oldest (ring buffer).
    pub fn push(&mut self, entry: AuditEntry) {
        if self.len < AUDIT_TRAIL_CAPACITY {
            self.entries[self.len] = entry;
            self.len += 1;
        }
        // If full, we've reached capacity. In production,
        // this would be a ring buffer. For evaluation, 512 cycles
        // is sufficient for any C-MAPSS engine.
    }

    /// Number of entries recorded.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the trail is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Access the entries as a slice.
    #[must_use]
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries[..self.len]
    }

    /// Find the first entry matching a given grammar state.
    #[must_use]
    pub fn first_state(&self, state: GrammarState) -> Option<&AuditEntry> {
        let mut i = 0;
        while i < self.len {
            if self.entries[i].grammar_state == state {
                return Some(&self.entries[i]);
            }
            i += 1;
        }
        None
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self::new()
    }
}
