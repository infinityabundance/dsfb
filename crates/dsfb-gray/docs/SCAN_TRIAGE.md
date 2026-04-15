# Scan Triage

This document records how to read current scanner findings with empirical discipline.

The goal is not to suppress uncomfortable outputs. The goal is to separate:

- `true/useful`
- `expected/noisy`
- `context-needed`

## Triage Rules

### True / Useful

Use this bucket when the finding is concrete, source-visible, and actionable without special domain assumptions.

Examples:

- `SHORT-WRITE`
- `DROP-PANIC`
- `TASK-LEAK`
- `ZERO-COPY`
- `CARGO-VERS`

### Expected / Noisy

Use this bucket when the finding is structurally real but too coarse to carry strong semantic meaning by itself.

Examples:

- high raw `H-SERDE-01` hit counts in large serialization-heavy crates
- `ITER-UNB` on code that is practically bounded by trusted upstream inputs but not mechanically obvious
- broad global/shared-resource motifs in crates that intentionally centralize registry state

These findings should not be hidden. They should be presented as review prompts rather than high-confidence defects.

### Context-Needed

Use this bucket when the finding only becomes truly meaningful after local code, domain, or operational context is supplied.

Examples:

- command buffering without TTL in a safety-control context
- mixed clock sources in lease / quorum / deadline logic
- dynamic loading in a supply-chain or high-assurance review
- Power-of-Ten proxies in industrial-safety review

## Current Discipline

The scanner keeps all three categories visible:

- the raw evidence remains in the report
- the remediation text stays bounded
- the report does not pretend that every elevated finding is equally severe

This is how the project stays broad without pretending every finding has the same certainty or urgency.
