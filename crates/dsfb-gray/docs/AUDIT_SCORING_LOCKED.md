# DSFB Locked Audit Scoring Specification

Version: `dsfb-assurance-score-v1`  
Status: Locked  
Applies to: static crate audit scoring emitted by `dsfb-gray`

## Purpose

This document defines the exact scoring method used by the DSFB crate scanner to
produce the `Assurance Audit Score` percentage.

The goal of the score is to summarize `source-visible assurance readiness`
across a stable set of audit checkpoints. It is not a certification result, not
a runtime reliability guarantee, and not a substitute for independent safety,
security, or verification review.

## Lock Rule

This specification is locked to method identifier `dsfb-assurance-score-v1`.

The following changes are considered scoring-breaking and **must** create a new
method identifier and a new locked specification document:

- changing section weights
- adding, removing, or renaming scored checkpoints
- changing checkpoint credit values
- changing threshold values
- changing band boundaries
- moving an informational-only signal into the scored denominator

Historical reports must continue to be interpreted against the method identifier
embedded in their text report, SARIF payload, and in-toto attestation.

## Formula

The overall score is a weighted sum of section scores.

For each section:

```text
section_ratio = sum(checkpoint_credit) / checkpoint_count
section_percent = section_ratio * 100
weighted_points = section_ratio * section_weight_percent
```

Overall score:

```text
overall_percent = sum(weighted_points)
```

The current denominator is fixed at `100.0` weighted points.

Displayed percentages are rounded to one decimal place.

## Checkpoint Credit

Each scored checkpoint contributes one of three values:

- `1.0` = pass, clear, or applied
- `0.5` = indeterminate or partial
- `0.0` = elevated, not applied, or failed

Raw motif counts do **not** scale the score directly. A crate is not penalized
 more heavily just because it is larger and contains more instances of the same
structural issue. Counts remain visible as evidence, but each checkpoint is
scored once.

## Section Weights

| Section | Weight |
|---|---:|
| Safety Surface | 15% |
| Verification Evidence | 15% |
| Build / Tooling Complexity | 10% |
| Lifecycle / Governance | 10% |
| NASA/JPL Power of Ten | 25% |
| Advanced Structural Checks | 25% |

## Scored Checkpoints

### 1. Safety Surface (15%)

Five checkpoints:

1. Unsafe policy declaration  
   `forbid(unsafe_code) = 1.0`, `deny(unsafe_code) = 0.5`, not declared = `0.0`
2. No explicit unsafe sites  
   `unsafe_sites == 0`
3. No panic-like sites  
   `panic_sites == 0`
4. No unwrap/expect-like sites  
   `unwrap_sites == 0`
5. FFI / unsafe justification posture  
   `1.0` if `ffi_sites == 0` and `unsafe_sites == 0`  
   `0.5` if FFI or unsafe is present and at least one `SAFETY:` justification comment exists  
   `0.0` otherwise

### 2. Verification Evidence (15%)

Five checkpoints:

1. Tests present  
   `tests/` directory or test markers detected
2. Property-testing signals present
3. Concurrency-exploration signals present
4. Fuzzing signals present
5. Formal-method signals present

Each checkpoint is binary: present = `1.0`, absent = `0.0`.

### 3. Build / Tooling Complexity (10%)

Six checkpoints:

1. Direct dependency count  
   `<= 10 = 1.0`, `11..=25 = 0.5`, `> 25 = 0.0`
2. Build dependency count  
   `<= 3 = 1.0`, `4..=8 = 0.5`, `> 8 = 0.0`
3. Dev dependency count  
   `<= 15 = 1.0`, `16..=30 = 0.5`, `> 30 = 0.0`
4. No `build.rs`
5. Not a proc-macro crate
6. No codegen / native-build signals

Items 4-6 are binary.

### 4. Lifecycle / Governance (10%)

Thirteen checkpoints:

1. `README` present
2. `CHANGELOG` present
3. `SECURITY.md` present
4. `SAFETY.md` present
5. Architecture/design document present
6. `docs/` content present
7. License evidence present  
   license file or manifest `license`
8. Manifest `rust-version` declared
9. Manifest `edition` declared
10. Manifest `repository` declared
11. Manifest `documentation` declared
12. Manifest `homepage` declared
13. Manifest `readme` declared or `README` present

Each checkpoint is binary.

### 5. NASA/JPL Power of Ten (25%)

Ten checkpoints, one per scored Power-of-Ten adaptation:

1. Simple control flow; no recursion or equivalent escapes
2. All loops have a fixed upper bound
3. No dynamic allocation after initialization
4. Functions stay within a single-sheet size budget
5. Assertion density averages at least two per function
6. Data objects remain at the smallest practical scope
7. Return values are checked and parameters are validated
8. Conditional compilation and metaprogramming stay minimal
9. Pointer use remains restricted
10. Pedantic warnings and static analyzers are enforced

Credit mapping:

- `Applied = 1.0`
- `Indeterminate = 0.5`
- `Not Applied = 0.0`

### 6. Advanced Structural Checks (25%)

Twenty-three checkpoints, one per advanced check:

1. `JPL-R0` Recursion and cyclic call graph audit
2. `JPL-R4` Data-flow traceability / interior mutability audit
3. `JPL-R9` Unchecked extraction / dereference safety audit
4. `NASA-CC` Cyclomatic complexity hotspot audit
5. `H-ASYNC-LOCK` Async lock contention / priority inversion proxy
6. `SAFE-STATE` Catch-all state handling / safe-state fallback audit
7. `TIME-WAIT` Hard-coded timing assumption audit
8. `PART-SPACE` Global shared-resource / partitioning-risk audit
9. `PLUGIN-LOAD` Dynamic loading / plugin sandbox audit
10. `CWE-404` Manual resource-lifecycle / shutdown audit
11. `CMD-BUF` Hazardous command buffering audit
12. `ITER-UNB` Unbounded iterator terminal-consumption audit
13. `ISR-SAFE` Interrupt-context allocation / lock audit
14. `FUTURE-WAKE` Manual Future pending-without-waker audit
15. `TASK-LEAK` Detached-task / discarded JoinHandle audit
16. `DROP-PANIC` Panic-in-Drop audit
17. `ATOMIC-RELAXED` Relaxed atomic ordering on critical-state paths
18. `CLOCK-MIX` Mixed monotonic/wall-clock duration audit
19. `SHORT-WRITE` Partial-write / Interrupted handling audit
20. `ASYNC-RECUR` Async recursion depth-bound audit
21. `CHAN-UNB` Unbounded async command-queue audit
22. `ZERO-COPY` Copy-on-read / zero-copy provenance audit
23. `CARGO-VERS` Dependency version drift / reproducibility audit

Credit mapping:

- `Clear = 1.0`
- `Indeterminate = 0.5`
- `Elevated = 0.0`

## Score Bands

| Overall score | Band |
|---|---|
| `>= 85.0%` | strong assurance posture |
| `>= 70.0% and < 85.0%` | developing but substantial assurance posture |
| `>= 55.0% and < 70.0%` | mixed assurance posture |
| `>= 40.0% and < 55.0%` | limited assurance evidence |
| `< 40.0%` | low assurance readiness |

## Fairness Guardrails

The scoring method intentionally avoids several unfair patterns:

- It does not multiply penalties by raw hit count.
- It does not punish `no_std` absence, `no_alloc` absence, or `no_unsafe`
  absence as capability choices by themselves.
- It does not deduct points for DSFB heuristic motif matches directly.
- It does not deduct points for criticality heatmap density directly.

These are still reported, but they are treated as contextual evidence rather
than denominator-expanding penalties.

## Informational-Only Signals

The following signals are visible in the report but excluded from the score
denominator:

- DSFB heuristic motif matches
- criticality heatmap entries
- raw motif hit totals
- constrained-runtime capability flags such as `no_std`, `no_alloc`, and
  `no_unsafe` candidates
- crate size measures such as file count

## Interpretation Constraints

The score means:

- how much source-visible assurance evidence is present
- how much adverse structural evidence is absent
- how well the crate aligns with the currently-scored assurance controls

The score does **not** mean:

- certification
- proof of correctness
- proof of runtime safety
- proof of WCET, determinism, or absence of gray failures
- proof that a crate is suitable for a specific DAL, SIL, ASIL, or mission role

## Audit Trail Requirement

Every emitted score should remain paired with:

- the method identifier
- the crate source digest
- the scanner version
- the generated timestamp

That requirement exists so the same percentage can always be traced back to the
exact scoring rules that produced it.
