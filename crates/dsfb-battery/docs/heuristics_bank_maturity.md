# Heuristics Bank Maturity

The NASA-grounded heuristics bank in this crate is a structured library of trajectory motifs for the current `dsfb-battery` capacity-only workflow. It is grounded in signals the crate already computes:

- capacity
- residual
- drift
- slew
- persistence counters
- envelope relation
- grammar-state transitions

What the bank currently supports:

- typed, versioned entries with conservative maturity labels
- supporting instances and counter-examples from the currently available NASA PCoE FY08Q4 cells
- ambiguity-aware retrieval against the current DSFB evidence path
- engineer-facing inventory, evidence, and retrieval artifacts in isolated helper outputs

What it does not support:

- unique physical mechanism identification
- chemistry-general transfer claims
- a complete degradation taxonomy
- replacement of the current mono-cell production path

Current maturity limits:

- the bank is grounded only in the crate's current capacity-centric NASA PCoE workflow
- most seeded entries remain `candidate`
- the accelerating-fade / knee entry remains `illustrative` because the current default NASA runs in this crate do not emit that reason code
- resistance-coupled or multi-channel motifs are not claimed because the current production path does not expose those channels cleanly

Next-step growth path:

- add new entries only when a repeated, auditable trajectory motif is supported by current crate signals
- attach both supporting instances and near-miss or counter-example cases where available
- promote entries conservatively based on repeated evidence rather than by expanding the label set

This bank is additive and engineer-facing. It does not modify the existing mono-cell production figure path or the current Stage II production artifact contract.
