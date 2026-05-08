# DSFB-Debug — Onboarding Recipe

Step-by-step adoption recipe for a team integrating DSFB-Debug into an
existing observability stack. Targets the most common path: Jaeger /
OTLP spans → residual-projection-v2 TSV → `cargo test` end-to-end →
JSON metric block on stdout → forward to incident-management.

## Prerequisites

- Rust toolchain ≥ 1.75.0 (pinned by `rust-toolchain.toml`).
- Trace export from your observability stack — the most common shapes:
  - Jaeger JSON (per-service trace files, as in TADBench / TrainTicket).
  - OTLP JSON / Protobuf (the W3C Trace Context Level 1 wire format).
  - Prometheus-style metric CSV (per-component time-series, as in LO2).
- A *healthy window* of trace data — at least
  `config::PAPER_LOCK_CONFIG.min_healthy_windows = 100` windows of
  nominal-load operation. Required for envelope construction.

## Step 1 — Project upstream traces to residual-projection-v2 TSV

The crate's harness consumes a TSV format documented in
[`../data/README.md`](../data/README.md). The format is a deterministic
projection of upstream span / metric / log streams onto a
per-window, per-signal residual matrix. Worked Python skeleton (zero
crate dependency on Python — only used as projection tooling outside
the no_std core):

```python
#!/usr/bin/env python3
"""
Project Jaeger spans -> residual-projection-v2 TSV.
Adapt paths and channel definitions to your environment.
"""
import json, statistics
from collections import defaultdict
from pathlib import Path

WINDOW_US = 10_000_000   # 10-second windows; adjust per your SLO budget
HEALTHY_FRACTION = 0.5   # First 50% of windows used as healthy baseline

def project(trace_root: Path, top_n_services: int = 8):
    service_spans = defaultdict(list)
    for f in sorted(trace_root.rglob('*.json')):
        try:
            with open(f) as fh:
                d = json.load(fh)
        except Exception:
            continue
        for trace in d.get('data', []):
            for s in trace.get('spans', []):
                proc = trace.get('processes', {}).get(s.get('processID'), {})
                svc = proc.get('serviceName', 'unknown')
                start = s.get('startTime', 0)
                dur = s.get('duration', 0)
                err = any(
                    (t.get('key') == 'http.status_code' and int(t.get('value', 0)) >= 400)
                    or (t.get('key') == 'error' and t.get('value') is True)
                    for t in s.get('tags', [])
                )
                service_spans[svc].append((start, dur, err))

    if not service_spans:
        raise SystemExit('no spans')
    all_starts = [s for spans in service_spans.values() for (s, _, _) in spans]
    t_min, t_max = min(all_starts), max(all_starts)
    n_windows = (t_max - t_min) // WINDOW_US + 1

    top_services = sorted(service_spans, key=lambda k: -len(service_spans[k]))[:top_n_services]
    n_signals = 2 * len(top_services)

    matrix = [[float('nan')] * n_signals for _ in range(n_windows)]
    for s_idx, svc in enumerate(top_services):
        for (start, dur, err) in service_spans[svc]:
            w = (start - t_min) // WINDOW_US
            ...  # bin into matrix[w][s_idx*2..s_idx*2+2]

    healthy_window_end = int(n_windows * HEALTHY_FRACTION)
    channels = [f'{svc}_latency_p50_ms' for svc in top_services] + \
               [f'{svc}_error_rate'     for svc in top_services]

    # Write residual-projection v2 TSV with the channel header line.
    print(f'# residual-projection v2')
    print(f'# num_windows={n_windows}')
    print(f'# num_signals={n_signals}')
    print(f'# healthy_window_end={healthy_window_end}')
    print(f'# fault_labels=')   # populate window indices if you have ground truth
    print(f'# channels={",".join(channels)}')
    for row in matrix:
        print('\t'.join(f'{v:.6f}' for v in row))
```

This is illustrative; the working extraction script lives at
`data/upstream/project_trainticket.py` and can be adapted per dataset.
Per-(window, service) `(latency_p50_ms, error_rate)` is the minimum
useful projection; richer projections add `(p99_ms, span_count,
log_volume, log_severity_high_pct)` channels per service.

## Step 2 — Place the TSV under `data/fixtures/` and update the manifest

```
mv my_extracted_slice.tsv crates/dsfb-debug/data/fixtures/<dataset_key>.tsv
sha256sum crates/dsfb-debug/data/fixtures/<dataset_key>.tsv
```

Edit `data/MANIFEST.toml`:

```toml
[<dataset_key>]
upstream_doi              = "<DOI or stable URL>"
upstream_url              = "<canonical URL>"
upstream_archive_sha256   = "<sha256 of the source archive>"
fixture_path              = "data/fixtures/<dataset_key>.tsv"
fixture_sha256            = "<recomputed SHA-256 from above>"
fixture_provenance        = "<single-line description: which spans, which window width, which fault case>"
upstream_license          = "<SPDX>"
fault_label_mapping       = "data/fault_labels/<dataset_key>.json"
expected_motif_class      = "<MotifClass variant or TBD>"
```

Add a corresponding `RealDatasetManifest` constant in
`src/real_data.rs` with the same SHA-256, then add an eval test under
`tests/eval_<dataset_key>.rs` that calls `evaluate_real_dataset` with
your manifest and `include_bytes!("../data/fixtures/<dataset_key>.tsv")`.

## Step 3 — Run the harness end-to-end

```
cd crates/dsfb-debug
cargo test --features "std paper-lock" --test eval_<dataset_key> -- --nocapture
```

Expected output: a structured JSON metric block on stdout. Example
(real, from the vendored TrainTicket-Anomaly F-11 fixture):

```json
{
  "manifest_name": "tadbench_trainticket_F11",
  "deterministic_replay_holds": true,
  "episode_count": 3,
  "metrics": {
    "total_windows": 431,
    "total_signals": 16,
    "raw_anomaly_count": 11,
    "dsfb_episode_count": 3,
    "rscr": 3.6666666666666665,
    "episode_precision": 0,
    "fault_recall": 1,
    "investigation_load_reduction_pct": 72.72727272727273,
    "clean_window_false_episode_rate": 0.0069605568445475635
  }
}
```

`deterministic_replay_holds: true` is the load-bearing engineering
claim (Theorem 9 verified on real bytes). RSCR is the
"trace-event-collapse" ratio (raw alerts / typed episodes); larger is
more compression. `clean_window_false_episode_rate` is the harness's
honest false-positive rate on the healthy-baseline windows.

## Step 4 — Forward the JSON to incident-management

The metric block is a single line of JSON on stdout. Consumers:

### PagerDuty webhook example

```bash
cargo test --features "std paper-lock" -- --nocapture 2>&1 \
  | grep -A 20 '"manifest_name"' \
  | curl -X POST 'https://events.pagerduty.com/v2/enqueue' \
      -H 'Content-Type: application/json' \
      -d @- \
      -d '{"routing_key":"...", "event_action":"trigger", "payload": ...}'
```

### Slack webhook example

```bash
cargo test --features "std paper-lock" -- --nocapture 2>&1 \
  | grep -A 20 '"manifest_name"' \
  | jq '{text: ("DSFB-Debug ep_count=" + (.episode_count | tostring))}' \
  | curl -X POST 'https://hooks.slack.com/services/...' \
      -H 'Content-Type: application/json' \
      -d @-
```

### Audit-trail emission (NIST SP 800-53 AU-12)

The JSON metric block is reproducible: identical fixture bytes →
identical metric numbers (Theorem 9). For ATO-pathway audit, append
the metric block to a structured audit sink with timestamp + signed
envelope. The crate itself emits to stdout; the audit-sink layer is
operator-side.

## Step 5 — Site calibration (when bank thresholds need tuning)

The 32-motif bank ships with hand-curated thresholds (drift_threshold,
slew_threshold, etc.) that work on public benchmarks but should be
re-fitted at your site. Use the calibration tool documented in
[`calibration.md`](calibration.md): pass a healthy-window slice; the
tool returns a per-motif recommended `(drift_threshold, slew_threshold)`
at your chosen percentile.

## Step 6 — Drop-in alongside existing observability stack

DSFB-Debug is augmentation. Recommended deployment topology:

1. Existing observability stack continues firing flat alerts as today.
2. DSFB-Debug runs as a sidecar batch job (e.g.\ every 5 minutes on a
   rolling window) over the same trace stream the alerts read from.
3. The DSFB-Debug episode summary lands in a dedicated panel on the
   on-call dashboard, ALONGSIDE the flat alerts.
4. On-call engineers see both views: the flat alert list (existing
   tools) AND the structured episode summary (DSFB-Debug). Cognitive
   load goes from ~1000 raw alerts to ~5 typed episodes.

Zero integration risk: read-only, observer-only, type-system-enforced
non-intrusion. The crate cannot mutate upstream telemetry by design.
