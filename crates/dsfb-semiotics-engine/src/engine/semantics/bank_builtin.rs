//! Builtin typed heuristic-bank entries and registry surface.
//!
//! Responsibilities are split so the public bank-loading entrypoint stays reviewable while the
//! large static bank content lives in a dedicated content module.

mod entries;

use crate::engine::types::HeuristicBankEntry;

pub(crate) fn builtin_heuristic_bank_entries() -> Vec<HeuristicBankEntry> {
    entries::builtin_heuristic_bank_entries_impl()
}
