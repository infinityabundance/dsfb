#!/usr/bin/env python3
"""
NASA IMS Bearings (Run 1) — Stage II Benchmark
DSFB Structural Semiotics Engine vs. magnitude threshold baseline.

Residual signal: bearing3_rms (known failure channel).
Nominal reference: mean RMS over first 10 steps (healthy baseline).
Residual: observed_rms - nominal_rms.

Detection metric: first detection hour for each method.
Ground truth failure onset: step 60 (~27.6 h), RMS > 0.35 threshold.
"""

import csv
import json
import math
import pathlib
import sys

# ── Configuration ────────────────────────────────────────────────────────────

DATA_PATH = pathlib.Path(__file__).parent.parent / \
    "data/public_dataset/raw/nasa_bearings_raw_summary.csv"

# Envelope radius: set to 3-sigma of the healthy residual
# (computed from first 10 steps below, not hand-tuned)
ENVELOPE_RADIUS_SIGMA_MULTIPLIER = 3.0

# Threshold baseline: alarm when residual exceeds N * healthy std
THRESHOLD_SIGMA_MULTIPLIER = 3.0

# Ground truth: failure onset step (step 60, ~27.6 h, RMS > 0.35)
FAILURE_ONSET_STEP = 60
FAILURE_ONSET_HOUR = 27.597

# Healthy window for nominal estimation
HEALTHY_WINDOW = 10

# ── Load data ─────────────────────────────────────────────────────────────────

rows = []
with open(DATA_PATH) as f:
    reader = csv.DictReader(f)
    for row in reader:
        rows.append({
            "step": int(row["step"]),
            "hour": float(row["relative_hours"]),
            "rms": float(row["bearing3_rms"]),
            "kurtosis": float(row["bearing3_kurtosis"]),
        })

# ── Nominal reference (healthy window) ───────────────────────────────────────

healthy = rows[:HEALTHY_WINDOW]
nominal_rms = sum(r["rms"] for r in healthy) / len(healthy)
residuals = [r["rms"] - nominal_rms for r in rows]

healthy_residuals = residuals[:HEALTHY_WINDOW]
healthy_std = math.sqrt(
    sum(x**2 for x in healthy_residuals) / len(healthy_residuals)
)
healthy_std = max(healthy_std, 1e-6)  # guard against degenerate case

envelope_radius = ENVELOPE_RADIUS_SIGMA_MULTIPLIER * healthy_std
threshold = THRESHOLD_SIGMA_MULTIPLIER * healthy_std

print(f"Nominal RMS (healthy mean):  {nominal_rms:.6f}")
print(f"Healthy residual std:         {healthy_std:.6f}")
print(f"Envelope radius (3σ):         {envelope_radius:.6f}")
print(f"Threshold baseline (3σ):      {threshold:.6f}")
print()

# ── Threshold baseline ────────────────────────────────────────────────────────

threshold_detection_step = None
threshold_detection_hour = None
for r in rows:
    residual = r["rms"] - nominal_rms
    if abs(residual) > threshold:
        threshold_detection_step = r["step"]
        threshold_detection_hour = r["hour"]
        break

# ── DSFB structural detection ─────────────────────────────────────────────────
# Drift-based detection: sustained outward drift over a window.
# Per Theorem 1: detection when drift rate > 0 sustained across W steps
# and residual norm is moving outward relative to envelope.
#
# Implementation: finite-difference drift estimate over window W=5,
# alarm when mean drift > alpha_threshold AND residual > envelope_radius * fraction.

DRIFT_WINDOW = 5
DRIFT_ALPHA = 0.002   # minimum drift rate (units: RMS/step) to count as "sustained"
ENVELOPE_FRACTION = 0.5  # alarm when residual > 50% of envelope AND drift sustained

dsfb_detection_step = None
dsfb_detection_hour = None

for i in range(DRIFT_WINDOW, len(rows)):
    window_residuals = [abs(residuals[j]) for j in range(i - DRIFT_WINDOW, i)]
    # finite-difference drift: mean of first differences
    diffs = [window_residuals[j+1] - window_residuals[j]
             for j in range(len(window_residuals) - 1)]
    mean_drift = sum(diffs) / len(diffs)

    current_residual = abs(residuals[i])

    # Theorem 1 preconditions: sustained outward drift AND inside envelope moving out
    if mean_drift >= DRIFT_ALPHA and current_residual > envelope_radius * ENVELOPE_FRACTION:
        dsfb_detection_step = rows[i]["step"]
        dsfb_detection_hour = rows[i]["hour"]
        break

# ── Results ───────────────────────────────────────────────────────────────────

print("=" * 60)
print("DETECTION RESULTS — NASA IMS Bearings Run 1, Bearing 3")
print("=" * 60)
print(f"Ground truth failure onset:   step {FAILURE_ONSET_STEP}"
      f" / {FAILURE_ONSET_HOUR:.2f} h")
print()

if threshold_detection_step is not None:
    lead_h = FAILURE_ONSET_HOUR - threshold_detection_hour
    print(f"Threshold baseline (3σ):      step {threshold_detection_step}"
          f" / {threshold_detection_hour:.2f} h"
          f"  [{lead_h:.2f} h before failure]")
else:
    print("Threshold baseline:           NO DETECTION")

if dsfb_detection_step is not None:
    lead_h = FAILURE_ONSET_HOUR - dsfb_detection_hour
    print(f"DSFB structural (Thm 1):      step {dsfb_detection_step}"
          f" / {dsfb_detection_hour:.2f} h"
          f"  [{lead_h:.2f} h before failure]")
else:
    print("DSFB structural:              NO DETECTION")

print()

if dsfb_detection_step is not None and threshold_detection_step is not None:
    advantage_h = threshold_detection_hour - dsfb_detection_hour
    print(f"Early-warning advantage:      {advantage_h:.2f} h earlier than threshold")

# ── JSON artifact ─────────────────────────────────────────────────────────────

artifact = {
    "dataset": "NASA IMS Bearings Run 1",
    "channel": "bearing3_rms",
    "nominal_rms": nominal_rms,
    "healthy_std": healthy_std,
    "envelope_radius_3sigma": envelope_radius,
    "threshold_3sigma": threshold,
    "ground_truth_failure_onset_step": FAILURE_ONSET_STEP,
    "ground_truth_failure_onset_hour": FAILURE_ONSET_HOUR,
    "threshold_baseline": {
        "detection_step": threshold_detection_step,
        "detection_hour": threshold_detection_hour,
    },
    "dsfb_structural": {
        "detection_step": dsfb_detection_step,
        "detection_hour": dsfb_detection_hour,
        "drift_window": DRIFT_WINDOW,
        "drift_alpha": DRIFT_ALPHA,
        "envelope_fraction": ENVELOPE_FRACTION,
    },
}

out = pathlib.Path(__file__).parent.parent / \
    "data/processed/nasa_bearings/stage2_detection_results.json"
out.parent.mkdir(parents=True, exist_ok=True)
with open(out, "w") as f:
    json.dump(artifact, f, indent=2)
print(f"\nArtifact written to: {out}")
