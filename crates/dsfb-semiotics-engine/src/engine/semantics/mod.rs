//! Governed semantics subsystem.
//!
//! Responsibilities are split so retrieval logic, builtin-bank content, compatibility handling,
//! bank loading, bank validation, explanation assembly, and scope-condition evaluation can be
//! reviewed independently.

mod bank_builtin;
mod bank_loader;
mod bank_validation;
mod compatibility;
mod explanations;
mod retrieval;
mod scope_eval;
mod types;

pub(crate) use bank_loader::{
    ensure_supported_bank_schema, load_builtin_registry, load_external_registry_json,
};
pub(crate) use bank_validation::build_bank_validation_report;
pub(crate) use retrieval::{
    benchmark_retrieval_scaling, build_retrieval_index, retrieve_semantics_with_context,
    SemanticRetrievalContext, SemanticRetrievalIndex,
};
pub use retrieval::{retrieve_semantics, retrieve_semantics_with_registry};
