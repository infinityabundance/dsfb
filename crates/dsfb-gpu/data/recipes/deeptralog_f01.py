#!/usr/bin/env python3
"""
Deterministic residual-projection v2 builder for DeepTraLog F01-02.

WHY:
The original dsfb-debug DeepTraLog fixture took only the first 16
rows of one slice and emitted a 16-window TSV in which 14 of 16
rows were NaN-only — a broken projection that produced zero
admitted episodes. This recipe replaces it with a rich projection
of the F01-02 ERROR fault period: 82 800 spans over 9 real minutes
across the TrainTicket microservice fleet, binned into 90-cell
5-second windows per service, with z-score residuals against the
first 12 windows as the healthy baseline.

Determinism contract (panel-locked):
- Single fixed input: `data/upstream/deeptralog/F01.zip` (SHA-256
  pinned in MANIFEST.toml).
- Single fixed inner file: `F01-02/ERROR_F012_SpanData2021-08-14_01-52-43.csv`.
- Window size: 5000 ms.
- Service selection: top 8 services by span count; ties broken by
  service name ascending. Top-8 is enforced from the canonical
  count over the full file.
- Per-service signals: latency-P50 (milliseconds) and span-count
  (spans per window). 8 services × 2 signals = 16 channels.
- Healthy baseline: first 12 windows = 60 seconds.
- z-score residual = (value - baseline_mean) / sample_stddev.
- NaN cells (window with zero spans for a service) emit `nan`
  in the TSV and are skipped by the audit's lowering rule.
- Six-decimal fixed-point formatting; UTF-8; \\n newlines.

Non-claims (preserved into limitations.md):
- No ground-truth label is asserted. The F01-02 file is named
  "ERROR" but DSFB does not depend on that label.
- The projection produces structural residuals; it does not
  identify a "fault" by physics or causality.
- The upstream DeepTraLog repository has no LICENSE file (gh api
  /license returns 404). This recipe and the vendored archive are
  redistributed under academic research-use convention.

License: Apache-2.0. Background IP: Invariant Forge LLC.
"""

import argparse
import csv
import hashlib
import io
import math
import pathlib
import sys
import zipfile

_parser = argparse.ArgumentParser()
_parser.add_argument("--archive", default="F01.zip", help="upstream archive filename in data/upstream/deeptralog/")
_parser.add_argument(
    "--inner-csv",
    default="F01-02/ERROR_F012_SpanData2021-08-14_01-52-43.csv",
    help="ZIP-internal path to the span CSV",
)
_parser.add_argument(
    "--label", default="F01-02 ERROR", help="human-readable fault-case label for metadata"
)
_args = _parser.parse_args()
SOURCE_ARCHIVE = (
    pathlib.Path(__file__).resolve().parents[1] / "upstream/deeptralog" / _args.archive
)
INNER_CSV = _args.inner_csv
LABEL = _args.label
WINDOW_MS = 5000
TOP_K = 8
HEALTHY_WINDOWS = 12
DECIMALS = 6


def sha256_hex(b: bytes) -> str:
    h = hashlib.sha256()
    h.update(b)
    return h.hexdigest()


def load_spans(zip_bytes: bytes, inner_csv: str):
    """Yield (start_ms, end_ms, service) for every span in the inner CSV."""
    with zipfile.ZipFile(io.BytesIO(zip_bytes)) as zf:
        with zf.open(inner_csv) as fh:
            text = io.TextIOWrapper(fh, encoding="utf-8", newline="")
            reader = csv.DictReader(text)
            for row in reader:
                try:
                    st = int(row["StartTime"])
                    et = int(row["EndTime"])
                except (KeyError, ValueError):
                    continue
                yield st, et, row["Service"]


def select_top_services(spans):
    counts: dict[str, int] = {}
    for _st, _et, svc in spans:
        counts[svc] = counts.get(svc, 0) + 1
    # Sort by -count, then by name asc.
    return sorted(counts.items(), key=lambda x: (-x[1], x[0]))[:TOP_K]


def bin_spans(spans, top_services, t0_ms, window_ms):
    """Return per-window per-service (latencies_ms_list, span_count)."""
    top_set = {s for s, _ in top_services}
    bins: dict[int, dict[str, list[float]]] = {}
    for st, et, svc in spans:
        if svc not in top_set:
            continue
        w = (st - t0_ms) // window_ms
        if w < 0:
            continue
        bins.setdefault(w, {})
        bins[w].setdefault(svc, []).append((et - st))
    return bins


def percentile_50_int_ms(values_ms: list[float]) -> float:
    """Deterministic median of integer milliseconds.

    Python's statistics.median uses float arithmetic with the
    standard tie-break (average of two middle elements when n is
    even). To keep byte-determinism, we use the same definition
    explicitly here.
    """
    s = sorted(values_ms)
    n = len(s)
    if n == 0:
        return float("nan")
    mid = n // 2
    if n % 2 == 1:
        return float(s[mid])
    return (s[mid - 1] + s[mid]) / 2.0


def zscore_residuals(matrix: list[list[float]], healthy_n: int) -> list[list[float]]:
    """Per-column z-score against the first `healthy_n` rows.

    `nan` cells in the healthy band drop out of the baseline mean
    and stddev for that column; `nan` cells outside the healthy
    band stay `nan` in the output.
    """
    n_rows = len(matrix)
    n_cols = len(matrix[0]) if matrix else 0
    out: list[list[float]] = [[float("nan")] * n_cols for _ in range(n_rows)]
    for c in range(n_cols):
        healthy_values = [
            matrix[r][c] for r in range(min(healthy_n, n_rows)) if not math.isnan(matrix[r][c])
        ]
        if len(healthy_values) < 2:
            # Cannot form a sample stddev with <2 points.
            continue
        mean = sum(healthy_values) / len(healthy_values)
        var = sum((v - mean) ** 2 for v in healthy_values) / (len(healthy_values) - 1)
        stddev = math.sqrt(var)
        if stddev <= 0:
            continue
        for r in range(n_rows):
            v = matrix[r][c]
            if math.isnan(v):
                continue
            out[r][c] = (v - mean) / stddev
    return out


def format_tsv(matrix: list[list[float]], metadata: dict[str, str]) -> bytes:
    lines: list[str] = ["# residual-projection v2"]
    header_order = [
        "upstream_doi",
        "upstream_url",
        "upstream_archive",
        "upstream_archive_sha256",
        "upstream_inner_file",
        "license",
        "attribution",
        "fixture_origin",
        "num_windows",
        "num_signals",
        "healthy_window_end",
        "window_ms",
        "channels",
        "top_services",
        "projection_law",
        "decimal_places",
        "notes",
    ]
    for key in header_order:
        if key not in metadata:
            raise KeyError(f"missing metadata key {key!r}")
        lines.append(f"# {key}={metadata[key]}")
    body: list[str] = []
    for row in matrix:
        cells: list[str] = []
        for v in row:
            if math.isnan(v):
                cells.append("nan")
            else:
                cells.append(f"{v:.{DECIMALS}f}")
        body.append("\t".join(cells))
    text = "\n".join(lines + body) + "\n"
    return text.encode("utf-8")


def main() -> None:
    zip_bytes = SOURCE_ARCHIVE.read_bytes()
    archive_sha = sha256_hex(zip_bytes)

    spans = list(load_spans(zip_bytes, INNER_CSV))
    if not spans:
        raise SystemExit("ERR: no spans loaded from inner CSV")

    top_services = select_top_services(spans)
    t0_ms = min(st for st, _et, _svc in spans)
    t_last_ms = max(st for st, _et, _svc in spans)
    n_windows = (t_last_ms - t0_ms) // WINDOW_MS + 1

    bins = bin_spans(spans, top_services, t0_ms, WINDOW_MS)

    # Build raw matrix: rows = windows, cols = (svc_p50, svc_count) for each top service.
    services_in_order = [s for s, _ in top_services]
    n_signals = len(services_in_order) * 2
    raw: list[list[float]] = []
    for w in range(n_windows):
        row: list[float] = []
        wbin = bins.get(w, {})
        for svc in services_in_order:
            lats = wbin.get(svc, [])
            row.append(percentile_50_int_ms(lats))
            row.append(float(len(lats)))
        raw.append(row)

    projected = zscore_residuals(raw, HEALTHY_WINDOWS)

    channels = []
    for svc in services_in_order:
        short = svc.replace("ts-", "").replace("-service", "")
        channels.append(f"{short}_p50_ms")
        channels.append(f"{short}_span_count")

    metadata = {
        "upstream_doi": "Zhang et al. ICSE 2022",
        "upstream_url": "https://github.com/FudanSELab/DeepTraLog",
        "upstream_archive": f"data/upstream/deeptralog/{_args.archive}",
        "upstream_archive_sha256": archive_sha,
        "upstream_inner_file": INNER_CSV,
        "license": "no-upstream-license",
        "attribution": "Zhang et al. ICSE 2022.",
        "fixture_origin": (
            f"{LABEL} fault-period spans ({len(spans):,} spans, "
            f"{(t_last_ms - t0_ms) / 1000:.0f}s) projected to "
            f"{n_windows}-window x {n_signals}-signal residual matrix"
        ),
        "num_windows": str(n_windows),
        "num_signals": str(n_signals),
        "healthy_window_end": str(HEALTHY_WINDOWS),
        "window_ms": str(WINDOW_MS),
        "channels": ",".join(channels),
        "top_services": ",".join(services_in_order),
        "projection_law": (
            "Per (window, service) bin: p50 = median of (EndTime - StartTime) "
            "milliseconds; span_count = number of spans. Residual = "
            "(value - baseline_mean) / sample_stddev over the first "
            "healthy_window_end windows. Bessel correction (n-1). NaN cell = "
            "service had zero spans in that window."
        ),
        "decimal_places": str(DECIMALS),
        "notes": (
            "DSFB interprets structural residual evidence; no ground-truth "
            "fault label is asserted. The F01-02 file is named ERROR but DSFB "
            "does not depend on that label."
        ),
    }

    sys.stdout.buffer.write(format_tsv(projected, metadata))


if __name__ == "__main__":
    main()
