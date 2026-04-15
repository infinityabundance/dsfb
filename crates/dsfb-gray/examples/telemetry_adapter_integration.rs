use dsfb_gray::{DsfbObserver, ObserverConfig, ResidualSample, ResidualSource, TelemetryAdapter};

#[derive(Debug, Clone, Copy)]
struct QueueSnapshot {
    depth: u64,
    baseline_depth: u64,
    timestamp_ns: u64,
}

struct QueueDepthAdapter;

impl TelemetryAdapter<QueueSnapshot> for QueueDepthAdapter {
    fn adapt(&self, input: &QueueSnapshot) -> ResidualSample {
        ResidualSample {
            value: input.depth as f64,
            baseline: input.baseline_depth as f64,
            timestamp_ns: input.timestamp_ns,
            source: ResidualSource::QueueDepth,
        }
    }
}

fn main() {
    let adapter = QueueDepthAdapter;
    let mut observer =
        DsfbObserver::new(ResidualSource::QueueDepth, &ObserverConfig::fast_response());

    for (idx, depth) in [8_u64, 9, 10, 12, 15].into_iter().enumerate() {
        let snapshot = QueueSnapshot {
            depth,
            baseline_depth: 8,
            timestamp_ns: (idx as u64) * 1_000_000_000,
        };
        let result = observer.observe_adapted(&adapter, &snapshot);
        println!(
            "step={idx} state={:?} reason={:?} confidence={:.2}",
            result.grammar_state,
            result.reason_evidence.reason_code,
            result.reason_evidence.confidence
        );
    }
}
