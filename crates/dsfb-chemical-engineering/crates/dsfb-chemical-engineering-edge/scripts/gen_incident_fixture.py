#!/usr/bin/env python3
"""Generate the fully-FICTIONAL forensic-incident historian fixture used by
`docs/forensic_incident_walkthrough.md`.

Everything here is invented: there is **no real company, plant, site, or agency**. The scenario exists
only to show, end to end, what an operator hands the `historian` command and what Chemical Court Record
comes back — a self-contained worked example on openly-shareable synthetic data (NOT plant data).

The (fictional) scenario
------------------------
"Northgate Specialty Chemicals", Plant 2, exothermic reactor **R-101**, is cooled by a jacket fed from
**cooling-water surge tank TK-110**. The tank is on level control:
  * LIT-110  — surge-tank level (mm), the controlled variable (carries SP / MV / controller mode);
  * FIT-110A — make-up cooling-water inflow (m3/h);
  * FIT-110B — draw to the R-101 jacket (m3/h).
The volume balance is  area * dLIT/dt = FIT-110A - FIT-110B  (area = dt = factor = 1), so under normal
control the level integrates the metered net flow and the closure residual is ~0.

The incident
------------
Mid-batch, LIT-110 is **spoofed** (the transmitter freezes) while the meters keep showing a sustained
net draw to the jacket: the tank is really draining (cooling-water inventory is being lost ahead of a
reactor temperature excursion) but the level reads flat. The balance witness recomputes the closure from
the raw tags and sees `dLIT (≈0) - (FIT-110A - FIT-110B) (<0)` break by the net-outflow magnitude every
step — the same sensor-spoof mass-balance mechanism as the real SWaT/BATADAL witnesses, here on invented
data. The operator drops the loop to manual at onset (controller_mode auto -> manual).

Deterministic: pure closed-form series, no RNG -> byte-identical on every run.
"""
import csv
import json
import math
import os

HERE = os.path.dirname(os.path.abspath(__file__))
OUT_DIR = os.path.join(HERE, "..", "data", "historian")
CSV_PATH = os.path.join(OUT_DIR, "northgate_r101_incident.csv")
ROLES_PATH = os.path.join(OUT_DIR, "northgate_r101_incident.roles.json")

N = 90               # timestamps (1-minute cadence)
FAULT_ONSET = 54     # the spoof begins here
SETPOINT_MM = 1200.0  # surge-tank level setpoint


def build_rows():
    rows = []
    lit = SETPOINT_MM
    spoof_level = None
    for t in range(N):
        if t < FAULT_ONSET:
            # Normal batch cooling: small benign swing in the make-up flow around a steady jacket draw;
            # the level integrates the metered net flow exactly => closure residual == 0.
            fit_b = 30.0                                   # steady jacket draw
            fit_a = 30.0 + 3.0 * math.sin(t / 5.0)         # make-up flow trims to hold level
            lit = lit + (fit_a - fit_b)                    # dL == net flow => balance closes
            phase, mode, quality = 0, "auto", "good"
        else:
            # Spoof: LIT-110 frozen while the meters show a sustained net draw (tank truly draining).
            fit_b = 34.0
            fit_a = 26.0                                   # net = -8 mm/step the level should fall by...
            if spoof_level is None:
                spoof_level = lit                          # ...but the frozen transmitter holds it flat
            lit = spoof_level
            phase, mode, quality = 1, "manual", "good"     # operator dropped the loop to manual at onset
        ts = f"2027-03-14T08:{t:02d}:00Z"
        mv = 50.0 + 4.0 * math.sin(t / 3.0)                # level-control valve output (%)
        rows.append([ts, "LIT-110", f"{lit:.4f}", "mm", quality, phase, mode, f"{SETPOINT_MM:.4f}", f"{mv:.4f}"])
        rows.append([ts, "FIT-110A", f"{fit_a:.4f}", "m3/h", quality, phase, "", "", ""])
        rows.append([ts, "FIT-110B", f"{fit_b:.4f}", "m3/h", quality, phase, "", "", ""])
    # One deliberately bad-quality sample to exercise the quality gate (a non-balance tag, early, benign).
    rows[7][4] = "bad"
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
        "dataset": "northgate_r101_incident",
        "kind": "synthetic",
        "description": ("FICTIONAL forensic-incident fixture (NOT real plant data; no real company/site). "
                        "Northgate Specialty Chemicals R-101 cooling-water surge tank TK-110: a mid-batch "
                        "LIT-110 sensor spoof freezes the level while the meters show a sustained jacket "
                        "draw, breaking the mass_tank_volume closure."),
        "variables": [
            {"name": "LIT-110", "role": "measured", "unit": "mm", "quantity": "level"},
            {"name": "FIT-110A", "role": "measured", "unit": "m3/h", "quantity": "inflow"},
            {"name": "FIT-110B", "role": "measured", "unit": "m3/h", "quantity": "outflow"},
        ],
        "balance": {
            "type": "mass_tank_volume",
            "area_m2": 1.0,
            "dt": 1.0,
            "level": "LIT-110",
            "inflows": ["FIT-110A"],
            "outflows": ["FIT-110B"],
            "flow_to_vol_per_dt": 1.0,
            "definition": ("residual = dLIT-110 - (FIT-110A - FIT-110B); ~0 under normal control, jumps to "
                           "the net-draw magnitude when the spoofed level contradicts the metered flow."),
        },
        "fault": {
            "onset_index_timestamps": FAULT_ONSET,
            "mode": "LIT-110 sensor spoof (frozen level) while meters show net jacket draw",
        },
    }
    with open(ROLES_PATH, "w") as f:
        json.dump(roles, f, indent=2)
    print(f"wrote {CSV_PATH} ({N} timestamps x 3 tags) and {ROLES_PATH}")


if __name__ == "__main__":
    main()
