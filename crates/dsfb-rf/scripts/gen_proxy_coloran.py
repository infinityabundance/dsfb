#!/usr/bin/env python3
"""
ColO-RAN synthetic-proxy slice generator.

Schema-preserving CSV stand-in (<=2 MB) for
github.com/wineslab/colosseum-oran-coloran-dataset when the public mirror
is unreachable. KPI columns mirror the published schema; numbers are
drawn from the crate's 3-slice O-RAN KPI model.

This is a non-IQ companion annex: ColO-RAN emits Distributed-Unit KPI
traces, not raw baseband. DSFB is never fed ColO-RAN data — the slice
is a contextual exhibit only.
"""

from __future__ import annotations

import json
import time
from pathlib import Path

import numpy as np

N_ROWS = 20000   # ~1 MB of CSV
COLUMNS = [
    "timestamp_ms",
    "slice_id",
    "tx_brate_downlink_mbps",
    "ratio_granted_req",
    "n_prb_granted",
    "active_ues",
]


def generate(
    out_csv: Path,
    out_meta: Path,
    rng: np.random.Generator,
    n_rows: int = N_ROWS,
) -> None:
    n_rows = int(n_rows)
    timestamps = np.arange(n_rows, dtype=np.int64) * 10  # 10 ms tick
    slices = rng.choice([0, 1, 2], size=n_rows, p=[0.4, 0.35, 0.25])  # eMBB / MTC / URLLC

    base_thr = np.where(slices == 0, 20.0, np.where(slices == 1, 5.0, 2.0))
    thr = np.abs(base_thr + rng.standard_normal(n_rows) * 1.5)
    ratio = np.clip(0.75 + 0.15 * rng.standard_normal(n_rows), 0.1, 1.0)
    prbs = np.clip((thr / 0.5 + rng.standard_normal(n_rows) * 5.0), 1.0, 100.0).astype(int)
    ues = np.clip(4 + rng.integers(-2, 3, size=n_rows), 1, 10)

    lines = [",".join(COLUMNS)]
    for i in range(n_rows):
        lines.append(
            f"{timestamps[i]},{int(slices[i])},{thr[i]:.3f},{ratio[i]:.3f},{int(prbs[i])},{int(ues[i])}"
        )
    out_csv.write_text("\n".join(lines) + "\n", encoding="utf-8")

    meta = {
        "dsfb_rf:provenance": "synthetic-proxy",
        "dsfb_rf:proxy_for": "wineslab/colosseum-oran-coloran-dataset (KPI CSVs)",
        "dsfb_rf:source_model": "Three-slice eMBB/MTC/URLLC KPI proxy; not from Colosseum.",
        "dsfb_rf:generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "columns": COLUMNS,
        "rows": n_rows,
        "notice": "[SYNTHETIC PROXY] Non-IQ companion annex; never fed to the DSFB FSM.",
    }
    out_meta.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
