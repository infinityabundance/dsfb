# DSFB-Debug — Operator Handbook

Single-screen reference for the on-call engineer reading a DSFB-Debug
episode at 3am. Every field documented; every symbol traceable;
no surprises.

## At-a-glance: what an episode tells you

When DSFB-Debug fires, the operator sees a `RenderedEpisodeSummary`
(produced by `render::render_episode_summary`):

```text
Episode #7  windows 100..=105  (6 windows)
Motif        : CascadingTimeoutSlew  (DatasetObserved, evidence: tadbench_trainticket_F04 / DOI 10.5281/zenodo.6979726)
Taxonomy     : IEEE 24765: 'fault propagation'; A-L-R: error → service-failure
Originator   : ts-station-service  (signal index 0)
Affected     : 4 services contributing
Peak slew    : 0.850
Policy state : Escalate
Confidence   : top 4.50, runner-up 1.50 (DeploymentRegressionSlew), margin 0.667
Hint         : Inspect ts-station-service (signal 0); 4 services contribute over 6 windows; peak slew 0.850
```

Every line above is mechanically derived from the `DebugEpisode`
struct + the bank's per-motif metadata. Nothing inferred. Nothing
"helpful AI suggestion" — every field has a source.

## The eight load-bearing questions (panel-derived)

| # | Question | Where to look in the summary |
|---|----------|------------------------------|
| 1 | How many real incidents? | `episode_count` (one summary per closed episode) |
| 2 | Which service is the originator? | `Originator` line (`root_cause_service` from auto-graph + `signal_names`) |
| 3 | What changed vs healthy? | Compare `peak_slew` and `contributing_signal_count` to the calibration tool's healthy stats |
| 4 | Do I trust this characterisation? | `Confidence` line (margin > 0.5 → trust; < 0.2 → see runner-up) |
| 5 | Have we seen this before? | Run `EpisodeCatalog::find_similar(episode)` — see "Catalog lookup" below |
| 6 | Where is the evidence trail? | `evidence_dataset` + DOI in the motif metadata; Theorem-9 deterministic replay verifies reproducibility |
| 7 | What should I investigate next? | `Hint` line (template-substituted; concrete service name + slew + duration) |
| 8 | Is this faster than my current tools? | Comparison run in `docs/incumbent_comparison.md` (numerical) |

## How to read the confidence margin

`MatchConfidence.margin = (top_score - runner_up_score) / top_score`,
clamped `[0, 1]`.

| Range | Operator action |
|-------|------------------|
| margin > 0.5 | Trust the top motif. Act on `Hint`. |
| 0.2 < margin ≤ 0.5 | Top dominates but runner-up is plausible. Note runner-up motif and consider whether the symptoms match it. |
| margin ≤ 0.2 | Top and runner-up are competitive. Surface BOTH. The episode is structurally ambiguous; further investigation needed. |
| top_score == 0 | `SemanticDisposition::Unknown` — endoductive mode. The structure is real (the episode fired) but no named motif matched. Treat as a candidate new motif for the field-validated provenance tier. |

## Catalog lookup — institutional memory

```rust
let mut catalog = EpisodeCatalog::<128>::new();   // 128-entry circular buffer

// During each evaluation run:
for ep in episodes {
    catalog.record(*ep);
}

// At incident time:
if let Some(similar) = catalog.find_similar(&current_ep) {
    println!(
        "Similar to past episode #{} (similarity {:.2})",
        similar.past_episode.episode_id,
        similar.similarity,
    );
    // Operator: pull the runbook entry for episode_id
}
```

Similarity is signature-vector cosine over `(motif, reason_code,
drift_direction, peak_slew, duration_windows)`. The catalog is
in-memory; persistence between runs is operator-side (serialise to
audit-sink).

## How to feed DSFB-Debug a service-call graph

Two paths:

1. **Operator-supplied (today's TrainTicket-Anomaly path).**
   Construct `&[(u16, u16)]` parent→child pairs and pass to
   `engine.run_evaluation_with_graph(...)`.

2. **Auto-inferred from spans (preferred).** Use
   `graph_inference::infer_service_graph_from_observed(observed_edges,
   num_services, &mut out_edges)` to walk span parent-child links into
   a deduplicated, canonical-ordered edge list. The graph is then fed
   into `run_evaluation_with_graph`. Tarjan's SCC handles cyclic
   service dependencies (operator gets the SCC count for cycle
   detection — if SCCs < num_services, the graph has cycles and
   causality attribution may surface ambiguous roots).

## Site calibration — when bank thresholds don't fit

If clean-window false-episode rate is too high (or fault recall too
low), recalibrate the bank's thresholds against your healthy slice:

```rust
use dsfb_debug::calibration::recommend_config_from_healthy;

let report = recommend_config_from_healthy(
    &healthy_data,    // healthy slice of your residual matrix
    num_signals,
    num_windows,
    0.9,              // percentile (0.9 = "fire on top 10%")
);

// Inspect report.healthy_stats — empirical p50/p90/p99 of residual norm
// Inspect report.motif_recommendations — per-motif (drift, slew) thresholds
// Decide whether to apply by mutating your bank copy
```

The crate does NOT silently rewrite the canonical bank. The operator
applies overrides at their discretion.

## Audit-trail emission (NIST SP 800-53 AU-12)

The harness emits one JSON metric block per evaluation on stdout. For
AU-12 minimum compliance:

1. Pipe stdout into a structured sink with timestamp.
2. Append-only log file with HMAC envelope (planned: `src/audit_sink.rs`).
3. Forward to your compliance pipeline (Splunk / Sentinel / etc.).

The `deterministic_replay_holds = true` field in every block is the
load-bearing reproducibility claim: the same fixture bytes produce the
same metric block, every time.

## Comparison vs incumbents (real numbers, F-11 fixture)

From `tests/incumbent_compare.rs` on the vendored TrainTicket-Anomaly
F-11 slice (35,604 real Jaeger spans):

| Detector | Raw alerts | Episodes | Clean-window FP rate | Wall-clock (µs) | Deterministic |
|----------|-----------:|---------:|---------------------:|----------------:|--------------:|
| **dsfb-debug** (Cargo v0.1.0, post-Session-18) | 11 | **3** | **0.0070** | 3850 | yes (Theorem 9) |
| scalar_threshold_3sigma | 10 | 6 | 0.0139 | 98 | yes |
| cusum_h4 | 10 | 5 | 0.0116 | 111 | yes |
| ewma_lambda0.2_L3 | 65 | 35 | 0.0812 | 103 | yes |

Honest reading on this fixture:

- DSFB-Debug delivers **half the clean-window FP rate** of scalar-3σ
  and CUSUM, and **1/12th of EWMA's**. Trace Event Collapse is
  observable (3 typed episodes vs 5–35 raw alert windows).
- Wall-clock per evaluation is ~35× higher than baselines, but
  evaluations run at window cadence (15-second windows here), so the
  cost is negligible at the scale that matters in practice.
- All four detectors are deterministic; DSFB-Debug has the strongest
  guarantee (Theorem 9 with formal proof + empirical replay).

This is one fixture. Per the academic-honesty discipline: do NOT
generalise these numbers without site-specific calibration. The
defence-mission and production-cloud regimes can produce different
RSCR, FP, and recall ratios; site engagement is required for those
empirical claims.

## Field reference

| Field | Type | Source | Notes |
|-------|------|--------|-------|
| `episode_id` | u32 | `aggregate_episodes` | sequential; resets per run |
| `start_window` / `end_window` | u64 | `aggregate_episodes` | inclusive, 0-indexed |
| `peak_grammar_state` | enum | episode peak | Admissible / Boundary / Violation |
| `primary_reason_code` | enum | episode peak | most-severe across the episode |
| `matched_motif` | enum | `match_episode` | `Named(MotifClass)` or `Unknown` |
| `policy_state` | enum | `apply_policy` | Silent / Watch / Review / Escalate |
| `contributing_signal_count` | u16 | `aggregate_episodes` | distinct signals reaching ≥ Boundary |
| `structural_signature.peak_slew_magnitude` | f64 | per-window max | slew tuple component |
| `structural_signature.duration_windows` | u64 | end - start + 1 | episode length |
| `structural_signature.signal_correlation` | f64 | contrib / num_signals | unitless ratio |
| `root_cause_signal_index` | Option\<u16\> | `causality::attribute_root_causes` | None if no graph or single-signal episode |

## What this handbook does NOT promise

- It does NOT claim DSFB-Debug detects faults faster than incumbent
  tools. The augmentation framing is intact.
- It does NOT claim the F-11 numbers generalise to your system. Run
  the calibration tool first.
- It does NOT replace your existing observability stack. DSFB-Debug
  is a side-panel summary; flat alerts continue to fire.
- It does NOT promise a streaming OTLP gRPC receiver out of the box;
  current shape is batch-mode evaluation against a residual matrix.
  Streaming is Phase II / partner-engagement scope.
