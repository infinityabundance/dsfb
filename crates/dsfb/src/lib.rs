//! DSFB - Drift-Slew Fusion Bootstrap
//!
//! A trust-adaptive nonlinear state estimation algorithm for tracking
//! position (phi), velocity/drift (omega), and acceleration/slew (alpha)
//! across multiple measurement channels with adaptive trust weighting.

pub mod observer;
pub mod params;
pub mod sim;
pub mod state;
pub mod trust;

// Re-export main types
pub use observer::DsfbObserver;
pub use params::DsfbParams;
pub use state::DsfbState;
pub use trust::TrustStats;
