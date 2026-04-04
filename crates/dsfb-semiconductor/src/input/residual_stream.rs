#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResidualSample {
    pub timestamp: f64,
    pub feature_id: String,
    pub value: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ResidualStream {
    samples: Vec<ResidualSample>,
}

impl ResidualStream {
    pub fn new(mut samples: Vec<ResidualSample>) -> Self {
        samples.sort_by(|left, right| {
            left.timestamp
                .total_cmp(&right.timestamp)
                .then_with(|| left.feature_id.cmp(&right.feature_id))
        });
        Self { samples }
    }

    pub fn from_samples(samples: &[ResidualSample]) -> Self {
        Self::new(samples.to_vec())
    }

    pub fn push_clone(&mut self, sample: &ResidualSample) {
        self.samples.push(sample.clone());
        self.samples.sort_by(|left, right| {
            left.timestamp
                .total_cmp(&right.timestamp)
                .then_with(|| left.feature_id.cmp(&right.feature_id))
        });
    }

    pub fn samples(&self) -> &[ResidualSample] {
        &self.samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn residual_stream_is_sorted_deterministically() {
        let stream = ResidualStream::new(vec![
            ResidualSample {
                timestamp: 2.0,
                feature_id: "S002".into(),
                value: 1.0,
            },
            ResidualSample {
                timestamp: 1.0,
                feature_id: "S003".into(),
                value: 1.0,
            },
            ResidualSample {
                timestamp: 1.0,
                feature_id: "S001".into(),
                value: 1.0,
            },
        ]);
        assert_eq!(stream.samples()[0].feature_id, "S001");
        assert_eq!(stream.samples()[1].feature_id, "S003");
        assert_eq!(stream.samples()[2].feature_id, "S002");
    }
}
