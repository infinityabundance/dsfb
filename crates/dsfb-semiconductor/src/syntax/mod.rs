#[cfg(feature = "std")]
use crate::error::Result;
use crate::sign::FeatureSignPoint;
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use std::collections::BTreeMap;
#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, string::{String, ToString}, vec::Vec};

#[cfg(not(feature = "std"))]
#[inline]
fn maybe_sqrt(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut s = x / 2.0;
    for _ in 0..32 {
        s = (s + x / s) * 0.5;
    }
    s
}

pub const ALLOWED_MOTIFS: [&str; 8] = [
    "slow_drift_precursor",
    "boundary_grazing",
    "transient_excursion",
    "persistent_instability",
    "burst_instability",
    "recovery_pattern",
    "noise_like",
    "null",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Motif {
    pub feature_id: String,
    pub motif_type: String,
    pub start_time: f64,
    pub end_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MotifTimelinePoint {
    pub feature_id: String,
    pub motif_type: String,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyntaxArtifacts {
    pub motifs: Vec<Motif>,
    pub timeline: Vec<MotifTimelinePoint>,
}

pub fn build_motifs(signs: &[FeatureSignPoint]) -> SyntaxArtifacts {
    let mut grouped = BTreeMap::<&str, Vec<&FeatureSignPoint>>::new();
    for sign in signs {
        grouped
            .entry(sign.feature_id.as_str())
            .or_default()
            .push(sign);
    }

    let mut timeline = Vec::new();
    let mut motifs = Vec::new();
    for (feature_id, series) in grouped {
        let envelope = feature_envelope(&series);
        let labels = series
            .iter()
            .enumerate()
            .map(|(index, point)| classify_point(&series, index, point, envelope))
            .collect::<Vec<_>>();

        for (point, label) in series.iter().zip(&labels) {
            timeline.push(MotifTimelinePoint {
                feature_id: feature_id.to_string(),
                motif_type: (*label).to_string(),
                timestamp: point.timestamp,
            });
        }

        let mut start = 0usize;
        while start < series.len() {
            let label = labels[start];
            let mut end = start;
            while end + 1 < series.len() && labels[end + 1] == label {
                end += 1;
            }
            motifs.push(Motif {
                feature_id: feature_id.to_string(),
                motif_type: label.to_string(),
                start_time: series[start].timestamp,
                end_time: series[end].timestamp,
            });
            start = end + 1;
        }
    }

    timeline.sort_by(|left, right| {
        left.timestamp
            .total_cmp(&right.timestamp)
            .then_with(|| left.feature_id.cmp(&right.feature_id))
    });
    motifs.sort_by(|left, right| {
        left.start_time
            .total_cmp(&right.start_time)
            .then_with(|| left.feature_id.cmp(&right.feature_id))
    });

    SyntaxArtifacts { motifs, timeline }
}

#[cfg(feature = "std")]
pub fn write_motifs_csv(path: &std::path::Path, rows: &[Motif]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

#[cfg(feature = "std")]
pub fn write_feature_motif_timeline_csv(
    path: &std::path::Path,
    rows: &[MotifTimelinePoint],
) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

fn feature_envelope(points: &[&FeatureSignPoint]) -> f64 {
    let mean = points.iter().map(|point| point.r.abs()).sum::<f64>() / points.len().max(1) as f64;
    let variance = points
        .iter()
        .map(|point| {
            let centered = point.r.abs() - mean;
            centered * centered
        })
        .sum::<f64>()
        / points.len().max(1) as f64;
    #[cfg(feature = "std")]
    return (mean + variance.sqrt()).max(1.0);
    #[cfg(not(feature = "std"))]
    return (mean + maybe_sqrt(variance)).max(1.0);
}

fn classify_point(
    series: &[&FeatureSignPoint],
    index: usize,
    point: &FeatureSignPoint,
    envelope: f64,
) -> &'static str {
    let abs_r = point.r.abs();
    let abs_s = point.s.abs();
    let prev = index
        .checked_sub(1)
        .and_then(|idx| series.get(idx))
        .copied();
    let prev_prev = index
        .checked_sub(2)
        .and_then(|idx| series.get(idx))
        .copied();
    let next = series.get(index + 1).copied();
    let prev_abs = prev.map(|row| row.r.abs()).unwrap_or(abs_r);
    let next_abs = next.map(|row| row.r.abs()).unwrap_or(abs_r);
    let drift_stable = prev
        .zip(prev_prev)
        .map(|(left, right)| {
            point.d.signum() != 0.0
                && point.d.signum() == left.d.signum()
                && left.d.signum() == right.d.signum()
                && point.d.abs() >= 0.05 * envelope
        })
        .unwrap_or(false);
    let oscillatory = prev
        .map(|row| {
            point.d.signum() != 0.0
                && row.d.signum() != 0.0
                && point.d.signum() != row.d.signum()
                && point.d.abs() >= 0.05 * envelope
        })
        .unwrap_or(false);
    let burst_cluster = index >= 2
        && series[index - 2..=index]
            .iter()
            .filter(|row| row.s.abs() >= 0.20 * envelope)
            .count()
            >= 2;
    let recovering = prev
        .map(|row| row.r.abs() >= 0.60 * envelope && row.r.abs() > abs_r && point.d < 0.0)
        .unwrap_or(false);

    if recovering {
        "recovery_pattern"
    } else if burst_cluster && abs_r >= 0.70 * envelope {
        "burst_instability"
    } else if abs_r >= envelope && oscillatory {
        "persistent_instability"
    } else if abs_s >= 0.25 * envelope && abs_r >= 0.50 * envelope {
        "transient_excursion"
    } else if drift_stable && abs_r >= 0.55 * envelope && next_abs >= abs_r {
        "slow_drift_precursor"
    } else if abs_r >= 0.55 * envelope
        && (oscillatory || (prev_abs >= 0.55 * envelope && next_abs >= 0.55 * envelope))
    {
        "boundary_grazing"
    } else if abs_s >= 0.20 * envelope && abs_r < 0.40 * envelope {
        "noise_like"
    } else {
        "null"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(timestamp: f64, r: f64, d: f64, s: f64) -> FeatureSignPoint {
        FeatureSignPoint {
            timestamp,
            feature_id: "S001".into(),
            r,
            d,
            s,
        }
    }

    #[test]
    fn motif_logic_is_not_just_threshold_labeling() {
        let flat_high = vec![
            point(0.0, 4.0, 0.0, 0.0),
            point(1.0, 4.0, 0.0, 0.0),
            point(2.0, 4.0, 0.0, 0.0),
        ];
        let drifting = vec![
            point(0.0, 1.0, 0.0, 0.0),
            point(1.0, 2.0, 1.0, 1.0),
            point(2.0, 3.0, 1.0, 0.0),
            point(3.0, 4.0, 1.0, 0.0),
        ];
        let flat = build_motifs(&flat_high);
        let drift = build_motifs(&drifting);

        assert!(flat
            .timeline
            .iter()
            .all(|row| row.motif_type != "slow_drift_precursor"));
        assert!(drift
            .timeline
            .iter()
            .any(|row| row.motif_type == "slow_drift_precursor"));
    }
}
