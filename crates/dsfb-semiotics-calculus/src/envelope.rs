//! # Admissibility Envelope (`τ_E`)
//!
//! Encodes the compact set `E ⊆ V` with `0 ∈ int(E)` and the uniform inner-ball condition
//! `B(0, ρ_min) ⊆ E ⊆ B(0, ρ_max)` from Section 2.2 of the DSSC paper.
//!
//! An [`AdmissibilityEnvelope`] classifies any residual magnitude as `Interior`, `Boundary`,
//! or `Exterior`, which drives the grammar FSM. The `δ`-band boundary layer (Definition 4.1)
//! is parameterized here as `delta`.

/// Classification of a residual magnitude relative to an admissibility envelope.
///
/// Maps directly to the three grammar states: `Interior` → `Adm`, `Boundary` → `Bdy`,
/// `Exterior` → `Vio`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeRegion {
    /// `r(k) ∈ int(E)`: strictly inside, beyond the δ-band. Grammar: `Adm`.
    Interior,
    /// `r(k) ∈ ∂_δE`: within the δ-band of the boundary. Grammar: `Bdy`.
    Boundary,
    /// `r(k) ∉ E`: outside the envelope. Grammar: `Vio`.
    Exterior,
}

/// A scalar admissibility envelope parameterized by `(ρ_min, ρ_max, δ)`.
///
/// The envelope is the ball `E = B(0, ρ_max)`. The boundary layer is the annulus
/// `[ρ_max − δ, ρ_max + δ]`. The constraint `δ ≤ ρ_min / 4` is checked at construction
/// (Definition 4.1 of the DSSC paper).
///
/// For multi-dimensional residuals, project each channel through its own envelope and
/// combine using [`EnvelopeFamily`].
#[derive(Debug, Clone, Copy)]
pub struct AdmissibilityEnvelope {
    /// Inner radius `ρ_min > 0`. Guarantees `B(0, ρ_min) ⊆ E`.
    pub rho_min: f64,
    /// Outer radius `ρ_max ≥ ρ_min`. Defines the envelope boundary.
    pub rho_max: f64,
    /// Boundary-layer half-width `δ ∈ (0, ρ_min/4]` (calibration parameter).
    pub delta: f64,
}

impl AdmissibilityEnvelope {
    /// Construct an envelope, enforcing `0 < ρ_min ≤ ρ_max` and `δ ≤ ρ_min/4`.
    ///
    /// # Panics
    /// Panics if the invariants are violated — this is a deployment-time calibration
    /// error, not a runtime error.
    pub fn new(rho_min: f64, rho_max: f64, delta: f64) -> Self {
        assert!(rho_min > 0.0, "ρ_min must be positive");
        assert!(rho_max >= rho_min, "ρ_max must be ≥ ρ_min");
        assert!(delta > 0.0 && delta <= rho_min / 4.0,
            "δ must satisfy 0 < δ ≤ ρ_min/4 (Definition 4.1)");
        Self { rho_min, rho_max, delta }
    }

    /// Classify a residual magnitude relative to this envelope.
    #[inline]
    pub fn classify(&self, magnitude: f64) -> EnvelopeRegion {
        if magnitude > self.rho_max + self.delta {
            EnvelopeRegion::Exterior
        } else if magnitude >= self.rho_max - self.delta {
            EnvelopeRegion::Boundary
        } else {
            EnvelopeRegion::Interior
        }
    }
}

/// A finite indexed family of admissibility envelopes `{E_λ}_{λ ∈ Λ}`.
///
/// The family satisfies regime monotonicity: `λ₁ ≤ λ₂ ⟹ E_λ₁ ⊆ E_λ₂`.
/// Envelopes are stored in increasing order of `ρ_max`.
#[derive(Debug, Clone)]
pub struct EnvelopeFamily {
    envelopes: Vec<AdmissibilityEnvelope>,
}

impl EnvelopeFamily {
    /// Construct a family from a sorted list of envelopes.
    ///
    /// # Panics
    /// Panics if envelopes are not monotonically increasing in `ρ_max`.
    pub fn new(envelopes: Vec<AdmissibilityEnvelope>) -> Self {
        for w in envelopes.windows(2) {
            assert!(w[1].rho_max >= w[0].rho_max,
                "EnvelopeFamily must be monotonically ordered by ρ_max");
        }
        Self { envelopes }
    }

    /// Return the active envelope at regime index `lambda`.
    pub fn get(&self, lambda: usize) -> Option<&AdmissibilityEnvelope> {
        self.envelopes.get(lambda)
    }

    /// Number of regimes in the family.
    pub fn len(&self) -> usize { self.envelopes.len() }

    /// `true` if the family is empty (degenerate; should not occur in deployment).
    pub fn is_empty(&self) -> bool { self.envelopes.is_empty() }
}
