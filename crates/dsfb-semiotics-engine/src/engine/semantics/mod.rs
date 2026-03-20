//! Governed semantics subsystem.
//!
//! Responsibilities are split so retrieval logic, builtin-bank content, compatibility handling,
//! and scope-condition evaluation can be reviewed independently.

mod bank_builtin;
mod compatibility;
mod retrieval;
mod scope_eval;

pub(crate) use bank_builtin::builtin_heuristic_bank_entries;
pub use retrieval::{retrieve_semantics, retrieve_semantics_with_registry};
