use creusot_std::prelude::*;

/// Mirrors `dsfb_chemical_engineering_core::SCALE`.
pub const SCALE: i64 = 1_000_000;

/// The three axis classes (bare enum — no derives needed for the safety proof).
pub enum CoordClass {
    Interior,
    Grazing,
    Outside,
}

/// Verbatim port of `dsfb_chemical_engineering_core::classify_axis`. The `i128` promotion is the
/// overflow-safety mechanism the core documents ("promoted to i128 so the product never overflows").
/// Creusot discharges the auto-generated no-panic / no-overflow VCs for ALL inputs satisfying the
/// documented preconditions (`lo < hi`; `band_scaled ∈ [0, SCALE)`) — the unbounded, all-inputs form
/// of the bounded cargo-fuzz result (85.6M random execs, 0 crashes).
#[requires(lo@ < hi@)]
#[requires(0i64@ <= band_scaled@ && band_scaled@ < SCALE@)]
pub fn classify_axis(v: i64, lo: i64, hi: i64, band_scaled: i64) -> CoordClass {
    let d2: i128 = 2i128 * (v as i128) - (hi as i128 + lo as i128);
    let width: i128 = hi as i128 - lo as i128;
    let ad2: i128 = if d2 < 0i128 { -d2 } else { d2 };
    if ad2 > width {
        CoordClass::Outside
    } else if ad2 * (SCALE as i128) >= width * (SCALE as i128 - band_scaled as i128) {
        CoordClass::Grazing
    } else {
        CoordClass::Interior
    }
}
