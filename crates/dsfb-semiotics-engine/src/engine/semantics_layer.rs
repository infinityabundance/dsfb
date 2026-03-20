//! Compatibility facade for the decomposed semantics subsystem.
//!
//! New code should prefer `crate::engine::semantics`, but this module keeps the historical
//! import path stable for the rest of the crate and for downstream examples.

pub use crate::engine::semantics::{retrieve_semantics, retrieve_semantics_with_registry};
