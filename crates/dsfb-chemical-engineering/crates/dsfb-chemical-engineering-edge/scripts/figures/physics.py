"""Figure group G — physics balance witnesses & soft sensors (4 designs).

The first-principles core: mass/energy-balance closure residuals over fully-metered control volumes, the
same witnesses firing on real labelled manipulations (BATADAL/SWaT), a deterministic soft-sensor witness,
and the honest applicability criterion (where the witness fires AND where it correctly stays blind). Balance
witnesses read the committed `data/instrumented/*.witness.csv` (the `normalized` column is the calibrated
closure residual; `label==1` marks the fault window). The soft sensor reads the exported witness JSON.
"""
import json
import os

import numpy as np
import matplotlib.pyplot as plt

from . import style as S

INSTR = os.path.join(S.CRATE, "data", "instrumented")


def _witness(name):
    """Load a balance-witness CSV (time_index, balance_residual, normalized, grammar_state, label)."""
    p = os.path.join(INSTR, f"{name}.witness.csv")
    return S.read_csv_rows(p)


def _plot_closure(ax, rows, title):
    """Plot the calibrated (normalized) closure residual over time, shading the labelled fault window."""
    t = np.array([int(r["time_index"]) for r in rows])
    z = np.array([S.fnum(r["normalized"]) for r in rows])
    lab = np.array([int(S.fnum(r.get("label", "0"))) for r in rows])
    ax.plot(t, z, lw=0.8, color=S.INK)
    # Shade the labelled fault window(s).
    inseg = False
    for i in range(len(t)):
        if lab[i] and not inseg:
            a = t[i]; inseg = True
        elif not lab[i] and inseg:
            ax.axvspan(a, t[i], color=S.WARN, alpha=0.15); inseg = False
    if inseg:
        ax.axvspan(a, t[-1], color=S.WARN, alpha=0.15)
    ax.axhline(0, color="#999", lw=0.6)
    # Robust y-limit: a single startup transient (e.g. CSTR sample 1) can be ~30× the rest and would hide
    # the fault-window signal. Cap the top at the 99th percentile when the max is a clear outlier.
    finite = z[np.isfinite(z)]
    if len(finite):
        p99 = np.percentile(finite, 99.0)
        hi = finite.max()
        top = p99 * 1.15 if (p99 > 0 and hi > 3.0 * p99) else hi * 1.08
        lo = min(finite.min(), 0.0)
        if top > lo:
            ax.set_ylim(lo - abs(lo) * 0.1 - 0.05, top)
    ax.set_title(title, fontsize=9)


def fig_synthetic_closures(run):
    """G46 ★ — calibrated balance-closure residuals for the four synthetic instrumented demonstrators.

    Three-tank (mass), CSTR (energy), CSTH (energy), quadruple-tank (mass): each closes a conservation
    balance over a fully-metered control volume; the closure residual stays near zero under nominal
    operation and breaks within the labelled fault window (orange).
    """
    panels = [("three_tank_instrumented", "Three-tank (mass balance) — leak"),
              ("cstr_instrumented", "CSTR (energy balance) — thermocouple drift"),
              ("csth_instrumented", "CSTH (energy balance) — insulation loss"),
              ("quadruple_tank_instrumented", "Quadruple-tank (mass balance) — tank-1 leak")]
    fig, axes = plt.subplots(2, 2, figsize=(8.0, 5.0))
    any_data = False
    for ax, (name, title) in zip(axes.ravel(), panels):
        rows = _witness(name)
        if rows:
            _plot_closure(ax, rows, title)
            any_data = True
        else:
            ax.text(0.5, 0.5, f"{name}\n(not generated)", ha="center", va="center", fontsize=8); ax.axis("off")
    if not any_data:
        plt.close(fig); S.log("  [G] SKIP physics_synthetic_closures: no witness CSVs"); return
    for ax in axes[-1]:
        ax.set_xlabel("sample index")
    for ax in axes[:, 0]:
        ax.set_ylabel("normalised closure")
    fig.suptitle("Balance-closure residuals — synthetic instrumented demonstrators (orange = labelled fault)", fontsize=10.5)
    fig.tight_layout(rect=[0, 0.03, 1, 0.96])
    S.figure_caption(axes[1, 0], "Closure ≈ 0 when conserved; breaks in the fault window. Witness fires only with a closed, fully-metered control volume.")
    S.save(fig, "physics_synthetic_closures", "G", "data/instrumented/*.witness.csv",
           "Calibrated balance-closure residuals for the four synthetic demonstrators; closure breaks in the labelled fault window.")


def fig_real_closures(run):
    """G47 ★ — balance closure on the REAL benchmarks: BATADAL T1 (mass/volume) and SWaT T101 (mass).

    The most persuasive physics result: the same conservation witness, applied to real labelled benchmark
    data, breaks on the labelled sensor-spoof / inflow-manipulation windows.
    """
    panels = [("batadal_t1_instrumented", "BATADAL T1 — volume balance (real C-Town SCADA)"),
              ("swat_t101_instrumented", "SWaT T101 — mass balance (real iTrust testbed)")]
    fig, axes = plt.subplots(2, 1, figsize=(7.4, 4.6))
    any_data = False
    for ax, (name, title) in zip(axes, panels):
        rows = _witness(name)
        if rows:
            _plot_closure(ax, rows, title)
            any_data = True
        else:
            ax.text(0.5, 0.5, f"{name}\n(gated data — recipe committed, bytes not redistributed)",
                    ha="center", va="center", fontsize=8); ax.axis("off")
    if not any_data:
        plt.close(fig); S.log("  [G] SKIP physics_real_closures: no real witness CSVs"); return
    axes[-1].set_xlabel("sample index")
    for ax in axes:
        ax.set_ylabel("normalised\nclosure", fontsize=8)
    fig.suptitle("Balance closure on real benchmarks (orange = labelled attack/spoof window)", fontsize=10.5)
    fig.tight_layout(rect=[0, 0.03, 1, 0.95])
    S.figure_caption(axes[1], "Same conservation witness on real labelled data; gated bytes not redistributed (recipe committed). " + S.DISCLAIMER["candidate"])
    S.save(fig, "physics_real_closures", "G", "data/instrumented/{batadal_t1,swat_t101}.witness.csv",
           "Balance-closure residuals on real BATADAL/SWaT benchmarks; closure breaks on labelled manipulation windows.")


def fig_softsensor(run):
    """G48 — the soft-sensor witness: measured vs deterministic prediction with the residual + interval band."""
    p = os.path.join(run, "figure_data", "softsensor_witness.json")
    if not os.path.exists(p):
        S.log("  [G] SKIP physics_softsensor: softsensor_witness.json not found"); return
    w = json.load(open(p))
    meas = np.array(w["measured"]); pred = np.array(w["prediction"])
    resid = np.array(w["residual"]); half = np.array(w["interval_half_width"])
    t = np.arange(len(meas))
    fig, ax = plt.subplots(2, 1, figsize=(7.2, 4.4), sharex=True, gridspec_kw={"height_ratios": [2, 1]})
    ax[0].plot(t, meas, color=S.INK, lw=1.1, label="measured")
    ax[0].plot(t, pred, color=S.ACCENT, lw=1.0, ls="--", label="deterministic prediction")
    ax[0].fill_between(t, pred - half, pred + half, color=S.ACCENT, alpha=0.15, label="interval")
    ax[0].legend(frameon=False, fontsize=8, loc="upper right")
    ax[0].set_ylabel(w.get("channel", "target"))
    ax[0].set_title(f"Soft-sensor witness — {w.get('channel','target')} ({w.get('model_family','')})", fontsize=10.5)
    ax[1].axhline(0, color="#999", lw=0.6)
    ax[1].plot(t, resid, color=S.WARN, lw=0.9)
    ax[1].fill_between(t, -half, half, color=S.MUTE, alpha=0.2)
    ax[1].set_ylabel("residual\n(meas − pred)", fontsize=8)
    ax[1].set_xlabel("sample index")
    S.figure_caption(ax[1], "Deterministic (envelope) soft sensor: residual = measured − prediction; interval is the envelope half-width. Advisory.")
    S.save(fig, "physics_softsensor", "G", "figure_data/softsensor_witness.json",
           "Soft-sensor witness: measured vs deterministic prediction with residual and interval band.")


def fig_applicability_criterion(run):
    """G49 — the balance-witness applicability criterion: where it fires AND where it correctly stays blind.

    The honest scope statement: a balance witness fires only with (1) a closed, fully-metered control volume
    AND (2) a fault that makes a conserved quantity appear non-conserved. A panel of correctly-rejected
    datasets (the witness must stay blind) makes the criterion doubly testable. From balance_witness_criterion.md.
    """
    rejected = [
        ("PRONTO multiphase facility", "diverted flow recirculates inside the metered boundary"),
        ("UCI WWTP / BattLeDIM", "incomplete consumer metering — demand swamps the closure"),
        ("RP-1043 chiller", "refrigerant-leak fault respects energy conservation (−4.1 vs −3.7 tons)"),
        ("HAI testbed", "tank outflow unmetered (only the valve command is available)"),
    ]
    fig, ax = plt.subplots(figsize=(8.0, 4.6))
    ax.axis("off")
    ax.text(0.02, 0.95, "Balance-witness applicability criterion", fontsize=12, weight="bold", transform=ax.transAxes)
    ax.text(0.02, 0.86, "A witness fires only when BOTH hold:", fontsize=9.5, transform=ax.transAxes)
    ax.text(0.05, 0.78, "(1) a closed, fully-metered control volume", fontsize=9.5, color=S.OK, transform=ax.transAxes)
    ax.text(0.05, 0.71, "(2) a fault makes a conserved quantity appear non-conserved\n      (a sensor spoof/drift, or a leak crossing the boundary)",
            fontsize=9.5, color=S.OK, transform=ax.transAxes)
    ax.text(0.02, 0.58, "Correctly REJECTED (the witness must stay blind — doubly testable):", fontsize=9.5,
            weight="bold", color=S.WARN, transform=ax.transAxes)
    y = 0.50
    for name, why in rejected:
        ax.text(0.05, y, f"×  {name}", fontsize=9, color=S.WARN, transform=ax.transAxes)
        ax.text(0.40, y, why, fontsize=8.2, color="#333", transform=ax.transAxes)
        y -= 0.11
    ax.text(0.02, 0.03, "Validated: fires on BATADAL T1-inflow + SWaT LIT101 spoof; correctly quiet on the above. "
                        "Physics, not tuning.", fontsize=7.5, style="italic", color="#555", transform=ax.transAxes)
    S.save(fig, "physics_applicability_criterion", "G", "docs/balance_witness_criterion.md",
           "The balance-witness applicability criterion: the two firing conditions and the correctly-rejected datasets.")


def render_all(run):
    """Render every group-G figure (balance witnesses + soft sensor + applicability criterion)."""
    S.log("group G — balance witnesses & soft sensors (physics)")
    fig_synthetic_closures(run)
    fig_real_closures(run)
    fig_softsensor(run)
    fig_applicability_criterion(run)
