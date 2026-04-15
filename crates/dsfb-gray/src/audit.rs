//! Audit trace: deterministic, reproducible record of every classification decision.
//!
//! Every grammar transition, heuristic match, and envelope classification is
//! recorded in an [`AuditTrace`] that can be replayed offline for verification.
//! This is the DSFB equivalent of a flight data recorder — if the system
//! says "Boundary due to ConsensusHeartbeatDegradation," the audit trace
//! shows exactly which residual signs, at which timestamps, with which
//! drift and slew values, produced that classification.

/// A single audit event recording one observation-classification cycle.
#[derive(Debug, Clone, Copy)]
pub struct AuditEvent {
    /// Monotonic timestamp (nanoseconds).
    pub timestamp_ns: u64,
    /// Raw residual value at this observation.
    pub residual: f64,
    /// Estimated drift at this observation.
    pub drift: f64,
    /// Estimated slew at this observation.
    pub slew: f64,
    /// Envelope position classification.
    pub envelope_position: u8, // 0=Interior, 1=Boundary, 2=Exterior
    /// Grammar state after this observation.
    pub grammar_state: u8, // 0=Admissible, 1=Boundary, 2=Violation
    /// Whether a grammar transition occurred at this step.
    pub transition_occurred: bool,
}

/// Fixed-capacity audit trace buffer (stack-allocated, no_alloc compatible).
///
/// Stores the most recent `N` audit events in a ring buffer. When the
/// buffer is full, the oldest events are overwritten. For full-history
/// audit, use the `std` feature to write events to a file or stream.
pub struct AuditTrace {
    /// Ring buffer of audit events.
    events: [AuditEvent; 256],
    /// Write head position.
    head: usize,
    /// Total events recorded (may exceed buffer capacity).
    total_count: u64,
}

impl AuditTrace {
    /// Create a new empty audit trace.
    pub fn new() -> Self {
        Self {
            events: [AuditEvent {
                timestamp_ns: 0,
                residual: 0.0,
                drift: 0.0,
                slew: 0.0,
                envelope_position: 0,
                grammar_state: 0,
                transition_occurred: false,
            }; 256],
            head: 0,
            total_count: 0,
        }
    }

    /// Record an audit event.
    pub fn record(&mut self, event: AuditEvent) {
        self.events[self.head] = event;
        self.head = (self.head + 1) % 256;
        self.total_count += 1;
    }

    /// Total number of events recorded (may exceed buffer capacity).
    pub fn total_count(&self) -> u64 {
        self.total_count
    }

    /// Number of events currently in the buffer (max 256).
    pub fn buffered_count(&self) -> usize {
        if self.total_count < 256 {
            self.total_count as usize
        } else {
            256
        }
    }

    /// Iterate over buffered events in chronological order.
    pub fn iter(&self) -> AuditTraceIter<'_> {
        let count = self.buffered_count();
        let start = if self.total_count < 256 { 0 } else { self.head };
        AuditTraceIter {
            trace: self,
            pos: start,
            remaining: count,
        }
    }

    /// Get the most recent audit event, if any.
    pub fn last(&self) -> Option<&AuditEvent> {
        if self.total_count == 0 {
            None
        } else {
            let idx = if self.head == 0 { 255 } else { self.head - 1 };
            Some(&self.events[idx])
        }
    }

    /// Reset the audit trace.
    pub fn reset(&mut self) {
        self.head = 0;
        self.total_count = 0;
    }
}

impl Default for AuditTrace {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over audit trace events in chronological order.
pub struct AuditTraceIter<'a> {
    trace: &'a AuditTrace,
    pos: usize,
    remaining: usize,
}

impl<'a> Iterator for AuditTraceIter<'a> {
    type Item = &'a AuditEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let event = &self.trace.events[self.pos];
        self.pos = (self.pos + 1) % 256;
        self.remaining -= 1;
        Some(event)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a> ExactSizeIterator for AuditTraceIter<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_trace() {
        let trace = AuditTrace::new();
        assert_eq!(trace.total_count(), 0);
        assert_eq!(trace.buffered_count(), 0);
        assert!(trace.last().is_none());
    }

    #[test]
    fn test_record_and_retrieve() {
        let mut trace = AuditTrace::new();
        trace.record(AuditEvent {
            timestamp_ns: 1000,
            residual: 0.5,
            drift: 0.01,
            slew: 0.001,
            envelope_position: 0,
            grammar_state: 0,
            transition_occurred: false,
        });
        assert_eq!(trace.total_count(), 1);
        assert_eq!(trace.buffered_count(), 1);
        let last = trace.last().unwrap();
        assert_eq!(last.timestamp_ns, 1000);
    }

    #[test]
    fn test_ring_buffer_wraps() {
        let mut trace = AuditTrace::new();
        for i in 0..300u64 {
            trace.record(AuditEvent {
                timestamp_ns: i,
                residual: i as f64,
                drift: 0.0,
                slew: 0.0,
                envelope_position: 0,
                grammar_state: 0,
                transition_occurred: false,
            });
        }
        assert_eq!(trace.total_count(), 300);
        assert_eq!(trace.buffered_count(), 256);
        // Most recent event should be 299
        assert_eq!(trace.last().unwrap().timestamp_ns, 299);
    }

    #[test]
    fn test_iter_chronological() {
        let mut trace = AuditTrace::new();
        for i in 0..10u64 {
            trace.record(AuditEvent {
                timestamp_ns: i * 100,
                residual: 0.0,
                drift: 0.0,
                slew: 0.0,
                envelope_position: 0,
                grammar_state: 0,
                transition_occurred: false,
            });
        }
        let timestamps: Vec<u64> = trace.iter().map(|e| e.timestamp_ns).collect();
        assert_eq!(
            timestamps,
            vec![0, 100, 200, 300, 400, 500, 600, 700, 800, 900]
        );
    }
}
