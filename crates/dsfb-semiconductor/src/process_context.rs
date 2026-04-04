//! Recipe-step and tool-state context for admissibility gating.
//!
//! # Step-Indexed Admissibility
//! The DSFB admissibility envelope radius ρ is not a global constant.
//! Industry practice demands that tolerance bands vary with the process
//! recipe step: a ±5 % gas-flow deviation is expected during a
//! "Gas Stabilise" ramp, but the same deviation during "Main Etch"
//! constitutes an out-of-control condition.
//!
//! This module encodes that domain knowledge as a look-up table (LUT)
//! keyed on [`RecipeStep`].  The LUT multiplier is applied to the
//! feature-level ρ values before the grammar layer evaluates admissibility.
//!
//! # Maintenance Hysteresis
//! Upon receipt of a [`ToolState::ChamberClean`] signal the engine
//! executes a "Warm Reset": accumulated grammar state is cleared and
//! a configurable guard window suppresses new alarms for the first
//! `post_clean_guard_runs` runs after clean completion.  This prevents
//! false escalations during the seasoning period that follows every
//! chamber clean.
//!
//! # No-std Compatibility
//! This module is `no_std`-compatible with `alloc`.

use serde::{Deserialize, Serialize};
#[cfg(not(feature = "std"))]
use alloc::{format, string::{String, ToString}};

// ─── Recipe Step ──────────────────────────────────────────────────────────────

/// Canonical set of recipe steps recognised by the DSFB engine.
///
/// Fabs that use non-standard step names should map their internal identifiers
/// to the closest canonical variant via [`RecipeStep::from_str`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RecipeStep {
    /// Pre-etch gas stabilisation — MFC setpoints ramp; relaxed tolerances
    /// apply because transient overshoots are physically expected.
    GasStabilize,
    /// Main active etch — tight tolerances; this is the yield-critical window.
    MainEtch,
    /// Deposition step — moderate tolerances.
    Deposition,
    /// Post-etch over-etch — slightly relaxed tolerances.
    OverEtch,
    /// Chamber conditioning after maintenance — widest tolerances.
    Seasoning,
    /// Any step not explicitly classified; baseline tolerances apply.
    Other(String),
}

impl Default for RecipeStep {
    fn default() -> Self {
        Self::Other("unknown".into())
    }
}

impl RecipeStep {
    /// Parse a step name string into the canonical variant.  Matching is
    /// case-insensitive.  Unknown names produce [`RecipeStep::Other`].
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "gas_stabilize" | "gas stabilize" | "gasstabilize" | "stabilize" => {
                Self::GasStabilize
            }
            "main_etch" | "main etch" | "mainetch" | "etch" => Self::MainEtch,
            "deposition" | "dep" | "cvd" | "pvd" | "ald" => Self::Deposition,
            "over_etch" | "over etch" | "overetch" => Self::OverEtch,
            "seasoning" | "season" | "conditioning" => Self::Seasoning,
            other => Self::Other(other.to_string()),
        }
    }

    /// Short display name for reporting / traceability manifests.
    pub fn display_name(&self) -> &str {
        match self {
            Self::GasStabilize => "GasStabilize",
            Self::MainEtch => "MainEtch",
            Self::Deposition => "Deposition",
            Self::OverEtch => "OverEtch",
            Self::Seasoning => "Seasoning",
            Self::Other(s) => s.as_str(),
        }
    }
}

// ─── Tool State ───────────────────────────────────────────────────────────────

/// Observable tool-level state that may suppress or modulate the grammar engine.
///
/// The tool-state is independent of the recipe step: a chamber clean can be
/// initiated between any two wafer runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum ToolState {
    /// Normal production run — grammar operates at full sensitivity.
    #[default]
    Production,
    /// Chamber clean cycle in progress.
    ///
    /// **Effect:** the grammar engine issues a Warm Reset; all deviations
    /// during this state are suppressed because the process is intentionally
    /// out-of-spec.
    ChamberClean,
    /// Post-clean seasoning cycle — grammar operates at reduced sensitivity
    /// because thin-film conditioning transients are expected.
    Seasoning,
    /// Tool is idle or in a maintenance hold — monitoring is paused.
    Maintenance,
}

// ─── Process Context ──────────────────────────────────────────────────────────

/// Full operational context for a single process run, passed to the DSFB
/// engine so that admissibility gating is recipe-step-aware.
///
/// # Example
/// ```
/// use dsfb_semiconductor::process_context::{ProcessContext, RecipeStep, ToolState};
///
/// let ctx = ProcessContext {
///     recipe_step_id: "STEP_002_MAIN_ETCH".into(),
///     recipe_step: RecipeStep::MainEtch,
///     tool_state: ToolState::Production,
///     lot_id: Some("LOT-2026-0401".into()),
///     chamber_id: Some("CH-A".into()),
/// };
///
/// // Main-etch tightens the envelope by 20 %.
/// assert!((ctx.admissibility_multiplier() - 0.80).abs() < 1e-9);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessContext {
    /// Fab-internal identifier string for the active recipe step.
    pub recipe_step_id: String,
    /// Canonical DSFB classification of the recipe step.
    pub recipe_step: RecipeStep,
    /// Observable tool-level state.
    pub tool_state: ToolState,
    /// Wafer lot / batch identifier (informational; included in traceability).
    pub lot_id: Option<String>,
    /// Chamber identifier (informational; included in traceability).
    pub chamber_id: Option<String>,
}

impl ProcessContext {
    /// Admissibility envelope multiplier for this context.
    ///
    /// The LUT below encodes industry-standard domain knowledge:
    ///
    /// | Recipe step    | Multiplier | Rationale                              |
    /// |----------------|-----------|---------------------------------------- |
    /// | `GasStabilize` | 1.50 ×    | Ramp transients are physically expected |
    /// | `MainEtch`     | 0.80 ×    | Yield-critical; tightest control window |
    /// | `Deposition`   | 1.10 ×    | Moderate tolerance                      |
    /// | `OverEtch`     | 1.20 ×    | Controlled endpoint; slightly relaxed   |
    /// | `Seasoning`    | 2.00 ×    | Post-clean transients expected          |
    /// | `Other`        | 1.00 ×    | Baseline                                |
    ///
    /// During a `ChamberClean` tool state, deviations are fully suppressed
    /// (`f64::INFINITY` — no finite residual can exceed an infinite envelope).
    #[must_use]
    pub fn admissibility_multiplier(&self) -> f64 {
        if self.tool_state == ToolState::ChamberClean {
            return f64::INFINITY; // suppress all deviations; chamber is intentionally dirty
        }
        match &self.recipe_step {
            RecipeStep::GasStabilize => 1.50,
            RecipeStep::MainEtch => 0.80,
            RecipeStep::Deposition => 1.10,
            RecipeStep::OverEtch => 1.20,
            RecipeStep::Seasoning => 2.00,
            RecipeStep::Other(_) => 1.00,
        }
    }

    /// Returns `true` when the grammar engine must issue a Warm Reset.
    ///
    /// A Warm Reset clears accumulated grammar state to prevent false alarms
    /// after a chamber clean or during a seasoning cycle.
    #[must_use]
    pub fn requires_warm_reset(&self) -> bool {
        matches!(
            self.tool_state,
            ToolState::ChamberClean | ToolState::Seasoning | ToolState::Maintenance
        )
    }

    /// A terse string representation for traceability manifests.
    pub fn traceability_tag(&self) -> String {
        format!(
            "step={} tool_state={:?} lot={} chamber={}",
            self.recipe_step.display_name(),
            self.tool_state,
            self.lot_id.as_deref().unwrap_or("none"),
            self.chamber_id.as_deref().unwrap_or("none"),
        )
    }
}

// ─── Maintenance Hysteresis ───────────────────────────────────────────────────

/// Tracks the hysteresis boundary around chamber-clean and seasoning events.
///
/// When a [`ToolState::ChamberClean`] signal is received, the engine
/// performs a "Warm Reset": the accumulated grammar state is cleared so that
/// post-clean transients during seasoning do not trigger false alarms.
/// A configurable guard window (`post_clean_guard_runs`) suppresses new alarms
/// for the first N wafer runs after clean completion.
///
/// # Integration Pattern
/// Call [`MaintenanceHysteresis::update`] at the start of every run with the
/// current [`ProcessContext`].  If it returns `true`, flush the grammar
/// accumulator for all features before processing the current run.
///
/// # Example
/// ```
/// use dsfb_semiconductor::process_context::{
///     MaintenanceHysteresis, ProcessContext, ToolState,
/// };
///
/// let mut hyst = MaintenanceHysteresis::new(10);
/// let mut ctx = ProcessContext::default();
///
/// ctx.tool_state = ToolState::ChamberClean;
/// assert!(hyst.update(&ctx), "clean signal must trigger warm reset");
/// assert!(hyst.is_suppressed(), "guard window should be active");
///
/// ctx.tool_state = ToolState::Production;
/// for _ in 0..10 {
///     hyst.update(&ctx);
/// }
/// assert!(!hyst.is_suppressed(), "guard window should have elapsed");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceHysteresis {
    /// Number of production runs after a clean event during which alarms are
    /// suppressed to allow the chamber seasoning transient to decay.
    pub post_clean_guard_runs: usize,

    reset_pending: bool,
    runs_since_reset: usize,
}

impl Default for MaintenanceHysteresis {
    fn default() -> Self {
        Self::new(10)
    }
}

impl MaintenanceHysteresis {
    /// Create a new tracker with the specified guard window.
    ///
    /// Setting `post_clean_guard_runs = 0` disables the guard window.
    pub fn new(post_clean_guard_runs: usize) -> Self {
        Self {
            post_clean_guard_runs,
            reset_pending: false,
            runs_since_reset: usize::MAX,
        }
    }

    /// Update the tracker with the current process context.
    ///
    /// Returns `true` if the grammar engine must perform a Warm Reset
    /// (i.e., the current context indicates a clean/maintenance event).
    pub fn update(&mut self, ctx: &ProcessContext) -> bool {
        if ctx.requires_warm_reset() {
            self.reset_pending = true;
            self.runs_since_reset = 0;
            return true;
        }

        if self.reset_pending {
            self.runs_since_reset = self.runs_since_reset.saturating_add(1);
            if self.runs_since_reset >= self.post_clean_guard_runs {
                self.reset_pending = false;
                self.runs_since_reset = usize::MAX;
            }
        }

        false
    }

    /// Returns `true` while the engine is inside the post-clean suppression
    /// window (i.e., alarms should be downgraded to `Watch` or suppressed).
    #[must_use]
    pub fn is_suppressed(&self) -> bool {
        self.reset_pending
    }

    /// Number of production runs elapsed since the last Warm Reset.
    /// Returns `usize::MAX` when no reset has been triggered.
    #[must_use]
    pub fn runs_since_last_reset(&self) -> usize {
        self.runs_since_reset
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_etch_tightens_envelope() {
        let ctx = ProcessContext {
            recipe_step: RecipeStep::MainEtch,
            tool_state: ToolState::Production,
            ..Default::default()
        };
        assert!(
            (ctx.admissibility_multiplier() - 0.80).abs() < 1e-9,
            "MainEtch should yield 0.80× multiplier"
        );
    }

    #[test]
    fn gas_stabilize_relaxes_envelope() {
        let ctx = ProcessContext {
            recipe_step: RecipeStep::GasStabilize,
            tool_state: ToolState::Production,
            ..Default::default()
        };
        assert!(
            (ctx.admissibility_multiplier() - 1.50).abs() < 1e-9,
            "GasStabilize should yield 1.50× multiplier"
        );
    }

    #[test]
    fn chamber_clean_suppresses_everything() {
        let ctx = ProcessContext {
            recipe_step: RecipeStep::MainEtch,
            tool_state: ToolState::ChamberClean,
            ..Default::default()
        };
        assert!(
            ctx.admissibility_multiplier().is_infinite(),
            "ChamberClean should yield MAX multiplier (full suppression)"
        );
    }

    #[test]
    fn chamber_clean_requires_warm_reset() {
        let ctx = ProcessContext {
            tool_state: ToolState::ChamberClean,
            ..Default::default()
        };
        assert!(ctx.requires_warm_reset());
    }

    #[test]
    fn production_does_not_require_warm_reset() {
        let ctx = ProcessContext {
            tool_state: ToolState::Production,
            ..Default::default()
        };
        assert!(!ctx.requires_warm_reset());
    }

    #[test]
    fn hysteresis_guard_window_elapses() {
        let mut hyst = MaintenanceHysteresis::new(3);
        let mut clean = ProcessContext::default();
        clean.tool_state = ToolState::ChamberClean;

        assert!(hyst.update(&clean));
        assert!(hyst.is_suppressed());

        let mut prod = ProcessContext::default();
        prod.tool_state = ToolState::Production;

        hyst.update(&prod);
        assert!(hyst.is_suppressed());
        hyst.update(&prod);
        assert!(hyst.is_suppressed());
        hyst.update(&prod);
        assert!(!hyst.is_suppressed(), "guard window should have expired after 3 runs");
    }

    #[test]
    fn recipe_step_from_str_round_trips() {
        assert_eq!(RecipeStep::from_str("main_etch"), RecipeStep::MainEtch);
        assert_eq!(RecipeStep::from_str("GAS STABILIZE"), RecipeStep::GasStabilize);
        assert_eq!(RecipeStep::from_str("seasoning"), RecipeStep::Seasoning);
        assert!(matches!(
            RecipeStep::from_str("plasma_clean"),
            RecipeStep::Other(_)
        ));
    }
}
