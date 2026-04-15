# DSFB Next-Phase Implementation Roadmap

Status: active roadmap  
Scope: scanner, attestation, and runtime-observer integration  
Date: 2026-04-13

## Purpose

This roadmap converts the recent research directions into concrete work items
for this crate.

It is organized by:

- `build now`: high-value features that fit the current codebase with bounded risk
- `build later`: valuable features that need deeper analysis or larger refactors
- `needs annotations`: features that require explicit system-context or task-role metadata
- `needs dataset`: features that should not be claimed until calibrated with real outcomes

This document maps each feature to the files that should own it in the current
crate.

## Fast View

Use this section as the quick navigation layer. The detailed prose that follows
is still the authoritative implementation guidance.

Legend:

- `status`: current roadmap bucket
- `value`: assurance value
- `fp risk`: false-positive risk
- `depth`: required analysis depth
- `ann.`: whether explicit annotations are required for a credible result
- `data`: whether a real calibration dataset is required before strong claims
- `score`: whether the feature would force a scoring-method version bump if it
  changed the audit denominator or weights

### Portfolio Matrix

| Feature | Status | Value | FP risk | Depth | Ann. | Data | Score | Primary owners |
|---|---|---|---|---|---|---|---|---|
| Prescriptive structural refactoring | build now | very high | low | medium | no | no | no, unless scored later | `scan.rs`, `dsfb-scan-crate.rs`, `lib.rs` |
| Static-to-runtime binding | build now | very high | medium | high | no | no | no, unless thresholds become score inputs | `scan.rs`, `observer.rs`, `heuristics.rs`, `audit.rs`, `inject.rs`, `dsfb-demo.rs` |
| Formal property bridge | build now | high | low | medium | no | no | yes, if it changes score weights/checkpoints | `scan.rs`, `lib.rs` |
| Hazard / FMEA interpretation mapping | build later | high | medium | high | explicit mission-context lens or annotations required | no | no, unless added to score | `scan.rs`, `dsfb-scan-crate.rs` |
| Monomorphization audit | build later | high | low | very high | no | no | no, unless added to score | `dsfb-scan-crate.rs`, `scan.rs` |
| Physical-systems audit family | build later / mixed | high | mixed | mixed | no | no | no, unless added to score | `scan.rs`, `observer.rs`, `heuristics.rs`, `report.rs` |
| Task-role / priority inversion risk | needs annotations | high | medium | high | yes | no | no, unless added to score | `scan.rs`, `observer.rs`, `lib.rs` |
| Stronger hazard/FMEA annotations | needs annotations | high | low | medium | yes | no | no, unless added to score | `scan.rs`, `lib.rs` |
| Systemic fragility index | needs dataset | high | high without calibration | high | no | yes | yes, if merged into audit score | `scan.rs`, `AUDIT_SCORING_LOCKED.md` |

### Physical-Systems Matrix

| Audit family | Status | Value | FP risk | Depth | Why it matters | Primary owners |
|---|---|---|---|---|---|---|
| Structural zero-copy provenance | build now | very high | medium | medium | exposes unnecessary ownership churn on ingress and parse paths | `scan.rs`, `lib.rs` |
| Backpressure damping / retry jitter | build now | very high | low | low | catches retry synchronization and storm-prone recovery paths | `scan.rs`, `heuristics.rs`, `observer.rs` |
| OS-resource entropy / lifecycle traceability | build now | high | medium | medium | surfaces retry/open/connect motifs that can leak or churn handles | `scan.rs`, `dsfb-scan-crate.rs` |
| Mechanical sympathy / false-sharing candidates | build later | high | medium | medium | highlights candidate cache-line contention and hidden coherence overhead | `scan.rs`, `report.rs` |
| Executor work-to-yield ratio | build later | high | medium-high | high | surfaces async paths that may monopolize executor time before yielding | `scan.rs`, `observer.rs`, `heuristics.rs` |
| Monomorphization / instruction-footprint audit | build later | high | low | very high | measures generic-instantiation pressure on binary/code footprint | `dsfb-scan-crate.rs`, `scan.rs` |

## Current File Ownership

- [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
  Static crate scanner, audit scoring, text report, SARIF, in-toto, DSSE export.
- [src/bin/dsfb-scan-crate.rs](/home/one/dsfb-gray/src/bin/dsfb-scan-crate.rs:1)
  Scanner CLI entrypoint and artifact emission.
- [src/heuristics.rs](/home/one/dsfb-gray/src/heuristics.rs:1)
  Runtime heuristic bank and reason-code mapping.
- [src/observer.rs](/home/one/dsfb-gray/src/observer.rs:1)
  Runtime observer decision logic and thresholding.
- [src/audit.rs](/home/one/dsfb-gray/src/audit.rs:1)
  Runtime audit trace and observation record path.
- [src/report.rs](/home/one/dsfb-gray/src/report.rs:1)
  Runtime report generation.
- [src/inject.rs](/home/one/dsfb-gray/src/inject.rs:1)
  Deterministic scenario harness for end-to-end validation.
- [src/bin/dsfb-demo.rs](/home/one/dsfb-gray/src/bin/dsfb-demo.rs:1)
  Demo/evaluation binary.
- [src/lib.rs](/home/one/dsfb-gray/src/lib.rs:1)
  Public exports and crate surface.

## Build Now

### 1. Prescriptive Structural Refactoring

Goal:
- Turn elevated findings into actionable guidance instead of passive alerts.

Implementation:
- Add a remediation model in [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1):
  - `recommended_refactor`
  - `why_it_matters`
  - `safer_alternative`
  - optional `example_snippet`
- Attach guidance to advanced checks and Power-of-Ten failures.
- Emit the guidance in:
  - text reports
  - SARIF `help` / `fix` metadata
  - in-toto predicate summary

First rules to cover:
- `P10-2`: recommend `.take(MAX_STEPS)` or bounded iteration
- `TIME-WAIT`: recommend deadline-driven wait or explicit timeout wrapper
- `SHORT-WRITE`: recommend `write_all` or explicit partial-write loop
- `TASK-LEAK`: recommend retaining `JoinHandle`, `JoinSet`, or shutdown tracking
- `CHAN-UNB`: recommend bounded channel or backpressure-aware path
- `ZERO-COPY`: recommend borrowed slices or `Bytes`-style ownership transfer

Files:
- primary: [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
- export wiring: [src/bin/dsfb-scan-crate.rs](/home/one/dsfb-gray/src/bin/dsfb-scan-crate.rs:1)
- public exports if needed: [src/lib.rs](/home/one/dsfb-gray/src/lib.rs:1)

Acceptance:
- every elevated advanced check has non-empty refactor guidance
- SARIF contains machine-readable remediation text
- at least one fixture test verifies a recommended refactor appears in output

### 2. Static-to-Runtime Binding

Goal:
- Let the runtime observer ingest the static attestation and bias monitoring
  toward known structural weak points.

Implementation:
- Extend the attestation predicate in [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
  with per-heuristic static priors:
  - heuristic id
  - confidence
  - structural density
  - recommended amplification bounds
- Add a runtime ingestion model:
  - attestation verification result
  - loaded priors
  - effective threshold deltas
- Add bounded threshold scaling in [src/observer.rs](/home/one/dsfb-gray/src/observer.rs:1)
  so static priors can only adjust thresholds within safe floors/ceilings.
- Record the loaded prior set in [src/audit.rs](/home/one/dsfb-gray/src/audit.rs:1).
- Expose a startup/demo path in [src/bin/dsfb-demo.rs](/home/one/dsfb-gray/src/bin/dsfb-demo.rs:1)
  and validate it in [src/inject.rs](/home/one/dsfb-gray/src/inject.rs:1).

Example:
- high lock density lowers the effective threshold range for `H-LOCK-01`
- many queue/backpressure motifs increase persistence sensitivity for `H-CHAN-01`
- heavy heap motifs increase scrutiny for `H-ALLOC-01`

Files:
- static priors and export: [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
- heuristic hooks: [src/heuristics.rs](/home/one/dsfb-gray/src/heuristics.rs:1)
- runtime application: [src/observer.rs](/home/one/dsfb-gray/src/observer.rs:1)
- audit trace: [src/audit.rs](/home/one/dsfb-gray/src/audit.rs:1)
- demo/injection: [src/inject.rs](/home/one/dsfb-gray/src/inject.rs:1), [src/bin/dsfb-demo.rs](/home/one/dsfb-gray/src/bin/dsfb-demo.rs:1)

Acceptance:
- runtime can load a DSSE/in-toto artifact at startup
- verification failure leaves runtime behavior unchanged
- successful load is visible in the audit trace
- deterministic tests prove bounded amplification only

### 3. Formal Property Bridge

Goal:
- recognize TLA+ / Alloy evidence and surface it as structured verification context

Implementation:
- detect doc comments and markdown references to:
  - TLA+
  - Alloy
  - model checker outputs
  - repo-local spec files
- emit these as verification evidence in the scan
- classify strength:
  - `linked-spec`
  - `repo-local-spec`
  - `checked-in-ci` or `verified-artifact` later

Important scoring constraint:
- the current score is locked by
  [docs/AUDIT_SCORING_LOCKED.md](/home/one/dsfb-gray/docs/AUDIT_SCORING_LOCKED.md:1)
- if formal-property links begin to change the score denominator or weights,
  that must become `dsfb-assurance-score-v2`

Files:
- detection and export: [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
- public documentation link surface: [src/lib.rs](/home/one/dsfb-gray/src/lib.rs:1)

Acceptance:
- the scanner detects TLA+/Alloy links in rustdoc and markdown
- output distinguishes “link present” from “verified formal evidence”
- no score changes are made under scoring v1

## Build Later

### 4. Hazard / FMEA Interpretation Mapping

Goal:
- translate structural findings into system-hazard hypotheses for explicit mission-context lenses

Implementation:
- add conclusion-lens / annotation-conditioned hazard mapping in [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1):
  - `dal_a_flight_control`
  - `industrial_plc`
  - `mission_network`
- for each elevated finding, emit:
  - hazard hypothesis
  - possible effect
  - severity candidate
  - confidence
  - assumptions

Important constraint:
- do not emit strong hazard claims without an explicit mission-context lens or annotation
- this is evidence support for a safety case, not a certification decision

Files:
- lens definitions and export: [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
- report rendering: [src/bin/dsfb-scan-crate.rs](/home/one/dsfb-gray/src/bin/dsfb-scan-crate.rs:1)

Acceptance:
- hazard mapping is opt-in via conclusion lens or annotations
- every mapped hazard includes assumptions and confidence
- unmapped crates remain on generic structural output

### 5. Monomorphization Audit

Goal:
- detect generic code bloat and structural instruction-footprint risk

Implementation direction:
- this likely exceeds the current pure source-scanner model
- use compiler-assisted data rather than line-pattern matching
- report:
  - monomorphized item count
  - largest generic families
  - repeated concrete instantiations
  - binary/code footprint proxies

Recommended architecture:
- start as a separate optional analysis path before folding it back into
  [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)

Files:
- likely new module later, but current integration points are:
  - [src/bin/dsfb-scan-crate.rs](/home/one/dsfb-gray/src/bin/dsfb-scan-crate.rs:1)
  - [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)

Acceptance:
- uses compiler-derived evidence, not regexes
- reports hotspots without claiming measured cache misses

### 6. Physical-Systems Audit Family

Goal:
- extend the scanner with audits that reflect cache behavior, executor tenure,
  copy pressure, OS resource exhaustion, and retry damping without making
  target-specific timing or cost claims

Ranking rule:
- the prioritization below uses `assurance value`, `false-positive risk`, and
  `required analysis depth`
- it does **not** include cost or time estimates

Priority order:

| Audit family | Assurance value | False-positive risk | Required analysis depth | Recommended phase |
|---|---|---|---|---|
| Structural zero-copy provenance | very high | medium | medium | build now |
| Backpressure damping / retry jitter | very high | low | low | build now |
| OS-resource entropy / lifecycle traceability | high | medium | medium | build now |
| Mechanical sympathy / false-sharing candidates | high | medium | medium | build later |
| Executor work-to-yield ratio | high | medium-high | high | build later |
| Monomorphization / instruction-footprint audit | high | low | very high | build later |

#### 6.1 Structural Zero-Copy Provenance

Goal:
- identify unnecessary owned-buffer creation on ingress and parsing paths

Implementation:
- extend [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1) to distinguish:
  - likely physical copies such as `to_vec`, `into_owned`, string materialization
  - likely cheap handle clones such as `Bytes`-style shared references where recognizable
- add a `copy-entropy` subsection to the text report and structured exports
- attach refactor guidance toward borrowed slices or shared-buffer idioms

Files:
- detection and export: [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
- public scanner exports if needed: [src/lib.rs](/home/one/dsfb-gray/src/lib.rs:1)

Acceptance:
- report suppresses or lowers severity for obvious cheap shared-handle clones
- report elevates likely ingress-to-owned-copy paths
- no claim of bus saturation or exact throughput loss is made from source alone

#### 6.2 Backpressure Damping / Retry Jitter

Goal:
- identify retry paths that can synchronize into storms rather than dissipate load

Implementation:
- extend [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1) to detect:
  - fixed retry intervals
  - linear retry ramps
  - no jitter
  - uncapped retry loops
  - visible timeout/retry resonance candidates
- attach guidance toward exponential backoff with randomized jitter
- fold findings into the existing error/retry and command-buffer audit surfaces

Files:
- primary: [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
- future runtime binding if used as a prior: [src/heuristics.rs](/home/one/dsfb-gray/src/heuristics.rs:1), [src/observer.rs](/home/one/dsfb-gray/src/observer.rs:1)

Acceptance:
- fixed-interval retry loops are detected reliably
- jitter presence lowers severity
- reports avoid claiming exact collapse timing

#### 6.3 OS-Resource Entropy / Lifecycle Traceability

Goal:
- expose structural motifs that can leak or churn file descriptors, sockets, or handles

Implementation:
- extend [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1) to track:
  - connect/open/listen/create patterns inside retry loops
  - `mem::forget`, raw-fd/raw-handle escape paths, and `ManuallyDrop`
  - resource creation without obvious supervisor ownership
  - detached tasks that may retain sockets or files across failure paths

Files:
- primary: [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
- attestation export: [src/bin/dsfb-scan-crate.rs](/home/one/dsfb-gray/src/bin/dsfb-scan-crate.rs:1)

Acceptance:
- resource-creation-in-retry-path motifs are surfaced separately from generic cleanup warnings
- reports use “candidate exhaustion path” language rather than deterministic exhaustion-time claims

#### 6.4 Mechanical Sympathy / False-Sharing Candidates

Goal:
- detect data layouts that are likely to amplify contention through shared cache-line traffic

Implementation:
- add a cache-contention audit in [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1) that flags:
  - structs with multiple atomic or lock fields
  - vectors or arrays of atomics
  - obvious hot shared counters without padding wrappers
- emit guidance toward `CachePadded<T>` or target-aware alignment separation

Important constraint:
- source-only scanning should call these `false-sharing candidates`
- do not claim proven same-line occupancy under Rust’s default representation

Files:
- primary: [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
- future report shaping: [src/report.rs](/home/one/dsfb-gray/src/report.rs:1) if this later appears in runtime reporting

Acceptance:
- obvious contiguous atomic collections are surfaced
- recommendation text prefers padding wrappers over universal `repr(align(64))` advice

#### 6.5 Executor Work-to-Yield Ratio

Goal:
- identify async paths that appear to perform large amounts of work before yielding

Implementation:
- extend [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1) with a static proxy based on:
  - async function size
  - branch/loop density
  - await density
  - parser/serde/transform motifs on async paths
  - blocking or heavy destructor work in async contexts

Important constraint:
- do not claim literal instruction counts or millisecond tenure from source alone
- this is a `work-to-yield` proxy, not a scheduler proof

Files:
- primary: [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
- future runtime bridge if tied to `H-ASYNC-01`: [src/observer.rs](/home/one/dsfb-gray/src/observer.rs:1), [src/heuristics.rs](/home/one/dsfb-gray/src/heuristics.rs:1)

Acceptance:
- large async functions with low await density are surfaced
- reports clearly label this as a proxy for executor tenure risk

#### 6.6 Monomorphization / Instruction-Footprint Audit

Goal:
- measure generic-instantiation density and cold-code growth pressure

Implementation:
- keep this out of the pure source scanner initially
- add a compiler-assisted analysis path later that reports:
  - mono-item counts
  - large generic families
  - repeated instantiations
  - code footprint hotspots

Important constraint:
- no blanket recommendation of trait objects
- recommendations must distinguish hot-path throughput from cold-path code-size reduction

Files:
- likely a future dedicated module plus integration through:
  - [src/bin/dsfb-scan-crate.rs](/home/one/dsfb-gray/src/bin/dsfb-scan-crate.rs:1)
  - [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)

Acceptance:
- evidence is compiler-derived
- reports speak about instruction-footprint pressure, not measured cache misses

## Needs Annotations

### 7. JPL-Style Task Priority Inversion Risk

Goal:
- find shared `Arc<Mutex<_>>` paths between critical tasks and housekeeping tasks

Why annotations are needed:
- Tokio does not provide a first-class real-time priority model
- “high priority” versus “housekeeping” must come from user intent or explicit system context

Suggested annotation model:
- `#[dsfb(task_role = "control")]`
- `#[dsfb(task_role = "housekeeping")]`
- `#[dsfb(task_role = "io")]`

Implementation:
- scan for shared mutex-like state crossing differently tagged roles
- combine with existing checks:
  - `H-ASYNC-LOCK`
  - lock-under-await
  - blocking under lock

Files:
- scanner detection: [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
- runtime use if later bound to monitoring: [src/observer.rs](/home/one/dsfb-gray/src/observer.rs:1)
- public annotation docs: [src/lib.rs](/home/one/dsfb-gray/src/lib.rs:1)

Acceptance:
- no priority-inversion claim without task-role metadata
- output clearly states when a result is annotation-derived

### 8. Stronger Hazard/FMEA Mapping

Goal:
- let developers mark safety-critical functions and safe states explicitly

Suggested annotations:
- `#[dsfb(safety_critical)]`
- `#[dsfb(safe_state = "...")]`
- `#[dsfb(hazard = "...")]`

Files:
- scanner parsing: [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
- public API/docs: [src/lib.rs](/home/one/dsfb-gray/src/lib.rs:1)

Acceptance:
- generic conclusion-lens hazard mapping works without annotations
- high-confidence hazard mapping requires annotations

## Needs Dataset

### 9. Systemic Fragility Index

Goal:
- move from ordinal structural concern to empirically calibrated risk stratification

What is safe to do now:
- emit a non-probabilistic ordinal fragility index based on current audit outputs

What is not safe to do yet:
- claims like “12x higher probability”
- recovery-window probabilities
- cost-of-failure projections

Dataset required:
- audited crate/version history
- observed incidents or fault-injection outcomes
- recovery-time distributions
- model calibration and validation data

Files:
- eventual scoring and export path: [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
- if it changes the scored audit percentage, it requires a new locked scoring spec:
  [docs/AUDIT_SCORING_LOCKED.md](/home/one/dsfb-gray/docs/AUDIT_SCORING_LOCKED.md:1)

Acceptance:
- use ordinal fragility labels first
- do not emit probabilistic financial-risk claims without a calibration corpus

## Recommended Delivery Order

1. Prescriptive structural refactoring
2. Static-to-runtime binding
3. Formal property bridge
4. Structural zero-copy provenance and retry-jitter auditing
5. OS-resource entropy auditing
6. Opt-in hazard interpretation lenses
7. Annotation-based task-role / safety-critical enhancements
8. Mechanical-sympathy and executor work-to-yield auditing
9. Compiler-assisted monomorphization audit
10. Empirical fragility calibration

## Immediate Patch Plan

### Patch Set A: Refactor Guidance

- extend advanced-check data structures in [src/scan.rs](/home/one/dsfb-gray/src/scan.rs:1)
- render guidance in text/SARIF/in-toto
- add regression fixtures for at least four elevated rules

### Patch Set B: Static Priors

- add per-heuristic prior payloads to scan artifacts
- define a runtime prior-ingestion struct
- add bounded threshold scaling in [src/observer.rs](/home/one/dsfb-gray/src/observer.rs:1)
- record priors in [src/audit.rs](/home/one/dsfb-gray/src/audit.rs:1)

### Patch Set C: Formal Links

- detect TLA+/Alloy links in rustdoc and markdown
- add a verification subsection to the scan output
- defer any scoring changes to a future score-method version

## Change-Control Note

This roadmap does not itself change the locked scoring method. Any roadmap item
that changes score weights, checkpoint sets, thresholds, or denominator rules
must create a new score method identifier and a new locked scoring document.
