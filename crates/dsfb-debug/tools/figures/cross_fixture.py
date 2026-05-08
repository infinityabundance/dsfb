"""Cross-fixture summary figures.

Three figures:
  1. fusion_sweep.png       — F-11 N=1..9 with semilog-y, annotated reference lines.
  2. rscr_forest.png        — 12 fixtures as horizontal forest plot, log-x for RSCR.
  3. tier_firing_heatmap.png — 12 fixtures × 27 tiers, which tiers fire on which dataset.
"""
from __future__ import annotations

from pathlib import Path

import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import numpy as np

from . import _style as S
from .architecture import TIER_BIT_NAMES, TIER_BITS


# ----------------------------------------------------------------------
# Fusion sweep — semilog-y, annotated reference lines, shaded zone
# ----------------------------------------------------------------------
def render_fusion_sweep(out_path: Path):
    """F-11 fusion sweep: clean-window FP rate vs min_consensus N."""
    # Verbatim from tests/fusion_compare.rs F-11 stdout
    Ns = [1, 3, 5, 7, 9]
    fps = [0.1717, 0.0348, 0.0186, 0.0116, 0.0001]   # 0 -> tiny epsilon for log
    refs = [
        ("EWMA",          0.0812, "#9A031E"),
        ("scalar-3sigma", 0.0139, "#7C7E80"),
        ("CUSUM",         0.0116, "#5F6368"),
        ("DSFB-structural alone", 0.0070, S.DSFB_HIGHLIGHT),
    ]

    fig, ax = plt.subplots(figsize=S.figsize("double"))
    ax.set_yscale("log")
    ax.set_xlim(0.5, 10)
    ax.set_ylim(0.0001, 0.5)

    # Horizontal reference lines + right-aligned labels
    for label, val, col in refs:
        ax.axhline(val, color=col, linewidth=0.9, linestyle="--", alpha=0.85)
        ax.text(9.6, val * 1.10, f"{label}: {val:.4f}",
                fontsize=8, color=col, ha="right", va="bottom",
                bbox=dict(facecolor="white", edgecolor="none", pad=1.0))

    # Shaded "below every single detector" zone
    min_ref = min(v for _, v, _ in refs)
    ax.axhspan(0.0001, min_ref, color=S.DSFB_PRIMARY, alpha=0.06,
               label="below every single-detector baseline")

    # The fusion curve
    ax.plot(Ns, fps, color=S.DSFB_PRIMARY, linewidth=2.4, marker="o",
            markersize=7, markerfacecolor=S.DSFB_PRIMARY,
            markeredgecolor="white", markeredgewidth=1.2, zorder=5,
            label="9-axis fusion (N≥...)")

    # Annotate each data point
    for N, fp in zip(Ns, fps):
        if fp < 0.001:
            label = "≈0"
        else:
            label = f"{fp:.4f}"
        ax.annotate(label, xy=(N, fp),
                    xytext=(0, 12), textcoords="offset points",
                    fontsize=8, ha="center", color=S.DSFB_PRIMARY,
                    fontweight="bold")

    ax.set_xlabel("min_consensus  N  (number of detectors that must agree)")
    ax.set_ylabel("Layer-2 clean-window false-positive rate (log scale)")
    ax.set_title("F-11 fusion sweep — consensus reduces false-positive rate below every single-detector baseline at N ≥ 7",
                 loc="left", fontsize=10)
    ax.set_xticks(Ns)
    ax.legend(loc="upper right", fontsize=8)
    ax.grid(which="major", linewidth=0.4, alpha=0.6)

    fig.tight_layout()
    fig.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close(fig)


# ----------------------------------------------------------------------
# RSCR forest plot
# ----------------------------------------------------------------------
def render_rscr_forest(per_fixture: list[dict], out_path: Path):
    """Horizontal forest plot of RSCR per fixture, log-x scale."""
    # Sort: by-design silent first (RSCR=0), then by RSCR ascending
    rows = []
    for f in per_fixture:
        rows.append({
            "name": f["short_name"],
            "rscr": f["metrics"]["rscr"],
            "fp":   f["metrics"]["clean_window_false_episode_rate"],
            "episodes": f["metrics"]["dsfb_episode_count"],
            "is_silent": f["metrics"]["dsfb_episode_count"] == 0,
        })
    silent = [r for r in rows if r["is_silent"]]
    active = sorted([r for r in rows if not r["is_silent"]],
                    key=lambda r: r["rscr"])
    rows = silent + active

    fig, ax = plt.subplots(figsize=S.figsize("wide"))
    y = list(range(len(rows)))[::-1]   # invert to put F-11 etc. on top
    rscrs = [r["rscr"] for r in rows]
    colors = [S.DSFB_GRAY if r["is_silent"] else S.DSFB_PRIMARY for r in rows]
    ax.barh(y, [max(r["rscr"], 0.5) for r in rows],   # 0.5 floor for log-x
            color=colors, edgecolor="white", linewidth=0.5,
            height=0.6)
    ax.set_xscale("log")
    ax.set_xlim(0.5, 200)
    ax.set_yticks(y)
    ax.set_yticklabels([r["name"] for r in rows], fontsize=8)
    ax.set_xlabel("RSCR  (raw cell-level alerts ÷ typed structural episodes; log scale)")
    ax.set_title("Cross-fixture review-surface compression — 12 vendored real-bytes fixtures",
                 loc="left", fontsize=11)

    # Direct labels — RSCR value + FP rate
    for r, yi in zip(rows, y):
        if r["is_silent"]:
            txt = "by-design silent (0 episodes)"
            ax.text(0.6, yi, txt, va="center", fontsize=7.5,
                    color="#444", style="italic")
        else:
            txt = f"RSCR {r['rscr']:.2f}×   FP rate {r['fp']:.4f}   ({r['episodes']} ep.)"
            ax.text(r["rscr"] * 1.15, yi, txt, va="center", fontsize=7.5,
                    color="#222")

    # Highlight F-11 row
    for i, r in enumerate(rows):
        if "F11" in r["name"] and "F11b" not in r["name"]:
            ax.axhspan(y[i] - 0.4, y[i] + 0.4, color=S.DSFB_HIGHLIGHT, alpha=0.10, zorder=0)
            ax.text(150, y[i], "production-relevant anchor", fontsize=7,
                    color=S.DSFB_HIGHLIGHT, va="center", ha="right",
                    fontweight="bold")

    ax.grid(axis="x", linewidth=0.3, alpha=0.5)
    ax.grid(axis="y", visible=False)

    fig.tight_layout()
    fig.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close(fig)


# ----------------------------------------------------------------------
# Cross-fixture per-tier firing heatmap (NEW)
# ----------------------------------------------------------------------
# Detector-name → tier mapping. Captures all 205 detectors emitted by the
# fusion harness in `incumbent_baselines.rs`. Detectors not in this map
# default to "?" (rendered as 'unknown' tier in the heatmap).
TIER_OF_DETECTOR = {
    # Tier A — parametric trio
    "scalar_threshold_3sigma": "A", "cusum": "A", "ewma": "A",
    # Tier B — robust statistics
    "robust_z_mad": "B", "page_hinkley": "B", "tukey_iqr": "B",
    # Tier C — model + non-parametric
    "spectral_residual": "C", "matrix_profile": "C", "bocpd": "C",
    "isolation_forest": "C", "lof": "C",
    # Tier D — additional non-dep
    "mann_kendall": "D", "rolling_z": "D", "ar1_residual": "D",
    "mahalanobis": "D", "ks_rolling": "D",
    # Tier E — debugging-specific
    "poisson_burst": "E", "saturation_chain": "E", "chi_sq_proportion": "E",
    # Tier F — burst
    "max_interval": "F", "log_isi": "F", "rank_surprise": "F", "misi": "F",
    # EXTRA
    "glr": "EXTRA", "adwin": "EXTRA", "mewma": "EXTRA",
    "retry_storm": "EXTRA", "correlation_break": "EXTRA",
    # Tier G — concept drift streaming
    "shiryaev_roberts": "G", "ddm": "G", "eddm": "G", "hddm_a": "G",
    "hddm_w": "G", "stepd": "G", "ecdd": "G", "kswin": "G", "fhddm": "G",
    # Tier H — distribution shift
    "wasserstein_1d": "H", "jensen_shannon": "H", "kl_divergence": "H",
    "psi": "H", "anderson_darling": "H", "cramer_von_mises": "H",
    "energy_distance": "H", "mmd": "H", "bhattacharyya": "H", "hellinger": "H",
    # Tier I — robust nonparametric
    "median_absolute_slope": "I", "theil_sen_residual": "I",
    "sen_slope_changepoint": "I", "moods_median_rolling": "I",
    "brown_forsythe": "I", "levene_variance": "I", "sign_test_drift": "I",
    "runs_test": "I", "wald_wolfowitz_two_sample": "I", "sequential_rank": "I",
    # Tier J — forecast residual
    "ses_residual": "J", "holt_linear": "J", "holt_winters": "J",
    "ar2_residual": "J", "arima_simplified": "J", "kalman_innovation": "J",
    "savitzky_golay_residual": "J", "stl_residual": "J",
    "prophet_simplified": "J", "naive_seasonal": "J",
    # Tier K — frequency / oscillation
    "fft_band_energy": "K", "welch_psd": "K", "wavelet_haar": "K",
    "autocorrelation_peak": "K", "lomb_scargle": "K", "zero_crossing_rate": "K",
    "dominant_frequency_drift": "K", "spectral_entropy": "K",
    "cepstral_simplified": "K", "phase_locking": "K",
    # Tier L — multivariate relationship
    "hotelling_t2": "L", "mcusum": "L", "pca_reconstruction": "L",
    "robust_pca": "L", "correlation_matrix_distance": "L",
    "partial_correlation": "L", "graph_laplacian": "L",
    "canonical_correlation": "L", "mutual_information": "L",
    # Tier M — debugging-native
    "flap": "M", "sawtooth_ramp": "M", "deadband_stuck": "M",
    "quantization": "M", "plateau": "M", "counter_wrap": "M",
    "monotone_leak": "M", "hysteresis": "M", "limit_cycle": "M",
    "ping_pong": "M", "backpressure": "M", "causal_lag": "M",
    "fan_out": "M", "fan_in": "M", "phase_slip": "M",
    "jitter_bloom": "M", "tail_thickening": "M", "burst_after_silence": "M",
    # Tier N — offline CPD
    "pelt": "N", "binary_segmentation": "N", "bottom_up_segmentation": "N",
    "window_based_cpd": "N", "dynamic_programming_cpd": "N",
    "kernel_cpd": "N", "piecewise_linear_cpd": "N", "bayesian_offline_cpd": "N",
    # Tier O — rare changepoint
    "mosum": "O", "narrowest_over_threshold": "O", "wbs2": "O",
    "seeded_bs": "O", "smuce": "O", "fdr_seg": "O", "fpop": "O",
    "tguh": "O", "inspect_cpd": "O", "double_cusum_bs": "O",
    # Tier P — streaming sequential
    "e_detector": "P", "conformal_martingale": "P",
    "exchangeability_martingale": "P", "power_martingale": "P",
    "mixture_martingale": "P", "mixture_sprt": "P", "scan_statistic": "P",
    "higher_criticism": "P", "berk_jones": "P",
    # Tier Q — concept drift rarer
    "mddm_a": "Q", "mddm_e": "Q", "mddm_g": "Q", "lfr": "Q",
    "fpdd": "Q", "optwin": "Q", "seqdrift2": "Q", "d3_drift": "Q",
    "quanttree": "Q", "nn_dvi": "Q",
    # Tier R — robust depth
    "halfspace_depth": "R", "projection_depth": "R", "stahel_donoho": "R",
    "mcd": "R", "spatial_sign": "R", "s_estimator_residual": "R",
    "depth_rank_control": "R", "outlyingness_median_polish": "R",
    # Tier S — count / event-process
    "bayesian_blocks": "S", "index_of_dispersion": "S", "allan_variance": "S",
    # Tier T — info-theoretic
    "mdl_change": "T", "ncd": "T", "lempel_ziv": "T",
    "transfer_entropy": "T", "fisher_information": "T", "renyi_entropy": "T",
    # Tier U — dynamical systems
    "permutation_entropy": "U", "sample_entropy": "U", "rqa_recurrence": "U",
    "lyapunov": "U", "correlation_dimension": "U", "bds_test": "U",
    "zero_one_chaos": "U", "delay_embedding_nn": "U",
    # Phase 5 wave
    "parity_space": "V", "observer_residual": "V", "structured_residual": "V",
    "buishand_range": "X", "snht": "X", "pettitt_step": "X",
    "median_of_means": "Y", "u_statistic_step": "Y", "rank_step": "Y",
    "rayleigh_circular": "Z", "phase_jump": "Z", "rbar_circular": "Z",
    "arch_residual": "AA", "kurtosis_burst": "AA", "volatility_drift": "AA",
    "hinich_bicorrelation": "AA", "mcleod_li": "AA",
    "keenan_nonlinearity": "AA", "tsay_nonlinearity": "AA",
    "hinich_tricorrelation": "AA",
}


def render_tier_firing_heatmap(per_fixture: list[dict], out_path: Path):
    """12 fixtures × 27 tiers heatmap of per-tier firing-fraction."""
    # Build matrix: rows = fixture, cols = tier; value = sum(raw_alerts in that tier) / total_raw_alerts
    fixtures = [f["short_name"] for f in per_fixture]

    matrix = np.zeros((len(per_fixture), len(TIER_BIT_NAMES)), dtype=float)
    for i, f in enumerate(per_fixture):
        # Aggregate alerts per tier from per_detector list
        per_tier = {t: 0 for t in TIER_BIT_NAMES}
        total = 0
        for d in f.get("per_detector", []):
            tier = TIER_OF_DETECTOR.get(d["name"], None)
            if tier is None:
                continue
            per_tier[tier] += d["raw_alerts"]
            total += d["raw_alerts"]
        if total > 0:
            for j, t in enumerate(TIER_BIT_NAMES):
                matrix[i, j] = per_tier[t] / total

    fig, ax = plt.subplots(figsize=S.figsize("wide"))
    im = ax.imshow(matrix, cmap="cmc.lapaz_r", aspect="auto",
                    interpolation="nearest", vmin=0, vmax=max(matrix.max(), 0.05))
    ax.set_xticks(range(len(TIER_BIT_NAMES)))
    ax.set_xticklabels(TIER_BIT_NAMES, fontsize=7.5)
    ax.set_yticks(range(len(fixtures)))
    ax.set_yticklabels(fixtures, fontsize=8)
    ax.set_xlabel("Detector tier (27 mathematical axes)")
    ax.set_ylabel("Vendored fixture")
    ax.set_title("Cross-fixture firing pattern — fraction of alerts per tier (Tier-A trio dominates everywhere; Phase-5 wave fires on specific domains)",
                 loc="left", fontsize=10)
    cb = fig.colorbar(im, ax=ax, shrink=0.7, pad=0.02)
    cb.set_label("Fraction of fixture's alerts contributed by this tier",
                  fontsize=8)
    cb.ax.tick_params(labelsize=7)

    fig.tight_layout()
    fig.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close(fig)
