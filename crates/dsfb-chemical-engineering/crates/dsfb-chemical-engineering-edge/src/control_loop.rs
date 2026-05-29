//! Control-loop context (Wave-4 historian layer): `SetpointResidualSeparationV1`, `ControllerModeGuardV1`,
//! and `ControlLoopInteractionMapV1` — the PV/MV/SP awareness that removes a major source of false alarms.
//!
//! A statistical monitor that watches a *controlled* variable raw will flag every setpoint change and every
//! auto↔manual handover as an "anomaly". These three objects give DSFB the controller context to tell those
//! apart from genuine process deviations:
//!
//!   * [`SetpointResidualSeparationV1`] — separates a PV's motion into **setpoint-driven** (the SP moved) vs a
//!     genuine **tracking error** (SP steady but PV off target). The tracking error `PV − SP`, not the raw PV
//!     deviation, is the process-relevant residual; a clean setpoint change is *not* an anomaly.
//!   * [`ControllerModeGuardV1`] — flags episodes that overlap a **controller-mode transition** (auto / manual
//!     / cascade), so a handover transient is read as mode-conditioned context, not a pure process fault.
//!   * [`ControlLoopInteractionMapV1`] — a candidate **loop-interaction map** from the correlation of
//!     controller-effort (MV) signals: loops whose manipulated variables move together may be fighting or
//!     coupled.
//!
//! Bounded (non-claims): the setpoint separation attributes PV motion under documented thresholds, not via a
//! controller model; a mode-transition flag is *context, not exoneration* (the episode may still be real); a
//! controller-effort correlation is a *candidate* interaction indicator, never proven causal coupling.
//! Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// Pearson correlation of two equal-length series over their finite, paired samples. Returns 0 when either
/// side is constant or fewer than two paired points exist (no spurious correlation from a flat signal).
pub fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let pairs: Vec<(f64, f64)> = a
        .iter()
        .zip(b)
        .filter(|(x, y)| x.is_finite() && y.is_finite())
        .map(|(&x, &y)| (x, y))
        .collect();
    let n = pairs.len();
    if n < 2 {
        return 0.0;
    }
    let nf = n as f64;
    let (mx, my) = pairs
        .iter()
        .fold((0.0, 0.0), |(sx, sy), &(x, y)| (sx + x, sy + y));
    let (mx, my) = (mx / nf, my / nf);
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (x, y) in pairs {
        let (dx, dy) = (x - mx, y - my);
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx <= 0.0 || syy <= 0.0 {
        return 0.0;
    }
    sxy / (sxx.sqrt() * syy.sqrt())
}

// ── SetpointResidualSeparationV1 ────────────────────────────────────────────────────────────────────

/// A hash-sealed setpoint/process separation (schema v1) for one controlled variable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetpointResidualSeparationV1 {
    pub pv_name: String,
    pub sp_name: String,
    pub n_samples: usize,
    /// Step `|ΔSP|` above which a sample is judged setpoint-driven.
    pub sp_change_threshold: f64,
    /// `|PV − SP|` above which a *steady-SP* sample is judged a genuine tracking error.
    pub tracking_tol: f64,
    /// Samples whose PV moved because the setpoint moved (a clean change a naive monitor would false-alarm).
    pub n_setpoint_driven: usize,
    /// Samples with steady SP but PV off target beyond `tracking_tol` (the process-relevant deviations).
    pub n_tracking_error: usize,
    pub n_on_target: usize,
    /// Largest `|PV − SP|` among steady-SP samples (the worst genuine tracking deviation).
    pub peak_tracking_error: f64,
    /// SHA-256 of the tracking-error stream `PV − SP`.
    pub tracking_error_hash: String,
    pub witness_hash: String,
}

impl SetpointResidualSeparationV1 {
    fn hash_te(te: &[f64]) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"tracking_error_stream_v1");
        for &v in te {
            h.f64q("te", v);
        }
        h.finalize_hex()
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"setpoint_residual_separation_v1");
        h.field("pv_name", self.pv_name.as_bytes());
        h.field("sp_name", self.sp_name.as_bytes());
        h.u64("n_samples", self.n_samples as u64);
        h.f64q("sp_change_threshold", self.sp_change_threshold);
        h.f64q("tracking_tol", self.tracking_tol);
        h.u64("n_setpoint_driven", self.n_setpoint_driven as u64);
        h.u64("n_tracking_error", self.n_tracking_error as u64);
        h.u64("n_on_target", self.n_on_target as u64);
        h.f64q("peak_tracking_error", self.peak_tracking_error);
        h.field("tracking_error_hash", self.tracking_error_hash.as_bytes());
        h.finalize_hex()
    }

    /// Separate a controlled variable's motion into setpoint-driven vs tracking-error vs on-target, given the
    /// PV and SP series and the two documented thresholds.
    pub fn build(
        pv_name: impl Into<String>,
        sp_name: impl Into<String>,
        pv: &[f64],
        sp: &[f64],
        sp_change_threshold: f64,
        tracking_tol: f64,
    ) -> Self {
        let n = pv.len().min(sp.len());
        let te: Vec<f64> = (0..n).map(|k| pv[k] - sp[k]).collect();
        let (mut n_setpoint_driven, mut n_tracking_error, mut n_on_target) =
            (0usize, 0usize, 0usize);
        let mut peak_tracking_error = 0.0f64;
        for k in 0..n {
            let sp_move = if k == 0 {
                0.0
            } else {
                (sp[k] - sp[k - 1]).abs()
            };
            let te_abs = te[k].abs();
            if sp_move > sp_change_threshold {
                n_setpoint_driven += 1;
            } else {
                // Steady setpoint: the tracking error is the process-relevant residual.
                peak_tracking_error = peak_tracking_error.max(te_abs);
                if te_abs > tracking_tol {
                    n_tracking_error += 1;
                } else {
                    n_on_target += 1;
                }
            }
        }
        let mut w = SetpointResidualSeparationV1 {
            pv_name: pv_name.into(),
            sp_name: sp_name.into(),
            n_samples: n,
            sp_change_threshold,
            tracking_tol,
            n_setpoint_driven,
            n_tracking_error,
            n_on_target,
            peak_tracking_error,
            tracking_error_hash: Self::hash_te(&te),
            witness_hash: String::new(),
        };
        w.witness_hash = w.seal();
        w
    }

    /// PV excursions explained by setpoint moves — the false alarms a setpoint-blind monitor would raise.
    pub fn false_alarms_avoided(&self) -> usize {
        self.n_setpoint_driven
    }

    pub fn verify(&self, pv: &[f64], sp: &[f64]) -> bool {
        let n = pv.len().min(sp.len());
        let te: Vec<f64> = (0..n).map(|k| pv[k] - sp[k]).collect();
        Self::hash_te(&te) == self.tracking_error_hash && self.seal() == self.witness_hash
    }
}

// ── ControllerModeGuardV1 ───────────────────────────────────────────────────────────────────────────

/// One episode's mode-transition guard record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeGuard {
    pub episode_start: usize,
    pub episode_end: usize,
    /// True iff a controller-mode transition occurred within the episode (incl. the lookback window before it).
    pub guarded: bool,
    /// The sample index of the transition that guards this episode (first one found), if any.
    pub transition_index: Option<usize>,
}

/// A hash-sealed controller-mode guard (schema v1) over a set of episodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControllerModeGuardV1 {
    pub guard_lookback: usize,
    pub guards: Vec<EpisodeGuard>,
    pub n_guarded: usize,
    /// SHA-256 of the mode-label series (so the exact mode trajectory is sealed).
    pub mode_hash: String,
    pub guard_hash: String,
}

impl ControllerModeGuardV1 {
    fn hash_modes(modes: &[String]) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"controller_mode_series_v1");
        for m in modes {
            h.field("mode", m.as_bytes());
        }
        h.finalize_hex()
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"controller_mode_guard_v1");
        h.u64("guard_lookback", self.guard_lookback as u64);
        for g in &self.guards {
            h.u64("episode_start", g.episode_start as u64);
            h.u64("episode_end", g.episode_end as u64);
            h.u64("guarded", g.guarded as u64);
            h.u64("has_transition", g.transition_index.is_some() as u64);
            h.u64("transition_index", g.transition_index.unwrap_or(0) as u64);
        }
        h.u64("n_guarded", self.n_guarded as u64);
        h.field("mode_hash", self.mode_hash.as_bytes());
        h.finalize_hex()
    }

    /// Flag every episode that overlaps a controller-mode transition (a sample where the mode label differs
    /// from the previous sample), including a `guard_lookback`-sample window before the episode start.
    pub fn build(modes: &[String], episodes: &[(usize, usize)], guard_lookback: usize) -> Self {
        // Transition indices: k where modes[k] != modes[k-1].
        let transitions: Vec<usize> = (1..modes.len())
            .filter(|&k| modes[k] != modes[k - 1])
            .collect();
        let guards: Vec<EpisodeGuard> = episodes
            .iter()
            .map(|&(start, end)| {
                let lo = start.saturating_sub(guard_lookback);
                let transition_index = transitions.iter().copied().find(|&t| t >= lo && t <= end);
                EpisodeGuard {
                    episode_start: start,
                    episode_end: end,
                    guarded: transition_index.is_some(),
                    transition_index,
                }
            })
            .collect();
        let n_guarded = guards.iter().filter(|g| g.guarded).count();
        let mut court = ControllerModeGuardV1 {
            guard_lookback,
            guards,
            n_guarded,
            mode_hash: Self::hash_modes(modes),
            guard_hash: String::new(),
        };
        court.guard_hash = court.seal();
        court
    }

    pub fn verify(&self, modes: &[String]) -> bool {
        let n_guarded = self.guards.iter().filter(|g| g.guarded).count();
        n_guarded == self.n_guarded
            && Self::hash_modes(modes) == self.mode_hash
            && self.seal() == self.guard_hash
    }
}

// ── ControlLoopInteractionMapV1 ─────────────────────────────────────────────────────────────────────

/// A candidate interaction edge between two loops, weighted by their MV correlation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteractionEdge {
    pub loop_a: String,
    pub loop_b: String,
    pub mv_correlation: f64,
}

/// A hash-sealed control-loop interaction map (schema v1) from controller-effort (MV) correlations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlLoopInteractionMapV1 {
    pub loop_ids: Vec<String>,
    pub threshold: f64,
    /// Strong edges only (|corr| ≥ threshold), in `(i<j)` order.
    pub edges: Vec<InteractionEdge>,
    pub n_strong_interactions: usize,
    pub map_hash: String,
}

impl ControlLoopInteractionMapV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"control_loop_interaction_map_v1");
        for id in &self.loop_ids {
            h.field("loop_id", id.as_bytes());
        }
        h.f64q("threshold", self.threshold);
        for e in &self.edges {
            h.field("loop_a", e.loop_a.as_bytes());
            h.field("loop_b", e.loop_b.as_bytes());
            h.f64q("mv_correlation", e.mv_correlation);
        }
        h.u64("n_strong_interactions", self.n_strong_interactions as u64);
        h.finalize_hex()
    }

    /// Build the map from `(loop_id, mv_series)` pairs: an edge is emitted for every loop pair whose MV
    /// correlation magnitude is at least `threshold`.
    pub fn build(loops: &[(String, Vec<f64>)], threshold: f64) -> Self {
        let loop_ids: Vec<String> = loops.iter().map(|(id, _)| id.clone()).collect();
        let mut edges = Vec::new();
        for i in 0..loops.len() {
            for j in (i + 1)..loops.len() {
                let c = pearson(&loops[i].1, &loops[j].1);
                if c.abs() >= threshold {
                    edges.push(InteractionEdge {
                        loop_a: loops[i].0.clone(),
                        loop_b: loops[j].0.clone(),
                        mv_correlation: c,
                    });
                }
            }
        }
        let n_strong_interactions = edges.len();
        let mut map = ControlLoopInteractionMapV1 {
            loop_ids,
            threshold,
            edges,
            n_strong_interactions,
            map_hash: String::new(),
        };
        map.map_hash = map.seal();
        map
    }

    pub fn verify(&self) -> bool {
        self.n_strong_interactions == self.edges.len() && self.seal() == self.map_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setpoint_change_is_not_a_tracking_error() {
        // PV tracks SP perfectly; SP steps from 50→60 at k=2. Every sample is on-target or setpoint-driven;
        // none is a tracking error (the step is explained by the setpoint move, not a process anomaly).
        let sp = vec![50.0, 50.0, 60.0, 60.0, 60.0];
        let pv = vec![50.0, 50.0, 60.0, 60.0, 60.0];
        let w = SetpointResidualSeparationV1::build("PV", "SP", &pv, &sp, 1.0, 0.5);
        assert_eq!(w.n_tracking_error, 0);
        assert_eq!(w.n_setpoint_driven, 1); // the k=2 step
        assert_eq!(w.false_alarms_avoided(), 1);
        assert!(w.peak_tracking_error < 1e-12);
        assert!(w.witness_hash.len() == 64 && w.verify(&pv, &sp));
    }

    #[test]
    fn steady_setpoint_with_pv_drift_is_a_tracking_error() {
        // SP held at 50; PV drifts off to 53 while SP is steady ⇒ a genuine tracking error of 3.
        let sp = vec![50.0, 50.0, 50.0, 50.0];
        let pv = vec![50.0, 51.0, 52.0, 53.0];
        let w = SetpointResidualSeparationV1::build("PV", "SP", &pv, &sp, 1.0, 0.5);
        assert_eq!(w.n_setpoint_driven, 0);
        assert!(w.n_tracking_error >= 2);
        assert!((w.peak_tracking_error - 3.0).abs() < 1e-12);
        assert!(w.verify(&pv, &sp));
        assert!(!w.verify(&[0.0; 4], &sp)); // a different PV stream fails verification
    }

    #[test]
    fn mode_guard_flags_episodes_overlapping_a_transition() {
        // Mode auto→manual at k=5. Episode [4,7] overlaps it (guarded); episode [10,12] does not.
        let modes: Vec<String> = (0..14)
            .map(|k| if k < 5 { "auto" } else { "manual" }.to_string())
            .collect();
        let g = ControllerModeGuardV1::build(&modes, &[(4, 7), (10, 12)], 1);
        assert!(g.guards[0].guarded && g.guards[0].transition_index == Some(5));
        assert!(!g.guards[1].guarded);
        assert_eq!(g.n_guarded, 1);
        assert!(g.guard_hash.len() == 64 && g.verify(&modes));
        // Tampering the mode series (where a transition was) breaks the seal.
        let mut modes2 = modes.clone();
        modes2[5] = "auto".into();
        assert!(!g.verify(&modes2));
    }

    #[test]
    fn interaction_map_links_loops_with_correlated_controller_effort() {
        // Loops A and B have identical MV traces (corr 1); C is anti-correlated with A (corr -1); D is flat.
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![1.0, 2.0, 3.0, 4.0];
        let c = vec![4.0, 3.0, 2.0, 1.0];
        let d = vec![2.0, 2.0, 2.0, 2.0];
        let loops = vec![
            ("A".into(), a),
            ("B".into(), b),
            ("C".into(), c),
            ("D".into(), d),
        ];
        let m = ControlLoopInteractionMapV1::build(&loops, 0.9);
        // A–B (+1), A–C (−1), B–C (−1) are strong; anything with the flat D is 0 (excluded).
        assert_eq!(m.n_strong_interactions, 3);
        assert!(m
            .edges
            .iter()
            .any(|e| e.loop_a == "A" && e.loop_b == "B" && (e.mv_correlation - 1.0).abs() < 1e-9));
        assert!(m
            .edges
            .iter()
            .any(|e| e.loop_a == "A" && e.loop_b == "C" && (e.mv_correlation + 1.0).abs() < 1e-9));
        assert!(m.edges.iter().all(|e| e.loop_a != "D" && e.loop_b != "D"));
        assert!(m.map_hash.len() == 64 && m.verify());
    }

    #[test]
    fn pearson_is_zero_for_a_constant_signal() {
        assert_eq!(pearson(&[1.0, 1.0, 1.0], &[1.0, 2.0, 3.0]), 0.0);
        assert!((pearson(&[1.0, 2.0, 3.0], &[2.0, 4.0, 6.0]) - 1.0).abs() < 1e-12);
    }
}
