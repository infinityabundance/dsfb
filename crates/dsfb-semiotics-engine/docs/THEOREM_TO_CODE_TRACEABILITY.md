# Theorem-to-Code Traceability Matrix

This document is machine-generated from structured `TRACE:TYPE:ID:SHORT_TITLE[:NOTE]` tags embedded in the crate implementation source.

It is a traceability aid for auditors, reviewers, and systems engineers. It is not a proof of correctness by itself.

Regenerate it from the crate root with:

```bash
cargo run --manifest-path Cargo.toml --bin dsfb-traceability
```

Check freshness without rewriting files:

```bash
cargo run --manifest-path Cargo.toml --bin dsfb-traceability -- --check
```

The generator scans `src/`, `tests/`, `examples/`, `ffi/`, and `python/src/` for implementation-linked trace tags.

| Paper Item Type | Paper Item ID | Short Title | File | Line | Notes / Implementation Role |
| --- | --- | --- | --- | ---: | --- |
| THEOREM | THM-DETECTABILITY-BOUND | Configured detectability upper bound | `src/math/detectability.rs` | 12 | Computes theorem-aligned exit-time summaries when explicit bound assumptions are attached. |
| DEFINITION | DEF-DRIFT | Finite-difference drift | `src/math/derivatives.rs` | 6 | Implements channel-wise first derivative of the residual trajectory. |
| DEFINITION | DEF-ENVELOPE-RADIUS | Admissibility envelope radius | `src/math/envelope.rs` | 67 | Maps configured envelope mode to per-sample radius, derivative bound, and regime label. |
| DEFINITION | DEF-GRAMMAR-EVIDENCE | Grammar evidence summary | `src/engine/semantics/retrieval.rs` | 477 | Reduces grammar trajectory state into counts and regime tags used by semantic retrieval. |
| DEFINITION | DEF-RESIDUAL | Residual construction | `src/math/residual.rs` | 6 | Implements sample-wise observed minus predicted residual formation. |
| DEFINITION | DEF-SLEW | Nonuniform finite-difference slew | `src/math/derivatives.rs` | 54 | Implements channel-wise second derivative over nonuniform sampled times. |
| ASSUMPTION | ASM-BOUNDED-ONLINE-HISTORY | Bounded online history window | `src/live/mod.rs` | 111 | The deployment path retains only the last N residual samples in a fixed-capacity ring buffer. |
| ASSUMPTION | ASM-FIXED-POINT-QUANTIZATION | Fixed-point ingress quantization | `src/math/fixed_point.rs` | 12 | The experimental embedded backend quantizes online residual inputs to q16.16 with saturating conversion before the conservative layered path runs. |
| ASSUMPTION | ASM-SMOOTHING-PRECONDITIONING | Deterministic smoothing before differencing | `src/math/smoothing.rs` | 10 | Optional low-latency preconditioning attenuates jitter before drift and slew estimation. |
| ALGORITHM | ALG-BOUNDED-ONLINE-STEP | Bounded online layered step | `src/live/mod.rs` | 357 | Replays residual to semantics over the fixed trailing window without unbounded live-state growth. |
| ALGORITHM | ALG-COMPARATOR-BASELINES | Deterministic comparator baselines | `src/evaluation/baselines.rs` | 7 | Computes threshold, moving-average, slew, envelope, CUSUM, and innovation-style comparison outputs. |
| ALGORITHM | ALG-ENVELOPE-BUILD | Envelope materialization | `src/math/envelope.rs` | 111 | Builds the typed admissibility envelope trajectory used by grammar evaluation. |
| ALGORITHM | ALG-FIRST-EXIT | First envelope exit detection | `src/math/detectability.rs` | 5 | Finds the earliest grammar violation sample used by detectability reporting. |
| ALGORITHM | ALG-GRAMMAR-EVALUATION | Admissibility grammar evaluation | `src/math/envelope.rs` | 140 | Assigns admissible, boundary, and violation states with typed reason codes and supporting metrics. |
| ALGORITHM | ALG-NONUNIFORM-SECOND-DERIVATIVE | Nonuniform three-point curvature estimate | `src/math/derivatives.rs` | 111 | Used by slew construction near boundaries and interior samples. |
| ALGORITHM | ALG-SEMANTIC-INDEX | Deterministic semantic prefilter index | `src/engine/semantics/retrieval.rs` | 356 | Builds reproducible candidate buckets for larger heuristic banks without replacing exact validation. |
| ALGORITHM | ALG-SEMANTIC-RETRIEVAL | Typed semantic retrieval | `src/engine/semantics/retrieval.rs` | 96 | Applies admissibility, regime, scope, and compatibility filtering to conservative semantic interpretation. |
| ALGORITHM | ALG-SIGN-PROJECTION | Sign-space projection | `src/engine/sign_layer.rs` | 6 | Maps residual, drift, and slew into the exported three-coordinate sign representation. |
| ALGORITHM | ALG-SMOOTHED-RESIDUAL-TRAJECTORY | Channel-wise residual smoothing | `src/math/smoothing.rs` | 63 | Preserves raw residual exports while producing a smoothed derivative input path. |
| ALGORITHM | ALG-SYNTAX-FORMATION | Deterministic syntax formation | `src/engine/syntax_layer.rs` | 33 | Combines sign, grammar, and coordination summaries into conservative syntax headlines. |
| CLAIM | CLM-COMPUTATIONAL-REPRODUCIBILITY | Layered output reproducibility | `src/engine/pipeline_evaluation.rs` | 8 | Hashes full scenario outputs twice under identical deterministic configuration. |
| CLAIM | CLM-REPRODUCIBILITY-SUMMARY | Aggregate reproducibility summary | `src/engine/pipeline_evaluation.rs` | 38 | Summarizes per-scenario identical reruns over the full layered output bundle. |
| CLAIM | CLM-RETRIEVAL-SCALING-REPORT | Retrieval scaling evidence | `src/engine/semantics/retrieval.rs` | 403 | Exports deterministic candidate-count scaling observations for indexed versus fallback retrieval paths. |
| CLAIM | CLM-TEST-BOUNDED-ONLINE-HISTORY | Executable bounded-history evidence | `tests/deployment_readiness.rs` | 340 | Long-stream smoke test confirms online live state remains bounded. |
| CLAIM | CLM-TEST-CONSTANT-DRIFT-ZERO | Executable constant-drift evidence | `tests/proptest_invariants.rs` | 285 | Property test confirms constant scalar paths yield zero drift. |
| CLAIM | CLM-TEST-REPRODUCIBILITY-HASH | Executable reproducibility evidence | `tests/proptest_invariants.rs` | 450 | Property test confirms identical inputs keep deterministic layered hashes stable. |
| CLAIM | CLM-TRUST-SEVERITY-MAPPING | Trust scalar from grammar severity | `src/math/envelope.rs` | 237 | Maps grammar reason and envelope gap to a bounded deterministic trust scalar. |
| INTERFACE | IFACE-C-ABI-LIFECYCLE | C ABI engine lifecycle | `ffi/src/lib.rs` | 289 | Creates the bounded live engine handle exposed to legacy hosts. |
| INTERFACE | IFACE-C-ABI-STATUS | Stable C ABI status surface | `ffi/src/lib.rs` | 120 | Maps layered live-engine status into a repr-C numeric interface for external callers. |
| INTERFACE | IFACE-CPP-WRAPPER | Header-only C plus plus wrapper | `ffi/include/dsfb.hpp` | 34 | RAII wrapper exposes the C ABI through an idiomatic C++17 surface. |
| INTERFACE | IFACE-PYTHON-BINDING | Python bindings over deterministic engine | `python/src/lib.rs` | 15 | Exposes bounded live status and deterministic batch summaries to Python and Jupyter users. |
| INTERFACE | PUBDATA-001 | PUBLIC_DATASET_PATHS | `src/public_dataset.rs` | 19 | crate-local demo paths for NASA datasets |
