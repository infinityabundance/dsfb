//! Full DSFB evaluation pipeline.
//!
//! This module is std-gated and alloc-using. It is not part of the
//! crate's embedded `no_std` / `no_alloc` core surface.
pub mod engine_eval;
pub mod metrics;
pub mod fleet;
pub mod sweep;
pub mod negative_control;
pub mod trace_chain;
pub mod discrimination;
pub mod regime_eval;
