//! Window-size descriptor for the algebra's third coordinate.
//!
//! At S1.1 the window is represented as a `u32` cell count.
//! Canonical wire names render as `W{N}` (e.g. `W64`). The
//! Atlas does NOT require windows to be powers of two — the
//! generator may emit arbitrary positive integers, and the
//! verifier enforces only positivity.
//!
//! Why a single `u32` field instead of a `WindowKind` enum: at
//! S1.1 the algebra needs only the cell count for naming and
//! identity. Future Atlas commits may introduce window shapes
//! (rectangular vs exponential vs adaptive); when that lands it
//! will compose with `WindowSpec` rather than replacing it.

use core::fmt::{self, Display};

/// Concrete window-size descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowSpec {
    /// Window size in cells. MUST be ≥ 1; the verifier rejects 0.
    pub cells: u32,
}

impl WindowSpec {
    /// Construct a window of the given cell count. Panics on 0
    /// — callers should construct windows via the verifier path
    /// in production code; this constructor is for test
    /// fixtures.
    #[must_use]
    pub const fn new(cells: u32) -> Self {
        Self { cells }
    }

    /// Common canonical sizes used by the v1 detector ladder.
    /// Provided for convenience; the algebra is not restricted
    /// to these values.
    pub const W8: WindowSpec = WindowSpec { cells: 8 };
    /// 16-cell window.
    pub const W16: WindowSpec = WindowSpec { cells: 16 };
    /// 32-cell window.
    pub const W32: WindowSpec = WindowSpec { cells: 32 };
    /// 64-cell window (the canonical wide-mask reference size).
    pub const W64: WindowSpec = WindowSpec { cells: 64 };
    /// 128-cell window.
    pub const W128: WindowSpec = WindowSpec { cells: 128 };
}

impl Display for WindowSpec {
    /// Canonical wire format: `W{cells}` (e.g. `W64`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "W{}", self.cells)
    }
}
