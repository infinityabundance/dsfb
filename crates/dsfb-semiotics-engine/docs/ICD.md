# DSFB Semiotics Engine Interface Control Document (ICD)

## 1. Purpose

This document defines the software interface boundary for the `dsfb-semiotics-engine` deployment surface, with particular emphasis on the C ABI / FFI boundary intended for integration into larger systems.

Transition-critical companion references:

- [`REAL_TIME_CONTRACT.md`](REAL_TIME_CONTRACT.md)
- [`TIMING_DETERMINISM_REPORT.md`](TIMING_DETERMINISM_REPORT.md)
- [`generated/real_time_contract_summary.json`](generated/real_time_contract_summary.json)

This ICD is written for systems engineers, flight-software engineers, and integration teams who require:
- deterministic interface behavior,
- bounded online state behavior,
- explicit error semantics,
- and auditable ownership / lifecycle rules.

This document does **not** claim certification, field validation, or formal platform approval. It specifies the current interface contract of the software artifact.

---

## 2. Interface Boundary Summary

The semiotics engine is architecturally layered:

1. residual extraction
2. sign construction
3. syntax generation
4. grammar evaluation
5. semantic retrieval

The deployment boundary exposed through the FFI layer is intentionally narrower than the internal layered architecture. It provides a stable external control surface while preserving internal implementation flexibility.

**Boundary principle:** external callers interact with an opaque engine handle and receive typed, reproducible outputs; internal representation details are not part of the ABI contract.

---

## 3. Integration Model

### 3.1 External inputs
The engine accepts time-ordered residual-like or measurement-derived inputs through a stepwise push interface.

### 3.2 External outputs
The engine exposes, at minimum:
- syntax status / label
- grammar reason / code
- semantic disposition / code
- trust scalar
- error status

### 3.3 Ownership model
- engine state is owned by the engine handle
- callers own their input buffers
- returned strings, if any, are display-oriented only and must not be used as machine contracts
- machine-readable integration should use numeric codes and `repr(C)`-safe output structures

---

## 4. Memory Model

### 4.1 Online bounded state
The online engine path uses bounded history rather than unbounded trajectory growth. This is critical for long-duration or embedded-style operation.

### 4.2 Mode-dependent memory footprint
This software supports multiple numeric/runtime modes in the crate architecture, including `f32`-oriented operation where enabled. Memory footprint depends on:
- numeric mode (`f32` vs `f64`)
- configured online history buffer capacity
- channel dimensionality
- enabled optional features

### 4.3 Memory footprint table
Current documented bounded-live profiles from the generated contract summary:

| Build Mode | Buffer Capacity | Channels | Approx. Online State Bytes | Notes |
|---|---:|---:|---:|---|
| `f64` | 32 | 1 | 3200 | bounded live handle + ring slots + retained value storage |
| `f64` | 64 | 1 | 4992 | default single-channel profile |
| `f64` | 64 | 3 | 6064 | default 3-axis profile |
| `f64` | 128 | 3 | 10672 | enlarged bounded profile |

Interpretation:

- these values cover the bounded live path only
- bank registry and retrieval index remain initialization-time assets and are excluded from this
  growth budget
- the authoritative machine-readable source is
  [`generated/real_time_contract_summary.json`](generated/real_time_contract_summary.json)

---

## 5. State Determinism

The engine is designed as a deterministic mapping under fixed:
- inputs,
- preprocessing rules,
- admissibility envelopes,
- and heuristics bank contents.

The paper explicitly ties the structural semiotics engine to deterministic interpretability and certifiable early-warning inference under fixed conditions. 

### 5.1 Error handling contract
The current FFI and bounded-live error policy is numeric-code-first and string-second:

| Error Code | Meaning | Engine State After Error | Recoverable? | Caller Action |
|---|---|---|---|---|
| invalid handle | opaque handle was null or stale | unchanged / invalid | no | recreate handle |
| invalid input | non-finite time, wrong channel width, or malformed batch | unchanged | yes | correct input and retry |
| bank/config init failure | bank or envelope setup rejected | engine not initialized | no | fix configuration |
| replay end-of-stream | replay consumed all rows | unchanged | yes | stop or reset |

### 5.2 Determinism rule
For any error that returns to the caller:
- document whether the engine state is unchanged,
- partially advanced,
- or invalidated.

This matters more than the code number itself.

---

## 6. Timing Behavior

### 6.1 Timing requirement
The interface shall be auditable with respect to execution time under documented build conditions.

### 6.2 Measured timing
Observed host-side measurements from [`TIMING_DETERMINISM_REPORT.md`](TIMING_DETERMINISM_REPORT.md):

| Target | Build Mode | Scenario | Median | p95 | p99 | Max | Notes |
|---|---|---|---:|---:|---:|---|
| Host x86_64 | release | scalar online step | 616728 ns | 763010 ns | 981176 ns | 992276 ns | observed only |
| Host x86_64 | release | batch online step | 1873025 ns | 1908592 ns | 1951953 ns | 2117250 ns | observed only |
| Host x86_64 | release | grammar admissible | 1373 ns | 1403 ns | 1422 ns | 1433 ns | observed only |
| Host x86_64 | release | grammar violation-like | 1473 ns | 1522 ns | 2314 ns | 3046 ns | observed only |
| Host x86_64 | release | semantic retrieval builtin bank | 38762 ns | 41457 ns | 62627 ns | 112379 ns | observed only |
| Host x86_64 | release | semantic retrieval enlarged bank | 187509 ns | 191617 ns | 197086 ns | 201735 ns | observed only |

Observed-vs-certified rule:

- the ICD may cite observed timing
- the ICD does not claim certified WCET
- target-hardware timing must be remeasured on the target

---

## 7. Data Structures

Document all externally stable structures, especially:
- opaque handle type
- `repr(C)` output structs
- numeric status code enums
- trust scalar range
- semantic disposition enums
- batch-ingestion ordering semantics

---

## 8. Build and Linking

Document:
- static library build
- shared library build
- header generation / inclusion
- compiler assumptions
- C++ wrapper usage if provided
- reference batch-ingestion examples for multi-axis integration

---

## 9. Out-of-Scope / Non-Claims

This ICD does not claim:
- platform certification,
- field qualification,
- formal hard-real-time certification,
- or closed-loop control validation.

It documents the current software interface boundary only.
