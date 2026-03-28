# IEC 61131-3 Mapping

Status: Partial

This file documents the deterministic Structured Text wrapper added for engineering portability. It is not a deployed PLC control package.

Relevant crate components:
- `wrappers/plc/structured_text.st`
- `src/detection.rs::evaluate_grammar_state`
- `src/types.rs::GrammarState`

Traceability matrix:

| PLC-oriented concern | Crate component | Evidence | Status | Notes |
|---|---|---|---|---|
| Deterministic state-machine translation | `wrappers/plc/structured_text.st` | Structured Text translation of the current grammar-state conditions | Partial | Translation is advisory and expects counters from upstream logic |
| State coloring | `wrappers/plc/structured_text.st`, operator overlay | Green / Yellow / Red mapping | Partial | Included as operator support only |
| Control independence | External host system | No actuation logic in wrapper | Partial | Wrapper does not implement protection commands |
