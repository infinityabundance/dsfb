#!/usr/bin/env python3
"""
Deterministic residual-projection v2 builder for NASA C-MAPSS FD001 unit 1.

WHY THIS FILE EXISTS:
The S-REAL.2 dataset gauntlet needed an F4 (Reliability/Industrial)
anchor. NASA C-MAPSS FD001 is the canonical run-to-failure turbofan
dataset for that family. Its native form is a 26-column space-separated
text file with unit/cycle/operational-settings/21-sensor rows; the
S-REAL audit pipeline ingests `# residual-projection v2` TSVs
(window-major x signal-minor matrices with a `# healthy_window_end`
header). This script is the audit-recordable extraction recipe that
bridges the two formats with no probabilistic, learned, or
machine-dependent step.

Determinism contract (panel-locked):
  - Single fixed input: `data/CMAPSSData.zip` (SHA-256 pinned in
    MANIFEST.toml). We do NOT redownload, fetch, or fall back to
    alternative sources.
  - Single fixed engine: train_FD001 unit 1. 192 cycles.
  - Single fixed healthy band: cycles 1..30. The same 30 cycles
    define baseline mean + sample-stddev per sensor.
  - Single fixed projection: z-score residual
    z[w][c] = (raw[w][c] - baseline_mean[c]) / baseline_stddev[c]
    where baseline_mean / baseline_stddev are computed over cycles
    1..30. Sample stddev uses Bessel correction (n-1).
  - Single fixed sensor-drop rule: any sensor whose baseline stddev
    is < 1e-6 is dropped as non-informative. The dropped-sensor list
    is recorded in the TSV header so the audit is reproducible.
  - Single fixed numeric formatting: every cell is rendered as
    `f"{v:.6f}"` (six decimal places, fixed-point). No scientific
    notation, no locale-dependent separators. NaN cells (would only
    arise if a sensor is non-constant in the healthy band but
    becomes NaN later -- never happens for C-MAPSS) are emitted as
    the literal token `nan`.
  - Output written via binary stdout with explicit \n newlines and
    no trailing whitespace; two runs of this script against the
    same `data/CMAPSSData.zip` produce byte-identical TSV output.

Non-claims (preserved into the audit's limitations.md):
  - This recipe does NOT recover the upstream's original sensor
    readings. It produces a deterministic residual projection
    against the engine's own healthy baseline.
  - This recipe does NOT claim a fault diagnosis. C-MAPSS is run-
    to-failure; we report deterministic structure, not the failure
    mechanism.
  - This recipe does NOT claim cross-engine residuals or
    cross-fixture comparability. Unit 1's baseline is internal to
    unit 1.

License: Apache-2.0. Background IP: Invariant Forge LLC.
"""

import hashlib
import io
import math
import pathlib
import sys
import zipfile

SOURCE_ARCHIVE = pathlib.Path(__file__).resolve().parents[1] / "CMAPSSData.zip"
# CLI: --variant FD001|FD002|FD003|FD004 (default FD001); --unit N
# (default 1). Output to stdout as before; recipe stays byte-
# deterministic for fixed (variant, unit) pair.
import argparse

_parser = argparse.ArgumentParser()
_parser.add_argument("--variant", default="FD001", choices=["FD001", "FD002", "FD003", "FD004"])
_parser.add_argument("--unit", type=int, default=1)
_args = _parser.parse_args()
INNER_FILE = f"train_{_args.variant}.txt"
SELECTED_VARIANT = _args.variant
SELECTED_UNIT = _args.unit
HEALTHY_WINDOW_END = 30  # cycles 1..30 (inclusive) define the baseline.
STD_DROP_THRESHOLD = 1e-6
DECIMALS = 6

# Column layout per NASA C-MAPSS readme.txt: unit, time-in-cycles, three
# operational settings, then sensors 1..21. We treat sensors 1..21 as
# the projection candidates; operational settings are dropped because
# they are workload knobs, not degradation evidence.
COLUMN_NAMES = (
    ["unit", "cycle", "op_setting_1", "op_setting_2", "op_setting_3"]
    + [f"sensor_{i}" for i in range(1, 22)]
)
SENSOR_COLUMNS = [i for i, name in enumerate(COLUMN_NAMES) if name.startswith("sensor_")]


def sha256_hex(b: bytes) -> str:
    h = hashlib.sha256()
    h.update(b)
    return h.hexdigest()


def load_unit_rows(zip_bytes: bytes, inner_file: str, unit: int) -> list[list[float]]:
    """Return the rows of `inner_file` (space-separated) whose first column equals `unit`.

    Each returned row is a list of 26 floats in column order.
    """
    rows: list[list[float]] = []
    with zipfile.ZipFile(io.BytesIO(zip_bytes)) as zf:
        with zf.open(inner_file) as fh:
            for raw in fh:
                fields = raw.split()
                if not fields:
                    continue
                if int(fields[0]) != unit:
                    continue
                # NASA C-MAPSS rows may have a trailing token because of
                # double-spaces; accept exactly 26 fields.
                if len(fields) != 26:
                    raise ValueError(
                        f"unexpected column count {len(fields)} in {inner_file} for unit {unit}"
                    )
                rows.append([float(x) for x in fields])
    return rows


def healthy_baseline(rows: list[list[float]], end_cycle: int) -> dict[int, tuple[float, float]]:
    """Compute (mean, sample_stddev) per sensor column over cycles 1..end_cycle.

    Cycles are 1-indexed in C-MAPSS; we use row index < end_cycle which
    corresponds to cycles 1..end_cycle inclusive (the first `end_cycle`
    rows of the engine).
    """
    healthy_rows = rows[:end_cycle]
    out: dict[int, tuple[float, float]] = {}
    n = len(healthy_rows)
    for col in SENSOR_COLUMNS:
        values = [r[col] for r in healthy_rows]
        mean = sum(values) / n
        # Sample stddev (Bessel correction). When n=1 this would divide
        # by zero; healthy_window_end >= 2 is enforced by an assertion
        # below to make the recipe explicit.
        var = sum((v - mean) ** 2 for v in values) / (n - 1)
        stddev = math.sqrt(var)
        out[col] = (mean, stddev)
    return out


def select_informative_sensors(
    baseline: dict[int, tuple[float, float]],
) -> tuple[list[int], list[int]]:
    """Return (kept_sensor_columns, dropped_sensor_columns).

    Kept = baseline stddev > STD_DROP_THRESHOLD. Dropped = constant
    (or near-constant) in the healthy band, which carries no residual
    signal under the z-score projection.
    """
    kept: list[int] = []
    dropped: list[int] = []
    for col in SENSOR_COLUMNS:
        _mean, stddev = baseline[col]
        if stddev > STD_DROP_THRESHOLD:
            kept.append(col)
        else:
            dropped.append(col)
    return kept, dropped


def project_z_residuals(
    rows: list[list[float]],
    baseline: dict[int, tuple[float, float]],
    kept_cols: list[int],
) -> list[list[float]]:
    """Per (row, sensor in kept_cols) compute (value - baseline_mean) / baseline_stddev."""
    out: list[list[float]] = []
    for row in rows:
        projected: list[float] = []
        for col in kept_cols:
            mean, stddev = baseline[col]
            projected.append((row[col] - mean) / stddev)
        out.append(projected)
    return out


def format_tsv(
    projected: list[list[float]],
    metadata: dict[str, str],
) -> bytes:
    """Emit `# residual-projection v2` formatted bytes.

    Header order is fixed for byte-determinism: the contract is in
    `# key=value` lines, in the order this function inserts them.
    """
    lines: list[str] = []
    lines.append("# residual-projection v2")
    # Header order is deterministic across runs because we iterate a
    # fixed list, not a dict-order-dependent sequence.
    header_order = [
        "upstream_doi",
        "upstream_url",
        "upstream_archive",
        "upstream_archive_sha256",
        "upstream_inner_file",
        "fixture_origin",
        "license",
        "num_windows",
        "num_signals",
        "healthy_window_end",
        "channels",
        "sensor_columns_kept",
        "sensor_columns_dropped",
        "projection_law",
        "baseline_stddev_drop_threshold",
        "decimal_places",
        "notes",
    ]
    for key in header_order:
        if key not in metadata:
            raise KeyError(f"missing metadata key {key!r}")
        lines.append(f"# {key}={metadata[key]}")
    body_lines: list[str] = []
    for row in projected:
        body_lines.append("\t".join(f"{v:.{DECIMALS}f}" for v in row))
    text = "\n".join(lines + body_lines) + "\n"
    return text.encode("utf-8")


def main() -> None:
    if not SOURCE_ARCHIVE.is_file():
        print(f"ERR: {SOURCE_ARCHIVE} not found", file=sys.stderr)
        sys.exit(1)
    zip_bytes = SOURCE_ARCHIVE.read_bytes()
    archive_sha = sha256_hex(zip_bytes)

    rows = load_unit_rows(zip_bytes, INNER_FILE, SELECTED_UNIT)
    assert HEALTHY_WINDOW_END >= 2, "healthy window must have >=2 cycles for Bessel stddev"
    assert len(rows) > HEALTHY_WINDOW_END, "engine must have cycles beyond the healthy band"

    baseline = healthy_baseline(rows, HEALTHY_WINDOW_END)
    kept_cols, dropped_cols = select_informative_sensors(baseline)
    projected = project_z_residuals(rows, baseline, kept_cols)

    kept_names = [COLUMN_NAMES[c] for c in kept_cols]
    dropped_names = [COLUMN_NAMES[c] for c in dropped_cols]

    metadata = {
        "upstream_doi": "Saxena, Goebel, Simon, Eklund (PHM08); NASA PCoE Prognostics Data Repository",
        "upstream_url": "https://www.nasa.gov/intelligent-systems-division/discovery-and-systems-health/pcoe/pcoe-data-set-repository/",
        "upstream_archive": "data/CMAPSSData.zip",
        "upstream_archive_sha256": archive_sha,
        "upstream_inner_file": INNER_FILE,
        "fixture_origin": (
            f"{INNER_FILE} unit {SELECTED_UNIT} (run-to-failure, {len(rows)} cycles)"
        ),
        "license": "Public Domain (NASA Open Data)",
        "num_windows": str(len(projected)),
        "num_signals": str(len(kept_cols)),
        "healthy_window_end": str(HEALTHY_WINDOW_END),
        "channels": ",".join(kept_names),
        "sensor_columns_kept": ",".join(str(c + 1 - 5) for c in kept_cols),  # sensor index 1..21
        "sensor_columns_dropped": (
            ",".join(str(c + 1 - 5) for c in dropped_cols) if dropped_cols else "(none)"
        ),
        "projection_law": (
            "z_residual = (value - baseline_mean) / baseline_sample_stddev; "
            "baseline computed over the first healthy_window_end cycles; "
            "sample stddev uses Bessel correction (n-1)"
        ),
        "baseline_stddev_drop_threshold": f"{STD_DROP_THRESHOLD:.0e}",
        "decimal_places": str(DECIMALS),
        "notes": (
            f"Per-engine z-score residual projection of NASA C-MAPSS {SELECTED_VARIANT} "
            f"unit {SELECTED_UNIT} against its own first {HEALTHY_WINDOW_END} healthy "
            "cycles. DSFB interprets the structural residual evidence; the audit "
            "does NOT claim a fault diagnosis or a remaining-useful-life prediction."
        ),
    }

    sys.stdout.buffer.write(format_tsv(projected, metadata))


if __name__ == "__main__":
    main()
