#!/usr/bin/env python3

import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
from pathlib import Path

ZOOM_MIN, ZOOM_MAX = 3.6, 4.0
IMPULSE_MIN, IMPULSE_MAX = 3.0, 4.0

REQUIRED_COLUMNS = [
    "t", "phi_true", "phi_mean", "phi_freqonly", "phi_dsfb",
    "err_mean", "err_freqonly", "err_dsfb", "w2", "s2"
]

def resolve_csv_path() -> str:
    run_candidates = sorted(Path("output-dsfb").glob("*/sim-dsfb.csv"))
    if run_candidates:
        return str(run_candidates[-1])

    run_candidates = sorted(Path("output-dsfb").glob("*/sim.csv"))
    if run_candidates:
        return str(run_candidates[-1])

    static_candidates = [
        Path("output-dsfb/sim-dsfb.csv"),
        Path("sim-dsfb.csv"),
        Path("output-dsfb/sim.csv"),
        Path("sim.csv"),
        Path("out/sim.csv"),
    ]
    for candidate in static_candidates:
        if candidate.exists():
            return str(candidate)

    raise FileNotFoundError(
        "Could not find DSFB simulation CSV. Expected sim-dsfb.csv or output-dsfb/<timestamp>/sim-dsfb.csv."
    )

CSV_PATH = resolve_csv_path()

df = pd.read_csv(CSV_PATH)

missing = [c for c in REQUIRED_COLUMNS if c not in df.columns]
if missing:
    raise ValueError(f"Missing required columns in {CSV_PATH}: {missing}")


def rms(x: pd.Series) -> float:
    a = x.to_numpy(dtype=float)
    return float(np.sqrt(np.mean(a ** 2)))


impulse_mask = (df["t"] >= IMPULSE_MIN) & (df["t"] <= IMPULSE_MAX)
if not impulse_mask.any():
    raise ValueError(f"No samples in impulse window [{IMPULSE_MIN}, {IMPULSE_MAX}]")

stats = {
    "mean": {
        "rms": rms(df["err_mean"]),
        "peak_impulse": float(df.loc[impulse_mask, "err_mean"].max()),
    },
    "freqonly": {
        "rms": rms(df["err_freqonly"]),
        "peak_impulse": float(df.loc[impulse_mask, "err_freqonly"].max()),
    },
    "dsfb": {
        "rms": rms(df["err_dsfb"]),
        "peak_impulse": float(df.loc[impulse_mask, "err_dsfb"].max()),
    },
}

print("Summary Statistics")
print("==================")
print(f"Impulse window: t in [{IMPULSE_MIN}, {IMPULSE_MAX}]")
print(f"{'method':<10}{'RMS error':>14}{'Peak impulse error':>22}")
for m in ("mean", "freqonly", "dsfb"):
    print(f"{m:<10}{stats[m]['rms']:>14.6f}{stats[m]['peak_impulse']:>22.6f}")

# 1) Estimation error vs time
fig1, ax1 = plt.subplots(figsize=(10, 4.8))
ax1.plot(df["t"], df["err_mean"], label="err_mean", linewidth=1.8)
ax1.plot(df["t"], df["err_freqonly"], label="err_freqonly", linewidth=1.8)
ax1.plot(df["t"], df["err_dsfb"], label="err_dsfb", linewidth=1.8)
ax1.set_title("Estimation Error vs Time")
ax1.set_xlabel("t")
ax1.set_ylabel("Absolute Error")
ax1.grid(True, alpha=0.3)
ax1.legend()
fig1.tight_layout()
fig1.savefig("estimation_error_vs_time.png", dpi=300, bbox_inches="tight")

# 2) Zoomed-in estimation error
fig2, ax2 = plt.subplots(figsize=(10, 4.8))
ax2.plot(df["t"], df["err_mean"], label="err_mean", linewidth=1.8)
ax2.plot(df["t"], df["err_freqonly"], label="err_freqonly", linewidth=1.8)
ax2.plot(df["t"], df["err_dsfb"], label="err_dsfb", linewidth=1.8)
ax2.set_xlim(ZOOM_MIN, ZOOM_MAX)
ax2.set_title(f"Estimation Error (Zoomed), t in [{ZOOM_MIN}, {ZOOM_MAX}]")
ax2.set_xlabel("t")
ax2.set_ylabel("Absolute Error")
ax2.grid(True, alpha=0.3)
ax2.legend()
fig2.tight_layout()
fig2.savefig("estimation_error_zoom.png", dpi=300, bbox_inches="tight")

# 3) Trust weight w2 vs time
fig3, ax3 = plt.subplots(figsize=(10, 4.8))
ax3.plot(df["t"], df["w2"], label="w2", color="tab:blue", linewidth=1.8)
ax3.set_title("Trust Weight w2 vs Time")
ax3.set_xlabel("t")
ax3.set_ylabel("w2")
ax3.grid(True, alpha=0.3)
ax3.legend()
fig3.tight_layout()
fig3.savefig("trust_weight_w2.png", dpi=300, bbox_inches="tight")

# 4) EMA residual s2 vs time
fig4, ax4 = plt.subplots(figsize=(10, 4.8))
ax4.plot(df["t"], df["s2"], label="s2", color="tab:orange", linewidth=1.8)
ax4.set_title("EMA Residual s2 vs Time")
ax4.set_xlabel("t")
ax4.set_ylabel("s2")
ax4.grid(True, alpha=0.3)
ax4.legend()
fig4.tight_layout()
fig4.savefig("ema_residual_s2.png", dpi=300, bbox_inches="tight")

plt.show()
