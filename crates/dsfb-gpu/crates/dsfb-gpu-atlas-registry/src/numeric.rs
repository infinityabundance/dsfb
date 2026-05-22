//! Numeric mode — the fixed-point representation a detector uses.
//!
//! The default for Atlas audit mode is Q16.16 (matches the v0
//! `dsfb-gpu-debug` baseline). The verifier enforces this for
//! audit-mode specs; throughput-mode specs may use richer
//! representations once registered.

/// Numeric mode for a detector's arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumericMode {
    /// Q16.16 fixed-point (16 integer bits + 16 fractional bits)
    /// — the canonical audit-mode default. Mirrors
    /// dsfb-gpu-debug-core's `Q16` representation.
    Q16_16,
    /// Q32.32 fixed-point — reserved for future expansion.
    Q32_32,
    /// Q8.24 — high-precision fractional, low-range integer.
    Q8_24,
}

impl NumericMode {
    /// The canonical default for audit-mode specs.
    pub const AUDIT_DEFAULT: NumericMode = NumericMode::Q16_16;

    /// Canonical wire name, uppercase snake-case.
    #[must_use]
    pub const fn canonical_wire_name(self) -> &'static str {
        match self {
            Self::Q16_16 => "Q16_16",
            Self::Q32_32 => "Q32_32",
            Self::Q8_24 => "Q8_24",
        }
    }
}
