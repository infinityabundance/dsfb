#!/usr/bin/env python3
"""Generate a small, fully-synthetic plant-historian fixture for the `historian` command.

Why this exists
---------------
S-CHEM.2 (operator-evaluation protocol) needs a *shareable* historian export that exercises the full
long-format schema -- timestamp, tag, value, unit, quality, phase_id, controller_mode, setpoint,
manipulated_variable -- AND carries a mass/energy balance the witness can close, so the end-to-end
"historian dump -> Chemical Court Record + balance witness" path is demonstrable on openly-shareable
data (NOT real plant data).

The plant
---------
A single raw-water tank under level control: tags LIT (level, mm), FIT_IN (inflow), FIT_OUT (outflow).
The level loop (on LIT) carries setpoint / manipulated_variable / controller_mode. The mass balance is
the volume balance  area*dL/dt = FIT_IN - FIT_OUT  with area = factor = dt = 1, so in normal operation
we set  dL = FIT_IN - FIT_OUT  exactly => closure residual = 0. In the fault region the level sensor is
*spoofed* (frozen) while the meters show a net outflow, so the closure breaks by the net-outflow
magnitude every step -- a textbook sensor-spoof mass-balance violation, the same mechanism as the real
SWaT/BATADAL witnesses, here on synthetic data.

Deterministic: no RNG; pure closed-form series. Re-running reproduces byte-identical files.
"""
import csv
import json
import math
import os

HERE = os.path.dirname(os.path.abspath(__file__))
OUT_DIR = os.path.join(HERE, "..", "data", "historian")
CSV_PATH = os.path.join(OUT_DIR, "synthetic_tank_historian.csv")
ROLES_PATH = os.path.join(OUT_DIR, "synthetic_tank_historian.roles.json")

N = 60                 # timestamps
FAULT_ONSET = 40       # spoof begins here
BASELINE_FRAC = 0.4    # matches the loader default; baseline window = first 40% of timestamps


def build_rows():
    rows = []
    lit = 700.0  # starting level (mm)
    spoof_level = None
    for t in range(N):
        if t < FAULT_ONSET:
            # Normal: a gentle net-flow oscillation; level integrates net flow exactly => balance closes.
            fit_out = 10.0
            fit_in = 10.0 + 2.0 * math.sin(t / 4.0)
            net = fit_in - fit_out
            lit = lit + net          # dL == net  => residual == 0
            phase = 0
            mode = "auto"
            quality = "good"
        else:
            # Sensor-spoof: level frozen while meters show a sustained net outflow (tank truly draining).
            fit_out = 12.0
            fit_in = 8.0             # net = -4 each step: tank should fall 4 mm/step...
            if spoof_level is None:
                spoof_level = lit    # ...but the spoofed level is held flat
            lit = spoof_level
            phase = 1
            mode = "manual"          # operator dropped the loop to manual during the upset
            quality = "good"
        ts = f"2026-01-01T00:{t:02d}:00Z"
        # Level row carries the control-loop context; flow rows carry only value+unit.
        sp = 700.0                   # level setpoint (mm)
        mv = 50.0 + 5.0 * math.sin(t / 3.0)  # control output (% valve)
        rows.append([ts, "LIT", f"{lit:.4f}", "mm", quality, phase, mode, f"{sp:.4f}", f"{mv:.4f}"])
        rows.append([ts, "FIT_IN", f"{fit_in:.4f}", "m3/h", quality, phase, "", "", ""])
        rows.append([ts, "FIT_OUT", f"{fit_out:.4f}", "m3/h", quality, phase, "", "", ""])
    # One deliberately bad-quality sample to exercise the quality gate (does not affect the balance tag).
    rows[3][4] = "bad"
    return rows


def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    header = ["timestamp", "tag", "value", "unit", "quality", "phase_id",
              "controller_mode", "setpoint", "manipulated_variable"]
    with open(CSV_PATH, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(header)
        w.writerows(build_rows())

    roles = {
        "dataset": "synthetic_tank_historian",
        "kind": "synthetic",
        "description": ("Fully-synthetic level-controlled tank historian (NOT real plant data). Exercises "
                        "the full long-format schema and a mass_tank_volume balance; a sensor-spoof in the "
                        "fault region freezes LIT while the meters show net outflow, breaking the closure."),
        "variables": [
            {"name": "LIT", "role": "measured", "unit": "mm", "quantity": "level"},
            {"name": "FIT_IN", "role": "measured", "unit": "m3/h", "quantity": "inflow"},
            {"name": "FIT_OUT", "role": "measured", "unit": "m3/h", "quantity": "outflow"},
        ],
        "balance": {
            "type": "mass_tank_volume",
            "area_m2": 1.0,
            "dt": 1.0,
            "level": "LIT",
            "inflows": ["FIT_IN"],
            "outflows": ["FIT_OUT"],
            "flow_to_vol_per_dt": 1.0,
            "definition": ("residual = dLIT - (FIT_IN - FIT_OUT); ~0 under normal control, jumps to the "
                           "net-outflow magnitude when the spoofed LIT contradicts the metered flow."),
        },
        "fault": {
            "onset_index_timestamps": FAULT_ONSET,
            "mode": "LIT sensor spoof (frozen level) while meters show net outflow",
        },
    }
    with open(ROLES_PATH, "w") as f:
        json.dump(roles, f, indent=2)
    print(f"wrote {CSV_PATH} ({N} timestamps x 3 tags) and {ROLES_PATH}")


if __name__ == "__main__":
    main()
