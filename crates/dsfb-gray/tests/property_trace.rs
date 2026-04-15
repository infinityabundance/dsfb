use dsfb_gray::{AuditEvent, AuditTrace};
use proptest::prelude::*;

proptest! {
    #[test]
    fn audit_trace_reports_the_number_of_recorded_events(event_count in 0usize..128) {
        let mut trace = AuditTrace::new();

        for idx in 0..event_count {
            trace.record(AuditEvent {
                timestamp_ns: idx as u64,
                residual: idx as f64,
                drift: 0.0,
                slew: 0.0,
                envelope_position: 0,
                grammar_state: 0,
                transition_occurred: false,
            });
        }

        prop_assert_eq!(trace.total_count(), event_count as u64);
        prop_assert_eq!(trace.buffered_count(), event_count.min(256));
    }
}
