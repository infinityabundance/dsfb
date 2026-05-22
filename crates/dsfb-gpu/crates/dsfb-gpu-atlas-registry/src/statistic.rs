//! Per-window statistic — the algebra's fourth coordinate.
//!
//! Reduces a window of transformed cells to a scalar. Pairs with
//! the family + transform + window to determine the detector's
//! firing decision before the comparator + gate are applied.

/// Per-window scalar statistic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Statistic {
    /// Arithmetic mean.
    Mean,
    /// Median.
    Median,
    /// Median absolute deviation.
    Mad,
    /// Trimmed mean (e.g. 10% trim).
    TrimmedMean,
    /// Winsorised mean.
    WinsorisedMean,
    /// Variance.
    Variance,
    /// Standard deviation.
    StdDev,
    /// Sum.
    Sum,
    /// Maximum.
    Max,
    /// Minimum.
    Min,
    /// Range (max - min).
    Range,
    /// Quantile (level pinned by parameter set).
    Quantile,
    /// Inter-quartile range.
    Iqr,
    /// Rank statistic.
    Rank,
    /// Signed-rank statistic.
    SignedRank,
    /// Run length.
    RunLength,
    /// Auto-correlation at a fixed lag (lag pinned by parameter set).
    Autocorrelation,
    /// Spectral-band energy.
    BandEnergy,
}

impl Statistic {
    /// Canonical wire name, uppercase snake-case.
    #[must_use]
    pub const fn canonical_wire_name(self) -> &'static str {
        match self {
            Self::Mean => "MEAN",
            Self::Median => "MEDIAN",
            Self::Mad => "MAD",
            Self::TrimmedMean => "TRIMMED_MEAN",
            Self::WinsorisedMean => "WINSORISED_MEAN",
            Self::Variance => "VARIANCE",
            Self::StdDev => "STD_DEV",
            Self::Sum => "SUM",
            Self::Max => "MAX",
            Self::Min => "MIN",
            Self::Range => "RANGE",
            Self::Quantile => "QUANTILE",
            Self::Iqr => "IQR",
            Self::Rank => "RANK",
            Self::SignedRank => "SIGNED_RANK",
            Self::RunLength => "RUN_LENGTH",
            Self::Autocorrelation => "AUTOCORRELATION",
            Self::BandEnergy => "BAND_ENERGY",
        }
    }
}
