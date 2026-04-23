//! Renormalization Group (RG) Flow on TDA Persistence Diagrams.
//!
//! ## Motivation
//!
//! A Topological Data Analysis (TDA) persistence diagram contains features
//! at *all scales simultaneously*: local noise creates short-lived (low-
//! persistence) Betti features while structural interference creates
//! long-lived (high-persistence) ones. The standard approach — a fixed
//! persistence threshold — is scale-dependent and requires manual tuning.
//!
//! **Renormalization Group (RG) Flow** (Wilson 1971, 1975) is a
//! physics-inspired technique for systematically "coarse-graining" a
//! system: integrating out short-scale degrees of freedom while tracking
//! how long-scale behaviour evolves. Applied to persistence diagrams:
//!
//! 1. Construct the persistence diagram at scale ε_0 (fine).
//! 2. **RG-coarsen** the diagram by merging features with
//!    $\text{persistence} < \delta\epsilon$ (integrating out noise).
//! 3. Repeat for $\epsilon_1 = \epsilon_0 + \delta\epsilon$, $\epsilon_2$, ...
//! 4. The **RG trajectory** — how the Betti numbers evolve across scales —
//!    distinguishes:
//!    - *Hardware flukes*: Betti features that vanish quickly as ε grows.
//!    - *Structural changes*: Betti features that persist across RG scales.
//!    - *Systemic environment changes*: new Betti-0 components that appear
//!      (birth) at coarse scales, indicating global topological phase
//!      transitions in the interference environment.
//!
//! ## Design
//!
//! This module operates on the `[PersistenceEvent; MAX_EVENTS]` array
//! produced by `tda::detect_topological_innovation`. It applies an
//! iterative scale-coarsening loop to compute the RG flow:
//!
//! - **`RgScale`**: one scale level in the flow.
//! - **`RgFlowResult`**: the full flow trajectory.
//! - **`RgFlowClassification`**: final classification of the topological
//!   change as `LocalNoise`, `HardwareFluke`, `StructuralOnset`, or
//!   `SystemicEnvironmentChange`.
//!
//! ## no_std / no_alloc / zero-unsafe
//!
//! Fixed arrays; `MAX_RG_SCALES = 8`. No heap. No unsafe.
//!
//! ## References
//!
//! - Wilson & Kogut (1974), "The renormalization group and the ε expansion",
//!   Phys. Rep. 12(2):75-200.
//! - Edelsbrunner & Harer (2010), Computational Topology, AMS, Ch. VII.
//! - Chazal et al. (2016), "The Structure and Stability of Persistence
//!   Modules", Springer Briefs.
//! - Adcock, Carlsson & Carlsson (2016), "The Ring of Algebraic Functions
//!   on Persistence Bar Codes".

use crate::tda::PersistenceEvent;

// ── Capacity ───────────────────────────────────────────────────────────────

/// Maximum number of RG scale levels.
pub const MAX_RG_SCALES: usize = 8;

/// Maximum number of persistence events per scale (matches tda::MAX_EVENTS).
pub const MAX_RG_EVENTS: usize = 32;

// ── RG Scale ───────────────────────────────────────────────────────────────

/// One coarse-graining scale level in the RG flow.
#[derive(Debug, Clone, Copy)]
pub struct RgScale {
    /// Coarse-graining scale ε at this level.
    pub epsilon: f32,
    /// Number of Betti-0 features surviving at this scale.
    pub betti0_surviving: u16,
    /// Number of features coarse-grained away (merged/eliminated) at this step.
    pub features_merged: u16,
    /// Mean persistence of surviving features at this scale.
    pub mean_persistence: f32,
    /// Maximum persistence of any surviving feature (the "infinite bar").
    pub max_persistence: f32,
    /// Topological innovation score at this scale
    /// (fraction of features that are new relative to ε_0).
    pub innovation_fraction: f32,
}

// ── RG Flow Result ─────────────────────────────────────────────────────────

/// Full RG flow trajectory over MAX_RG_SCALES scale levels.
pub struct RgFlowResult {
    /// Scales in order from fine (ε_0) to coarse.
    pub scales:      [RgScale; MAX_RG_SCALES],
    /// Number of valid scales computed.
    pub n_scales:    usize,
    /// Classification of the topological change.
    pub class:       RgFlowClass,
    /// Scale at which the dominant Betti-0 structure becomes stable.
    /// `None` if no stability is found (transient noise at all scales).
    pub stable_at:   Option<f32>,
    /// Persistence decay exponent β_RG fit to Betti₀(ε) ∝ ε^{-β_RG}.
    /// β_RG ≈ 0: persistent structure (systemic change).
    /// β_RG >> 1: rapid decay (local noise).
    pub beta_rg:     f32,
}

// ── Classification ─────────────────────────────────────────────────────────

/// Classification of a topological change based on its RG flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RgFlowClass {
    /// All Betti-0 features vanish within 1-2 coarse-graining steps.
    /// Consistent with local hardware noise, ADC quantisation, or
    /// short-lived atmospheric scintillation.
    LocalNoise,
    /// Features survive 2-4 steps then vanish. Consistent with a hardware
    /// fluke (e.g., thermal transient, memory effect) rather than structural
    /// interference.
    HardwareFluke,
    /// Features survive 4-6 steps. Consistent with coherent structural
    /// interference onset (carrier-class jammer, CW tone injection).
    StructuralOnset,
    /// Features persist across all scale levels. Consistent with a systemic
    /// environment change (new permanent transmitter, infrastructure change,
    /// propagation-path structural shift).
    SystemicEnvironmentChange,
    /// Insufficient persistence events to classify.
    Indeterminate,
}

impl RgFlowClass {
    /// Human-readable label.
    pub const fn label(self) -> &'static str {
        match self {
            RgFlowClass::LocalNoise               => "LocalNoise",
            RgFlowClass::HardwareFluke            => "HardwareFluke",
            RgFlowClass::StructuralOnset          => "StructuralOnset",
            RgFlowClass::SystemicEnvironmentChange => "SystemicEnvironmentChange",
            RgFlowClass::Indeterminate            => "Indeterminate",
        }
    }

    /// Whether this classification warrants grammar escalation to Boundary.
    pub const fn warrants_boundary_escalation(self) -> bool {
        matches!(self, RgFlowClass::StructuralOnset | RgFlowClass::SystemicEnvironmentChange)
    }
}

// ── RG Flow Engine ─────────────────────────────────────────────────────────

/// Compute the RG flow for a set of persistence events.
///
/// # Arguments
/// - `events`      — persistence events from `tda::detect_topological_innovation`
/// - `n_events`    — number of valid events in `events`
/// - `epsilon_0`   — initial (finest) scale ε₀
/// - `delta_eps`   — coarse-graining step Δε per level
///
/// Each RG step merges all features with `persistence < epsilon` into the
/// background (coarse-grained away). The resulting Betti₀ count at each
/// scale forms the flow trajectory.
pub fn compute_rg_flow(
    events:    &[PersistenceEvent],
    n_events:  usize,
    epsilon_0: f32,
    delta_eps: f32,
) -> RgFlowResult {
    let n = n_events.min(events.len()).min(MAX_RG_EVENTS);
    if n == 0 { return empty_rg_result(); }

    let persistence_vals = extract_persistence_values(events, n);
    let eps0_betti0 = n as u16;
    let scales = build_rg_scales(&persistence_vals, n, eps0_betti0, epsilon_0, delta_eps);
    let beta_rg = fit_beta_rg(&scales, eps0_betti0, epsilon_0);
    let stable_at = find_stable_scale(&scales);
    let class = classify_rg_flow(&scales, n);

    RgFlowResult {
        scales,
        n_scales: MAX_RG_SCALES,
        class,
        stable_at,
        beta_rg,
    }
}

fn empty_rg_result() -> RgFlowResult {
    RgFlowResult {
        scales: [RgScale {
            epsilon: 0.0, betti0_surviving: 0, features_merged: 0,
            mean_persistence: 0.0, max_persistence: 0.0, innovation_fraction: 0.0,
        }; MAX_RG_SCALES],
        n_scales: 0,
        class: RgFlowClass::Indeterminate,
        stable_at: None,
        beta_rg: 0.0,
    }
}

fn extract_persistence_values(events: &[PersistenceEvent], n: usize) -> [f32; MAX_RG_EVENTS] {
    let mut persistence_vals = [0.0_f32; MAX_RG_EVENTS];
    for i in 0..n {
        let lt = events[i].lifetime();
        persistence_vals[i] = if lt == f32::MAX { 1e6_f32 } else { lt };
    }
    persistence_vals
}

fn build_rg_scales(
    persistence_vals: &[f32; MAX_RG_EVENTS],
    n: usize,
    eps0_betti0: u16,
    epsilon_0: f32,
    delta_eps: f32,
) -> [RgScale; MAX_RG_SCALES] {
    let mut scales = [RgScale {
        epsilon: 0.0, betti0_surviving: 0, features_merged: 0,
        mean_persistence: 0.0, max_persistence: 0.0, innovation_fraction: 0.0,
    }; MAX_RG_SCALES];

    for level in 0..MAX_RG_SCALES {
        let eps = epsilon_0 + (level as f32) * delta_eps;
        let (surviving, sum_p, max_p) = count_surviving(persistence_vals, n, eps);
        let merged = if level == 0 {
            0u16
        } else {
            scales[level - 1].betti0_surviving.saturating_sub(surviving)
        };
        let mean_p = if surviving > 0 { sum_p / surviving as f32 } else { 0.0 };
        let innovation_fraction = 1.0 - (surviving as f32 / eps0_betti0 as f32);

        scales[level] = RgScale {
            epsilon: eps,
            betti0_surviving: surviving,
            features_merged: merged,
            mean_persistence: mean_p,
            max_persistence: max_p,
            innovation_fraction,
        };
    }
    scales
}

fn count_surviving(persistence_vals: &[f32; MAX_RG_EVENTS], n: usize, eps: f32) -> (u16, f32, f32) {
    let mut surviving = 0u16;
    let mut sum_p = 0.0_f32;
    let mut max_p = 0.0_f32;
    for i in 0..n {
        if persistence_vals[i] >= eps {
            surviving += 1;
            sum_p += persistence_vals[i];
            if persistence_vals[i] > max_p { max_p = persistence_vals[i]; }
        }
    }
    (surviving, sum_p, max_p)
}

fn fit_beta_rg(scales: &[RgScale; MAX_RG_SCALES], eps0_betti0: u16, epsilon_0: f32) -> f32 {
    let mut sum_xy = 0.0_f32;
    let mut sum_xx = 0.0_f32;
    let mut fit_n  = 0u32;
    for level in 0..MAX_RG_SCALES {
        let b = scales[level].betti0_surviving;
        if b > 0 && b < eps0_betti0 {
            let log_eps = crate::math::ln_f32((scales[level].epsilon / epsilon_0.max(1e-9)).max(1e-9));
            let log_b   = crate::math::ln_f32(b as f32);
            sum_xy += log_eps * log_b;
            sum_xx += log_eps * log_eps;
            fit_n  += 1;
        }
    }
    if sum_xx > 1e-9 && fit_n >= 2 { -(sum_xy / sum_xx) } else { 0.0 }
}

fn find_stable_scale(scales: &[RgScale; MAX_RG_SCALES]) -> Option<f32> {
    for level in 1..MAX_RG_SCALES {
        if scales[level].betti0_surviving == scales[level - 1].betti0_surviving
            && scales[level].betti0_surviving > 0 {
            return Some(scales[level].epsilon);
        }
    }
    None
}

fn classify_rg_flow(scales: &[RgScale; MAX_RG_SCALES], n: usize) -> RgFlowClass {
    if n == 0 { return RgFlowClass::Indeterminate; }
    let n_surviving_levels = scales.iter()
        .take(MAX_RG_SCALES)
        .filter(|s| s.betti0_surviving > 0)
        .count();
    if n_surviving_levels <= 1 { RgFlowClass::LocalNoise }
    else if n_surviving_levels <= 3 { RgFlowClass::HardwareFluke }
    else if n_surviving_levels <= 6 { RgFlowClass::StructuralOnset }
    else { RgFlowClass::SystemicEnvironmentChange }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tda::PersistenceEvent;

    fn make_events(persistences: &[f32]) -> ([PersistenceEvent; MAX_RG_EVENTS], usize) {
        let mut events = [PersistenceEvent { birth_radius: 0.0, death_radius: 0.0 }; MAX_RG_EVENTS];
        let n = persistences.len().min(MAX_RG_EVENTS);
        for (i, &p) in persistences.iter().enumerate().take(n) {
            events[i] = PersistenceEvent { birth_radius: 0.0, death_radius: p };
        }
        (events, n)
    }

    #[test]
    fn empty_events_returns_indeterminate() {
        let events = [PersistenceEvent { birth_radius: 0.0, death_radius: 0.0 }; MAX_RG_EVENTS];
        let result = compute_rg_flow(&events, 0, 0.01, 0.01);
        assert_eq!(result.class, RgFlowClass::Indeterminate);
        assert_eq!(result.n_scales, 0);
    }

    #[test]
    fn all_low_persistence_is_local_noise() {
        // All features vanish at eps_0 + 1 step
        let (events, n) = make_events(&[0.001, 0.002, 0.003]);
        let result = compute_rg_flow(&events, n, 0.1, 0.01);
        assert_eq!(result.class, RgFlowClass::LocalNoise,
            "tiny persistence: {:?}", result.class);
    }

    #[test]
    fn high_persistence_is_systemic() {
        // All features persist across all scale levels
        let (events, n) = make_events(&[100.0, 200.0, 300.0, 150.0, 250.0]);
        let result = compute_rg_flow(&events, n, 0.01, 0.01);
        assert_eq!(result.class, RgFlowClass::SystemicEnvironmentChange,
            "persistent features: {:?}", result.class);
    }

    #[test]
    fn surviving_features_monotone_decrease_with_scale() {
        let (events, n) = make_events(&[0.05, 0.10, 0.20, 0.30, 0.50]);
        let result = compute_rg_flow(&events, n, 0.01, 0.03);
        for i in 1..result.n_scales {
            assert!(result.scales[i].betti0_surviving <= result.scales[i-1].betti0_surviving,
                "Betti₀ must be monotone non-increasing: level {}", i);
        }
    }

    #[test]
    fn flow_class_warrants_boundary_escalation_for_onset() {
        assert!(RgFlowClass::StructuralOnset.warrants_boundary_escalation());
        assert!(RgFlowClass::SystemicEnvironmentChange.warrants_boundary_escalation());
        assert!(!RgFlowClass::LocalNoise.warrants_boundary_escalation());
        assert!(!RgFlowClass::HardwareFluke.warrants_boundary_escalation());
    }

    #[test]
    fn labels_are_distinct_and_correct() {
        assert_eq!(RgFlowClass::LocalNoise.label(),               "LocalNoise");
        assert_eq!(RgFlowClass::HardwareFluke.label(),            "HardwareFluke");
        assert_eq!(RgFlowClass::StructuralOnset.label(),          "StructuralOnset");
        assert_eq!(RgFlowClass::SystemicEnvironmentChange.label(), "SystemicEnvironmentChange");
    }

    #[test]
    fn stable_at_found_for_persistent_features() {
        // Two features persist past scale 0.2
        let (events, n) = make_events(&[10.0, 10.0, 10.0]);
        let result = compute_rg_flow(&events, n, 0.01, 0.01);
        // stable_at: all features survive all levels → stability at earliest level
        assert!(result.stable_at.is_some(), "stable at should be found for persistent features");
    }
}
