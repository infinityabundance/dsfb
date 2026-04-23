#!/usr/bin/env python3
"""
ColO-RAN-commag synthetic-proxy slice generator.

Schema-preserving CSV stand-in for
github.com/wineslab/colosseum-oran-commag-dataset. Adds a scheduling-policy
dimension on top of the ColO-RAN schema. Non-IQ companion annex; never
fed to the DSFB FSM.
"""

from __future__ import annotations

import json
import time
from pathlib import Path

import numpy as np

N_ROWS = 20000
POLICIES = ["round_robin", "proportional_fair", "waterfilling"]
COLUMNS = [
    "timestamp_ms",
    "bs_id",
    "policy",
    "slice_id",
    "tx_brate_downlink_mbps",
    "ratio_granted_req",
    "latency_ms_p95",
]


def generate(
    out_csv: Path,
    out_meta: Path,
    rng: np.random.Generator,
    n_rows: int = N_ROWS,
) -> None:
    n_rows = int(n_rows)
    timestamps = np.arange(n_rows, dtype=np.int64) * 20
    bs_ids = rng.integers(1, 5, size=n_rows)
    policies = rng.choice(POLICIES, size=n_rows)
    slices = rng.choice([0, 1, 2], size=n_rows)
    base_thr = np.where(slices == 0, 25.0, np.where(slices == 1, 6.0, 2.5))
    thr = np.abs(base_thr + rng.standard_normal(n_rows) * 1.5)
    ratio = np.clip(0.75 + 0.15 * rng.standard_normal(n_rows), 0.1, 1.0)
    lat = np.clip(5.0 + rng.exponential(scale=2.5, size=n_rows), 0.5, 100.0)

    lines = [",".join(COLUMNS)]
    for i in range(n_rows):
        lines.append(
            f"{timestamps[i]},{int(bs_ids[i])},{policies[i]},{int(slices[i])},"
            f"{thr[i]:.3f},{ratio[i]:.3f},{lat[i]:.3f}"
        )
    out_csv.write_text("\n".join(lines) + "\n", encoding="utf-8")

    meta = {
        "dsfb_rf:provenance": "synthetic-proxy",
        "dsfb_rf:proxy_for": "wineslab/colosseum-oran-commag-dataset (KPI × policy CSVs)",
        "dsfb_rf:source_model": "Three-policy × three-slice KPI proxy; not from Colosseum.",
        "dsfb_rf:generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "columns": COLUMNS,
        "rows": n_rows,
        "policies": POLICIES,
        "notice": "[SYNTHETIC PROXY] Non-IQ companion annex; never fed to the DSFB FSM.",
    }
    out_meta.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
