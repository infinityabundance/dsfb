//! DSFB State representation
//!
//! The DSFB state consists of three components:
//! - phi: position/phase
//! - omega: velocity/frequency (drift)
//! - alpha: acceleration/slew

/// State of the DSFB observer
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DsfbState {
    /// Position/phase
    pub phi: f64,
    /// Velocity/frequency (drift)
    pub omega: f64,
    /// Acceleration/slew
    pub alpha: f64,
}

impl DsfbState {
    /// Create a new DSFB state
    pub fn new(phi: f64, omega: f64, alpha: f64) -> Self {
        Self { phi, omega, alpha }
    }

    /// Create a zero state
    pub fn zero() -> Self {
        Self {
            phi: 0.0,
            omega: 0.0,
            alpha: 0.0,
        }
    }
}

impl Default for DsfbState {
    fn default() -> Self {
        Self::zero()
    }
}
