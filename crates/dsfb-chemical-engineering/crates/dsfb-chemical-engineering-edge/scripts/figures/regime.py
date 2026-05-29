"""Figure group H — regime-conditioned envelopes & case studies (3 designs).

The Tier-3 (newly demonstrated, bounded) result and its honest boundary. Reads the MEASURED
`reports/regime_comparison.csv` written by `dsfb-chem-edge regime-eval` (global vs regime-conditioned
baseline false-positive rate, with the unknown rate shown to confirm episodes are not relabelled to
manufacture a lower FP). The gas-sensor case is shown alongside penicillin precisely because it does NOT
improve — a batch-level label is too coarse — and that boundary is disclosed, not hidden.
"""
import os

import numpy as np
import matplotlib.pyplot as plt

from . import style as S

REGIME_CSV = os.path.join(S.CRATE, "reports", "regime_comparison.csv")


def fig_regime_before_after(run):
    """H50 ★ — global vs regime-conditioned baseline false-positive rate (both datasets, honest).

    Penicillin fed-batch improves (54%→39%) when per-phase envelopes track the non-stationarity; the
    gas-sensor drift case does NOT improve (74%→76%) because its batch-level label is too coarse — shown
    side by side so the bounded scope of the result is explicit. The unknown rate is annotated to confirm
    the honesty signal is preserved (episodes are not relabelled to lower the FP).
    """
    rows = S.read_csv_rows(REGIME_CSV)
    if not rows:
        S.log("  [H] SKIP regime_before_after: reports/regime_comparison.csv not found (run regime-eval)")
        return
    names = [S.disp(r["dataset"]) for r in rows]
    g = [S.fnum(r["baseline_fp_global"]) * 100 for r in rows]
    rg = [S.fnum(r["baseline_fp_regime"]) * 100 for r in rows]
    fig, ax = plt.subplots(figsize=(7.2, 4.0))
    x = np.arange(len(names))
    w = 0.38
    ax.bar(x - w / 2, g, w, color=S.MUTE, label="global envelope")
    ax.bar(x + w / 2, rg, w, color=S.OK, label="regime-conditioned")
    for xi, (a, b, r) in enumerate(zip(g, rg, rows)):
        arrow = "↓ improves" if b < a - 0.5 else ("↑ no help (boundary)" if b > a + 0.5 else "≈")
        col = S.OK if b < a - 0.5 else S.WARN
        ax.text(xi, max(a, b) + 2, f"{a:.0f}% → {b:.0f}%\n{arrow}", ha="center", fontsize=8, color=col)
        ax.text(xi, 2, f"unk {S.fnum(r['unknown_global']):.0%}→{S.fnum(r['unknown_regime']):.0%}",
                ha="center", fontsize=6.8, color="#555")
    ax.set_xticks(x); ax.set_xticklabels(names)
    ax.set_ylabel("baseline false-positive rate (%)")
    ax.set_ylim(0, max(g + rg) + 12)
    ax.set_title("Regime-conditioned vs global envelope (per-phase labels; honest both ways)")
    ax.legend(frameon=False, fontsize=8, loc="upper right")
    S.figure_caption(ax, "Penicillin improves with per-phase envelopes; the coarse gas-sensor batch label does not (boundary disclosed). Unknown rate preserved. Measured via `regime-eval`.")
    S.save(fig, "regime_before_after", "H", "reports/regime_comparison.csv",
           "Global vs regime-conditioned baseline FP: penicillin improves 54→39%; the gas-sensor boundary case does not.")


def fig_recipe_phase_overlay(run):
    """H51 — a batch residual timeline split into recipe phases, each with its own calibrated envelope.

    Penicillin fed-batch is non-stationary by design (growth → production). Overlaying the recipe-phase
    segmentation on the residual makes the case for per-phase envelopes visible: a single global envelope
    mis-flags the phase change itself as a fault.
    """
    rows = [r for r in S.read_csv_rows(S.ds_csv(run, "penicillin_fedbatch", "residual_streams.csv"))
            if r["detector_id"] == "pca_spe_q"]
    if not rows:
        S.log("  [H] SKIP regime_recipe_phase_overlay: no penicillin residual streams"); return
    t = np.array([int(r["time_index"]) for r in rows])
    z = np.array([S.fnum(r["r"]) for r in rows])
    n = len(t)
    # Three regimes (from regime-eval: penicillin has 3) — illustrate as equal recipe-phase thirds.
    bounds = [0, n // 3, 2 * n // 3, n]
    phase_names = ["lag / growth", "exponential growth", "production"]
    phase_cols = [S.CB["skyblue"], S.CB["green"], S.CB["orange"]]
    fig, ax = plt.subplots(figsize=(7.4, 3.6))
    ax.plot(t, z, lw=0.8, color=S.INK)
    for i in range(3):
        a, b = t[bounds[i]], t[min(bounds[i + 1], n - 1)]
        ax.axvspan(a, b, color=phase_cols[i], alpha=0.10)
        ax.text((a + b) / 2, ax.get_ylim()[1] if False else np.nanmax(z) * 0.95, phase_names[i],
                ha="center", fontsize=8, color=phase_cols[i])
    ax.set_xlabel("sample index")
    ax.set_ylabel("residual r (SPE/Q)")
    ax.set_title("Penicillin fed-batch: recipe phases each get their own calibrated envelope")
    S.figure_caption(ax, "Phase segmentation illustrative (3 regimes per regime-eval); a single global envelope mis-flags phase changes as faults.")
    S.save(fig, "regime_recipe_phase_overlay", "H", "penicillin_fedbatch/residual_streams.csv + regime-eval",
           "A batch residual timeline split into recipe phases, motivating per-phase admissibility envelopes.")


def fig_gas_sensor_unknown(run):
    """H52 — the gas-sensor drift case: the honest hard problem (high baseline FP, high unknown rate).

    Long-horizon sensor drift with a batch-level (not per-sample) regime label: DSFB reports a high baseline
    FP and a high unknown rate rather than manufacturing a clean answer. The honesty signal in action.
    """
    rows = [r for r in S.read_csv_rows(S.ds_csv(run, "gas_sensor_array_drift", "residual_streams.csv"))
            if r["detector_id"] == "pca_spe_q"]
    if not rows:
        S.log("  [H] SKIP regime_gas_sensor_unknown: no gas-sensor residual streams"); return
    t = np.array([int(r["time_index"]) for r in rows])
    z = np.array([S.fnum(r["r"]) for r in rows])
    # Pull the dataset's metrics for the honesty annotation.
    fp = unk = float("nan")
    for m in S.read_csv_rows(os.path.join(run, "metrics.csv")):
        if m["dataset"] == "gas_sensor_array_drift":
            fp = S.fnum(m["baseline_false_positive_rate"]) * 100
            unk = S.fnum(m["unknown_rate"]) * 100
    fig, ax = plt.subplots(figsize=(7.4, 3.6))
    ax.plot(t, z, lw=0.8, color=S.INK)
    ax.set_xlabel("sample index")
    ax.set_ylabel("residual r (SPE/Q)")
    ax.set_title("Gas-sensor array drift — the honest hard case")
    ax.text(0.02, 0.92, f"baseline FP ≈ {fp:.0f}%   unknown rate ≈ {unk:.0f}%", transform=ax.transAxes,
            fontsize=9, color=S.WARN,
            bbox=dict(boxstyle="round", fc="white", ec=S.WARN, alpha=0.9))
    S.figure_caption(ax, "Heterogeneous baseline (batch-1 vs late batches) → high FP; DSFB reports unknown rather than forcing a clean answer. Reported honestly.")
    S.save(fig, "regime_gas_sensor_unknown", "H", "gas_sensor_array_drift/residual_streams.csv + metrics.csv",
           "The gas-sensor drift hard case: high baseline FP + high unknown rate, reported honestly rather than suppressed.")


def render_all(run):
    """Render every group-H figure (regime before/after + recipe-phase overlay + gas-sensor honest case)."""
    S.log("group H — regime-conditioned envelopes & case studies")
    fig_regime_before_after(run)
    fig_recipe_phase_overlay(run)
    fig_gas_sensor_unknown(run)
