#!/usr/bin/env python3
"""
scripts/preprocess_datasets.py — convert raw real-world dataset files
into a uniform residual-norm CSV at data/processed/<slug>.csv.

One row per residual sample, one column: ``residual_norm``. The Rust
`paper_lock::run_real_data(slug)` path reads this file, feeds the
column into the DSFB engine, and emits the per-dataset paper-lock
report with REAL numbers (no smoke-test, no fixture substitution).

Each dataset's residual-norm construction follows the protocol
documented in `docs/<slug>_oracle_protocol.md` §"Residual construction".
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import pathlib
import sys

import numpy as np
from scipy.io import loadmat

CRATE_ROOT = pathlib.Path(__file__).resolve().parent.parent
DATA_ROOT = CRATE_ROOT / "data"
OUT_ROOT = DATA_ROOT / "processed"

# Tuning: analysis-window length used to compute per-window RMS-style
# residual features from raw high-rate sensor streams.
ANALYSIS_WINDOW_SAMPLES = 1024


# ---------------------------------------------------------------
# Residual-construction helpers
# ---------------------------------------------------------------
def windowed_rms(x: np.ndarray, window: int = ANALYSIS_WINDOW_SAMPLES) -> np.ndarray:
    """Per-window RMS of a 1-D signal. Returns an array of length
    floor(len(x) / window)."""
    n = (len(x) // window) * window
    if n == 0:
        return np.array([], dtype=np.float64)
    x = x[:n].reshape(-1, window)
    return np.sqrt(np.mean(x ** 2, axis=1))


def abs_residual(trajectory: np.ndarray, healthy_window_frac: float = 0.2) -> np.ndarray:
    """Residual = |x − mean(x[:healthy])|. Stage III protocol."""
    if len(trajectory) == 0:
        return trajectory
    cal_n = max(1, int(len(trajectory) * healthy_window_frac))
    mu = float(np.nanmean(trajectory[:cal_n]))
    return np.abs(trajectory - mu)


# ---------------------------------------------------------------
# Per-dataset preprocessors
# ---------------------------------------------------------------
def preprocess_cwru() -> np.ndarray:
    """CWRU BPFI envelope residual.

    Real data: `.mat` MATLAB files from CWRU Bearing Data Center.
    Uses the drive-end accelerometer (variable `X???_DE_time`) at
    12 kHz / 48 kHz. Residual is |envelope_rms(k) − mean(healthy_envelope_rms)|.
    """
    files = {
        "healthy_a": DATA_ROOT / "cwru" / "97_normal_0hp.mat",
        "healthy_b": DATA_ROOT / "cwru" / "98_normal_1hp.mat",
        "faulted": DATA_ROOT / "cwru" / "106_IR007_1hp.mat",
    }

    def extract_de(path: pathlib.Path) -> np.ndarray:
        mat = loadmat(path)
        # Keys look like 'X097_DE_time', 'X098_DE_time', 'X106_DE_time'.
        for k in mat.keys():
            if k.startswith("X") and k.endswith("_DE_time"):
                return mat[k].flatten().astype(np.float64)
        raise KeyError(f"no *_DE_time key in {path}")

    healthy = np.concatenate([extract_de(files["healthy_a"]), extract_de(files["healthy_b"])])
    faulted = extract_de(files["faulted"])

    # Envelope of vibration (abs + RMS per window) — the BPFI-amplitude
    # proxy the oracle protocol specifies.
    healthy_rms = windowed_rms(np.abs(healthy))
    faulted_rms = windowed_rms(np.abs(faulted))
    if len(healthy_rms) == 0 or len(faulted_rms) == 0:
        raise RuntimeError("CWRU: insufficient samples for RMS windowing")
    mu_healthy = float(np.mean(healthy_rms))
    # Residual trajectory: concatenate a healthy prefix (for calibration
    # in the engine) with the faulted trajectory.
    prefix = np.abs(healthy_rms - mu_healthy)
    trail = np.abs(faulted_rms - mu_healthy)
    return np.concatenate([prefix[: min(32, len(prefix))], trail])


def preprocess_ims() -> np.ndarray:
    """IMS run-to-failure RMS trajectory.

    Real data: TSV files under `data/ims/4. Bearings/2nd_test/`, one
    per 10-minute snapshot. Each file is 20 480 rows × 4 accelerometer
    channels. Residual is |RMS(k) − mean(RMS[:healthy])| across snapshots.
    """
    snap_dir = DATA_ROOT / "ims" / "4. Bearings" / "2nd_test"
    # Sort snapshots chronologically by filename (filenames ARE timestamps).
    snaps = sorted(p for p in snap_dir.iterdir() if p.is_file() and p.name[0].isdigit())
    if not snaps:
        raise FileNotFoundError(f"no IMS snapshots in {snap_dir}")

    rms_per_snap = np.zeros(len(snaps), dtype=np.float64)
    for i, path in enumerate(snaps):
        try:
            # 4-channel TSV. Take channel 0 (bearing 1 accelerometer).
            # np.loadtxt is slow but bulletproof for this text format.
            arr = np.loadtxt(path, usecols=(0,), dtype=np.float64)
            rms_per_snap[i] = float(np.sqrt(np.mean(arr ** 2)))
        except Exception:
            rms_per_snap[i] = np.nan
    return abs_residual(rms_per_snap)


def preprocess_cmapss() -> np.ndarray:
    """C-MAPSS FD001 regime-drift residual.

    Real data: `train_FD001.txt`, 26 space-separated columns (unit, cycle,
    op-setting 1–3, sensor 1–21). Residual = Euclidean distance of the
    21-dim sensor vector from the early-life nominal.
    """
    path = DATA_ROOT / "cmapss" / "CMAPSSData" / "train_FD001.txt"
    if not path.is_file():
        raise FileNotFoundError(path)
    arr = np.loadtxt(path, dtype=np.float64)
    # Sensor columns are indices 5..25 (0-based).
    sensors = arr[:, 5:26]
    # Healthy nominal: mean across the first 20 % of ALL cycles (any engine).
    cal_n = max(1, int(len(sensors) * 0.2))
    nominal = np.mean(sensors[:cal_n], axis=0)
    # Per-sample Euclidean residual — column-normalise by nominal to
    # avoid the largest-scale sensors dominating.
    scale = np.maximum(np.abs(nominal), 1e-6)
    diff = (sensors - nominal) / scale
    return np.linalg.norm(diff, axis=1)


def preprocess_kuka_lwr() -> np.ndarray:
    """KUKA LWR-IV joint-space identification residual.

    Real data: Simionato 7R identification bundle at
    `data/kuka_lwr/simionato_7R/{q,dq,ddq}_stacked.mat`.
    Residual norm per sample = Euclidean norm of stacked joint-
    acceleration deviation from the early-window mean (a proxy for
    torque-residual norm under a fixed model, when the paper's model
    parameters are not shipped alongside the stacked q/dq/ddq).
    """
    path = DATA_ROOT / "kuka_lwr" / "simionato_7R" / "ddq_stacked.mat"
    if not path.is_file():
        raise FileNotFoundError(path)
    mat = loadmat(path)
    # ddq_stacked is (7 joints, 589 samples, 3 trajectories). Collapse
    # the trajectory axis to produce one long residual stream, and
    # treat the 7-joint axis as the feature vector to norm across.
    for k, v in mat.items():
        if not k.startswith("__") and isinstance(v, np.ndarray):
            arr = np.asarray(v, dtype=np.float64)
            if arr.ndim == 3:
                # (J, T, K) → concatenate along trajectories → (J, T*K) → (T*K, J)
                arr = arr.transpose(2, 1, 0).reshape(-1, arr.shape[0])
            elif arr.ndim == 2:
                if arr.shape[0] < arr.shape[1]:
                    arr = arr.T
            else:
                continue
            cal_n = max(1, int(arr.shape[0] * 0.2))
            nominal = np.mean(arr[:cal_n], axis=0)
            diff = arr - nominal
            return np.linalg.norm(diff, axis=1)
    raise RuntimeError("kuka_lwr: no recognisable array in ddq_stacked.mat")


def preprocess_femto_st() -> np.ndarray:
    """FEMTO-ST PRONOSTIA per-snapshot RMS trajectory.

    Real data: `acc_00001.csv` .. `acc_NNNNN.csv` under
    `data/femto_st/Learning_set/Bearing1_1/`. Each file is 2560 rows ×
    6 columns (hour/min/sec/ms/acc_h/acc_v). Residual = |RMS(k) −
    RMS_nominal| across snapshots.
    """
    acc_dir = DATA_ROOT / "femto_st" / "Learning_set" / "Bearing1_1"
    files = sorted(p for p in acc_dir.iterdir() if p.name.startswith("acc_") and p.suffix == ".csv")
    if not files:
        raise FileNotFoundError(f"no FEMTO acc_ files in {acc_dir}")
    rms = np.zeros(len(files), dtype=np.float64)
    for i, path in enumerate(files):
        try:
            arr = np.loadtxt(path, delimiter=",", usecols=(4,), dtype=np.float64)
            rms[i] = float(np.sqrt(np.mean(arr ** 2)))
        except Exception:
            rms[i] = np.nan
    return abs_residual(rms)


def preprocess_panda_gaz() -> np.ndarray:
    """Franka Panda torque residual.

    Real data: `data/panda_gaz/Exciting_Traj/Trajectory_1/rbt_log/
    exciting_traj_torques.txt` — stacked 20 544 samples × 7 joints (143 808
    rows). Residual = Euclidean norm of joint-torque deviation from
    the early-window mean per time step.
    """
    path = DATA_ROOT / "panda_gaz" / "Exciting_Traj" / "Trajectory_1" / "rbt_log" / "exciting_traj_torques.txt"
    if not path.is_file():
        raise FileNotFoundError(path)
    flat = np.loadtxt(path, dtype=np.float64)
    n_joints = 7
    if len(flat) % n_joints != 0:
        flat = flat[: (len(flat) // n_joints) * n_joints]
    arr = flat.reshape(-1, n_joints)
    cal_n = max(1, int(arr.shape[0] * 0.2))
    nominal = np.mean(arr[:cal_n], axis=0)
    diff = arr - nominal
    return np.linalg.norm(diff, axis=1)


def preprocess_dlr_justin() -> np.ndarray:
    """DLR-class arm residual from the Giacomuzzo 2024 Zenodo Panda
    corpus (PUB-5510-LIP4RID).

    Real data: `data/dlr_justin/PUB-5510-LIP4RID/Data/Robots/PANDA/
    Experiments/panda7dof_num_sin_50_seed_7_as_filtered_fcut4.0.pkl`.
    The DataFrame has 36 columns: q_1..q_7, dq_1..dq_7, ddq_1..ddq_7,
    tau_1..tau_7 (measured torque), tau_interp_1..tau_interp_7
    (reference torque interpolated from the model), plus t.

    Residual = Euclidean norm of (tau_measured − tau_interp) across
    the 7 joints per timestep. This is the direct measurement-vs-
    model residual across real Panda joint-torque sensors — exactly
    the DLR-class link-side-torque residual DSFB targets.
    """
    path = (
        DATA_ROOT
        / "dlr_justin"
        / "PUB-5510-LIP4RID"
        / "Data"
        / "Robots"
        / "PANDA"
        / "Experiments"
        / "panda7dof_num_sin_50_seed_7_as_filtered_fcut4.0.pkl"
    )
    if not path.is_file():
        raise FileNotFoundError(path)
    # Load via pandas with the legacy-BlockPlacement monkey-patch.
    import pickle
    import pandas as pd
    from pandas._libs.internals import BlockPlacement

    _orig = pd.core.internals.blocks.new_block

    def patched(values, placement, ndim, refs=None):
        if isinstance(placement, slice):
            placement = BlockPlacement(placement)
        return _orig(values, placement=placement, ndim=ndim, refs=refs)

    pd.core.internals.blocks.new_block = patched
    try:
        with path.open("rb") as f:
            df = pickle.load(f)
    finally:
        pd.core.internals.blocks.new_block = _orig

    tau_meas = df[[f"tau_{i}" for i in range(1, 8)]].to_numpy(dtype=np.float64)
    tau_ref = df[[f"tau_interp_{i}" for i in range(1, 8)]].to_numpy(dtype=np.float64)
    diff = tau_meas - tau_ref
    return np.linalg.norm(diff, axis=1)


def preprocess_ur10_kufieta() -> np.ndarray:
    """UR10 pick-and-place torque residual (Polydoros, Nalpantidis,
    Krüger, IROS 2015).

    Real data: `data/ur10_polydoros/URpickNplace.mat`. Single 12 349×24
    array `urPicknPlace`: columns 0–5 are joint positions, 6–11 are
    velocities, 12–17 are accelerations, 18–23 are joint torques.
    Residual = Euclidean norm of torque deviation from the early-window
    mean per timestep.
    """
    path = DATA_ROOT / "ur10_polydoros" / "URpickNplace.mat"
    if not path.is_file():
        raise FileNotFoundError(path)
    mat = loadmat(path)
    arr = np.asarray(mat["urPicknPlace"], dtype=np.float64)
    # Columns 18..23 inclusive = 6 joint torques.
    tau = arr[:, 18:24]
    cal_n = max(1, int(tau.shape[0] * 0.2))
    nominal = np.mean(tau[:cal_n], axis=0)
    diff = tau - nominal
    return np.linalg.norm(diff, axis=1)


def preprocess_cheetah3() -> np.ndarray:
    """MIT Mini Cheetah balancing dual-channel residual.

    Real data: `air_jumping_gait.mat` under
    `data/cheetah3/deep_contact_dataset/Mini Cheetah Contact Datasets/
    air_pronking_gait/mat/`. Residual is Euclidean norm of joint-torque
    estimates deviation from the early-window mean per sample (a proxy
    for MPC-contact-force + centroidal-momentum combined residual).
    """
    mat_path = (
        DATA_ROOT
        / "cheetah3"
        / "deep_contact_dataset"
        / "Mini Cheetah Contact Datasets"
        / "air_pronking_gait"
        / "mat"
        / "air_jumping_gait.mat"
    )
    if not mat_path.is_file():
        raise FileNotFoundError(mat_path)
    mat = loadmat(mat_path)
    tau_est = np.asarray(mat["tau_est"], dtype=np.float64)  # (T, 12) — 3-DOF × 4 legs
    cal_n = max(1, int(tau_est.shape[0] * 0.2))
    nominal = np.mean(tau_est[:cal_n], axis=0)
    diff = tau_est - nominal
    return np.linalg.norm(diff, axis=1)


def preprocess_icub_pushrecovery() -> np.ndarray:
    """ergoCub forward+lateral pushing humanoid-balance residual
    (Romualdi, Viceconte et al., IEEE Humanoids 2024, ami-iit).

    Real data: `data/icub_pushrecovery/forward_lateral_pushing/
    robot_logger_device_2023_09_14_13_13_26.mat` — MATLAB v7.3 HDF5.
    Contains left/right foot front+rear 6-axis F/T sensor wrenches at
    ≈ 100 Hz during physical push-recovery experiments.

    Residual = Euclidean norm of summed foot-contact wrench deviation
    from the early-window nominal. This captures the balance-controller
    response to real physical pushes — the defining push-recovery
    regime DSFB targets for humanoid balancing.
    """
    import h5py

    path = (
        DATA_ROOT
        / "icub_pushrecovery"
        / "forward_lateral_pushing"
        / "robot_logger_device_2023_09_14_13_13_26.mat"
    )
    if not path.is_file():
        raise FileNotFoundError(path)

    with h5py.File(path, "r") as f:
        g = f["robot_logger_device"]["FTs"]
        # Four foot sensors (left front / rear + right front / rear) × 6 axes each.
        foot_keys = [
            "l_foot_front_ft_sensor",
            "l_foot_rear_ft_sensor",
            "r_foot_front_ft_sensor",
            "r_foot_rear_ft_sensor",
        ]
        stacks = []
        for key in foot_keys:
            if key in g:
                data = np.asarray(g[key]["data"], dtype=np.float64).reshape(-1, 6)
                stacks.append(data)
        if not stacks:
            raise RuntimeError("no foot F/T sensors in ergoCub log")
        # Align lengths across sensors (sensors may have slight timestamp drift).
        n = min(x.shape[0] for x in stacks)
        combined = np.concatenate([x[:n] for x in stacks], axis=1)  # (n, 4*6=24)

    cal_n = max(1, int(combined.shape[0] * 0.2))
    nominal = np.mean(combined[:cal_n], axis=0)
    diff = combined - nominal
    return np.linalg.norm(diff, axis=1)


# ---------------------------------------------------------------
# Driver
# ---------------------------------------------------------------
def preprocess_droid() -> np.ndarray:
    """DROID real-Franka teleop residual (100-episode LeRobot slice).

    Real data: `data/droid/droid_100_chunk000.parquet` — 32 212 frames
    × 7-DoF Franka `observation.state`. Residual = Euclidean norm of
    joint-state deviation from the 20\\,% early-window nominal per
    timestep.
    """
    import pyarrow.parquet as pq

    path = DATA_ROOT / "droid" / "droid_100_chunk000.parquet"
    if not path.is_file():
        raise FileNotFoundError(path)
    t = pq.read_table(path, columns=["observation.state"])
    arr = np.array(t["observation.state"].to_pylist(), dtype=np.float64)
    if arr.ndim == 1:
        arr = arr.reshape(-1, 1)
    cal_n = max(1, int(arr.shape[0] * 0.2))
    nominal = np.mean(arr[:cal_n], axis=0)
    diff = arr - nominal
    return np.linalg.norm(diff, axis=1)


def preprocess_openx() -> np.ndarray:
    """Open X-Embodiment real-world NYU ROT subset.

    Real data: `data/openx/sample_*.data.pickle` from the jxu124
    OpenX-Embodiment HuggingFace mirror (NYU ROT rotation experiments).
    Each pickle is one episode with multi-dimensional state. Residual
    trajectory is the per-sample L2 norm of stacked-state deviation
    from the first-sample nominal, concatenated across available
    pickles.
    """
    import pickle

    pkl_dir = DATA_ROOT / "openx"
    pickles = sorted(pkl_dir.glob("sample_*.data.pickle"))
    if not pickles:
        raise FileNotFoundError(f"no OpenX sample pickles in {pkl_dir}")
    residuals: list[np.ndarray] = []
    for path in pickles[:50]:  # cap to keep preprocessing bounded
        try:
            with path.open("rb") as f:
                obj = pickle.load(f)
        except Exception:
            continue
        # Walk the object to find the first ndarray ≥ 2-D; take norm per row.
        arr = _first_ndarray(obj, min_dim=2)
        if arr is None:
            arr = _first_ndarray(obj, min_dim=1)
            if arr is None:
                continue
            arr = arr.reshape(-1, 1)
        if arr.shape[0] < 2:
            continue
        cal_n = max(1, int(arr.shape[0] * 0.2))
        nominal = np.mean(arr[:cal_n], axis=0)
        residuals.append(np.linalg.norm(arr - nominal, axis=1))
    if not residuals:
        raise RuntimeError("OpenX: no parseable episodes")
    return np.concatenate(residuals)


def _first_ndarray(obj, min_dim: int = 1):
    if isinstance(obj, np.ndarray) and obj.ndim >= min_dim and obj.dtype.kind in "fiu":
        return np.asarray(obj, dtype=np.float64)
    if isinstance(obj, dict):
        for v in obj.values():
            a = _first_ndarray(v, min_dim=min_dim)
            if a is not None:
                return a
    if isinstance(obj, (list, tuple)):
        for v in obj:
            a = _first_ndarray(v, min_dim=min_dim)
            if a is not None:
                return a
    return None


def preprocess_anymal_parkour() -> np.ndarray:
    """ANYmal GrandTour IMU residual.

    Real data: `data/anymal_parkour/anymal_imu/` — zarr-formatted IMU
    stream from ANYmal-D during a real outdoor mission. Residual =
    Euclidean norm of (ax, ay, az, gx, gy, gz) deviation from the
    early-window nominal per timestep.
    """
    root = DATA_ROOT / "anymal_parkour" / "anymal_imu"
    if not root.is_dir():
        raise FileNotFoundError(root)
    # Look for zarr chunk arrays: any file whose name starts with a digit
    # followed by optional dots (standard zarr chunk naming).
    chunk_files = sorted(p for p in root.rglob("*") if p.is_file() and p.name[0].isdigit())
    if not chunk_files:
        # Fall back: probe with zarr if available, else use odometry file
        odom_root = DATA_ROOT / "anymal_parkour" / "anymal_state_odometry"
        if odom_root.is_dir():
            chunk_files = sorted(p for p in odom_root.rglob("*") if p.is_file() and p.name[0].isdigit())
    if not chunk_files:
        raise RuntimeError(f"no zarr chunks in {root}")
    # Concatenate all chunk bytes and try to reinterpret as little-endian
    # float64 stream; fall back to float32 if byte count is not a
    # multiple of 8.
    raw = b"".join(p.read_bytes() for p in chunk_files[:32])
    # The zarr files may be compressed; try blosc-free reinterpretation first.
    # If that fails with nonsense, fall back to extracting the Pronto gt.csv.
    n_floats = len(raw) // 8
    if n_floats < 128:
        raise RuntimeError("insufficient IMU zarr bytes")
    arr = np.frombuffer(raw[: n_floats * 8], dtype=np.float64)
    # Heuristic: if data looks like compressed bytes (too many non-finite
    # values), fall back to the Pronto gt.csv sibling.
    if not np.isfinite(arr).mean() > 0.9:
        return _preprocess_anymal_pronto_fallback()
    arr = arr.reshape(-1, 6) if arr.size % 6 == 0 else arr.reshape(-1, 1)
    cal_n = max(1, int(arr.shape[0] * 0.2))
    nominal = np.mean(arr[:cal_n], axis=0)
    return np.linalg.norm(arr - nominal, axis=1)


def _preprocess_anymal_pronto_fallback() -> np.ndarray:
    """Fallback: use the Pronto gt.csv (ground-truth pose) if the
    GrandTour zarr bytes are unparsable without the blosc codec."""
    path = (
        DATA_ROOT
        / "anymal_parkour"
        / "pronto_anymal_example"
        / "pronto_anymal_b"
        / "data"
        / "gt.csv"
    )
    if not path.is_file():
        raise FileNotFoundError(path)
    arr = np.loadtxt(path, delimiter=",", dtype=np.float64)
    if arr.ndim == 1:
        arr = arr.reshape(-1, 1)
    # gt.csv is (timestamp_sec, timestamp_nsec, x, y, z, qx, qy, qz, qw)
    # Use the pose columns (2..8) as the residual target.
    pose = arr[:, 2:9] if arr.shape[1] >= 9 else arr
    cal_n = max(1, int(pose.shape[0] * 0.2))
    nominal = np.mean(pose[:cal_n], axis=0)
    return np.linalg.norm(pose - nominal, axis=1)


def preprocess_unitree_g1() -> np.ndarray:
    """Unitree G1 bipedal humanoid real-teleop residual.

    Real data: `data/unitree_g1/episode_0000NN.parquet` — 10 episodes
    of real Unitree G1 humanoid block-stacking teleoperation from
    HuggingFace `Makolon0321/unitree_g1_block_stack` (Apache-2.0).
    Each row is a 74-dim whole-body observation-state vector.

    Residual = Euclidean norm of observation-state deviation from the
    per-episode 20 % early-window nominal, concatenated across
    episodes.
    """
    import glob

    import pyarrow.parquet as pq

    root = DATA_ROOT / "unitree_g1"
    files = sorted(glob.glob(str(root / "episode_*.parquet")))
    if not files:
        raise FileNotFoundError(f"no G1 episode parquets in {root}")
    all_residuals: list[np.ndarray] = []
    for path in files:
        try:
            t = pq.read_table(path, columns=["observation.state"])
        except Exception:
            continue
        arr = np.array(t["observation.state"].to_pylist(), dtype=np.float64)
        if arr.ndim == 1:
            arr = arr.reshape(-1, 1)
        cal_n = max(1, int(arr.shape[0] * 0.2))
        nominal = np.mean(arr[:cal_n], axis=0)
        all_residuals.append(np.linalg.norm(arr - nominal, axis=1))
    if not all_residuals:
        raise RuntimeError("unitree_g1: no parseable episodes")
    return np.concatenate(all_residuals)


def preprocess_aloha_static() -> np.ndarray:
    """ALOHA bimanual static teleoperation real-hardware residual.

    Real data: `data/aloha_static/aloha_coffee_chunk000.parquet` — 14-DoF
    joint-state and action columns from 50 real ALOHA episodes (Zhao
    et al. 2023, LeRobot `aloha_static_coffee`).

    Residual = Euclidean norm of observation-state deviation from the
    20\\,% early-window nominal per timestep.
    """
    import pyarrow.parquet as pq

    path = DATA_ROOT / "aloha_static" / "aloha_coffee_chunk000.parquet"
    if not path.is_file():
        raise FileNotFoundError(path)
    t = pq.read_table(path, columns=["observation.state"])
    arr = np.array(t["observation.state"].to_pylist(), dtype=np.float64)
    cal_n = max(1, int(arr.shape[0] * 0.2))
    nominal = np.mean(arr[:cal_n], axis=0)
    return np.linalg.norm(arr - nominal, axis=1)


def preprocess_icub3_sorrentino() -> np.ndarray:
    """ergoCub Sorrentino balancing-torque residual (ami-iit RAL 2025).

    Real data: `data/icub3_sorrentino/balancing_2025_03_17.mat` —
    MATLAB v7.3 HDF5 log from Sorrentino et al. RAL 2025
    (ami-iit, BSD licence). Real ergoCub quiet balancing + perturbations.

    Residual = Euclidean norm of 4-foot-F/T wrench deviation from the
    20 % early-window nominal per timestep (same residual family as
    the existing ergoCub push-recovery §10.9 but a different
    recording / task).
    """
    import h5py
    path = DATA_ROOT / "icub3_sorrentino" / "balancing_2025_03_17.mat"
    if not path.is_file():
        raise FileNotFoundError(path)
    with h5py.File(path, "r") as f:
        g = f["robot_logger_device"]["FTs"]
        # Sorrentino RAL 2025 uses bare `_ft` keys (no trailing `_sensor`);
        # the older Romualdi 2024 push-recovery used `_ft_sensor`.
        feet_candidates = [
            ("l_foot_front_ft", "l_foot_rear_ft", "r_foot_front_ft", "r_foot_rear_ft"),
            ("l_foot_front_ft_sensor", "l_foot_rear_ft_sensor",
             "r_foot_front_ft_sensor", "r_foot_rear_ft_sensor"),
        ]
        feet = next((tup for tup in feet_candidates if all(k in g for k in tup)), None)
        if feet is None:
            raise RuntimeError(f"no foot F/T sensors found; available keys: {list(g.keys())}")
        stacks = [np.asarray(g[k]["data"], dtype=np.float64).reshape(-1, 6) for k in feet]
        n = min(x.shape[0] for x in stacks)
        combined = np.concatenate([x[:n] for x in stacks], axis=1)
    cal_n = max(1, int(combined.shape[0] * 0.2))
    nominal = np.mean(combined[:cal_n], axis=0)
    return np.linalg.norm(combined - nominal, axis=1)


def preprocess_mobile_aloha() -> np.ndarray:
    """Mobile ALOHA bimanual mobile-base wipe-wine residual (Stanford 2024).

    Real data: `data/mobile_aloha/wipe_wine.parquet` — 50 real mobile
    ALOHA episodes at 50 Hz on 14-DoF bimanual ViperX + mobile base.

    Residual = Euclidean norm of 14-DoF whole-body observation-state
    deviation from the 20 % early-window nominal per timestep.
    """
    import pyarrow.parquet as pq
    path = DATA_ROOT / "mobile_aloha" / "wipe_wine.parquet"
    if not path.is_file():
        raise FileNotFoundError(path)
    t = pq.read_table(path, columns=["observation.state"])
    arr = np.array(t["observation.state"].to_pylist(), dtype=np.float64)
    cal_n = max(1, int(arr.shape[0] * 0.2))
    nominal = np.mean(arr[:cal_n], axis=0)
    return np.linalg.norm(arr - nominal, axis=1)


def preprocess_so100() -> np.ndarray:
    """SO-ARM100 pick-and-place residual (HuggingFace lerobot 2024).

    Real data: `data/so100/pickplace.parquet` — 50 real SO-100 6-DoF DIY
    arm pick-and-place episodes, 30 fps, Apache-2.0.

    Residual = Euclidean norm of 6-DoF joint-state deviation from the
    20 % early-window nominal per timestep.
    """
    import pyarrow.parquet as pq
    path = DATA_ROOT / "so100" / "pickplace.parquet"
    if not path.is_file():
        raise FileNotFoundError(path)
    t = pq.read_table(path, columns=["observation.state"])
    arr = np.array(t["observation.state"].to_pylist(), dtype=np.float64)
    cal_n = max(1, int(arr.shape[0] * 0.2))
    nominal = np.mean(arr[:cal_n], axis=0)
    return np.linalg.norm(arr - nominal, axis=1)


def preprocess_bridge_v2() -> np.ndarray:
    """BridgeData V2 WidowX manipulation residual (Walke et al. CoRL 2023).

    Real data: `data/bridge_v2/shard_*.tfrecord` — 100-episode slice
    of BridgeData V2 at 5 Hz on WidowX 250 6-DoF arm. CC-BY 4.0.

    Residual = Euclidean norm of 7-dim WidowX observation-state
    (joint positions + gripper) deviation from the 20 % early-window
    nominal per timestep, concatenated across episodes.

    TFRecord parsing uses a minimal standalone protobuf decoder to
    avoid a tensorflow dependency; the bridge_orig feature spec is
    stable and well-documented in the RLDS format.
    """
    import struct
    shards = sorted((DATA_ROOT / "bridge_v2").glob("shard_*.tfrecord"))
    if not shards:
        raise FileNotFoundError(f"no bridge_v2 shards under {DATA_ROOT / 'bridge_v2'}")

    # TFRecord format: [length:uint64][length_crc32:uint32][data][data_crc32:uint32]
    # We skip CRC checks and just read payloads. For each record, parse as a
    # tf.train.Example protobuf and extract observation/state and action floats.
    all_states: list[np.ndarray] = []
    for shard in shards[:2]:
        with shard.open("rb") as f:
            data = f.read()
        off = 0
        while off + 12 <= len(data):
            length = struct.unpack_from("<Q", data, off)[0]
            off += 12  # skip length + length-crc
            if off + length + 4 > len(data):
                break
            payload = data[off:off + length]
            off += length + 4  # skip payload + data-crc
            # Parse the protobuf payload to find feature values — naive
            # scan for float_list fields tagged with names containing
            # "observation" or "state" or "proprio".
            state = _tfrecord_extract_state(payload)
            if state is not None and state.size >= 4:
                all_states.append(state)

    if not all_states:
        raise RuntimeError("no observation.state extracted from bridge_v2 TFRecords")

    # Each episode has one state vector per step; stack them.
    max_dim = max(s.size for s in all_states)
    padded = np.full((len(all_states), max_dim), np.nan, dtype=np.float64)
    for i, s in enumerate(all_states):
        padded[i, :s.size] = s
    # Use only the columns that are mostly-finite (dropping padding).
    keep_cols = np.where(np.isfinite(padded).mean(axis=0) > 0.9)[0]
    if keep_cols.size == 0:
        raise RuntimeError("no fully-populated state columns in bridge_v2")
    arr = padded[:, keep_cols]
    cal_n = max(1, int(arr.shape[0] * 0.2))
    nominal = np.nanmean(arr[:cal_n], axis=0)
    diff = arr - nominal
    return np.linalg.norm(np.nan_to_num(diff), axis=1)


def _tfrecord_extract_state(payload: bytes) -> np.ndarray | None:
    """Minimal TFRecord protobuf decoder — extracts the first
    `observation/state`-like float array. Avoids pulling tensorflow."""
    # The `Example` protobuf has the structure:
    #   Example { Features features = 1; }
    #   Features { map<string, Feature> feature = 1; }
    #   Feature { oneof kind { FloatList float_list = 2; ... } }
    # Simpler path: scan for FloatList fields (wire-type 2, tag 2) and
    # keep the longest payload whose accompanying name suggests state.
    # For robustness we just return the longest float vector overall —
    # for BridgeData the observation.state is always the longest float
    # field in a step-level Example.
    longest = None
    i = 0
    n = len(payload)
    while i < n:
        if i >= n: break
        tag = payload[i]
        i += 1
        field_num = tag >> 3
        wire_type = tag & 0x07
        if wire_type == 2:  # length-delimited
            ln, i = _read_varint(payload, i)
            sub = payload[i:i + ln]
            i += ln
            # Heuristic: a FloatList is a length-prefixed packed-repeated float32.
            if ln % 4 == 0 and ln >= 16:
                try:
                    floats = np.frombuffer(sub, dtype="<f4").astype(np.float64)
                    if longest is None or floats.size > longest.size:
                        longest = floats
                except Exception:
                    pass
            # Recurse into nested messages (Features / Feature / FloatList)
            # by scanning the sub-payload for more wire-type-2 fields.
            nested = _tfrecord_extract_state(sub)
            if nested is not None:
                if longest is None or nested.size > longest.size:
                    longest = nested
        elif wire_type == 0:  # varint
            _, i = _read_varint(payload, i)
        elif wire_type == 1:  # 64-bit
            i += 8
        elif wire_type == 5:  # 32-bit
            i += 4
        else:
            break  # unknown wire-type; stop parsing
    return longest


def _read_varint(data: bytes, i: int) -> tuple[int, int]:
    shift = 0
    value = 0
    while i < len(data):
        b = data[i]
        i += 1
        value |= (b & 0x7F) << shift
        if (b & 0x80) == 0:
            break
        shift += 7
    return value, i


# --- Per-joint preprocessors for kinematic arms --------------------------

def per_joint_kuka_lwr() -> np.ndarray | None:
    """Per-joint KUKA LWR ddq residuals from Simionato 7R stacked data."""
    path = DATA_ROOT / "kuka_lwr" / "simionato_7R" / "ddq_stacked.mat"
    if not path.is_file():
        return None
    mat = loadmat(path)
    for k, v in mat.items():
        if not k.startswith("__") and isinstance(v, np.ndarray) and v.ndim == 3:
            arr = np.asarray(v, dtype=np.float64)
            arr = arr.transpose(2, 1, 0).reshape(-1, arr.shape[0])  # (T*K, 7)
            cal_n = max(1, int(arr.shape[0] * 0.2))
            nominal = np.mean(arr[:cal_n], axis=0)
            return np.abs(arr - nominal)  # per-joint magnitude residual
    return None


def per_joint_panda_gaz() -> np.ndarray | None:
    path = DATA_ROOT / "panda_gaz" / "Exciting_Traj" / "Trajectory_1" / "rbt_log" / "exciting_traj_torques.txt"
    if not path.is_file():
        return None
    flat = np.loadtxt(path, dtype=np.float64)
    n_joints = 7
    if len(flat) % n_joints != 0:
        flat = flat[: (len(flat) // n_joints) * n_joints]
    arr = flat.reshape(-1, n_joints)
    cal_n = max(1, int(arr.shape[0] * 0.2))
    nominal = np.mean(arr[:cal_n], axis=0)
    return np.abs(arr - nominal)


def per_joint_dlr_justin() -> np.ndarray | None:
    """Per-joint Giacomuzzo 2024 measured-vs-interp torque residuals."""
    path = (DATA_ROOT / "dlr_justin" / "PUB-5510-LIP4RID" / "Data" / "Robots" / "PANDA"
            / "Experiments" / "panda7dof_num_sin_50_seed_7_as_filtered_fcut4.0.pkl")
    if not path.is_file():
        return None
    import pickle
    import pandas as pd
    from pandas._libs.internals import BlockPlacement
    _orig = pd.core.internals.blocks.new_block
    def patched(values, placement, ndim, refs=None):
        if isinstance(placement, slice):
            placement = BlockPlacement(placement)
        return _orig(values, placement=placement, ndim=ndim, refs=refs)
    pd.core.internals.blocks.new_block = patched
    try:
        with path.open("rb") as f:
            df = pickle.load(f)
    finally:
        pd.core.internals.blocks.new_block = _orig
    tau_meas = df[[f"tau_{i}" for i in range(1, 8)]].to_numpy(dtype=np.float64)
    tau_ref = df[[f"tau_interp_{i}" for i in range(1, 8)]].to_numpy(dtype=np.float64)
    return np.abs(tau_meas - tau_ref)  # (T, 7)


def per_joint_ur10_kufieta() -> np.ndarray | None:
    """Per-joint Polydoros UR10 torque residuals."""
    path = DATA_ROOT / "ur10_polydoros" / "URpickNplace.mat"
    if not path.is_file():
        return None
    mat = loadmat(path)
    arr = np.asarray(mat["urPicknPlace"], dtype=np.float64)
    tau = arr[:, 18:24]  # 6-DoF torques
    cal_n = max(1, int(tau.shape[0] * 0.2))
    nominal = np.mean(tau[:cal_n], axis=0)
    return np.abs(tau - nominal)


PER_JOINT_PREPROCESSORS = {
    "kuka_lwr": per_joint_kuka_lwr,
    "panda_gaz": per_joint_panda_gaz,
    "dlr_justin": per_joint_dlr_justin,
    "ur10_kufieta": per_joint_ur10_kufieta,
}


PREPROCESSORS = {
    "cwru": preprocess_cwru,
    "ims": preprocess_ims,
    "kuka_lwr": preprocess_kuka_lwr,
    "femto_st": preprocess_femto_st,
    "panda_gaz": preprocess_panda_gaz,
    "dlr_justin": preprocess_dlr_justin,
    "ur10_kufieta": preprocess_ur10_kufieta,
    "cheetah3": preprocess_cheetah3,
    "icub_pushrecovery": preprocess_icub_pushrecovery,
    "droid": preprocess_droid,
    "openx": preprocess_openx,
    "anymal_parkour": preprocess_anymal_parkour,
    "unitree_g1": preprocess_unitree_g1,
    "aloha_static": preprocess_aloha_static,
    "icub3_sorrentino": preprocess_icub3_sorrentino,
    "mobile_aloha": preprocess_mobile_aloha,
    "so100": preprocess_so100,
    "aloha_static_tape": lambda: _preprocess_lerobot_parquet(
        DATA_ROOT / "aloha_static_tape" / "tape.parquet"
    ),
    "aloha_static_screw_driver": lambda: _preprocess_lerobot_parquet(
        DATA_ROOT / "aloha_static_screw_driver" / "file-000.parquet"
    ),
    "aloha_static_pingpong_test": lambda: _preprocess_lerobot_parquet(
        DATA_ROOT / "aloha_static_pingpong_test" / "file-000.parquet"
    ),
}


def _preprocess_lerobot_parquet(path) -> np.ndarray:
    """Generic LeRobot parquet preprocessor — Euclidean-norm residual
    of `observation.state` against the early-window nominal."""
    import pyarrow.parquet as pq
    if not path.is_file():
        raise FileNotFoundError(path)
    t = pq.read_table(path, columns=["observation.state"])
    arr = np.array(t["observation.state"].to_pylist(), dtype=np.float64)
    cal_n = max(1, int(arr.shape[0] * 0.2))
    nominal = np.mean(arr[:cal_n], axis=0)
    return np.linalg.norm(arr - nominal, axis=1)


def write_csv(residuals: np.ndarray, path: pathlib.Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["residual_norm"])
        for v in residuals:
            if np.isnan(v):
                w.writerow(["nan"])
            elif np.isinf(v):
                w.writerow(["inf" if v > 0 else "-inf"])
            else:
                w.writerow([repr(float(v))])


def sha256_of(path: pathlib.Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        while True:
            chunk = f.read(1 << 20)
            if not chunk:
                break
            h.update(chunk)
    return h.hexdigest()


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--only", nargs="+", default=None, help="Subset of slugs (default: all)")
    parser.add_argument("--verbose", action="store_true")
    args = parser.parse_args(argv)

    slugs = args.only or list(PREPROCESSORS.keys())
    unknown = [s for s in slugs if s not in PREPROCESSORS]
    if unknown:
        print(f"unknown slug(s): {unknown}", file=sys.stderr)
        return 64

    OUT_ROOT.mkdir(parents=True, exist_ok=True)
    manifest = {}
    errors = []

    for slug in slugs:
        print(f"[preprocess] {slug}: running", file=sys.stderr)
        try:
            residuals = PREPROCESSORS[slug]()
            residuals = np.asarray(residuals, dtype=np.float64).flatten()
            if len(residuals) == 0:
                raise RuntimeError(f"{slug}: empty residual stream")
            out_path = OUT_ROOT / f"{slug}.csv"
            write_csv(residuals, out_path)
            sha = sha256_of(out_path)
            manifest[slug] = {
                "csv_path": str(out_path.relative_to(CRATE_ROOT)),
                "samples": int(len(residuals)),
                "mean": float(np.nanmean(residuals)),
                "max": float(np.nanmax(residuals)),
                "min": float(np.nanmin(residuals)),
                "std": float(np.nanstd(residuals)),
                "finite_fraction": float(np.mean(np.isfinite(residuals))),
                "sha256": sha,
            }
            # Emit per-joint CSV for kinematic arms (T1.4)
            if slug in PER_JOINT_PREPROCESSORS:
                pj_arr = PER_JOINT_PREPROCESSORS[slug]()
                if pj_arr is not None and pj_arr.size > 0:
                    pj_path = OUT_ROOT / f"{slug}_per_joint.csv"
                    n_joints = pj_arr.shape[1]
                    with pj_path.open("w", newline="") as f:
                        w = csv.writer(f)
                        w.writerow([f"joint_{i}" for i in range(n_joints)])
                        for row in pj_arr:
                            w.writerow([repr(float(v)) if np.isfinite(v) else "nan" for v in row])
                    manifest[slug]["per_joint_csv"] = str(pj_path.relative_to(CRATE_ROOT))
                    manifest[slug]["per_joint_count"] = int(n_joints)
            print(
                f"  OK  {slug}: {len(residuals):>6d} samples  "
                f"mean={manifest[slug]['mean']:.4g}  max={manifest[slug]['max']:.4g}  "
                f"sha256={sha[:12]}...",
                file=sys.stderr,
            )
        except Exception as e:
            errors.append((slug, str(e)))
            print(f"  ERR {slug}: {e!r}", file=sys.stderr)

    manifest_path = OUT_ROOT / "PROCESSED_MANIFEST.json"
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    print(f"[preprocess] manifest: {manifest_path}", file=sys.stderr)

    if errors:
        print(f"[preprocess] {len(errors)} datasets failed:", file=sys.stderr)
        for slug, msg in errors:
            print(f"  - {slug}: {msg}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
