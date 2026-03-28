# IEEE 1547-2018 Mapping

Status: Partial

This mapping is advisory only. It does not implement interconnection control logic.

Relevant crate components:
- `src/types.rs::GrammarState`
- `src/detection.rs::evaluate_grammar_state`
- `src/compliance.rs` operator overlay

Traceability matrix:

| IEEE 1547-style concern | Crate component | Evidence | Status | Notes |
|---|---|---|---|---|
| Structural deviation escalation | `GrammarState::Violation` | Explicit `Violation` state in the grammar | Partial | Can be mapped to a cease-to-energize advisory in a host system |
| Operator advisory | operator overlay output | Red state with advisory text | Partial | Output is informational only |
| Control action | Outside crate scope | None | Not supported | The crate does not trip, disconnect, or command plant behavior |
