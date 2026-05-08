# DSFB-Debug Standards Alignment

## Control-Level Mapping

| Standard | Control/Section | Crate Implementation | Test |
|---|---|---|---|
| NIST SP 800-53 AU-2 | Event Logging | `grammar.rs` generates state transitions as auditable events | `grammar_correctness` |
| NIST SP 800-53 AU-3 | Audit Record Content | `types::AuditRecord` — what, when, where, source, outcome. Per-motif AU-3 content satisfied by `HeuristicEntry.{provenance, evidence_dataset, evidence_dataset_doi, taxonomy_ref}` (29 entries, see `docs/heuristics_bank.md`). | `test_observer_only_contract`; `taxonomy_ref_present_on_every_entry`; `dataset_observed_entries_have_doi` |
| NIST SP 800-53 AU-6 | Audit Review & Analysis | `episode.rs` + `compute_metrics()` = structured review | `tests/eval_tadbench.rs`, `tests/eval_illinois.rs` |
| NIST SP 800-53 AU-12 | Audit Record Generation | Deterministic replay (Theorem 9) ensures reproducible records | `RealDatasetEvaluation::deterministic_replay_holds` (real-data eval tests) |
| NIST SP 800-92 §4.2 | Log Analysis | Grammar evaluation over log-frequency residuals | Integration tests |
| NIST SP 800-171 §3.3 | Audit & Accountability | Full traceability chain: residual → sign → grammar → policy | All integration tests |
| CMMC L2 AU.L2-3.3.1 | Audit Events | Grammar state transitions are typed, logged audit events | Grammar tests |
| CMMC L2 AU.L2-3.3.2 | Audit Record Content (CUI) | `episode::compute_metrics` produces structured episode records satisfying the AU-3 content schema for CUI handling | `tests/eval_tadbench.rs` |
| DO-178C §6.3 | Verification of Outputs | Deterministic, traceable analysis outputs (foresight; not a certification claim) | `RealDatasetEvaluation::deterministic_replay_holds` |
| IEEE 1012-2016 §7 | V&V Activities | DSFB-Debug is a verification tool per IEEE 1012 taxonomy | Architecture |
| ISO/IEC 25010:2023 | Analysability | Typed episode output improves system analysability | Architecture |
| ISO/IEC 25010:2023 | Testability | Deterministic replay = testable, verifiable | `RealDatasetEvaluation::deterministic_replay_holds` |
| W3C Trace Context L1 | §3 traceparent | Causal ordering support (window_index-based) | Pipeline tests |
| SOC 2 CC7.2 | Monitoring Activities | Continuous structural monitoring of telemetry residuals | `tests/eval_tadbench.rs` |
| SOC 2 CC7.3 | Evaluating Anomalies | Grammar + episode evaluation of anomalies | Episode tests |

## Certification-Pathway-Eligible Properties

The crate exhibits the following properties relevant to certification arguments.
These are architectural properties, NOT certification claims:

1. **Deterministic**: identical inputs → identical outputs (Theorem 9, tested)
2. **Reproducible**: no randomness, no time-dependent state, no runtime initialization
3. **Traceable**: full (r, d, s) → sign → grammar → semantic → policy chain
4. **Bounded memory**: `no_std`, `no_alloc` — all buffers statically sized
5. **No undefined behavior**: `#![forbid(unsafe_code)]`
6. **No panic paths**: `#![deny(clippy::unwrap_used)]`
7. **Observer-only**: compile-time enforced non-intrusion

## What This Does NOT Constitute

- NOT a certified monitoring system under any standard
- NOT ISO 25010, SOC 2, FedRAMP, or DO-178C compliant from this crate alone
- Certification requires site-specific calibration, integration testing, and assessor review

## DO-178C §6.3 — Foresight, not certification

DO-178C §6.3 ("Verification of Outputs") expects software outputs to be
produced by a verifiable, deterministic process whose intermediate
representations are retained and traceable. The DSFB-Debug architectural
properties listed above (deterministic, reproducible, traceable, bounded
memory, no UB, no panic paths, observer-only) are precisely the
properties §6.3 expects. The crate is therefore **certification-pathway
eligible**: a future qualification effort would not need to redesign the
architecture, only to produce the §6.3 verification evidence (Software
Verification Plan, Test Case Procedures, Test Results, Trace Matrix).
This is a **foresight** claim — an architectural alignment for future
work — not a certification claim, not an attestation of DAL-A or DAL-B
status, and not an SBIR Phase II deliverable.

## Real-data evaluation traces (Phase I)

The control-ID rows in the table above whose `Test` column points at
`tests/eval_tadbench.rs` / `tests/eval_illinois.rs` produce structured
JSON metric blocks on stdout when run with the fixtures vendored
(`cargo test --features "std paper-lock" -- --nocapture`). The metric
blocks are the artefacts referenced by an external auditor performing
AU-6 / AU-12 / SOC 2 CC7.2 review. See
[`../data/README.md`](../data/README.md) for the extraction recipe.
