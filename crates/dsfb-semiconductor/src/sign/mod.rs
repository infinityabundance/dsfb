use crate::error::Result;
use crate::input::residual_stream::ResidualStream;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Sign {
    pub r: f64,
    pub d: f64,
    pub s: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeatureSignPoint {
    pub timestamp: f64,
    pub feature_id: String,
    pub r: f64,
    pub d: f64,
    pub s: f64,
}

impl FeatureSignPoint {
    pub fn sign(&self) -> Sign {
        Sign {
            r: self.r,
            d: self.d,
            s: self.s,
        }
    }
}

pub fn build_feature_signs(stream: &ResidualStream) -> Vec<FeatureSignPoint> {
    let mut grouped = BTreeMap::<&str, Vec<_>>::new();
    for sample in stream.samples() {
        grouped
            .entry(sample.feature_id.as_str())
            .or_default()
            .push(sample);
    }

    let mut rows = Vec::new();
    for (feature_id, samples) in grouped {
        let mut previous_r = None;
        let mut previous_d = 0.0;
        for sample in samples {
            let r = sample.value;
            let d = previous_r.map(|value| r - value).unwrap_or(0.0);
            let s = if previous_r.is_some() {
                d - previous_d
            } else {
                0.0
            };
            rows.push(FeatureSignPoint {
                timestamp: sample.timestamp,
                feature_id: feature_id.to_string(),
                r,
                d,
                s,
            });
            previous_r = Some(r);
            previous_d = d;
        }
    }

    rows.sort_by(|left, right| {
        left.timestamp
            .total_cmp(&right.timestamp)
            .then_with(|| left.feature_id.cmp(&right.feature_id))
    });
    rows
}

pub fn write_feature_signs_csv(path: &Path, rows: &[FeatureSignPoint]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::residual_stream::{ResidualSample, ResidualStream};

    #[test]
    fn sign_construction_uses_first_and_second_difference() {
        let stream = ResidualStream::new(vec![
            ResidualSample {
                timestamp: 0.0,
                feature_id: "S001".into(),
                value: 1.0,
            },
            ResidualSample {
                timestamp: 1.0,
                feature_id: "S001".into(),
                value: 2.5,
            },
            ResidualSample {
                timestamp: 2.0,
                feature_id: "S001".into(),
                value: 4.0,
            },
        ]);
        let rows = build_feature_signs(&stream);
        assert_eq!(
            rows[0].sign(),
            Sign {
                r: 1.0,
                d: 0.0,
                s: 0.0
            }
        );
        assert_eq!(
            rows[1].sign(),
            Sign {
                r: 2.5,
                d: 1.5,
                s: 1.5
            }
        );
        assert_eq!(
            rows[2].sign(),
            Sign {
                r: 4.0,
                d: 1.5,
                s: 0.0
            }
        );
    }
}
