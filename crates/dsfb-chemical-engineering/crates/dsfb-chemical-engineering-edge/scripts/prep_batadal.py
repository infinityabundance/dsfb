#!/usr/bin/env python3
"""Prepare a REAL public-data mass-balance witness slice from the BATADAL benchmark.

BATADAL (BATtle of the Attack Detection ALgorithms; Taormina, Galelli, et al., J. Water Resources
Planning & Management 144(8), 2018) distributes hourly SCADA logs from the C-Town water distribution
network: seven tank levels (m), eleven pump flows + statuses (LPS), one valve, junction pressures, and
a labelled attack flag. Unlike the twenty anonymised corpus slices, this data carries the metadata to
close a *rigorous* volume (mass) balance on a storage tank:

    area * dL/dt = sum(metered inflows) - (metered outflows + unmetered demand).

Tank T1 is the main storage tank, fed by source pumping station S1 (pumps PU1, PU2; PU3 is an idle
backup). Its cross-sectional area follows from the C-Town network file (EPANET ctown_map.inp, [TANKS]:
diameter 31.3 m -> area = pi*31.3^2/4 = 769.4 m^2). T1's outflow is gravity-fed DMA demand, which is
NOT metered -- so the closure equals minus that demand: a bounded, diurnal signal under normal
operation. Two of the labelled attacks in training-dataset-2 (Oct 9-11 and Oct 30-Nov 1, 2016) were
documented to manipulate pump PU2 (a T1 inflow pump); they drive the tank near overflow then let it
drain hard while the spoofed pump flows still report ~95 LPS inflow, so the measured dL/dt becomes
inconsistent with the reported inflow and the mass balance breaks. That is exactly what the DSFB
mass-balance witness keys on (sustained shift / sharp slew of the closure).

This script fetches the dataset directly from batadal.net (it is *not* redistributed in the repo) and
derives a small instrumented slice `batadal_t1_instrumented.csv` (+ roles sidecar) carrying only the
columns the T1 balance needs, with the attack flag mapped to the standard label column. The derived
slice is git-ignored; the roles sidecar (the recipe + provenance) is committed.

Run:  python3 scripts/prep_batadal.py
"""
import csv
import json
import math
import os
import urllib.request

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(HERE, "data", "instrumented")
RAW = os.path.join(OUT, "raw")
os.makedirs(RAW, exist_ok=True)
# The derived instrumented CSV is NEVER tracked → untracked research/ quarantine (DSFB_CONTROLLED_DIR
# overrides); only the metadata roles sidecar stays under OUT. BATADAL_DIR lets the user point at their own
# obtained copy of BATADAL_dataset04.csv instead of fetching from the web.
CONTROLLED = os.environ.get("DSFB_CONTROLLED_DIR", os.path.join(HERE, "..", "..", "research", "controlled"))
os.makedirs(CONTROLLED, exist_ok=True)

# Training dataset 2 (~6 months) carries the labelled attacks (ATT_FLAG). The first ~40% precedes the
# first attack, so it serves as the witness's clean baseline window (the balance-witness CLI calibrates
# its threshold from the leading baseline fraction of the loaded slice).
URL = "https://www.batadal.net/data/BATADAL_dataset04.csv"
RAW_CSV = os.path.join(os.environ.get("BATADAL_DIR", RAW), "BATADAL_dataset04.csv")

# T1 cross-sectional area from the C-Town network file (ctown_map.inp, [TANKS], diameter 31.3 m).
T1_DIAM_M = 31.3
T1_AREA_M2 = math.pi * T1_DIAM_M**2 / 4.0
DT_HOURS = 1.0
LPS_TO_M3_PER_H = 3.6  # 1 L/s * 3600 s/h / 1000 L/m^3


def fetch():
    if os.path.exists(RAW_CSV):
        print(f"using cached {RAW_CSV}")
        return
    print(f"fetching {URL} ...")
    # batadal.net presents a valid certificate; no verification bypass needed.
    urllib.request.urlretrieve(URL, RAW_CSV)
    print(f"wrote {RAW_CSV}")


def derive():
    with open(RAW_CSV) as f:
        r = csv.reader(f)
        hdr = [h.strip() for h in next(r)]
        rows = [[c.strip() for c in row] for row in r if row]
    fi = {h: i for i, h in enumerate(hdr)}
    needed = ["L_T1", "F_PU1", "F_PU2", "F_PU3", "ATT_FLAG"]
    missing = [c for c in needed if c not in fi]
    if missing:
        raise SystemExit(f"BATADAL csv missing columns {missing}; header was {hdr}")

    cols = ["L_T1", "F_PU1", "F_PU2", "F_PU3", "label", "phase"]
    out_rows = []
    n_attack = 0
    for row in rows:
        # ATT_FLAG is 1 during a labelled attack, -999 otherwise (unlabelled / normal).
        att = 1 if row[fi["ATT_FLAG"]] == "1" else 0
        n_attack += att
        out_rows.append(
            {
                "L_T1": float(row[fi["L_T1"]]),
                "F_PU1": float(row[fi["F_PU1"]]),
                "F_PU2": float(row[fi["F_PU2"]]),
                "F_PU3": float(row[fi["F_PU3"]]),
                "label": att,
                "phase": 0,
            }
        )

    csv_path = os.path.join(CONTROLLED, "batadal_t1_instrumented.csv")
    with open(csv_path, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(cols)
        for r in out_rows:
            w.writerow([f"{r[c]:.6f}" if isinstance(r[c], float) else r[c] for c in cols])

    roles = {
        "dataset": "batadal_t1_instrumented",
        "kind": "real",  # real public SCADA (simulated C-Town network, hourly), not a bespoke unit
        "description": "BATADAL C-Town tank T1 volume (mass) balance witness on real public SCADA "
        "with labelled cyber-attacks; PU2 attacks break the T1 mass balance.",
        "provenance": {
            "benchmark": "BATADAL (Battle of the Attack Detection Algorithms)",
            "citation": "Taormina, Galelli, et al., J. Water Resources Planning & Management 144(8), 2018",
            "source_url": URL,
            "network_file": "ctown_map.inp (EPANET), https://github.com/rtaormina/epanetCPA",
            "license": "freely available for research (batadal.net)",
            "note": "raw data fetched by this script, not redistributed in the repository",
        },
        "variables": [
            {"name": "L_T1", "role": "measured", "unit": "m", "quantity": "level"},
            {"name": "F_PU1", "role": "measured", "unit": "L/s", "quantity": "inflow"},
            {"name": "F_PU2", "role": "measured", "unit": "L/s", "quantity": "inflow"},
            {"name": "F_PU3", "role": "measured", "unit": "L/s", "quantity": "inflow"},
        ],
        "balance": {
            "type": "mass_tank_volume",
            "area_m2": round(T1_AREA_M2, 4),
            "dt": DT_HOURS,
            "level": "L_T1",
            "inflows": ["F_PU1", "F_PU2", "F_PU3"],
            "flow_to_vol_per_dt": LPS_TO_M3_PER_H,
            "definition": "residual = area*dL/dt - 3.6*(F_PU1+F_PU2+F_PU3)  [m^3/h]; "
            "= -(unmetered DMA demand) under normal operation",
        },
        "fault": {
            "mode": "cyber-attacks manipulating pump PU2 (T1 inflow); spoofed flow vs real drain",
            "note": "multiple disjoint labelled windows; only PU2/T1 attacks break the T1 balance",
        },
    }
    with open(os.path.join(OUT, "batadal_t1_instrumented.roles.json"), "w") as f:
        json.dump(roles, f, indent=2)

    print(f"derived {csv_path}  ({len(out_rows)} hourly rows, {n_attack} labelled attack-hours)")
    print(f"T1 area = {T1_AREA_M2:.1f} m^2 (diameter {T1_DIAM_M} m from ctown_map.inp)")


if __name__ == "__main__":
    fetch()
    derive()
