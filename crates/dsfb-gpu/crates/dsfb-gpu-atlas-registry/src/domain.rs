//! Domain-tag bitset — which input domain a detector is
//! applicable to.
//!
//! Mirrors the corpus crate's [`dsfb_gpu_atlas_corpus::types::DomainTagSet`]
//! at the bit-position level so a corpus literature primitive's
//! domain tags translate one-for-one into the registry. **Bit
//! positions are panel-locked** — changing a bit value would
//! break the `DomainTagSet` byte equivalence with the corpus.
//!
//! At S1.1 the activation planner is not yet present; domain
//! tags are pure schema metadata. The planner (Section S1.3+)
//! consumes them to gate detectors against the input manifest.

/// Single domain tag. Mirrors
/// [`dsfb_gpu_atlas_corpus::types::DomainTagSet`]'s bit positions
/// (0..12) but is exposed as a value-typed enum for ergonomics
/// at the algebra layer. Use [`DomainTag::bit`] to convert to a
/// `u16` for bitset operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DomainTag {
    /// Debug / trace catalogs (the dsfb-gpu-debug fixture domain).
    Debug,
    /// Distributed-system telemetry.
    Telemetry,
    /// Generic tabular data.
    Tabular,
    /// Univariate or multivariate time-series.
    TimeSeries,
    /// Graph / topology data.
    Graph,
    /// Industrial process monitoring / FDD.
    Industrial,
    /// Categorical-only data.
    Categorical,
    /// Pure missingness mask.
    Missingness,
    /// Continuous event stream.
    EventStream,
    /// Medical / biosignal domain.
    Medical,
    /// RF / communications signals.
    RfComms,
    /// Chemometrics / lab data.
    Chemometrics,
    /// Database / data-quality constraints.
    Database,
}

impl DomainTag {
    /// Bit value for this tag. Matches
    /// [`dsfb_gpu_atlas_corpus::types::DomainTagSet`] bit
    /// positions byte-for-byte.
    #[must_use]
    pub const fn bit(self) -> u16 {
        match self {
            Self::Debug => 1 << 0,
            Self::Telemetry => 1 << 1,
            Self::Tabular => 1 << 2,
            Self::TimeSeries => 1 << 3,
            Self::Graph => 1 << 4,
            Self::Industrial => 1 << 5,
            Self::Categorical => 1 << 6,
            Self::Missingness => 1 << 7,
            Self::EventStream => 1 << 8,
            Self::Medical => 1 << 9,
            Self::RfComms => 1 << 10,
            Self::Chemometrics => 1 << 11,
            Self::Database => 1 << 12,
        }
    }

    /// Canonical wire name, uppercase snake-case.
    #[must_use]
    pub const fn canonical_wire_name(self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Telemetry => "TELEMETRY",
            Self::Tabular => "TABULAR",
            Self::TimeSeries => "TIME_SERIES",
            Self::Graph => "GRAPH",
            Self::Industrial => "INDUSTRIAL",
            Self::Categorical => "CATEGORICAL",
            Self::Missingness => "MISSINGNESS",
            Self::EventStream => "EVENT_STREAM",
            Self::Medical => "MEDICAL",
            Self::RfComms => "RF_COMMS",
            Self::Chemometrics => "CHEMOMETRICS",
            Self::Database => "DATABASE",
        }
    }
}

/// Bitset of [`DomainTag`] values. The wire format matches
/// [`dsfb_gpu_atlas_corpus::types::DomainTagSet`] byte-for-byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DomainTagSet(pub u16);

impl DomainTagSet {
    /// Empty set.
    pub const EMPTY: DomainTagSet = DomainTagSet(0);

    /// True if no bits are set. The verifier rejects an empty
    /// tag set on a `DetectorSpec`.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Add a tag to the set (returns a new set).
    #[must_use]
    pub const fn with(self, tag: DomainTag) -> Self {
        Self(self.0 | tag.bit())
    }

    /// True if the set contains the tag.
    #[must_use]
    pub const fn contains(self, tag: DomainTag) -> bool {
        (self.0 & tag.bit()) != 0
    }

    /// Round-trip: build from a raw `u16` and immediately extract
    /// it. Useful for the `domain_tagset_roundtrips_bits`
    /// acceptance test.
    #[must_use]
    pub const fn from_raw(bits: u16) -> Self {
        Self(bits)
    }

    /// Raw bits.
    #[must_use]
    pub const fn to_raw(self) -> u16 {
        self.0
    }
}
