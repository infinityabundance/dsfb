#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlarmSample {
    pub timestamp: f64,
    pub source: String,
    pub active: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AlarmStream {
    samples: Vec<AlarmSample>,
}

impl AlarmStream {
    pub fn new(mut samples: Vec<AlarmSample>) -> Self {
        samples.sort_by(|left, right| {
            left.timestamp
                .total_cmp(&right.timestamp)
                .then_with(|| left.source.cmp(&right.source))
        });
        Self { samples }
    }

    pub fn from_samples(samples: &[AlarmSample]) -> Self {
        Self::new(samples.to_vec())
    }

    pub fn push_clone(&mut self, sample: &AlarmSample) {
        self.samples.push(sample.clone());
        self.samples.sort_by(|left, right| {
            left.timestamp
                .total_cmp(&right.timestamp)
                .then_with(|| left.source.cmp(&right.source))
        });
    }

    pub fn samples(&self) -> &[AlarmSample] {
        &self.samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alarm_stream_is_sorted_deterministically() {
        let stream = AlarmStream::new(vec![
            AlarmSample {
                timestamp: 5.0,
                source: "ewma".into(),
                active: true,
            },
            AlarmSample {
                timestamp: 1.0,
                source: "threshold".into(),
                active: false,
            },
        ]);
        assert_eq!(stream.samples()[0].timestamp, 1.0);
        assert_eq!(stream.samples()[1].timestamp, 5.0);
    }
}
