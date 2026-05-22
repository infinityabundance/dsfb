//! Concrete parameter values backing a single detector spec.
//!
//! `DetectorParamSet` carries the parameter-grid coordinates a
//! template uses to expand into one specific detector. Every
//! field is integer-typed so the parameter hash is host-
//! independent.

/// One concrete parameterisation point in the algebra grid.
///
/// The fields correspond to the panel-locked grammar's tail
/// coordinates (`{TRANSFORM}__W{WINDOW}__{STATISTIC}__{COMPARATOR}__P{PERSISTENCE}`).
/// The verifier rejects `window_cells = 0` and
/// `persistence_windows = 0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DetectorParamSet {
    /// Window size in cells (the `W{N}` token in the canonical
    /// name).
    pub window_cells: u32,
    /// Persistence threshold in windows (the `P{N}` token).
    pub persistence_windows: u32,
    /// Threshold value in Q16.16 raw representation. The
    /// concrete units depend on the family + statistic; the
    /// parameter hash treats this as opaque bytes.
    pub threshold_q16_raw: i32,
    /// Optional secondary parameter slot, also Q16.16 raw. Some
    /// families need a second scalar (e.g. EWMA's alpha,
    /// CUSUM's k). Zero when unused.
    pub secondary_q16_raw: i32,
}

impl DetectorParamSet {
    /// Construct a parameter set. Callers should run through the
    /// verifier for production use.
    #[must_use]
    pub const fn new(
        window_cells: u32,
        persistence_windows: u32,
        threshold_q16_raw: i32,
        secondary_q16_raw: i32,
    ) -> Self {
        Self {
            window_cells,
            persistence_windows,
            threshold_q16_raw,
            secondary_q16_raw,
        }
    }
}
