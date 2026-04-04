#[cfg(feature = "std")]
use crate::error::Result;
use crate::sign::FeatureSignPoint;
use crate::syntax::MotifTimelinePoint;
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

pub const ALLOWED_GRAMMAR_STATES: [&str; 6] = [
    "Admissible",
    "BoundaryGrazing",
    "SustainedDrift",
    "TransientViolation",
    "PersistentViolation",
    "Recovery",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GrammarState {
    pub feature_id: String,
    pub state: String,
    pub timestamp: f64,
}

pub fn build_grammar_states(
    signs: &[FeatureSignPoint],
    motifs: &[MotifTimelinePoint],
) -> Vec<GrammarState> {
    let mut motif_map = BTreeMap::<(&str, u64), &str>::new();
    for motif in motifs {
        motif_map.insert(
            (motif.feature_id.as_str(), motif.timestamp.to_bits()),
            motif.motif_type.as_str(),
        );
    }

    let mut grouped = BTreeMap::<&str, Vec<&FeatureSignPoint>>::new();
    for sign in signs {
        grouped
            .entry(sign.feature_id.as_str())
            .or_default()
            .push(sign);
    }

    let mut states = Vec::new();
    for (feature_id, series) in grouped {
        let envelope = feature_envelope(&series);
        let mut violation_streak = 0usize;
        let mut drift_streak = 0usize;
        let mut previous_non_admissible = false;

        for point in series {
            let motif = motif_map
                .get(&(feature_id, point.timestamp.to_bits()))
                .copied()
                .unwrap_or("null");
            let abs_r = point.r.abs();
            let state = if abs_r >= envelope {
                violation_streak += 1;
                drift_streak = 0;
                previous_non_admissible = true;
                if violation_streak >= 2
                    || motif == "persistent_instability"
                    || motif == "burst_instability"
                {
                    "PersistentViolation"
                } else {
                    "TransientViolation"
                }
            } else if motif == "slow_drift_precursor" && abs_r >= 0.60 * envelope && point.d > 0.0 {
                violation_streak = 0;
                drift_streak += 1;
                previous_non_admissible = true;
                if drift_streak >= 3 {
                    "SustainedDrift"
                } else {
                    "BoundaryGrazing"
                }
            } else if motif == "boundary_grazing" && abs_r >= 0.50 * envelope {
                violation_streak = 0;
                drift_streak = 0;
                previous_non_admissible = true;
                "BoundaryGrazing"
            } else if previous_non_admissible && motif == "recovery_pattern" {
                violation_streak = 0;
                drift_streak = 0;
                previous_non_admissible = false;
                "Recovery"
            } else {
                violation_streak = 0;
                drift_streak = 0;
                previous_non_admissible = false;
                "Admissible"
            };

            states.push(GrammarState {
                feature_id: feature_id.to_string(),
                state: state.to_string(),
                timestamp: point.timestamp,
            });
        }
    }

    states.sort_by(|left, right| {
        left.timestamp
            .total_cmp(&right.timestamp)
            .then_with(|| left.feature_id.cmp(&right.feature_id))
    });
    states
}

#[cfg(feature = "std")]
pub fn write_grammar_states_csv(path: &std::path::Path, rows: &[GrammarState]) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sign::FeatureSignPoint;
    use crate::syntax::MotifTimelinePoint;

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
    fn grammar_depends_on_persistence_and_trajectory() {
        let signs = vec![
            point(0.0, 1.0, 0.0, 0.0),
            point(1.0, 2.0, 1.0, 1.0),
            point(2.0, 3.0, 1.0, 0.0),
            point(3.0, 3.1, 0.1, -0.9),
            point(4.0, 3.2, 0.1, 0.0),
        ];
        let motifs = vec![
            MotifTimelinePoint {
                feature_id: "S001".into(),
                motif_type: "null".into(),
                timestamp: 0.0,
            },
            MotifTimelinePoint {
                feature_id: "S001".into(),
                motif_type: "slow_drift_precursor".into(),
                timestamp: 1.0,
            },
            MotifTimelinePoint {
                feature_id: "S001".into(),
                motif_type: "slow_drift_precursor".into(),
                timestamp: 2.0,
            },
            MotifTimelinePoint {
                feature_id: "S001".into(),
                motif_type: "slow_drift_precursor".into(),
                timestamp: 3.0,
            },
            MotifTimelinePoint {
                feature_id: "S001".into(),
                motif_type: "slow_drift_precursor".into(),
                timestamp: 4.0,
            },
        ];
        let grammar = build_grammar_states(&signs, &motifs);
        assert!(grammar.iter().any(|row| row.state == "SustainedDrift"));
    }

    #[test]
    fn recovery_requires_prior_non_admissible_state() {
        let signs = vec![
            point(0.0, 5.0, 0.0, 0.0),
            point(1.0, 6.0, 1.0, 1.0),
            point(2.0, 3.0, -3.0, -4.0),
        ];
        let motifs = vec![
            MotifTimelinePoint {
                feature_id: "S001".into(),
                motif_type: "persistent_instability".into(),
                timestamp: 0.0,
            },
            MotifTimelinePoint {
                feature_id: "S001".into(),
                motif_type: "burst_instability".into(),
                timestamp: 1.0,
            },
            MotifTimelinePoint {
                feature_id: "S001".into(),
                motif_type: "recovery_pattern".into(),
                timestamp: 2.0,
            },
        ];
        let grammar = build_grammar_states(&signs, &motifs);
        assert_eq!(grammar.last().unwrap().state, "Recovery");
    }
}
