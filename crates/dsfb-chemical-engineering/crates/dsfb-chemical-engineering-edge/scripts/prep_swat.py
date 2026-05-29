#!/usr/bin/env python3
"""Prepare a REAL physical-plant mass-balance witness slice from the SWaT testbed.

SWaT (Secure Water Treatment; Goh et al., iTrust/SUTD, 2016) is a real six-stage water-treatment
testbed logged at 1 Hz, with 41 cyber-physical attacks over four days. Stage 1 is a raw-water tank
T101 with a *closed, fully-metered* volume balance:

    area * dLIT101/dt = FIT101 (inflow) - FIT201 (outflow via pump P101 to stage 2).

Several SWaT attacks are sensor spoofs on this stage -- e.g. freezing LIT101 at 700 mm while P101
keeps pumping the tank down: the level reads flat while the flow meters say it must be draining, so
dLIT/dt contradicts (FIT101 - FIT201) and the mass-balance closure jumps. That is exactly what the
DSFB `mass_tank_volume` witness keys on (sustained shift), the same mechanism as the BATADAL PU2
attack -- here on a real physical plant rather than a network model.

DATA IS NOT REDISTRIBUTED. SWaT is released under an iTrust usage agreement; obtain it yourself and
point this script at the CSVs (env SWAT_DIR, else data/instrumented/raw/swat/). The Kaggle packaging
used here splits the record into `normal.csv` (the 7-day normal run) and `attack.csv` (the
attack-labelled rows extracted from the attack run). Because that packaging drops the normal periods
*between* attacks, we reconstruct a continuous normal-with-attacks sequence: real normal blocks form
the calibration baseline and the spacers between attack segments, and each attack segment becomes its
own labelled window. To beat the 1 s level-sensor quantisation (+/-5 mm jitter vs a ~0.5 mm/s signal)
the series is aggregated into 30 s blocks; segment joins are bridged so only *within-segment* level
changes enter the balance (absolute level is irrelevant -- the balance uses only dL).

Run:  python3 scripts/prep_swat.py
"""
import csv
import json
import math
import os
import statistics as st
from datetime import datetime

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(HERE, "data", "instrumented")
SWAT_DIR = os.environ.get("SWAT_DIR", os.path.join(OUT, "raw", "swat"))
# Controlled SWaT bytes are NEVER tracked. The derived instrumented CSV goes to the untracked research/
# quarantine (DSFB_CONTROLLED_DIR overrides); only the metadata roles sidecar (no data rows) stays under OUT.
CONTROLLED = os.environ.get("DSFB_CONTROLLED_DIR", os.path.join(HERE, "..", "..", "research", "controlled"))
os.makedirs(CONTROLLED, exist_ok=True)
BLOCK = 30          # seconds per aggregation block
BASELINE_BLOCKS = 2000
SPACER_BLOCKS = 20  # real normal blocks inserted before each attack segment


def parse_ts(s):
    s = s.strip()
    for fmt in ("%d/%m/%Y %I:%M:%S %p", "%d/%m/%Y %H:%M:%S"):
        try:
            return datetime.strptime(s, fmt)
        except ValueError:
            pass
    return None


def read_cols(path):
    """Yield (timestamp, FIT101, LIT101, FIT201, label) per row."""
    with open(path) as f:
        r = csv.reader(f)
        h = [c.strip() for c in next(r)]
        ci = {c: i for i, c in enumerate(h)}
        iL, i1, i2 = ci["LIT101"], ci["FIT101"], ci["FIT201"]
        ilab = ci.get("Normal/Attack")
        for row in r:
            try:
                yield (parse_ts(row[0]), float(row[i1]), float(row[iL]), float(row[i2]),
                       (row[ilab].strip() if ilab is not None else "Attack"))
            except (ValueError, IndexError):
                continue


def blockify(rows):
    """Aggregate a contiguous run of 1 s rows into 30 s blocks (mean of each channel)."""
    out = []
    for i in range(0, len(rows) - BLOCK + 1, BLOCK):
        seg = rows[i:i + BLOCK]
        out.append((
            st.mean(r[2] for r in seg),   # LIT101 mean
            st.mean(r[1] for r in seg),   # FIT101 mean
            st.mean(r[3] for r in seg),   # FIT201 mean
        ))
    return out


def main():
    npath = os.path.join(SWAT_DIR, "normal.csv")
    apath = os.path.join(SWAT_DIR, "attack.csv")
    if not (os.path.exists(npath) and os.path.exists(apath)):
        raise SystemExit(f"SWaT CSVs not found in {SWAT_DIR}. Set SWAT_DIR or place normal.csv/attack.csv there.")

    # Normal run -> a pool of 30 s blocks (calibration baseline + inter-attack spacers).
    norm_rows = list(read_cols(npath))
    # skip the first hours (plant start-up); take a stable mid-run stretch.
    norm_rows = norm_rows[100_000:100_000 + (BASELINE_BLOCKS + 40 * SPACER_BLOCKS + 200) * BLOCK]
    NORM = blockify(norm_rows)

    # Attack run -> split into contiguous-time segments (the Kaggle file drops inter-attack gaps).
    atk = list(read_cols(apath))
    segs, cur = [], [atk[0]]
    for i in range(1, len(atk)):
        if atk[i][0] is None or atk[i - 1][0] is None or (atk[i][0] - atk[i - 1][0]).total_seconds() > 2:
            segs.append(cur)
            cur = []
        cur.append(atk[i])
    segs.append(cur)
    seg_blocks = [blockify(s) for s in segs if len(s) >= BLOCK]

    # Build the continuous sequence with bridged joins (dL=0 at every run boundary; within-run dL real).
    rows = []          # (level, FIT101, FIT201, label)
    running = [None]

    def emit_run(blocks, lbl):
        for j, blk in enumerate(blocks):
            if running[0] is None:
                running[0] = blk[0]
            elif j > 0:
                running[0] += blk[0] - blocks[j - 1][0]   # within-run level change
            # j==0 of a later run: join -> dL contribution 0 (running unchanged)
            rows.append((running[0], blk[1], blk[2], lbl))

    emit_run(NORM[:BASELINE_BLOCKS], 0)
    ni = BASELINE_BLOCKS
    for sb in seg_blocks:
        emit_run(NORM[ni:ni + SPACER_BLOCKS], 0)
        ni += SPACER_BLOCKS
        emit_run(sb, 1)

    # Calibrate the level-per-net-flow factor on the baseline run (through-origin), so the closure is
    # ~0 in normal operation. Subsumes tank cross-section and the 30 s block.
    base = rows[:BASELINE_BLOCKS]
    num = den = 0.0
    for k in range(1, len(base)):
        net = base[k][1] - base[k][2]
        dL = base[k][0] - base[k - 1][0]
        num += net * dL
        den += net * net
    factor = num / den if den else 6.0

    cols = ["LIT101", "FIT101", "FIT201", "label", "phase"]
    csv_path = os.path.join(CONTROLLED, "swat_t101_instrumented.csv")
    with open(csv_path, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(cols)
        for lvl, f1, f2, lbl in rows:
            w.writerow([f"{lvl:.4f}", f"{f1:.5f}", f"{f2:.5f}", lbl, 0])

    roles = {
        "dataset": "swat_t101_instrumented",
        "kind": "real",  # real physical water-treatment testbed (iTrust SWaT), 1 Hz -> 30 s blocks
        "description": "SWaT stage-1 raw-water tank T101 volume (mass) balance witness on a real "
        "water-treatment testbed; LIT101 sensor-spoofing attacks break the closure.",
        "provenance": {
            "testbed": "SWaT (Secure Water Treatment), iTrust / SUTD",
            "citation": "Goh, Adepu, Junejo, Mathur, 'A Dataset to Support Research in the Design of "
            "Secure Water Treatment Systems', CRITIS 2016",
            "license": "iTrust usage agreement; data NOT redistributed in this repository",
            "note": "Kaggle packaging (normal.csv + attack.csv) reconstructed into a continuous "
            "normal-with-attacks sequence; 30 s aggregation; segment joins bridged (only within-"
            "segment level changes enter the balance).",
        },
        "variables": [
            {"name": "LIT101", "role": "measured", "unit": "mm", "quantity": "level"},
            {"name": "FIT101", "role": "measured", "unit": "m^3/h", "quantity": "inflow"},
            {"name": "FIT201", "role": "measured", "unit": "m^3/h", "quantity": "outflow"},
        ],
        "balance": {
            "type": "mass_tank_volume",
            "area_m2": 1.0,
            "dt": 1.0,
            "level": "LIT101",
            "inflows": ["FIT101"],
            "outflows": ["FIT201"],
            "flow_to_vol_per_dt": round(factor, 5),
            "definition": "residual = dLIT101 - factor*(FIT101 - FIT201) [mm per 30 s block]; ~0 under "
            "normal control, jumps when a spoofed LIT101 contradicts the metered net flow",
        },
        "fault": {
            "mode": "cyber-attacks spoofing LIT101 (and stage-1 flow) -- level frozen/biased while "
            "the flow meters keep moving",
            "note": "multiple disjoint labelled windows; only stage-1 (LIT101/flow) attacks break the "
            "T101 balance",
        },
    }
    with open(os.path.join(OUT, "swat_t101_instrumented.roles.json"), "w") as f:
        json.dump(roles, f, indent=2)

    n_attack = sum(1 for r in rows if r[3] == 1)
    print(f"wrote {csv_path}: {len(rows)} 30 s-blocks ({n_attack} attack, {len(seg_blocks)} segments)")
    print(f"calibrated factor (level-mm per m^3/h over 30 s) = {factor:.4f}")


if __name__ == "__main__":
    main()
