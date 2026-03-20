//! Compatibility facade for the decomposed semantics subsystem.
//!
//! New code should prefer `crate::engine::semantics`, but this module keeps the historical
//! import path stable for the rest of the crate and for downstream examples.

pub(crate) use crate::engine::semantics::builtin_heuristic_bank_entries;
pub use crate::engine::semantics::{retrieve_semantics, retrieve_semantics_with_registry};
