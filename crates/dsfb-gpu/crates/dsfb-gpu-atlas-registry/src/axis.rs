//! Axis binding — which fusion axis a detector feeds.
//!
//! Mirrors the corpus crate's
//! [`dsfb_gpu_atlas_corpus::types::AxisBindingSet`] bit positions
//! at S1.1. A detector spec MUST declare at least one axis
//! binding; the verifier rejects an empty binding.
//!
//! The 9 axes match the v1 9-axis fusion layout:
//!
//! 1. residual magnitude (axis 1)
//! 2. drift persistence (axis 2)
//! 3. slew shock (axis 3)
//! 4. temporal locality (axis 4)
//! 5. entity locality (axis 5)
//! 6. causal / topological adjacency (axis 6)
//! 7. detector motif consensus (axis 7)
//! 8. bank semantic admissibility (axis 8)
//! 9. confuser suppression (axis 9)
//!
//! Section S Phase 1+ expands the fusion layer to an 8-plane ×
//! 8-axis hierarchy; the axis-to-plane mapping lives in the
//! corpus crate's [`dsfb_gpu_atlas_corpus::fusion`].

/// Bitset of fusion-axis bindings. Mirrors
/// [`dsfb_gpu_atlas_corpus::types::AxisBindingSet`] byte-for-byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AxisBinding(pub u16);

impl AxisBinding {
    /// Axis 1 — residual magnitude.
    pub const AXIS_1_RESIDUAL_MAGNITUDE: u16 = 1 << 0;
    /// Axis 2 — drift persistence.
    pub const AXIS_2_DRIFT_PERSISTENCE: u16 = 1 << 1;
    /// Axis 3 — slew shock.
    pub const AXIS_3_SLEW_SHOCK: u16 = 1 << 2;
    /// Axis 4 — temporal locality.
    pub const AXIS_4_TEMPORAL_LOCALITY: u16 = 1 << 3;
    /// Axis 5 — entity locality.
    pub const AXIS_5_ENTITY_LOCALITY: u16 = 1 << 4;
    /// Axis 6 — causal / topological adjacency.
    pub const AXIS_6_CAUSAL_ADJACENCY: u16 = 1 << 5;
    /// Axis 7 — detector motif consensus.
    pub const AXIS_7_DETECTOR_CONSENSUS: u16 = 1 << 6;
    /// Axis 8 — bank semantic admissibility.
    pub const AXIS_8_BANK_ADMISSIBILITY: u16 = 1 << 7;
    /// Axis 9 — confuser suppression.
    pub const AXIS_9_CONFUSER_SUPPRESSION: u16 = 1 << 8;

    /// True if no axis bits are set. The verifier rejects this.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Construct an axis binding with one bit set.
    #[must_use]
    pub const fn single(axis_bit: u16) -> Self {
        Self(axis_bit)
    }
}
