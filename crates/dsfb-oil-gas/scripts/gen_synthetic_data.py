#!/usr/bin/env python3
"""Generate synthetic CSV datasets for the DSFB oil-and-gas crate.

All files use fully-qualified column names matching the Rust loader structs.
Run: python3 scripts/gen_synthetic_data.py
"""

import math
import os
import random

random.seed(42)
N = 201
DATA = os.path.join(os.path.dirname(__file__), "..", "data")
os.makedirs(DATA, exist_ok=True)


# ── Pipeline synthetic ────────────────────────────────────────────────────────
# Columns: timestamp, expected_flow_balance, observed_flow_balance,
#           inlet_pressure, outlet_pressure
rows = ["timestamp,expected_flow_balance,observed_flow_balance,inlet_pressure,outlet_pressure"]
for i in range(N):
    t = i * 0.5
    exp_flow = 100.0
    drift = 0.04 * i * math.sin(0.02 * i)         # slow sinusoidal wax build-up
    slew_impulse = 3.5 if i == 80 else (-2.8 if i == 140 else 0.0)  # pigging events
    obs_flow = exp_flow + drift + slew_impulse + random.gauss(0, 0.08)
    obs_dp = 5.0 + 0.003 * i + random.gauss(0, 0.05)   # slowly rising ΔP
    inp = 50.0 + obs_dp / 2.0
    outp = 50.0 - obs_dp / 2.0
    rows.append(f"{t:.3f},{exp_flow:.4f},{obs_flow:.6f},{inp:.6f},{outp:.6f}")
path = os.path.join(DATA, "pipeline_synthetic.csv")
open(path, "w").write("\n".join(rows) + "\n")
print(f"Wrote {len(rows)} lines to {path}")


# ── Drilling synthetic ────────────────────────────────────────────────────────
# Columns: timestamp, expected_torque, observed_torque, wob, rpm
rows = ["timestamp,expected_torque,observed_torque,wob,rpm"]
for i in range(N):
    t = i * 0.5
    exp_t = 18.0
    wear_drift = 0.015 * i
    stick_slip = 8.0 if (i % 20 < 3) else 0.0     # periodic torsional oscillation
    obs_t = exp_t + wear_drift + stick_slip + random.gauss(0, 0.3)
    wob = 150.0 + random.gauss(0, 2.0)
    rpm = 80.0 - 0.006 * i + random.gauss(0, 0.5)
    rows.append(f"{t:.3f},{exp_t:.4f},{obs_t:.6f},{wob:.4f},{rpm:.4f}")
path = os.path.join(DATA, "drilling_synthetic.csv")
open(path, "w").write("\n".join(rows) + "\n")
print(f"Wrote {len(rows)} lines to {path}")


# ── Rotating equipment synthetic ──────────────────────────────────────────────
# Columns: timestamp, expected_head, observed_head, vibration_rms, flow_rate
rows = ["timestamp,expected_head,observed_head,vibration_rms,flow_rate"]
for i in range(N):
    t = i * 0.5
    exp_h = 400.0
    scale_drift = 0.008 * i                        # scale deposit build-up
    gas = -15.0 if (60 <= i <= 63 or 110 <= i <= 112) else 0.0  # gas-lock events
    obs_h = exp_h + scale_drift + gas + random.gauss(0, 1.0)
    vib = 2.0 + 0.01 * i * abs(math.sin(0.1 * i)) + random.gauss(0, 0.1)
    flow = 280.0 + random.gauss(0, 3.0)
    rows.append(f"{t:.3f},{exp_h:.4f},{obs_h:.6f},{vib:.6f},{flow:.4f}")
path = os.path.join(DATA, "rotating_synthetic.csv")
open(path, "w").write("\n".join(rows) + "\n")
print(f"Wrote {len(rows)} lines to {path}")


# ── Subsea synthetic ──────────────────────────────────────────────────────────
# Columns: timestamp, expected_actuation_pressure,
#           observed_actuation_pressure, valve_command
rows = ["timestamp,expected_actuation_pressure,observed_actuation_pressure,valve_command"]
for i in range(N):
    t = i * 0.5
    exp_p = 340.0
    valve = 1.0 if (40 <= i <= 42 or 100 <= i <= 102 or 160 <= i <= 162) else 0.0
    slew_p = 80.0 * valve                          # actuation pressure spike
    seal_drift = 0.05 * i                          # seal degradation drift
    obs_p = exp_p + slew_p + seal_drift + random.gauss(0, 1.5)
    rows.append(f"{t:.3f},{exp_p:.4f},{obs_p:.6f},{valve:.1f}")
path = os.path.join(DATA, "subsea_synthetic.csv")
open(path, "w").write("\n".join(rows) + "\n")
print(f"Wrote {len(rows)} lines to {path}")

print("All done.")
