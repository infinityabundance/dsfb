#!/usr/bin/env python3
"""Compute the literal Polydoros 2015 / URSim-published-model torque residual
for the UR10 pick-and-place corpus.

For each timestep k in `data/ur10_polydoros/URpickNplace.mat`:

    tau_pred(k) = RNEA(q(k), dq(k), ddq(k); URSim-extracted UR10 params)
    r(k)        = ||tau_meas(k) - tau_pred(k)||

The URSim-extracted UR10 dynamic parameters are vendored at
`data/ur10_polydoros/upstream_model/{ur10.urdf, physical_parameters.yaml,
default_kinematics.yaml}` (sourced from
github.com/ros-industrial/universal_robot under BSD-3-Clause). Polydoros
2015 IROS uses these (or close variants) as the prior for their
joint-space identification, so r(k) here is the literal LS-identification
residual stream the Polydoros pipeline operates on — exactly the
"discarded residual" DSFB structures.

Output: `data/processed/ur10_kufieta_published.csv` (single-column
`residual_norm` CSV mirroring the schema paper-lock consumes).
"""

from __future__ import annotations

import sys
from pathlib import Path

import numpy as np
import pinocchio as pin
from scipy.io import loadmat

CRATE_ROOT = Path(__file__).resolve().parent.parent
URDF_PATH = CRATE_ROOT / "data" / "ur10_polydoros" / "upstream_model" / "ur10.urdf"
RAW_PATH = CRATE_ROOT / "data" / "ur10_polydoros" / "URpickNplace.mat"
OUT_PATH = CRATE_ROOT / "data" / "processed" / "ur10_kufieta_published.csv"


def main() -> int:
    if not URDF_PATH.is_file():
        print(f"ERROR: missing UR10 URDF at {URDF_PATH}")
        return 2
    if not RAW_PATH.is_file():
        print(f"ERROR: missing raw data at {RAW_PATH}")
        return 2

    model = pin.buildModelFromUrdf(str(URDF_PATH))
    data = model.createData()
    print(f"  UR10 pinocchio model loaded: nq={model.nq}, nv={model.nv}")

    mat = loadmat(str(RAW_PATH))
    arr = np.asarray(mat["urPicknPlace"], dtype=np.float64)
    print(f"  URpickNplace shape: {arr.shape}")

    # Layout per the existing preprocess_ur10_kufieta function:
    # cols  0..5  = q  (positions, rad)
    # cols  6..11 = dq (velocities, rad/s)
    # cols 12..17 = ddq (accelerations, rad/s^2)
    # cols 18..23 = tau_meas (joint torques, N·m)
    q_all = arr[:, 0:6]
    dq_all = arr[:, 6:12]
    ddq_all = arr[:, 12:18]
    tau_meas = arr[:, 18:24]
    n = arr.shape[0]

    res_norm = np.empty(n, dtype=np.float64)
    for k in range(n):
        tau_pred = pin.rnea(model, data, q_all[k], dq_all[k], ddq_all[k])
        res_norm[k] = float(np.linalg.norm(tau_meas[k] - tau_pred))

    OUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with OUT_PATH.open("w") as fh:
        fh.write("residual_norm\n")
        for v in res_norm:
            fh.write(f"{v:.17g}\n")

    print(
        f"  wrote {n} samples to {OUT_PATH.relative_to(CRATE_ROOT)}"
        f"  (mean={res_norm.mean():.3f}, peak={res_norm.max():.3f})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
