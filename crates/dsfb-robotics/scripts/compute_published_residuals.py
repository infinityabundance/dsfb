#!/usr/bin/env python3
"""Compute literal published-θ̂ identification residuals.

For datasets where the source paper publishes a parameter vector and a
dynamic-model evaluator that the crate can vendor, this script computes

    r(k) = ‖τ_meas(k) − τ_pred(q(k), q̇(k), q̈(k); θ̂_published)‖

where τ_pred comes from running the published model on the recorded
trajectory. The output is written to `data/processed/<slug>_published.csv`,
which the Rust paper-lock binary prefers over `<slug>.csv` when present.

Currently implemented:
- panda_gaz: vendored Gaz 2019 cpp model under
  `data/panda_gaz/upstream_model/`. Build it via
  `scripts/build_panda_gaz_model.sh` first.

The other three kinematic arms (kuka_lwr, ur10_kufieta, dlr_justin)
continue to use the early-window-nominal proxy residual described in
their respective `data/processed/<slug>.csv`. The paper §10 explicitly
documents this distinction; only panda_gaz is the literal published-θ̂
exemplar at this revision.
"""

from __future__ import annotations

import csv
import subprocess
import sys
from pathlib import Path

import numpy as np

CRATE_ROOT = Path(__file__).resolve().parent.parent
DATA_ROOT = CRATE_ROOT / "data"
PROCESSED_ROOT = DATA_ROOT / "processed"


def panda_gaz_published() -> Path:
    """Compute Gaz-2019-published-model residual norm stream for panda_gaz."""
    model_bin = DATA_ROOT / "panda_gaz" / "upstream_model" / "build" / "compute_tau_pred"
    if not model_bin.is_file():
        raise FileNotFoundError(
            f"missing {model_bin}; run scripts/build_panda_gaz_model.sh first"
        )

    rbt = DATA_ROOT / "panda_gaz" / "Exciting_Traj" / "Trajectory_1" / "rbt_log"
    positions = rbt / "exciting_traj_positions.txt"
    velocity = rbt / "exciting_traj_velocity.txt"
    time = rbt / "exciting_traj_time.txt"
    torques_meas = rbt / "exciting_traj_torques.txt"

    for p in (positions, velocity, time, torques_meas):
        if not p.is_file():
            raise FileNotFoundError(p)

    tau_pred_out = DATA_ROOT / "panda_gaz" / "tau_pred_gaz.txt"
    print(f"  running {model_bin.name} on recorded panda_gaz trajectory...")
    subprocess.run(
        [
            str(model_bin),
            "--positions", str(positions),
            "--velocity", str(velocity),
            "--time", str(time),
            "--out", str(tau_pred_out),
        ],
        check=True,
        stderr=subprocess.PIPE,
    )

    tau_meas = np.loadtxt(torques_meas).reshape(-1, 7)
    tau_pred = np.loadtxt(tau_pred_out)
    if tau_meas.shape != tau_pred.shape:
        raise ValueError(
            f"shape mismatch: tau_meas {tau_meas.shape} vs tau_pred {tau_pred.shape}"
        )
    residual = tau_meas - tau_pred
    res_norm = np.linalg.norm(residual, axis=1)

    out_csv = PROCESSED_ROOT / "panda_gaz_published.csv"
    PROCESSED_ROOT.mkdir(parents=True, exist_ok=True)
    # Match the existing single-column CSV schema used by paper-lock's
    # `load_residual_csv` (header `residual_norm`, then one float per line).
    with out_csv.open("w", newline="") as fh:
        fh.write("residual_norm\n")
        for v in res_norm:
            fh.write(f"{v:.17g}\n")

    print(
        f"  wrote {len(res_norm)} samples to {out_csv.relative_to(CRATE_ROOT)}"
        f"  (mean={res_norm.mean():.3f}, peak={res_norm.max():.3f})"
    )
    return out_csv


PUBLISHED_RESIDUAL_BUILDERS = {
    "panda_gaz": panda_gaz_published,
}


def main() -> int:
    targets = sys.argv[1:] or list(PUBLISHED_RESIDUAL_BUILDERS.keys())
    failures: list[str] = []
    for slug in targets:
        if slug not in PUBLISHED_RESIDUAL_BUILDERS:
            print(f"SKIP {slug}: no published-θ̂ builder registered")
            continue
        print(f"== {slug} ==")
        try:
            PUBLISHED_RESIDUAL_BUILDERS[slug]()
        except Exception as exc:
            print(f"FAIL {slug}: {exc}")
            failures.append(slug)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
