# DSFB Semiotics Engine Interface Control Document (ICD)

## 1. Purpose

This document defines the software interface boundary for the `dsfb-semiotics-engine` deployment surface, with particular emphasis on the C ABI / FFI boundary intended for integration into larger systems.

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
Populate this table with **measured values from the actual crate build**, not estimates:

| Build Mode | Buffer Capacity | Channels | Approx. Online State Bytes | Notes |
|---|---:|---:|---:|---|
| `f32` | 32 | 1 | TBD | Measure from compiled build |
| `f32` | 64 | 3 | TBD | Measure from compiled build |
| `f64` | 32 | 1 | TBD | Measure from compiled build |
| `f64` | 64 | 3 | TBD | Measure from compiled build |

**Rule:** this table must be generated from a measurement script or build-time report, not entered manually.

---

## 5. State Determinism

The engine is designed as a deterministic mapping under fixed:
- inputs,
- preprocessing rules,
- admissibility envelopes,
- and heuristics bank contents.

The paper explicitly ties the structural semiotics engine to deterministic interpretability and certifiable early-warning inference under fixed conditions. 

### 5.1 Error handling contract
Populate this table with actual codes from the FFI/API:

| Error Code | Meaning | Engine State After Error | Recoverable? | Caller Action |
|---|---|---|---|---|
| TBD | invalid handle | unchanged / invalid | no | recreate handle |
| TBD | malformed input | unchanged | yes | correct input |
| TBD | bank validation failure | engine not initialized | no | correct configuration |
| TBD | replay end-of-stream | finalized / unchanged | yes | stop or reset |

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
Populate this section from actual benchmark artifacts:

| Target | Build Mode | Scenario | Mean Step Time | p95 | p99 | Notes |
|---|---|---|---:|---:|---:|---|
| Host x86_64 | release | scalar online step | TBD | TBD | TBD | measured |
| ARM target | release | scalar online step | TBD | TBD | TBD | measured if available |

### 6.3 Timing jitter chart
Include:
- a 10,000-step run chart,
- histogram,
- and summary statistics.

If ARM measurements are not yet available, say so explicitly. Do not claim them.

---

## 7. Data Structures

Document all externally stable structures, especially:
- opaque handle type
- `repr(C)` output structs
- numeric status code enums
- trust scalar range
- semantic disposition enums

---

## 8. Build and Linking

Document:
- static library build
- shared library build
- header generation / inclusion
- compiler assumptions
- C++ wrapper usage if provided

---

## 9. Out-of-Scope / Non-Claims

This ICD does not claim:
- platform certification,
- field qualification,
- formal hard-real-time certification,
- or closed-loop control validation.

It documents the current software interface boundary only.
