//! Waveform transition context for grammar-escalation suppression.
//!
//! ## Motivation (paper §18.3)
//!
//! > "Deliberate waveform transitions — frequency hops, modulation changes,
//! > burst boundaries — produce residual signatures structurally
//! > indistinguishable from interference onset without a waveform-schedule
//! > context flag. The correct integration contract includes a
//! > regime-context channel that suppresses grammar escalation during
//! > flagged transition windows. This is a **near-term engineering
//! > extension**. The `platform_context.rs` module provides the hook;
//! > population of the transition schedule is deployment-specific."
//!
//! This module provides that extension: a fixed-capacity waveform schedule
//! that marks transition windows and suppresses spurious grammar escalation
//! during those intervals.
//!
//! ## Design
//!
//! - **`TransitionWindow`**: a half-open interval `[start_k, end_k + margin)`
//!   during which the grammar-state escalation to `Violation` is suppressed.
//!   The optional `suppression_margin` adds post-transition damping to absorb
//!   residual ringing from waveform changes.
//! - **`WaveformSchedule<N>`**: a fixed-capacity (no_alloc) collection of
//!   transition windows. `N` is a compile-time constant.
//! - **`suppress_escalation(k, schedule)`**: the query function the engine
//!   calls at observation `k`. Returns `true` during any active window.
//!
//! ## Non-Claims
//!
//! The schedule must be populated from a deployment-specific source
//! (e.g., TDMA frame schedule, FHSS channel plan, link-layer signaling).
//! This module provides the data structure and query logic only — it does
//! not infer transition boundaries from the IQ residual itself.
//!
//! ## no_std / no_alloc / zero-unsafe
//!
//! `WaveformSchedule<N>` uses a `[TransitionWindow; N]` array. No heap.
//! The crate-wide `#![forbid(unsafe_code)]` applies to all code here.
//!
//! ## References
//!
//! - de Beer (2026), §18.3 (Waveform Transition Artifacts)
//! - Rondeau et al. (2004), cognitive radio state machines (heuristic basis
//!   for suppression window design)

// ── Transition Kind ────────────────────────────────────────────────────────

/// Classification of a deliberate waveform transition event.
///
/// Used by the heuristics bank to distinguish deliberate transitions
/// (which should suppress grammar escalation) from structural interference
/// (which should not).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionKind {
    /// Frequency-hopping spread-spectrum (FHSS) hop event.
    /// Produces abrupt residual slew then fast recovery.
    FrequencyHop,

    /// Deliberate modulation-format change (e.g., BPSK→QPSK handshake).
    /// Produces transient residual peak during re-training period.
    ModulationChange,

    /// Burst-mode transmission onset (preamble + sync acquisition).
    BurstStart,

    /// Burst-mode transmission termination (demodulator idle flush).
    BurstEnd,

    /// Deliberate transmitter power-level change (power ramp).
    /// Produces monotone drift consistent with PA thermal motif.
    PowerLevelChange,

    /// Pre-planned time-slot boundary from a known TDMA/FDMA schedule.
    ScheduledSlotBoundary,

    /// Transition of unspecified or deployment-specific kind.
    Unknown,
}

impl TransitionKind {
    /// Human-readable label for SigMF `dsfb:transition_kind` field.
    pub const fn label(self) -> &'static str {
        match self {
            TransitionKind::FrequencyHop         => "FrequencyHop",
            TransitionKind::ModulationChange     => "ModulationChange",
            TransitionKind::BurstStart           => "BurstStart",
            TransitionKind::BurstEnd             => "BurstEnd",
            TransitionKind::PowerLevelChange     => "PowerLevelChange",
            TransitionKind::ScheduledSlotBoundary => "ScheduledSlotBoundary",
            TransitionKind::Unknown              => "Unknown",
        }
    }

    /// Whether this transition kind requires post-transition suppression margin.
    ///
    /// Frequency hops and modulation changes require extra margin because
    /// the receiver's equalizer/PLL needs time to re-acquire lock. Burst
    /// boundaries and power changes settle faster.
    pub const fn requires_margin(self) -> bool {
        matches!(self, TransitionKind::FrequencyHop | TransitionKind::ModulationChange)
    }
}

// ── Transition Window ──────────────────────────────────────────────────────

/// A single waveform transition window.
///
/// Suppresses grammar escalation to `Violation` during
/// `[start_k, end_k + suppression_margin)` observation indices.
///
/// The margin absorbs post-transition residual ringing; set to zero
/// for transitions with fast recovery (e.g., scheduled slot boundaries).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransitionWindow {
    /// Observation index at which the transition begins (inclusive).
    pub start_k: u32,
    /// Observation index at which the nominal waveform is expected to
    /// resume (inclusive). Suppression continues through `end_k + margin`.
    pub end_k: u32,
    /// Additional post-transition suppression window in observations.
    /// For `FrequencyHop` / `ModulationChange`: typically 2–10 samples
    /// (depending on receiver lock-time). For others: 0.
    pub suppression_margin: u32,
    /// Semantic classification of this transition.
    pub kind: TransitionKind,
}

impl TransitionWindow {
    /// First observation index where suppression is active (= `start_k`).
    #[inline]
    pub const fn suppression_start(&self) -> u32 {
        self.start_k
    }

    /// Last observation index where suppression is active (inclusive).
    #[inline]
    pub const fn suppression_end(&self) -> u32 {
        self.end_k.saturating_add(self.suppression_margin)
    }

    /// Returns `true` if observation `k` falls within this suppression window.
    #[inline]
    pub fn is_active(&self, k: u32) -> bool {
        k >= self.suppression_start() && k <= self.suppression_end()
    }

    /// Duration in observations (end_k − start_k + 1), excluding margin.
    #[inline]
    pub const fn duration_k(&self) -> u32 {
        self.end_k.saturating_sub(self.start_k) + 1
    }
}

// ── Waveform Schedule ──────────────────────────────────────────────────────

/// Fixed-capacity waveform transition schedule.
///
/// `N` is the maximum number of transition windows that can be registered.
/// For typical TDMA/FHSS protocols with ≤ 64 hops per evaluation window,
/// `N = 64` is sufficient. For burst-rich environments, use `N = 128`.
pub struct WaveformSchedule<const N: usize> {
    windows: [TransitionWindow; N],
    count: usize,
}

impl<const N: usize> WaveformSchedule<N> {
    /// Create an empty schedule.
    pub const fn new() -> Self {
        Self {
            // Safety: TransitionWindow is Copy + has all-zero constructible fields.
            // We initialize with a sentinel value (kind=Unknown, all zeros).
            windows: [TransitionWindow {
                start_k: 0,
                end_k: 0,
                suppression_margin: 0,
                kind: TransitionKind::Unknown,
            }; N],
            count: 0,
        }
    }

    /// Register a new transition window.
    ///
    /// Returns `true` on success; `false` if the schedule is full
    /// (capacity `N`). Caller must handle the full-schedule case
    /// (e.g., emit a grammar `Boundary` event noting schedule overflow).
    pub fn add(&mut self, window: TransitionWindow) -> bool {
        if self.count >= N { return false; }
        self.windows[self.count] = window;
        self.count += 1;
        true
    }

    /// Clear all registered windows (e.g., at frame boundary).
    pub fn clear(&mut self) {
        self.count = 0;
    }

    /// Number of registered windows.
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// `true` if no windows are registered.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns `true` if observation `k` falls within any suppression window.
    ///
    /// O(N) linear scan. For typical `N ≤ 128` this is always sub-microsecond.
    pub fn is_suppressed(&self, k: u32) -> bool {
        self.windows[..self.count].iter().any(|w| w.is_active(k))
    }

    /// Returns the first active `TransitionWindow` containing `k`, if any.
    pub fn active_window(&self, k: u32) -> Option<&TransitionWindow> {
        self.windows[..self.count].iter().find(|w| w.is_active(k))
    }

    /// Count of windows whose suppression interval overlaps observation `k`.
    ///
    /// Values > 1 indicate overlapping transitions (e.g., simultaneous
    /// frequency hop and power ramp), which warrants extended suppression.
    pub fn overlap_count(&self, k: u32) -> usize {
        self.windows[..self.count].iter().filter(|w| w.is_active(k)).count()
    }

    /// Returns `true` if the schedule is at capacity.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.count >= N
    }

    /// Fraction of capacity used (0.0–1.0).
    #[inline]
    pub fn capacity_fraction(&self) -> f32 {
        self.count as f32 / N as f32
    }
}

// ── Grammar Integration Hook ───────────────────────────────────────────────

/// Suppression decision returned to the grammar/policy layer.
///
/// The grammar layer calls `suppress_escalation()` at each observation.
/// If the result is `Suppressed`, it must not escalate to `Violation`
/// regardless of the DSA score or structural episode state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuppressionDecision {
    /// Normal operation — grammar escalation is permitted.
    Active,
    /// Suppressed — grammar must not escalate to `Violation`.
    /// Contains the `TransitionKind` of the suppressing window for diagnostics.
    Suppressed(TransitionKind),
}

/// Query whether grammar escalation should be suppressed at observation `k`.
///
/// Returns `Suppressed(kind)` if `k` falls within any registered transition
/// window; otherwise returns `Active`.
///
/// ## Usage in the Grammar Layer
///
/// ```rust,ignore
/// use dsfb_rf::waveform_context::{suppress_escalation, SuppressionDecision};
///
/// let decision = suppress_escalation(k, &schedule);
/// match decision {
///     SuppressionDecision::Active           => { /* normal grammar logic */ }
///     SuppressionDecision::Suppressed(kind) => {
///         // Downgrade Violation → Boundary, log suppression reason
///     }
/// }
/// ```
pub fn suppress_escalation<const N: usize>(
    k: u32,
    schedule: &WaveformSchedule<N>,
) -> SuppressionDecision {
    match schedule.active_window(k) {
        None      => SuppressionDecision::Active,
        Some(win) => SuppressionDecision::Suppressed(win.kind),
    }
}

// ── Builder helpers ────────────────────────────────────────────────────────

/// Convenience constructor for a frequency-hop transition window.
///
/// `margin` defaults to 5 observations (typical PLL re-lock for FHSS at
/// moderate hop rates; adjust for platform-specific lock time).
#[inline]
pub fn freq_hop_window(start_k: u32, end_k: u32, margin: u32) -> TransitionWindow {
    TransitionWindow {
        start_k,
        end_k,
        suppression_margin: margin,
        kind: TransitionKind::FrequencyHop,
    }
}

/// Convenience constructor for a burst-start transition window.
#[inline]
pub fn burst_start_window(start_k: u32, preamble_len: u32) -> TransitionWindow {
    TransitionWindow {
        start_k,
        end_k: start_k + preamble_len,
        suppression_margin: 0,
        kind: TransitionKind::BurstStart,
    }
}

/// Convenience constructor for a power-level change window.
#[inline]
pub fn power_change_window(start_k: u32, ramp_duration_k: u32) -> TransitionWindow {
    TransitionWindow {
        start_k,
        end_k: start_k + ramp_duration_k,
        suppression_margin: 2,
        kind: TransitionKind::PowerLevelChange,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_schedule_is_empty() {
        let s = WaveformSchedule::<8>::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn add_and_query_window() {
        let mut s = WaveformSchedule::<8>::new();
        let win = freq_hop_window(100, 105, 3);
        assert!(s.add(win));
        // Suppressed at k=100
        assert_eq!(s.is_suppressed(100), true);
        // Suppressed at k=108 (105 + margin 3 = 108)
        assert_eq!(s.is_suppressed(108), true);
        // Active before window
        assert_eq!(s.is_suppressed(99), false);
        // Active after window + margin
        assert_eq!(s.is_suppressed(109), false);
    }

    #[test]
    fn full_schedule_returns_false_on_add() {
        let mut s = WaveformSchedule::<2>::new();
        let w = freq_hop_window(0, 5, 0);
        assert!(s.add(w));
        assert!(s.add(w));
        assert!(!s.add(w), "full schedule must reject add");
        assert!(s.is_full());
    }

    #[test]
    fn clear_resets_schedule() {
        let mut s = WaveformSchedule::<4>::new();
        s.add(freq_hop_window(10, 20, 2));
        s.add(freq_hop_window(50, 60, 2));
        assert_eq!(s.len(), 2);
        s.clear();
        assert!(s.is_empty());
        assert!(!s.is_suppressed(15), "cleared schedule must not suppress");
    }

    #[test]
    fn suppress_escalation_returns_active_when_clear() {
        let s = WaveformSchedule::<8>::new();
        assert_eq!(suppress_escalation(42, &s), SuppressionDecision::Active);
    }

    #[test]
    fn suppress_escalation_returns_suppressed_in_window() {
        let mut s = WaveformSchedule::<8>::new();
        s.add(burst_start_window(200, 10));
        let dec = suppress_escalation(205, &s);
        assert_eq!(dec, SuppressionDecision::Suppressed(TransitionKind::BurstStart));
    }

    #[test]
    fn overlap_count_detects_simultaneous_transitions() {
        let mut s = WaveformSchedule::<8>::new();
        s.add(freq_hop_window(100, 110, 3));
        s.add(power_change_window(105, 8));
        // k=107 is in both windows
        assert_eq!(s.overlap_count(107), 2, "should detect 2 overlapping windows");
        assert_eq!(s.overlap_count(99),  0, "before all windows");
        assert_eq!(s.overlap_count(120), 0, "after all windows");
    }

    #[test]
    fn transition_kind_labels() {
        assert_eq!(TransitionKind::FrequencyHop.label(), "FrequencyHop");
        assert_eq!(TransitionKind::ModulationChange.label(), "ModulationChange");
        assert_eq!(TransitionKind::BurstStart.label(), "BurstStart");
        assert_eq!(TransitionKind::ScheduledSlotBoundary.label(), "ScheduledSlotBoundary");
        assert_eq!(TransitionKind::Unknown.label(), "Unknown");
    }

    #[test]
    fn requires_margin_correct() {
        assert!(TransitionKind::FrequencyHop.requires_margin());
        assert!(TransitionKind::ModulationChange.requires_margin());
        assert!(!TransitionKind::BurstStart.requires_margin());
        assert!(!TransitionKind::PowerLevelChange.requires_margin());
        assert!(!TransitionKind::ScheduledSlotBoundary.requires_margin());
    }

    #[test]
    fn window_duration_k_correct() {
        let w = TransitionWindow {
            start_k: 100, end_k: 110, suppression_margin: 0,
            kind: TransitionKind::FrequencyHop,
        };
        assert_eq!(w.duration_k(), 11); // 110 - 100 + 1
        assert_eq!(w.suppression_end(), 110);
    }

    #[test]
    fn window_margin_extends_suppression() {
        let w = freq_hop_window(100, 110, 5);
        assert!( w.is_active(115), "margin extends to 115");
        assert!(!w.is_active(116), "116 is past margin");
    }

    #[test]
    fn capacity_fraction_reports_correctly() {
        let mut s = WaveformSchedule::<4>::new();
        assert!((s.capacity_fraction() - 0.0).abs() < 1e-5);
        s.add(freq_hop_window(0, 5, 0));
        assert!((s.capacity_fraction() - 0.25).abs() < 1e-5);
        s.add(freq_hop_window(10, 15, 0));
        assert!((s.capacity_fraction() - 0.50).abs() < 1e-5);
    }

    #[test]
    fn multiple_windows_distinct_ranges_no_cross_suppression() {
        let mut s = WaveformSchedule::<8>::new();
        s.add(freq_hop_window(10, 20, 0));
        s.add(freq_hop_window(100, 110, 0));
        assert!( s.is_suppressed(15));
        assert!(!s.is_suppressed(50), "gap between windows is active");
        assert!( s.is_suppressed(105));
    }
}
