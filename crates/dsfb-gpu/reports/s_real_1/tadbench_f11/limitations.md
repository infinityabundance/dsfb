# S-REAL.1 audit — limitations and non-claims (tadbench_f11)

This file accompanies the `audit_report.html` for this dataset. The
audit's deliverable is **deterministic, replayable structural
evidence on real public dataset bytes** — not domain-truth
claims.

## Non-claims

- Does NOT claim DSFB has identified the "real" anomaly in the dataset.
- Does NOT claim DSFB outperforms any other anomaly detector.
- Does NOT claim DSFB has discovered causality.
- Does NOT claim DSFB has measured remediation effectiveness.
- Does NOT claim fitness-for-purpose on regulated or safety-critical use.
- Does NOT claim the dataset is "correctly labeled" or "ground truth".
- Does NOT claim the corpus or registry is exhaustive.
- Does NOT claim replay determinism across different driver / CUDA / hardware versions.

## Lowering disclosure

The upstream fixture is in `residual-projection v2` form
(window-major × signal-minor TSV). DSFB-GPU normally takes a
`Vec<TraceEvent>` and projects events into residuals via its
window-feature kernel; the upstream is already past that
projection. To run the deterministic engine on this form
without modifying the dispatcher, the audit lowers each
finite cell into one synthetic `TraceEvent` via a documented
rule (see `schema_map.toml` and section 2 of
`audit_report.html`). The audit does NOT claim to recover the
upstream's original trace events; it claims DSFB-GPU saw
exactly the events that rule produces from these bytes.
