//! Operating regime classification for gas turbine engines.
//!
//! In C-MAPSS, the three operational settings encode altitude, Mach number,
//! and throttle resolver angle. In FD001, these are approximately constant
//! (single operating condition). In FD002/FD004, they vary across six regimes.
//!
//! DSFB admissibility envelopes are regime-conditioned: a residual that is
//! admissible at cruise may not be admissible at take-off.

/// Operating regime for envelope conditioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingRegime {
    /// FD001: single condition (sea level static equivalent).
    SeaLevelStatic,
    /// Climb phase.
    Climb,
    /// Cruise phase.
    Cruise,
    /// Descent phase.
    Descent,
    /// Take-off (maximum power).
    TakeOff,
    /// Multi-condition dataset: regime identified by cluster index.
    MultiCondition {
        /// Regime cluster ID (0-based).
        regime_id: u8,
    },
    /// Unknown or unclassified regime.
    Unknown,
}

impl OperatingRegime {
    /// Classifies the operating regime from C-MAPSS operational settings.
    ///
    /// For FD001 (single condition), always returns `SeaLevelStatic`.
    /// For FD002/FD004 (six conditions), clusters by nearest centroid.
    ///
    /// FD002 centroids (from dataset analysis):
    ///   0: alt≈0,  Mach≈0.00, TRA=100
    ///   1: alt≈10, Mach≈0.25, TRA=100
    ///   2: alt≈20, Mach≈0.70, TRA=100
    ///   3: alt≈25, Mach≈0.62, TRA=60
    ///   4: alt≈35, Mach≈0.84, TRA=100
    ///   5: alt≈42, Mach≈0.84, TRA=100
    #[must_use]
    pub fn from_cmapss_settings(op1: f64, op2: f64, op3: f64) -> Self {
        // FD001: settings are approximately constant
        if op1.abs() < 1.0 && op2.abs() < 0.05 && (op3 - 100.0).abs() < 5.0 {
            return Self::SeaLevelStatic;
        }

        // FD002/FD004: nearest-centroid classification
        const CENTROIDS: [(f64, f64, f64); 6] = [
            (0.0,  0.00, 100.0),
            (10.0, 0.25, 100.0),
            (20.0, 0.70, 100.0),
            (25.0, 0.62,  60.0),
            (35.0, 0.84, 100.0),
            (42.0, 0.84, 100.0),
        ];

        let mut best_id = 0u8;
        let mut best_dist = f64::MAX;
        let mut i = 0;
        while i < 6 {
            let (ca, cm, ct) = CENTROIDS[i];
            let d = (op1 - ca) * (op1 - ca) + (op2 - cm) * 1000.0 * (op2 - cm) * 1000.0 + (op3 - ct) * (op3 - ct);
            if d < best_dist {
                best_dist = d;
                best_id = i as u8;
            }
            i += 1;
        }

        Self::MultiCondition { regime_id: best_id }
    }

    /// Human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SeaLevelStatic => "Sea Level Static",
            Self::Climb => "Climb",
            Self::Cruise => "Cruise",
            Self::Descent => "Descent",
            Self::TakeOff => "Take-Off",
            Self::MultiCondition { .. } => "Multi-Condition",
            Self::Unknown => "Unknown",
        }
    }
}
