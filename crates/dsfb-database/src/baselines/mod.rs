//! Published-baseline change-point detectors for the paper's §7 bake-off.
//!
//! The bake-off answers a reviewer's mandatory question: *on the same
//! residual streams that dsfb-database consumes, how do standard
//! change-point baselines score against the same ground-truth windows?*
//!
//! We implement three classics from the change-point literature:
//!
//! * [`adwin`] — ADWIN (Bifet & Gavaldà, 2007, *"Learning from
//!   time-changing data with adaptive windowing"*). Online, streaming,
//!   no distributional assumption.
//! * [`bocpd`] — Bayesian Online Change-Point Detection (Adams & MacKay,
//!   2007). Online, Gaussian Normal-Gamma conjugate predictive.
//! * [`pelt`] — Pruned Exact Linear Time (Killick, Fearnhead & Eckley,
//!   2012). Offline, optimal under an L2 cost for changes in mean.
//!
//! All three are faithful reference implementations rather than thin
//! calls to external crates — we want every bake-off number to be
//! reproducible from this file alone, with no hidden dependency drift.
//!
//! Each detector returns a list of *change-point timestamps*. The
//! [`run_detector`] helper wraps those into [`crate::grammar::Episode`]s
//! on the matching motif so that [`crate::metrics::evaluate`] scores the
//! baseline with the exact same TP / FP / FN rules that grade the
//! dsfb-database motif grammar — apples-to-apples.
//!
//! ### Charitable conversion choice
//!
//! Change-point detectors report points; motif episodes are intervals.
//! We convert a change-point at time `t` into an episode `[t, t + dwell]`
//! where `dwell` is the motif's `min_dwell_seconds`. This is deliberately
//! generous to the baselines — any ground-truth window within
//! `min_dwell` of the reported change-point counts as a TP, which
//! maximises the baselines' recall. An adversarial reviewer can
//! re-derive stricter numbers from the emitted CSVs by shrinking the
//! dwell; the raw change-point times are in the CSV alongside the
//! wrapped episodes.

use crate::grammar::{Episode, MotifClass, MotifParams};
use crate::residual::{ResidualClass, ResidualStream};
use std::collections::BTreeMap;

pub mod adwin;
pub mod bocpd;
pub mod pelt;

/// A detector operates on an ordered univariate `(t, value)` series and
/// reports change-point timestamps. Implementations must be deterministic
/// for a given input — the whole bake-off is pinned to byte-identical
/// CSVs, so any randomness kills the replay guarantee.
pub trait ChangePointDetector {
    /// Short kebab-case label embedded in CSV rows.
    fn name(&self) -> &'static str;
    fn detect(&self, series: &[(f64, f64)]) -> Vec<f64>;
}

/// Run a detector against one motif of a residual stream and emit
/// [`Episode`]s that the standard metrics path can score.
///
/// The series is built per-channel: for each distinct channel string
/// under the motif's residual class we invoke `detector.detect`
/// independently, then union the resulting change-points. This mirrors
/// the channel granularity the dsfb motif state machine gets for free
/// — without it the baseline would be unfairly starved of the channel
/// signal that the motif grammar consumes.
pub fn run_detector(
    detector: &dyn ChangePointDetector,
    motif: MotifClass,
    stream: &ResidualStream,
) -> Vec<Episode> {
    let class: ResidualClass = motif.residual_class();
    let mut by_channel: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for s in stream.iter_class(class) {
        let ch = s.channel.clone().unwrap_or_default();
        by_channel.entry(ch).or_default().push((s.t, s.value));
    }
    let dwell = MotifParams::default_for(motif).min_dwell_seconds;
    let mut eps = Vec::new();
    for (channel, series) in by_channel {
        let cps = detector.detect(&series);
        for t in cps {
            eps.push(Episode {
                motif,
                channel: if channel.is_empty() {
                    None
                } else {
                    Some(channel.clone())
                },
                t_start: t,
                t_end: t + dwell,
                peak: 0.0,
                ema_at_boundary: 0.0,
                trust_sum: 1.0,
            });
        }
    }
    eps.sort_by(|a, b| {
        a.t_start
            .partial_cmp(&b.t_start)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    eps
}
