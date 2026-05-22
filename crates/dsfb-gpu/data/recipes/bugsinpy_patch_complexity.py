#!/usr/bin/env python3
"""
Deterministic residual-projection v2 builder for BugsInPy patch complexity.

WHY:
The original dsfb-debug BugsInPy fixture took only the SHA-1
prefix of each buggy commit and decoded it to a 28-bit integer
— essentially random noise, no structural signal. This recipe
replaces it with a real per-project bug-fix complexity
time-series: bug_patch.txt line count over bug-id order for the
6 most-bug-rich projects in soarsmu/BugsInPy.

The input is a deterministic composed snapshot
(`data/upstream/bugsinpy/bugsinpy_snapshot.csv`) extracted from
the soarsmu/BugsInPy `projects/` tree. The snapshot carries per
(project, bug_id) the bug_patch.txt line count + byte size +
setup.sh size + run_test.sh size. We use the patch-line count
as the primary signal — real per-bug complexity variation.

Determinism contract:
- Single fixed input: `data/upstream/bugsinpy/bugsinpy_snapshot.csv`
  (SHA-256 pinned in MANIFEST.toml).
- Projects selected: 6 most-bug-rich in upstream-order:
  pandas (169), keras (45), youtube-dl (43), scrapy (40),
  luigi (33), thefuck (32).
- One signal per project = 6 signals.
- One window per bug_id (1..N where N = max bug-id across
  the 6 projects = 169). Projects with fewer bugs emit `nan`
  for the windows beyond their last bug.
- Signal value: bug_patch.txt line count.
- z-score residual per project against the first 20 bugs.
- Six-decimal fixed-point formatting; UTF-8; \\n newlines.

Non-claims:
- Patch line count is a proxy for fix complexity, not a measure
  of bug severity. DSFB reports structural outliers; it does
  NOT claim any bug is "complex" in a domain sense.
- The upstream BugsInPy repo has no LICENSE file; redistributed
  under academic research-use convention.

License: Apache-2.0. Background IP: Invariant Forge LLC.
"""

import csv
import hashlib
import math
import pathlib
import sys

SOURCE_CSV = (
    pathlib.Path(__file__).resolve().parents[1]
    / "upstream/bugsinpy/bugsinpy_snapshot.csv"
)
SELECTED_PROJECTS = ["pandas", "keras", "youtube-dl", "scrapy", "luigi", "thefuck"]
HEALTHY_WINDOWS = 20
DECIMALS = 6


def sha256_hex(b: bytes) -> str:
    h = hashlib.sha256()
    h.update(b)
    return h.hexdigest()


def load_snapshot(csv_bytes: bytes) -> dict[str, dict[int, int]]:
    """Return {project: {bug_id: bug_patch_lines}}."""
    text = csv_bytes.decode("utf-8")
    reader = csv.DictReader(text.splitlines())
    out: dict[str, dict[int, int]] = {}
    for row in reader:
        try:
            proj = row["project"]
            bid = int(row["bug_id"])
            patch_lines = int(row["bug_patch_lines"])
        except (KeyError, ValueError):
            continue
        out.setdefault(proj, {})[bid] = patch_lines
    return out


def project_matrix(
    snapshot: dict[str, dict[int, int]], projects: list[str]
) -> list[list[float]]:
    """Rows = bug_id (1..max), cols = projects (in order). nan = missing bug for that project."""
    max_bid = max(
        max(snapshot.get(p, {}).keys() or [0]) for p in projects
    )
    matrix: list[list[float]] = []
    for bid in range(1, max_bid + 1):
        row: list[float] = []
        for p in projects:
            v = snapshot.get(p, {}).get(bid)
            row.append(float(v) if v is not None else float("nan"))
        matrix.append(row)
    return matrix


def zscore(matrix: list[list[float]], healthy_n: int) -> list[list[float]]:
    n_rows = len(matrix)
    n_cols = len(matrix[0]) if matrix else 0
    out: list[list[float]] = [[float("nan")] * n_cols for _ in range(n_rows)]
    for c in range(n_cols):
        healthy = [
            matrix[r][c]
            for r in range(min(healthy_n, n_rows))
            if not math.isnan(matrix[r][c])
        ]
        if len(healthy) < 2:
            continue
        mean = sum(healthy) / len(healthy)
        var = sum((v - mean) ** 2 for v in healthy) / (len(healthy) - 1)
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
    body = []
    for row in matrix:
        cells = [
            "nan" if math.isnan(v) else f"{v:.{DECIMALS}f}" for v in row
        ]
        body.append("\t".join(cells))
    return ("\n".join(lines + body) + "\n").encode("utf-8")


def main() -> None:
    csv_bytes = SOURCE_CSV.read_bytes()
    csv_sha = sha256_hex(csv_bytes)
    snapshot = load_snapshot(csv_bytes)
    raw = project_matrix(snapshot, SELECTED_PROJECTS)
    projected = zscore(raw, HEALTHY_WINDOWS)

    metadata = {
        "upstream_doi": "Widyasari et al. ESEC/FSE 2020",
        "upstream_url": "https://github.com/soarsmu/BugsInPy",
        "upstream_archive": "data/upstream/bugsinpy/bugsinpy_snapshot.csv",
        "upstream_archive_sha256": csv_sha,
        "license": "no-upstream-license",
        "attribution": "Widyasari et al. ESEC/FSE 2020. github.com/soarsmu/BugsInPy.",
        "fixture_origin": (
            f"BugsInPy {len(SELECTED_PROJECTS)}-project bug-patch-line "
            f"complexity time-series over the first 169 bug-ids; "
            f"projected to {len(projected)}-window x "
            f"{len(SELECTED_PROJECTS)}-signal z-score residual matrix"
        ),
        "num_windows": str(len(projected)),
        "num_signals": str(len(SELECTED_PROJECTS)),
        "healthy_window_end": str(HEALTHY_WINDOWS),
        "channels": ",".join(f"{p}_bug_patch_lines" for p in SELECTED_PROJECTS),
        "projection_law": (
            "Per project: z = (bug_patch.txt line count - baseline_mean) / "
            "sample_stddev where baseline = first healthy_window_end bug-ids. "
            "Bessel correction. NaN = project has no bug at that bug-id."
        ),
        "decimal_places": str(DECIMALS),
        "notes": (
            "Patch line count is a proxy for fix complexity, not bug severity. "
            "DSFB reports structural outliers; no domain claim about which "
            "bugs are 'complex' in a software-engineering sense."
        ),
    }
    sys.stdout.buffer.write(format_tsv(projected, metadata))


if __name__ == "__main__":
    main()
