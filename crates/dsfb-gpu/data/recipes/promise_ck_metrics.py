#!/usr/bin/env python3
"""
Deterministic residual-projection v2 builder for PROMISE CK metrics.

WHY:
The original dsfb-debug PROMISE fixture used per-module defect
counts across 6 separate CSVs — sparse zero-dominant integers
with no per-row structural variation, producing zero admitted
episodes. This recipe replaces it with a rich projection of the
canonical CK object-oriented design metrics from the single
PROMISE/1.csv project (Ant 1.3 — 124 Java classes), exposing 6
structurally-varied metrics over the class-index axis.

CK metrics carry real structural signal: WMC (weighted methods
per class) and RFC (response-for-class) measure complexity; CBO
(coupling between objects) and LCOM (lack of cohesion) measure
modular hygiene; LOC and AMC (average method complexity) measure
size and per-method complexity. Classes vary by these metrics in
deterministic ways the DSFB-GPU detector court can witness as
structural episodes (high-coupling outliers, low-cohesion
plateaus, complexity spikes).

Determinism contract (panel-locked):
- Single fixed input: `data/upstream/promise/1.csv` (SHA-256 pinned
  in MANIFEST.toml). Equivalent to ssea-lab/PROMISE/PROMISE/1.csv.
- 6 signals in fixed order: WMC, RFC, CBO, LCOM, LOC, AMC.
- 1 window per class. Classes appear in upstream CSV order
  (deterministic).
- Healthy baseline: first 20 classes. Long enough for a stable
  baseline mean + sample stddev under Bessel correction.
- z-score residual = (value - baseline_mean) / sample_stddev per
  signal.
- Six-decimal fixed-point formatting; UTF-8; \\n newlines.

Non-claims:
- CK metrics correlate with defect density in the literature
  (Basili, Briand, Melo 1996) but DSFB does NOT assert that any
  high-residual class is "buggy". The audit reports structural
  outliers, not bug predictions.
- The upstream PROMISE corpus has no canonical license; this is
  research-use redistribution under fair-use convention.

License: Apache-2.0. Background IP: Invariant Forge LLC.
"""

import argparse
import csv
import hashlib
import math
import pathlib
import sys

_parser = argparse.ArgumentParser()
_parser.add_argument("--csv-id", default="1", help="upstream PROMISE CSV id (1, 2, ...)")
_parser.add_argument(
    "--project-name", default="Ant 1.3", help="human-readable project name for metadata"
)
_args = _parser.parse_args()
SOURCE_CSV = (
    pathlib.Path(__file__).resolve().parents[1] / f"upstream/promise/{_args.csv_id}.csv"
)
PROJECT_NAME = _args.project_name
SIGNAL_COLUMNS = ["wmc", "rfc", "cbo", "lcom", "loc", "amc"]
HEALTHY_WINDOWS = 20
DECIMALS = 6


def sha256_hex(b: bytes) -> str:
    h = hashlib.sha256()
    h.update(b)
    return h.hexdigest()


def load_metrics(csv_bytes: bytes) -> list[list[float]]:
    """Return one row per class with the 6 signal values in order."""
    text = csv_bytes.decode("utf-8")
    reader = csv.DictReader(text.splitlines())
    rows: list[list[float]] = []
    for r in reader:
        try:
            rows.append([float(r[c]) for c in SIGNAL_COLUMNS])
        except (KeyError, ValueError) as e:
            raise SystemExit(f"ERR: malformed CSV row {r}: {e}")
    return rows


def zscore(matrix: list[list[float]], healthy_n: int) -> list[list[float]]:
    n_rows = len(matrix)
    n_cols = len(matrix[0]) if matrix else 0
    out: list[list[float]] = [[0.0] * n_cols for _ in range(n_rows)]
    for c in range(n_cols):
        healthy = [matrix[r][c] for r in range(min(healthy_n, n_rows))]
        mean = sum(healthy) / len(healthy)
        var = sum((v - mean) ** 2 for v in healthy) / (len(healthy) - 1)
        stddev = math.sqrt(var)
        if stddev <= 0:
            # constant column in the healthy band; emit zeros so the
            # signal is honest "no structural variation".
            continue
        for r in range(n_rows):
            out[r][c] = (matrix[r][c] - mean) / stddev
    return out


def format_tsv(matrix: list[list[float]], metadata: dict[str, str]) -> bytes:
    lines: list[str] = ["# residual-projection v2"]
    header_order = [
        "upstream_doi",
        "upstream_url",
        "upstream_archive",
        "upstream_archive_sha256",
        "license",
        "attribution",
        "fixture_origin",
        "num_windows",
        "num_signals",
        "healthy_window_end",
        "channels",
        "projection_law",
        "decimal_places",
        "notes",
    ]
    for key in header_order:
        if key not in metadata:
            raise KeyError(f"missing metadata key {key!r}")
        lines.append(f"# {key}={metadata[key]}")
    body = [
        "\t".join(f"{v:.{DECIMALS}f}" for v in row) for row in matrix
    ]
    return ("\n".join(lines + body) + "\n").encode("utf-8")


def main() -> None:
    csv_bytes = SOURCE_CSV.read_bytes()
    csv_sha = sha256_hex(csv_bytes)
    raw = load_metrics(csv_bytes)
    projected = zscore(raw, HEALTHY_WINDOWS)

    metadata = {
        "upstream_doi": "Basili, Briand, Melo 1996 (CK metric suite); PROMISE corpus",
        "upstream_url": "https://github.com/ssea-lab/PROMISE",
        "upstream_archive": f"data/upstream/promise/{_args.csv_id}.csv",
        "upstream_archive_sha256": csv_sha,
        "license": "no-upstream-license",
        "attribution": "Sayyad Shirabad, Menzies. PROMISE Repository. github.com/ssea-lab/PROMISE.",
        "fixture_origin": (
            f"PROMISE {_args.csv_id}.csv ({PROJECT_NAME}, {len(projected)} Java classes) "
            f"projected to {len(projected)}-window x {len(SIGNAL_COLUMNS)}-signal "
            "CK-metric z-score residual matrix"
        ),
        "num_windows": str(len(projected)),
        "num_signals": str(len(SIGNAL_COLUMNS)),
        "healthy_window_end": str(HEALTHY_WINDOWS),
        "channels": ",".join(SIGNAL_COLUMNS),
        "projection_law": (
            "Per signal: z = (value - baseline_mean) / sample_stddev where "
            "baseline = first healthy_window_end classes, Bessel correction. "
            "Class ordering is upstream-CSV row order (deterministic)."
        ),
        "decimal_places": str(DECIMALS),
        "notes": (
            "CK metrics correlate with defect density in the literature "
            "(Basili-Briand-Melo 1996) but DSFB does NOT assert that any "
            "high-residual class is buggy. Structural outliers only."
        ),
    }
    sys.stdout.buffer.write(format_tsv(projected, metadata))


if __name__ == "__main__":
    main()
