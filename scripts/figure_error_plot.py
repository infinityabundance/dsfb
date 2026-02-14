#!/usr/bin/env python3

import pandas as pd
import matplotlib.pyplot as plt
from mpl_toolkits.axes_grid1.inset_locator import inset_axes, mark_inset

CSV_PATH = "sim.csv"
X_ZOOM_MIN, X_ZOOM_MAX = 3.6, 4.0

required_cols = [
    "t", "phi_true", "phi_mean", "phi_freqonly", "phi_dsfb",
    "err_mean", "err_freqonly", "err_dsfb", "w2", "s2"
]

df = pd.read_csv(CSV_PATH)
missing = [c for c in required_cols if c not in df.columns]
if missing:
    raise ValueError(f"Missing columns in {CSV_PATH}: {missing}")

plt.rcParams.update({
    "font.family": "serif",
    "font.size": 8,
    "axes.labelsize": 8,
    "axes.titlesize": 8,
    "legend.fontsize": 7,
    "xtick.labelsize": 7,
    "ytick.labelsize": 7,
    "lines.linewidth": 0.9,
    "axes.linewidth": 0.6,
    "xtick.major.width": 0.6,
    "ytick.major.width": 0.6,
})

fig, ax = plt.subplots(figsize=(3.5, 2.4), constrained_layout=True)

ax.plot(df["t"], df["err_mean"], label="err_mean")
ax.plot(df["t"], df["err_freqonly"], label="err_freqonly")
ax.plot(df["t"], df["err_dsfb"], label="err_dsfb")

ax.set_xlabel("t")
ax.set_ylabel("Absolute error")
ax.grid(True, linewidth=0.3, alpha=0.4)
ax.legend(loc="upper left", frameon=False)

zoom_df = df[(df["t"] >= X_ZOOM_MIN) & (df["t"] <= X_ZOOM_MAX)]
if zoom_df.empty:
    raise ValueError(f"No data in zoom window [{X_ZOOM_MIN}, {X_ZOOM_MAX}]")

ymin = zoom_df[["err_mean", "err_freqonly", "err_dsfb"]].min().min()
ymax = zoom_df[["err_mean", "err_freqonly", "err_dsfb"]].max().max()
ypad = (ymax - ymin) * 0.08 if ymax > ymin else 0.01

axins = inset_axes(ax, width="45%", height="45%", loc="upper right", borderpad=0.8)
axins.plot(df["t"], df["err_mean"], linewidth=0.8)
axins.plot(df["t"], df["err_freqonly"], linewidth=0.8)
axins.plot(df["t"], df["err_dsfb"], linewidth=0.8)

axins.set_xlim(X_ZOOM_MIN, X_ZOOM_MAX)
axins.set_ylim(ymin - ypad, ymax + ypad)
axins.grid(True, linewidth=0.25, alpha=0.35)
axins.tick_params(labelsize=6, width=0.5, length=2)
for spine in axins.spines.values():
    spine.set_linewidth(0.5)

mark_inset(ax, axins, loc1=2, loc2=4, fc="none", ec="0.4", lw=0.5)

fig.savefig("figure_error.pdf", bbox_inches="tight")
fig.savefig("figure_error.png", dpi=300, bbox_inches="tight")
